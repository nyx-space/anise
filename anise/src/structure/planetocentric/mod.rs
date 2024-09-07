/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{
    astro::PhysicsResult,
    constants::orientations::orientation_name_from_id,
    math::{
        rotation::{r1, r3, DCM},
        Matrix3,
    },
    prelude::{Frame, FrameUid},
    NaifId,
};
use core::f64::consts::FRAC_PI_2;
use core::fmt;
pub mod ellipsoid;
pub mod phaseangle;
use der::{Decode, Encode, Reader, Writer};
use ellipsoid::Ellipsoid;
use hifitime::{Epoch, TimeUnits, Unit};
use phaseangle::PhaseAngle;

use super::dataset::DataSetT;

pub const MAX_NUT_PREC_ANGLES: usize = 32;

/// ANISE supports two different kinds of orientation data. High precision, with spline based interpolations, and constants right ascension, declination, and prime meridian, typically used for planetary constant data.
///
/// # Documentation of rotation angles
/// Source: <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/pck.html#Models%20for%20the%20Sun,%20Planets,%20and%20some%20Minor%20Bodies%20in%20Text%20PCK%20Kernels>
///  The angles RA, DEC, and W are defined as follows:
///
/// ```text
///                                  2
///                             RA2*t
/// RA  =  RA0  +  RA1*t/T  +  ------  + [optional trig polynomials]
///                                2
///                               T
///
///                                  2
///                            DEC2*t
/// DEC =  DEC0 + DEC1*t/T  +  ------- + [optional trig polynomials]
///                                2
///                               T
///
///                                 2
///                             W2*t
/// W   =  W0   + W1*t/d    +  -----   + [optional trig polynomials]
///                               2
///                              d
/// ```
///
/// where
///
/// d = seconds/day
/// T = seconds/Julian century
/// t = ephemeris time, expressed as seconds past the reference epoch
/// for this body or planetary system
///
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PlanetaryData {
    /// The NAIF ID of this object
    pub object_id: NaifId,
    /// The NAIF ID of the parent orientation, NOT the parent translation
    pub parent_id: NaifId,
    /// Gravitational parameter (μ) of this planetary object.
    pub mu_km3_s2: f64,
    /// The shape is always a tri axial ellipsoid
    pub shape: Option<Ellipsoid>,
    pub pole_right_ascension: Option<PhaseAngle<MAX_NUT_PREC_ANGLES>>,
    pub pole_declination: Option<PhaseAngle<MAX_NUT_PREC_ANGLES>>,
    pub prime_meridian: Option<PhaseAngle<MAX_NUT_PREC_ANGLES>>,
    pub long_axis: Option<f64>,
    /// These are the nutation precession angles as a list of tuples to rebuild them.
    /// E.g. For `E1 = 125.045 -  0.052992 d`, this would be stored as a single entry `(125.045, -0.052992)`.
    pub num_nut_prec_angles: u8,
    pub nut_prec_angles: [PhaseAngle<0>; MAX_NUT_PREC_ANGLES],
}

impl DataSetT for PlanetaryData {
    const NAME: &'static str = "planetary data";
}

impl PlanetaryData {
    /// Converts this planetary data into a Frame, unsetting any shape data for non-body-fixed frames (ID < 100).
    pub fn to_frame(&self, uid: FrameUid) -> Frame {
        Frame {
            ephemeris_id: uid.ephemeris_id,
            orientation_id: uid.orientation_id,
            mu_km3_s2: Some(self.mu_km3_s2),
            shape: self.shape,
        }
    }
    /// Specifies what data is available in this structure.
    ///
    /// Returns:
    /// + Bit 0 is set if `shape` is available
    /// + Bit 1 is set if `pole_right_ascension` is available
    /// + Bit 2 is set if `pole_declination` is available
    /// + Bit 3 is set if `prime_meridian` is available
    /// + Bit 4 is set if `long_axis` is available
    fn available_data(&self) -> u8 {
        let mut bits: u8 = 0;

        if self.shape.is_some() {
            bits |= 1 << 0;
        }
        if self.pole_right_ascension.is_some() {
            bits |= 1 << 1;
        }
        if self.pole_declination.is_some() {
            bits |= 1 << 2;
        }
        if self.prime_meridian.is_some() {
            bits |= 1 << 3;
        }
        if self.long_axis.is_some() {
            bits |= 1 << 4;
        }

        bits
    }

