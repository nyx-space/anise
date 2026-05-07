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
use core::str::FromStr;
use der::{Decode, Encode, Reader, Writer};
use log::error;
use serde_derive::{Deserialize, Serialize};
use snafu::ResultExt;

#[cfg(feature = "metaload")]
use serde_dhall::{SimpleType, StaticType};

use crate::astro::PhysicsResult;
use crate::constants::celestial_objects::{
    celestial_name_from_id, id_from_celestial_name, SOLAR_SYSTEM_BARYCENTER,
};
use crate::constants::orientations::{id_from_orientation_name, orientation_name_from_id, J2000};
use crate::errors::{AlmanacError, EphemerisSnafu, OrientationSnafu, PhysicsError};
use crate::frames::DynamicFrame;
use crate::prelude::FrameUid;
use crate::structure::planetocentric::ellipsoid::Ellipsoid;
use crate::time::{Epoch, TimeScale, Unit};
use crate::NaifId;

#[cfg(feature = "python")]
use pyo3::exceptions::{PyTypeError, PyValueError};
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::pyclass::CompareOp;
#[cfg(feature = "python")]
use pyo3::types::{PyBytes, PyType};

/// A Frame uniquely defined by its ephemeris center and orientation. Refer to FrameDetail for frames combined with parameters.
///
/// :type ephemeris_id: int
/// :type orientation_id: int
/// :type force_inertial: bool
/// :type mu_km3_s2: float, optional
/// :type shape: Ellipsoid, optional
/// :type frozen_epoch: Epoch, optional
/// :rtype: Frame
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct Frame {
    pub ephemeris_id: NaifId,
    pub orientation_id: NaifId,
    /// If true, the DCM of this frame will always force its time derivative to zero, inertially fixing the frame.
    pub force_inertial: bool,
    /// Gravity parameter of this frame, only defined on celestial frames
    pub mu_km3_s2: Option<f64>,
    /// Shape of the geoid of this frame, only defined on geodetic frames
    pub shape: Option<Ellipsoid>,
    /// If set, the DCM will always be evaluated at the provided epoch, freezing it in time.
    pub frozen_epoch: Option<Epoch>,
}

