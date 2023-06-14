/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use der::{Decode, Encode, Reader, Writer};

pub const MAX_NUT_PREC_ANGLES: usize = 16;

use super::ellipsoid::Ellipsoid;
use super::{phaseangle::PhaseAngle, trigangle::TrigAngle};
use crate::structure::array::DataArray;
use crate::NaifId;

/// ANISE supports two different kinds of orientation data. High precision, with spline based interpolations, and constants right ascension, declination, and prime meridian, typically used for planetary constant data.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PlanetaryConstant<'a> {
    /// The NAIF ID of this object
    pub object_id: NaifId,
    /// Gravitational parameter (μ) of this planetary object.
    pub mu_km3_s2: f64,
    /// The shape is always a tri axial ellipsoid
    pub shape: Option<Ellipsoid>,
    ///     TODO: Create a PoleOrientation structure which is optional. If defined, it includes the stuff below, and none optional (DataArray can be empty).
    pub pole_right_ascension: Option<PhaseAngle>,
    pub pole_declination: Option<PhaseAngle>,
    pub prime_meridian: Option<PhaseAngle>,
    pub nut_prec_angles: Option<DataArray<'a, TrigAngle>>,
}

/*
Consider this option since the nut prec angles are shared

Also consider that this data is either entirely nil, like when we have a BPC entry, or
all four exist.
But I also need to specify in the flag whether it's a BPC defined or constants defined.

// The maximum expected number of polynomial coefficients.
const MAX_COEFFICIENTS: usize = 10;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PhaseAngle {
    pub offset_deg: f64,
    pub rate_deg: f64,
    pub accel_deg: f64,
    // Add an array for the coefficients tuple
    pub coefficients: [(f64, f64); MAX_COEFFICIENTS],
    // Track how many coefficients are actually used
    pub num_coefficients: usize,
}

// Usage:
impl PhaseAngle {
    pub fn new() -> Self {
        PhaseAngle {
            offset_deg: 0.0,
            rate_deg: 0.0,
            accel_deg: 0.0,
            coefficients: [0.0; MAX_COEFFICIENTS],
            num_coefficients: 0,
        }
    }

    // Adds a new coefficient, if there is room.
    // Returns whether the addition was successful.
    pub fn add_coefficient(&mut self, coefficient: f64) -> bool {
        if self.num_coefficients < MAX_COEFFICIENTS {
            self.coefficients[self.num_coefficients] = coefficient;
            self.num_coefficients += 1;
            true
        } else {
            false
        }
    }
}



 */

impl<'a> PlanetaryConstant<'a> {
    /// Specifies what data is available in this structure.
    ///
    /// Returns:
    /// + Bit 0 is set if `shape` is available
    /// + Bit 1 is set if `pole_right_ascension` is available
    /// + Bit 2 is set if `pole_declination` is available
    /// + Bit 3 is set if `prime_meridian` is available
    /// + Bit 4 is set if nut_prec_angles` is available.
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
        if self.nut_prec_angles.is_some() {
            bits |= 1 << 4;
        }

        bits
    }
}

impl<'a> Encode for PlanetaryConstant<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let available_flags = self.available_data();
        self.object_id.encoded_len()?
            + self.mu_km3_s2.encoded_len()?
            + available_flags.encoded_len()?
            + self.shape.encoded_len()?
            + self.pole_right_ascension.encoded_len()?
            + self.pole_declination.encoded_len()?
            + self.prime_meridian.encoded_len()?
            + self.nut_prec_angles.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.object_id.encode(encoder)?;
        self.mu_km3_s2.encode(encoder)?;
        self.available_data().encode(encoder)?;
        self.shape.encode(encoder)?;
        self.pole_right_ascension.encode(encoder)?;
        self.pole_declination.encode(encoder)?;
        self.prime_meridian.encode(encoder)?;
        self.nut_prec_angles.encode(encoder)
    }
}

impl<'a> Decode<'a> for PlanetaryConstant<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let object_id: NaifId = decoder.decode()?;
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

        let nut_prec_angles = if data_flags & (1 << 4) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        Ok(Self {
            object_id,
            mu_km3_s2,
            shape,
            pole_right_ascension,
            pole_declination,
            prime_meridian,
            nut_prec_angles,
        })
    }
}

#[cfg(test)]
mod planetary_constants_ut {
    use super::{Ellipsoid, PhaseAngle, PlanetaryConstant};
    use crate::structure::der::{Decode, Encode};

    #[test]
    fn pc_encdec_min_repr() {
        // A minimal representation of a planetary constant.
        let repr = PlanetaryConstant {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = PlanetaryConstant::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn pc_encdec_with_shape_only() {
        let earth_data = Ellipsoid::from_spheroid(6378.1366, 6356.7519);
        let repr = PlanetaryConstant {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            shape: Some(earth_data),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = PlanetaryConstant::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn pc_encdec_with_pole_ra_only() {
        let earth_data = PhaseAngle {
            offset_deg: 270.0,
            rate_deg: 0.003,
            accel_deg: 0.0,
        };
        let repr = PlanetaryConstant {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            pole_right_ascension: Some(earth_data),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = PlanetaryConstant::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn pc_encdec_with_pole_dec_only() {
        let earth_data = PhaseAngle {
            offset_deg: 66.541,
            rate_deg: 0.013,
            accel_deg: 0.0,
        };
        let repr = PlanetaryConstant {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            pole_declination: Some(earth_data),
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = PlanetaryConstant::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn pc_encdec_with_pm_only() {
        let earth_data = PhaseAngle {
            offset_deg: 38.317,
            rate_deg: 13.1763582,
            accel_deg: 0.0,
        };
        let min_repr = PlanetaryConstant {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            prime_meridian: Some(earth_data),
            ..Default::default()
        };

        let mut buf = vec![];
        min_repr.encode_to_vec(&mut buf).unwrap();

        let min_repr_dec = PlanetaryConstant::from_der(&buf).unwrap();

        assert_eq!(min_repr, min_repr_dec);
    }

    #[test]
    fn pc_encdec_with_dec_pm_only() {
        let earth_data_dec = PhaseAngle {
            offset_deg: 66.541,
            rate_deg: 0.013,
            accel_deg: 0.0,
        };
        let earth_data_pm = PhaseAngle {
            offset_deg: 38.317,
            rate_deg: 13.1763582,
            accel_deg: 0.0,
        };
        let min_repr = PlanetaryConstant {
            object_id: 1234,
            mu_km3_s2: 12345.6789,
            pole_declination: Some(earth_data_dec),
            prime_meridian: Some(earth_data_pm),
            ..Default::default()
        };

        let mut buf = vec![];
        min_repr.encode_to_vec(&mut buf).unwrap();

        let min_repr_dec = PlanetaryConstant::from_der(&buf).unwrap();

        assert_eq!(min_repr, min_repr_dec);
    }
}