    fn uses_trig_polynomial(&self) -> bool {
        if let Some(phase) = self.pole_right_ascension {
            if phase.coeffs_count > 0 {
                return true;
            }
        }

        if let Some(phase) = self.pole_declination {
            if phase.coeffs_count > 0 {
                return true;
            }
        }

        if let Some(phase) = self.prime_meridian {
            if phase.coeffs_count > 0 {
                return true;
            }
        }

        false
    }

    /// Computes the rotation to the parent frame, returning only the rotation matrix
    fn dcm_to_parent(&self, epoch: Epoch, system: &Self) -> PhysicsResult<Matrix3> {
        if self.pole_declination.is_none()
            && self.prime_meridian.is_none()
            && self.pole_right_ascension.is_none()
        {
            Ok(Matrix3::identity())
        } else {
            let mut variable_angles_rad = [0.0_f64; MAX_NUT_PREC_ANGLES];
            // Skip the computation of the nutation and precession angles of the system if we won't be using them.
            if self.uses_trig_polynomial() {
                for (ii, nut_prec_angle) in system
                    .nut_prec_angles
                    .iter()
                    .enumerate()
                    .take(system.num_nut_prec_angles.into())
                {
                    variable_angles_rad[ii] = nut_prec_angle
                        .evaluate_deg(epoch, Unit::Century)
                        .to_radians();
                }
            }

            let right_asc_rad = match self.pole_right_ascension {
                Some(right_asc_deg) => {
                    let mut angle_deg = right_asc_deg.evaluate_deg(epoch, Unit::Century);
                    // Add the nutation and precession angles for this phase angle
                    for (ii, coeff) in right_asc_deg
                        .coeffs
                        .iter()
                        .enumerate()
                        .take(right_asc_deg.coeffs_count as usize)
                    {
                        angle_deg += coeff * variable_angles_rad[ii].sin();
                    }
                    angle_deg.to_radians() + FRAC_PI_2
                }
                None => 0.0,
            };

            let dec_rad = match self.pole_declination {
                Some(decl_deg) => {
                    let mut angle_deg = decl_deg.evaluate_deg(epoch, Unit::Century);
                    // Add the nutation and precession angles for this phase angle
                    for (ii, coeff) in decl_deg
                        .coeffs
                        .iter()
                        .enumerate()
                        .take(decl_deg.coeffs_count as usize)
                    {
                        angle_deg += coeff * variable_angles_rad[ii].cos();
                    }
                    FRAC_PI_2 - angle_deg.to_radians()
                }
                None => 0.0,
            };

            let twist_rad = match self.prime_meridian {
                Some(twist_deg) => {
                    let mut angle_deg = twist_deg.evaluate_deg(epoch, Unit::Day);
                    // Add the nutation and precession angles for this phase angle
                    for (ii, coeff) in twist_deg
                        .coeffs
                        .iter()
                        .enumerate()
                        .take(twist_deg.coeffs_count as usize)
                    {
                        angle_deg += coeff * variable_angles_rad[ii].sin();
                    }
                    angle_deg.to_radians()
                }
                None => 0.0,
            };

            let ra_dcm = r3(right_asc_rad);
            let dec_dcm = r1(dec_rad);
            let w_dcm = r3(twist_rad);
            // Perform a multiplication of the DCMs, regardless of frames.
            Ok(w_dcm * dec_dcm * ra_dcm)
        }
    }

    /// Computes the rotation to the parent frame, including its time derivative.
    ///
    /// Source: <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/rotation.html#Working%20with%20RA,%20Dec%20and%20Twist>
    pub fn rotation_to_parent(&self, epoch: Epoch, system: &Self) -> PhysicsResult<DCM> {
        if self.pole_declination.is_none()
            && self.prime_meridian.is_none()
            && self.pole_right_ascension.is_none()
        {
            Ok(DCM::identity(self.object_id, self.parent_id))
        } else {
            // For planetary constants data, we perform a finite differencing to compute the time derivative.
            let mut dcm = DCM {
                rot_mat: self.dcm_to_parent(epoch, system)?,
                from: self.parent_id,
                to: self.object_id,
                rot_mat_dt: None,
            };
            // Compute rotation matrix one second before
            let pre_rot_dcm = self.dcm_to_parent(epoch - 1.seconds(), system)?;
            let post_rot_dcm = self.dcm_to_parent(epoch + 1.seconds(), system)?;

            dcm.rot_mat_dt = Some((post_rot_dcm - pre_rot_dcm) / 2.0);

            Ok(dcm)
        }
    }
}

