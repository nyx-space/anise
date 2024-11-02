/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt;
use core::fmt::Debug;
use serde_derive::{Deserialize, Serialize};
use snafu::ResultExt;

#[cfg(feature = "metaload")]
use serde_dhall::StaticType;

use crate::astro::PhysicsResult;
use crate::constants::celestial_objects::{
    celestial_name_from_id, id_to_celestial_name, SOLAR_SYSTEM_BARYCENTER,
};
use crate::constants::orientations::{id_to_orientation_name, orientation_name_from_id, J2000};
use crate::errors::{AlmanacError, EphemerisSnafu, OrientationSnafu, PhysicsError};
use crate::prelude::FrameUid;
use crate::structure::planetocentric::ellipsoid::Ellipsoid;
use crate::NaifId;

#[cfg(feature = "python")]
use pyo3::exceptions::PyTypeError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::pyclass::CompareOp;

/// A Frame uniquely defined by its ephemeris center and orientation. Refer to FrameDetail for frames combined with parameters.
///
/// :type ephemeris_id: int
/// :type orientation_id: int
/// :type mu_km3_s2: float, optional
/// :type shape: Ellipsoid, optional
/// :rtype: Frame
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "metaload", derive(StaticType))]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct Frame {
    pub ephemeris_id: NaifId,
    pub orientation_id: NaifId,
    /// Gravity parameter of this frame, only defined on celestial frames
    pub mu_km3_s2: Option<f64>,
    /// Shape of the geoid of this frame, only defined on geodetic frames
    pub shape: Option<Ellipsoid>,
}

impl Frame {
    /// Constructs a new frame given its ephemeris and orientations IDs, without defining anything else (so this is not a valid celestial frame, although the data could be populated later).
    pub const fn new(ephemeris_id: NaifId, orientation_id: NaifId) -> Self {
        Self {
            ephemeris_id,
            orientation_id,
            mu_km3_s2: None,
            shape: None,
        }
    }

    pub const fn from_ephem_j2000(ephemeris_id: NaifId) -> Self {
        Self::new(ephemeris_id, J2000)
    }

    pub const fn from_orient_ssb(orientation_id: NaifId) -> Self {
        Self::new(SOLAR_SYSTEM_BARYCENTER, orientation_id)
    }

    /// Attempts to create a new frame from its center and reference frame name.
    /// This function is compatible with the CCSDS OEM names.
    pub fn from_name(center: &str, ref_frame: &str) -> Result<Self, AlmanacError> {
        let ephemeris_id = id_to_celestial_name(center).context(EphemerisSnafu {
            action: "converting center name to its ID",
        })?;

        let orientation_id = id_to_orientation_name(ref_frame).context(OrientationSnafu {
            action: "converting reference frame to its ID",
        })?;

        Ok(Self::new(ephemeris_id, orientation_id))
    }