impl Frame {
    /// Constructs a new frame given its ephemeris and orientations IDs, without defining anything else (so this is not a valid celestial frame, although the data could be populated later).
    pub const fn new(ephemeris_id: NaifId, orientation_id: NaifId) -> Self {
        Self {
            ephemeris_id,
            orientation_id,
            frozen_epoch: None,
            force_inertial: false,
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

    pub const fn new_inertial(ephemeris_id: NaifId, orientation_id: NaifId) -> Self {
        Self {
            ephemeris_id,
            orientation_id,
            frozen_epoch: None,
            force_inertial: true,
            mu_km3_s2: None,
            shape: None,
        }
    }
    /// Attempts to create a new frame from its center and reference frame name.

    /// This function is compatible with the CCSDS OEM names.
    pub fn from_name(center: &str, ref_frame: &str) -> Result<Self, AlmanacError> {
        let ephemeris_id = id_from_celestial_name(center).context(EphemerisSnafu {
            action: "converting center name to its ID",
        })?;

        let orientation_id = id_from_orientation_name(ref_frame).context(OrientationSnafu {
            action: "converting reference frame to its ID",
        })?;

        Ok(Self::new(ephemeris_id, orientation_id))
    }

    /// Define Ellipsoid shape and return a new [Frame]
    pub fn with_ellipsoid(mut self, shape: Ellipsoid) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Returns a copy of this frame with the graviational parameter and the shape information from this frame.
    /// Use this to prevent astrodynamical computations.
    ///
    /// :rtype: None
    pub fn stripped(mut self) -> Self {
        self.strip();
        self
    }

    /// Specifies what data is available in this structure.
    ///
    /// Returns:
    /// + Bit 0 is set if `mu_km3_s2` is available
    /// + Bit 1 is set if `shape` is available
    /// + Bit 2 is set if `frozen_eval_epoch` is available
    fn available_data(&self) -> u8 {
        let mut bits: u8 = 0;

        if self.mu_km3_s2.is_some() {
            bits |= 1 << 0;
        }
        if self.shape.is_some() {
            bits |= 1 << 1;
        }
        if self.frozen_epoch.is_some() {
            bits |= 1 << 2;
        }

        bits
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl Frame {
    /// Initializes a new [Frame] provided its ephemeris and orientation identifiers, and optionally its gravitational parameter (in km^3/s^2) and optionally its shape (cf. [Ellipsoid]).
    #[new]
    #[pyo3(signature=(ephemeris_id, orientation_id, mu_km3_s2=None, shape=None))]
    pub fn py_new(
        ephemeris_id: NaifId,
        orientation_id: NaifId,
        mu_km3_s2: Option<f64>,
        shape: Option<Ellipsoid>,
    ) -> Self {
        Self {
            ephemeris_id,
            orientation_id,
            force_inertial: false,
            mu_km3_s2,
            shape,
            frozen_epoch: None,
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
    fn get_ephemeris_id(&self) -> NaifId {
        self.ephemeris_id
    }
    /// :type ephemeris_id: int
    #[setter]
    fn set_ephemeris_id(&mut self, ephemeris_id: NaifId) {
        self.ephemeris_id = ephemeris_id;
    }
    /// :rtype: int
    #[getter]
    fn get_orientation_id(&self) -> NaifId {
        self.orientation_id
    }
    /// :type orientation_id: int
    #[setter]
    fn set_orientation_id(&mut self, orientation_id: NaifId) {
        self.orientation_id = orientation_id;
    }
    #[getter]
    fn get_force_inertial(&self) -> bool {
        self.force_inertial
    }
    /// :type ephemeris_id: int
    #[setter]
    fn set_force_inertial(&mut self, force_inertial: bool) {
        self.force_inertial = force_inertial;
    }
    /// :rtype: float
    #[getter]
    fn get_mu_km3_s2(&self) -> Option<f64> {
        self.mu_km3_s2
    }
    /// :type mu_km3_s2: float
    #[setter]
    fn set_mu_km3_s2(&mut self, mu_km3_s2: Option<f64>) {
        self.mu_km3_s2 = mu_km3_s2;
    }
    /// :rtype: Ellipsoid
    #[getter]
    fn get_shape(&self) -> Option<Ellipsoid> {
        self.shape
    }
    /// :type shape: Ellipsoid
    #[setter]
    fn set_shape(&mut self, shape: Option<Ellipsoid>) {
        self.shape = shape;
    }
    /// :rtype: Epoch
    #[getter]
    fn get_frozen_epoch(&self) -> Option<Epoch> {
        self.frozen_epoch
    }
    /// :type frozen_epoch: Epoch, optional
    #[setter]
    fn set_frozen_epoch(&mut self, frozen_epoch: Option<Epoch>) {
        self.frozen_epoch = frozen_epoch;
    }

    /// Decodes an ASN.1 DER encoded byte array into a Frame.
    ///
    /// :type data: bytes
    /// :rtype: Frame
    #[classmethod]
    pub fn from_asn1(_cls: &Bound<'_, PyType>, data: &[u8]) -> PyResult<Self> {
        match Self::from_der(data) {
            Ok(obj) => Ok(obj),
            Err(e) => Err(PyValueError::new_err(format!("ASN.1 decoding error: {e}"))),
        }
    }

    /// Encodes this Frame into an ASN.1 DER encoded byte array.
    ///
    /// :rtype: bytes
    pub fn to_asn1<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let mut buf = Vec::new();
        match self.encode_to_vec(&mut buf) {
            Ok(_) => Ok(PyBytes::new(py, &buf)),
            Err(e) => Err(PyValueError::new_err(format!("ASN.1 encoding error: {e}"))),
        }
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

    /// Returns true if this is a dynamic frame, e.g. Mean/True of Date/Epoch
    ///
    /// :rtype: bool
    pub fn is_dynamic(&self) -> bool {
        DynamicFrame::try_from(self.orientation_id as u32).is_ok()
    }
}

#[cfg(feature = "metaload")]
impl StaticType for Frame {
    fn static_type() -> serde_dhall::SimpleType {
        use std::collections::HashMap;
        let mut repr = HashMap::new();

        repr.insert("ephemeris_id".to_string(), SimpleType::Integer);
        repr.insert("orientation_id".to_string(), SimpleType::Integer);
        repr.insert("force_inertial".to_string(), SimpleType::Bool);
        repr.insert(
            "mu_km3_s2".to_string(),
            SimpleType::Optional(Box::new(SimpleType::Double)),
        );
        repr.insert(
            "shape".to_string(),
            SimpleType::Optional(Box::new(Ellipsoid::static_type())),
        );
        repr.insert(
            "frozen_epoch".to_string(),
            SimpleType::Optional(Box::new(SimpleType::Text)),
        );
        SimpleType::Record(repr)
    }
}

impl Encode for Frame {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let available_flags = self.available_data();

        self.ephemeris_id.encoded_len()?
            + self.orientation_id.encoded_len()?
            + self.force_inertial.encoded_len()?
            + available_flags.encoded_len()?
            + self.mu_km3_s2.encoded_len()?
            + self.shape.encoded_len()?
            + self.frozen_epoch.map(|e| e.to_string()).encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.ephemeris_id.encode(encoder)?;
        self.orientation_id.encode(encoder)?;
        self.force_inertial.encode(encoder)?;
        self.available_data().encode(encoder)?;
        self.mu_km3_s2.encode(encoder)?;
        self.shape.encode(encoder)?;
        self.frozen_epoch.map(|e| e.to_string()).encode(encoder)
    }
}

impl<'a> Decode<'a> for Frame {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let ephemeris_id: NaifId = decoder.decode()?;
        let orientation_id: NaifId = decoder.decode()?;
        let force_inertial: bool = decoder.decode()?;

        let data_flags: u8 = decoder.decode()?;

        let mu_km3_s2 = if data_flags & (1 << 0) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        let shape = if data_flags & (1 << 1) != 0 {
            Some(decoder.decode()?)
        } else {
            None
        };

        let frozen_epoch = if data_flags & (1 << 2) != 0 {
            let epoch_str: String = decoder.decode()?;
            match Epoch::from_str(&epoch_str) {
                Ok(epoch) => Some(epoch),
                Err(e) => {
                    error!("frozen epoch in frame kernel invalid: {e}");
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            ephemeris_id,
            orientation_id,
            force_inertial,
            mu_km3_s2,
            shape,
            frozen_epoch,
        })
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match celestial_name_from_id(self.ephemeris_id) {
            Some(name) => write!(f, "{name}")?,
            None => write!(f, "body {}", self.ephemeris_id)?,
        };

        let skip_orient_print = if self.force_inertial {
            write!(f, " inertial")?;
            self.ephemeris_id == self.orientation_id
        } else {
            false
        };

        if !skip_orient_print {
            write!(f, "{self:o}")?;
        }

        // Add the frozen epoch if applicable, trying to match on common frames.
        if let Some(frozen_epoch) = self.frozen_epoch {
            if (frozen_epoch - Epoch::from_et_duration(Unit::Second * 0)).abs() < Unit::Second * 1 {
                write!(f, " @ J2000")?;
            } else if (frozen_epoch - Epoch::from_gregorian_at_noon(2010, 1, 1, TimeScale::ET))
                .abs()
                < Unit::Second * 1
            {
                write!(f, " @ J2010")?;
            } else if (frozen_epoch - Epoch::from_gregorian_at_noon(2020, 1, 1, TimeScale::ET))
                .abs()
                < Unit::Second * 1
            {
                write!(f, " @ J2020")?;
            } else {
                write!(f, " @ {frozen_epoch}")?;
            }
        }

        if self.is_geodetic() {
            write!(
                f,
                " (μ = {} km^3/s^2, {})",
                self.mu_km3_s2.expect("mu must be set for geodetic frame"),
                self.shape.expect("shape must be set for geodetic frame")
            )?;
        } else if self.is_celestial() {
            write!(
                f,
                " (μ = {} km^3/s^2)",
                self.mu_km3_s2.expect("mu must be set for celestial frame")
            )?;
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
        if let Ok(dyn_frame) = DynamicFrame::try_from(self.orientation_id as u32) {
            let source_id = match dyn_frame {
                DynamicFrame::EarthMeanOfDate { .. }
                | DynamicFrame::EarthTrueOfDate { .. }
                | DynamicFrame::EarthTrueEquatorMeanEquinox { .. } => 399,
                DynamicFrame::BodyMeanOfDate { source_id }
                | DynamicFrame::BodyTrueOfDate { source_id } => source_id,
            };

            let mut name = if self.ephemeris_id == source_id {
                // Skip orientation
                dyn_frame.family().to_string()
            } else {
                format!("{dyn_frame}")
            };
            if self.frozen_epoch.is_some() {
                // Frame is now of Epoch not of Date
                name = name.replace("TOD", "TOE").replace("MOD", "MOE");
            }
            write!(f, " {name}")
        } else {
            match orientation_name_from_id(self.orientation_id) {
                Some(name) => write!(f, " {name}"),
                None => write!(f, " orientation {}", self.orientation_id),
            }
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
    use hifitime::Epoch;

    use super::Frame;
    use crate::constants::frames::EME2000;

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
        assert_eq!(
            serialized,
            "{ ephemeris_id = +399, force_inertial = False, frozen_epoch = None Text, mu_km3_s2 = None Double, orientation_id = +1, shape = None { polar_radius_km : Double, semi_major_equatorial_radius_km : Double, semi_minor_equatorial_radius_km : Double } }"
        );
        assert_eq!(
            serde_dhall::from_str(&serialized).parse::<Frame>().unwrap(),
            EME2000
        );
    }

    #[test]
    fn ccsds_name_to_frame() {
        use crate::constants::celestial_objects::EARTH;
        use crate::constants::orientations::ICRS;
        // "ICRF" now correctly resolves to bias-corrected GCRF (EARTH + ICRS),
        // not EARTH_J2000. See #686.
        assert_eq!(
            Frame::from_name("Earth", "ICRF").unwrap(),
            Frame::new(EARTH, ICRS)
        );
    }

    #[test]
    fn mars_centered_inertial() {
        use crate::constants::frames::MARS_INERTIAL_FRAME;
        assert!(MARS_INERTIAL_FRAME.force_inertial);
        assert_eq!(
            MARS_INERTIAL_FRAME.frozen_epoch.unwrap(),
            Epoch::from_et_seconds(0.0)
        );
        assert_eq!(format!("{MARS_INERTIAL_FRAME}"), "Mars inertial @ J2000");
    }
}