impl Encode for PlanetaryData {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let available_flags = self.available_data();
        self.object_id.encoded_len()?
            + self.parent_id.encoded_len()?
            + self.mu_km3_s2.encoded_len()?
            + available_flags.encoded_len()?
            + self.shape.encoded_len()?
            + self.pole_right_ascension.encoded_len()?
            + self.pole_declination.encoded_len()?
            + self.prime_meridian.encoded_len()?
            + self.long_axis.encoded_len()?
            + self.num_nut_prec_angles.encoded_len()?
            + self.nut_prec_angles.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.object_id.encode(encoder)?;
        self.parent_id.encode(encoder)?;
        self.mu_km3_s2.encode(encoder)?;
        self.available_data().encode(encoder)?;
        self.shape.encode(encoder)?;
        self.pole_right_ascension.encode(encoder)?;
        self.pole_declination.encode(encoder)?;
        self.prime_meridian.encode(encoder)?;
        self.long_axis.encode(encoder)?;
        self.num_nut_prec_angles.encode(encoder)?;
        self.nut_prec_angles.encode(encoder)
    }
}

impl<'a> Decode<'a> for PlanetaryData {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let object_id: NaifId = decoder.decode()?;
        let parent_id: NaifId = decoder.decode()?;
        let mu_km3_s2: f64 = decoder.decode()?;

        let data_flags: u8 = decoder.decode()?;

        let shape = if data_flags & (1 << 0) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        let pole_right_ascension = if data_flags & (1 << 1) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        let pole_declination = if data_flags & (1 << 2) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        let prime_meridian = if data_flags & (1 << 3) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        let long_axis = if data_flags & (1 << 4) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        Ok(Self {
            object_id,
            parent_id,
            mu_km3_s2,
            shape,
            pole_right_ascension,
            pole_declination,
            prime_meridian,
            long_axis,
            num_nut_prec_angles: decoder.decode()?,
            nut_prec_angles: decoder.decode()?,
        })
    }
}