    /// Define Ellipsoid shape and return a new [Frame]
    pub fn with_ellipsoid(mut self, shape: Ellipsoid) -> Self {
        self.shape = Some(shape);
        self
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl Frame {
    /// Initializes a new [Frame] provided its ephemeris and orientation identifiers, and optionally its gravitational parameter (in km^3/s^2) and optionally its shape (cf. [Ellipsoid]).
    #[new]
    pub fn py_new(
        ephemeris_id: NaifId,
        orientation_id: NaifId,
        mu_km3_s2: Option<f64>,
        shape: Option<Ellipsoid>,
    ) -> Self {
        Self {
            ephemeris_id,
            orientation_id,
            mu_km3_s2,
            shape,
        }
    }

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self} (@{self:p})")
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> Result<bool, PyErr> {
        match op {
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            _ => Err(PyErr::new::<PyTypeError, _>(format!(
                "{op:?} not available"
            ))),
        }
    }

    /// Allows for pickling the object
    ///
    /// :rtype: typing.Tuple
    fn __getnewargs__(&self) -> Result<(NaifId, NaifId, Option<f64>, Option<Ellipsoid>), PyErr> {
        Ok((
            self.ephemeris_id,
            self.orientation_id,
            self.mu_km3_s2,
            self.shape,
        ))
    }

    /// :rtype: int
    #[getter]
    fn get_ephemeris_id(&self) -> PyResult<NaifId> {
        Ok(self.ephemeris_id)
    }
    /// :type ephemeris_id: int
    #[setter]
    fn set_ephemeris_id(&mut self, ephemeris_id: NaifId) -> PyResult<()> {
        self.ephemeris_id = ephemeris_id;
        Ok(())
    }
    /// :rtype: int
    #[getter]
    fn get_orientation_id(&self) -> PyResult<NaifId> {
        Ok(self.orientation_id)
    }
    /// :type orientation_id: int
    #[setter]
    fn set_orientation_id(&mut self, orientation_id: NaifId) -> PyResult<()> {
        self.orientation_id = orientation_id;
        Ok(())
    }
    /// :rtype: float
    #[getter]
    fn get_mu_km3_s2(&self) -> PyResult<Option<f64>> {
        Ok(self.mu_km3_s2)
    }
    /// :type mu_km3_s2: float
    #[setter]
    fn set_mu_km3_s2(&mut self, mu_km3_s2: Option<f64>) -> PyResult<()> {
        self.mu_km3_s2 = mu_km3_s2;
        Ok(())
    }
    /// :rtype: Ellipsoid
    #[getter]
    fn get_shape(&self) -> PyResult<Option<Ellipsoid>> {
        Ok(self.shape)
    }
    /// :type shape: Ellipsoid
    #[setter]
    fn set_shape(&mut self, shape: Option<Ellipsoid>) -> PyResult<()> {
        self.shape = shape;
        Ok(())
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Frame {
    /// Returns a copy of this Frame whose ephemeris ID is set to the provided ID
    ///
    /// :type new_ephem_id: int
    /// :rtype: Frame
    pub const fn with_ephem(&self, new_ephem_id: NaifId) -> Self {
        let mut me = *self;
        me.ephemeris_id = new_ephem_id;
        me
    }

    /// Returns a copy of this Frame whose orientation ID is set to the provided ID
    ///
    /// :type new_orient_id: int
    /// :rtype: Frame
    pub const fn with_orient(&self, new_orient_id: NaifId) -> Self {
        let mut me = *self;
        me.orientation_id = new_orient_id;
        me
    }

    /// Returns whether this is a celestial frame
    ///
    /// :rtype: bool
    pub const fn is_celestial(&self) -> bool {
        self.mu_km3_s2.is_some()
    }

    /// Returns whether this is a geodetic frame
    ///
    /// :rtype: bool
    pub const fn is_geodetic(&self) -> bool {
        self.mu_km3_s2.is_some() && self.shape.is_some()
    }

    /// Returns true if the ephemeris origin is equal to the provided ID
    ///
    /// :type other_id: int
    /// :rtype: bool
    pub const fn ephem_origin_id_match(&self, other_id: NaifId) -> bool {
        self.ephemeris_id == other_id
    }
    /// Returns true if the orientation origin is equal to the provided ID
    ///
    /// :type other_id: int
    /// :rtype: bool
    pub const fn orient_origin_id_match(&self, other_id: NaifId) -> bool {
        self.orientation_id == other_id
    }
    /// Returns true if the ephemeris origin is equal to the provided frame
    ///
    /// :type other: Frame
    /// :rtype: bool
    pub const fn ephem_origin_match(&self, other: Self) -> bool {
        self.ephem_origin_id_match(other.ephemeris_id)
    }
    /// Returns true if the orientation origin is equal to the provided frame
    ///
    /// :type other: Frame
    /// :rtype: bool
    pub const fn orient_origin_match(&self, other: Self) -> bool {
        self.orient_origin_id_match(other.orientation_id)
    }

    /// Removes the graviational parameter and the shape information from this frame.
    /// Use this to prevent astrodynamical computations.
    ///
    /// :rtype: None
    pub fn strip(&mut self) {
        self.mu_km3_s2 = None;
        self.shape = None;
    }

    /// Returns the gravitational parameters of this frame, if defined
    ///
    /// :rtype: float
    pub fn mu_km3_s2(&self) -> PhysicsResult<f64> {
        self.mu_km3_s2.ok_or(PhysicsError::MissingFrameData {
            action: "retrieving gravitational parameter",
            data: "mu_km3_s2",
            frame: self.into(),
        })
    }

    /// Returns a copy of this frame with the graviational parameter set to the new value.
    ///
    /// :type mu_km3_s2: float
    /// :rtype: Frame
    pub fn with_mu_km3_s2(&self, mu_km3_s2: f64) -> Self {
        let mut me = *self;
        me.mu_km3_s2 = Some(mu_km3_s2);
        me
    }

    /// Returns the mean equatorial radius in km, if defined
    ///
    /// :rtype: float
    pub fn mean_equatorial_radius_km(&self) -> PhysicsResult<f64> {
        Ok(self
            .shape
            .ok_or(PhysicsError::MissingFrameData {
                action: "retrieving mean equatorial radius",
                data: "shape",
                frame: self.into(),
            })?
            .mean_equatorial_radius_km())
    }

    /// Returns the semi major radius of the tri-axial ellipoid shape of this frame, if defined
    ///
    /// :rtype: float
    pub fn semi_major_radius_km(&self) -> PhysicsResult<f64> {
        Ok(self
            .shape
            .ok_or(PhysicsError::MissingFrameData {
                action: "retrieving semi major axis radius",
                data: "shape",
                frame: self.into(),
            })?
            .semi_major_equatorial_radius_km)
    }

    /// Returns the flattening ratio (unitless)
    ///
    /// :rtype: float
    pub fn flattening(&self) -> PhysicsResult<f64> {
        Ok(self
            .shape
            .ok_or(PhysicsError::MissingFrameData {
                action: "retrieving flattening ratio",
                data: "shape",
                frame: self.into(),
            })?
            .flattening())
    }

    /// Returns the polar radius in km, if defined
    ///
    /// :rtype: float
    pub fn polar_radius_km(&self) -> PhysicsResult<f64> {
        Ok(self
            .shape
            .ok_or(PhysicsError::MissingFrameData {
                action: "retrieving polar radius",
                data: "shape",
                frame: self.into(),
            })?
            .polar_radius_km)
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let body_name = match celestial_name_from_id(self.ephemeris_id) {
            Some(name) => name.to_string(),
            None => format!("body {}", self.ephemeris_id),
        };

        let orientation_name = match orientation_name_from_id(self.orientation_id) {
            Some(name) => name.to_string(),
            None => format!("orientation {}", self.orientation_id),
        };

        write!(f, "{body_name} {orientation_name}")?;
        if self.is_geodetic() {
            write!(
                f,
                " (μ = {} km^3/s^2, {})",
                self.mu_km3_s2.unwrap(),
                self.shape.unwrap()
            )?;
        } else if self.is_celestial() {
            write!(f, " (μ = {} km^3/s^2)", self.mu_km3_s2.unwrap())?;
        }
        Ok(())
    }
}

impl fmt::LowerExp for Frame {
    /// Only prints the ephemeris name
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match celestial_name_from_id(self.ephemeris_id) {
            Some(name) => write!(f, "{name}"),
            None => write!(f, "{}", self.ephemeris_id),
        }
    }
}

impl fmt::Octal for Frame {
    /// Only prints the orientation name
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match orientation_name_from_id(self.orientation_id) {
            Some(name) => write!(f, "{name}"),
            None => write!(f, "orientation {}", self.orientation_id),
        }
    }
}

impl fmt::LowerHex for Frame {
    /// Only prints the UID
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let uid: FrameUid = self.into();
        write!(f, "{uid}")
    }
}

#[cfg(test)]
mod frame_ut {
    use super::Frame;
    use crate::constants::frames::{EARTH_J2000, EME2000};

    #[test]
    fn format_frame() {
        assert_eq!(format!("{EME2000}"), "Earth J2000");
        assert_eq!(format!("{EME2000:x}"), "Earth J2000");
        assert_eq!(format!("{EME2000:o}"), "J2000");
        assert_eq!(format!("{EME2000:e}"), "Earth");
    }

    #[cfg(feature = "metaload")]
    #[test]
    fn dhall_serde() {
        let serialized = serde_dhall::serialize(&EME2000)
            .static_type_annotation()
            .to_string()
            .unwrap();
        assert_eq!(serialized, "{ ephemeris_id = +399, mu_km3_s2 = None Double, orientation_id = +1, shape = None { polar_radius_km : Double, semi_major_equatorial_radius_km : Double, semi_minor_equatorial_radius_km : Double } }");
        assert_eq!(
            serde_dhall::from_str(&serialized).parse::<Frame>().unwrap(),
            EME2000
        );
    }

    #[test]
    fn ccsds_name_to_frame() {
        assert_eq!(Frame::from_name("Earth", "ICRF").unwrap(), EARTH_J2000);
    }
}