impl fmt::Display for PlanetaryData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Initialize new frame UIDs with arbitrary ephemeris centers, and we don't print those.
        let orientation_name = match orientation_name_from_id(self.object_id) {
            Some(name) => name.to_string(),
            None => format!("planetary data {}", self.object_id),
        };

        write!(f, "{orientation_name}")?;
        match self.shape {
            Some(shape) => {
                write!(f, " (μ = {} km^3/s^2, {})", self.mu_km3_s2, shape)?;
            }
            None => {
                write!(f, " (μ = {} km^3/s^2)", self.mu_km3_s2)?;
            }
        }

        if let Some(ra) = self.pole_right_ascension {
            write!(f, " RA = {}", ra)?;
        }
        if let Some(dec) = self.pole_declination {
            write!(f, " Dec = {}", dec)?;
        }
        if let Some(pm) = self.prime_meridian {
            write!(f, " PM = {}", pm)?;
        }
        if self.num_nut_prec_angles > 0 {
            write!(f, " + {} nut/prec angles", self.num_nut_prec_angles)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod planetary_constants_ut {
    use super::{Ellipsoid, PhaseAngle, PlanetaryData};
    use der::{Decode, Encode};

    #[test]
    fn pc_encdec_min_repr() {
        // A minimal representation of a planetary constant.
        let repr = PlanetaryData {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = PlanetaryData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
        assert_eq!(
            format!("{repr}"),
            "planetary data 1234 (μ = 12345.6789 km^3/s^2)"
        );
    }

    #[test]
    fn pc_encdec_with_shape_only() {
        let earth_data = Ellipsoid::from_spheroid(6378.1366, 6356.7519);
        let repr = PlanetaryData {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            shape: Some(earth_data),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = PlanetaryData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
        assert_eq!(
            format!("{repr}"),
            "planetary data 1234 (μ = 12345.6789 km^3/s^2, eq. radius = 6378.1366 km, polar radius = 6356.7519 km, f = 0.0033528131084554717)"
        );
    }

    #[test]
    fn pc_encdec_with_pole_ra_only() {
        let earth_data = PhaseAngle {
            offset_deg: 270.0,
            rate_deg: 0.003,
            ..Default::default()
        };
        let repr = PlanetaryData {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            pole_right_ascension: Some(earth_data),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = PlanetaryData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
        assert_eq!(
            format!("{repr}"),
            "planetary data 1234 (μ = 12345.6789 km^3/s^2) RA = 270 + 0.003 t"
        );
    }

    #[test]
    fn pc_encdec_with_pole_dec_only() {
        let earth_data = PhaseAngle {
            offset_deg: 66.541,
            rate_deg: 0.013,
            ..Default::default()
        };
        let repr = PlanetaryData {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            pole_declination: Some(earth_data),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = PlanetaryData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
        assert_eq!(
            format!("{repr}"),
            "planetary data 1234 (μ = 12345.6789 km^3/s^2) Dec = 66.541 + 0.013 t"
        );
    }

    #[test]
    fn pc_encdec_with_pm_only() {
        let earth_data = PhaseAngle {
            offset_deg: 38.317,
            rate_deg: 13.1763582,
            ..Default::default()
        };
        let repr = PlanetaryData {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            prime_meridian: Some(earth_data),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let min_repr_dec = PlanetaryData::from_der(&buf).unwrap();

        assert_eq!(repr, min_repr_dec);
        assert_eq!(
            format!("{repr}"),
            "planetary data 1234 (μ = 12345.6789 km^3/s^2) PM = 38.317 + 13.1763582 t"
        );
    }

    #[test]
    fn pc_encdec_with_dec_pm_only() {
        let earth_data_dec = PhaseAngle {
            offset_deg: 66.541,
            rate_deg: 0.013,
            ..Default::default()
        };
        let earth_data_pm = PhaseAngle {
            offset_deg: 38.317,
            rate_deg: 13.1763582,
            ..Default::default()
        };
        let repr = PlanetaryData {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            pole_declination: Some(earth_data_dec),
            prime_meridian: Some(earth_data_pm),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();
        assert_eq!(buf.len(), 566);

        let min_repr_dec = PlanetaryData::from_der(&buf).unwrap();

        assert_eq!(repr, min_repr_dec);

        assert_eq!(core::mem::size_of::<PlanetaryData>(), 1984);

        assert_eq!(format!("{repr}"), "planetary data 1234 (μ = 12345.6789 km^3/s^2) Dec = 66.541 + 0.013 t PM = 38.317 + 13.1763582 t");
    }

    #[test]
    fn pc_encdec_with_long_axis_only() {
        let min_repr = PlanetaryData {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            long_axis: Some(1789.4),
            ..Default::default()
        };

        let mut buf = vec![];
        min_repr.encode_to_vec(&mut buf).unwrap();

        let min_repr_dec = PlanetaryData::from_der(&buf).unwrap();

        assert_eq!(min_repr, min_repr_dec);
    }

    #[test]
    fn test_301() {
        // Build the Moon 301 representation from pck00008.tpc data
        // We build it from the slice that concats the POLA_RA and NUT_PREC_RA
        let pole_ra = PhaseAngle::maybe_new(&[
            269.9949, 0.0031, 0.0, -3.8787, -0.1204, 0.0700, -0.0172, 0.0, 0.0072, 0.0, 0.0, 0.0,
            -0.0052, 0.0, 0.0, 0.0043,
        ]);
        assert_eq!(pole_ra.unwrap().coeffs_count, 13);
        let pole_dec = PhaseAngle::maybe_new(&[
            66.5392, 0.0130, 0., 1.5419, 0.0239, -0.0278, 0.0068, 0.0, -0.0029, 0.0009, 0.0, 0.0,
            0.0008, 0.0, 0.0, -0.0009,
        ]);
        assert_eq!(pole_dec.unwrap().coeffs_count, 13);
        let prime_m = PhaseAngle::maybe_new(&[
            38.3213,
            13.17635815,
            -1.4e-12,
            3.5610,
            0.1208,
            -0.0642,
            0.0158,
            0.0252,
            -0.0066,
            -0.0047,
            -0.0046,
            0.0028,
            0.0052,
            0.0040,
            0.0019,
            -0.0044,
        ]);
        assert_eq!(prime_m.as_ref().unwrap().coeffs_count, 13);

        let gm_moon = 4.902_800_066_163_796E3;

        let moon = PlanetaryData {
            object_id: 301,
            parent_id: 0,
            mu_km3_s2: gm_moon,
            shape: None,
            pole_right_ascension: pole_ra,
            pole_declination: pole_dec,
            prime_meridian: prime_m,
            long_axis: None,
            num_nut_prec_angles: 0,
            nut_prec_angles: Default::default(),
        };

        // Encode
        let mut buf = vec![];
        moon.encode_to_vec(&mut buf).unwrap();
        assert_eq!(buf.len(), 946);

        let moon_dec = PlanetaryData::from_der(&buf).unwrap();

        assert_eq!(moon, moon_dec);

        assert_eq!(format!("{moon}"), "IAU_MOON (μ = 4902.800066163796 km^3/s^2) RA = 269.9949 + 0.0031 t Dec = 66.5392 + 0.013 t PM = 38.3213 + 13.17635815 t + -0.0000000000014 t^2");
    }
}
