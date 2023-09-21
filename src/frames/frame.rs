/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt;
use core::fmt::Debug;

use crate::constants::celestial_objects::celestial_name_from_id;
use crate::constants::orientations::{orientation_name_from_id, J2000};
use crate::errors::PhysicsError;
use crate::prelude::FrameUid;
use crate::structure::planetocentric::ellipsoid::Ellipsoid;
use crate::NaifId;

/// A Frame uniquely defined by its ephemeris center and orientation. Refer to FrameDetail for frames combined with parameters.
#[derive(Copy, Clone, Debug, PartialEq)]
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
    pub const fn from_ephem_orient(ephemeris_id: NaifId, orientation_id: NaifId) -> Self {
        Self {
            ephemeris_id,
            orientation_id,
            mu_km3_s2: None,
            shape: None,
        }
    }

    pub const fn from_ephem_j2000(ephemeris_id: NaifId) -> Self {
        Self::from_ephem_orient(ephemeris_id, J2000)
    }

    /// Returns a copy of this Frame whose ephemeris ID is set to the provided ID
    pub const fn with_ephem(&self, new_ephem_id: NaifId) -> Self {
        let mut me = *self;
        me.ephemeris_id = new_ephem_id;
        me
    }

    /// Returns a copy of this Frame whose orientation ID is set to the provided ID
    pub const fn with_orient(&self, new_orient_id: NaifId) -> Self {
        let mut me = *self;
        me.orientation_id = new_orient_id;
        me
    }

    /// Returns whether this is a celestial frame
    pub const fn is_celestial(&self) -> bool {
        self.mu_km3_s2.is_some()
    }

    /// Returns whether this is a geodetic frame
    pub const fn is_geodetic(&self) -> bool {
        self.mu_km3_s2.is_some() && self.shape.is_some()
    }

    /// Returns true if the ephemeris origin is equal to the provided ID
    pub fn ephem_origin_id_match(&self, other_id: NaifId) -> bool {
        self.ephemeris_id == other_id
    }
    /// Returns true if the orientation origin is equal to the provided ID
    pub fn orient_origin_id_match(&self, other_id: NaifId) -> bool {
        self.orientation_id == other_id
    }
    /// Returns true if the ephemeris origin is equal to the provided frame
    pub fn ephem_origin_match(&self, other: Self) -> bool {
        self.ephem_origin_id_match(other.ephemeris_id)
    }
    /// Returns true if the orientation origin is equal to the provided frame
    pub fn orient_origin_match(&self, other: Self) -> bool {
        self.orient_origin_id_match(other.orientation_id)
    }

    /// Returns the gravitational parameters of this frame, if defined
    pub fn mu_km3_s2(&self) -> Result<f64, PhysicsError> {
        self.mu_km3_s2.ok_or(PhysicsError::MissingFrameData {
            action: "retrieving mean equatorial radius",
            data: "shape",
            frame: self.into(),
        })
    }

    /// Returns the mean equatorial radius in km, if defined
    pub fn mean_equatorial_radius_km(&self) -> Result<f64, PhysicsError> {
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
    pub fn semi_major_radius_km(&self) -> Result<f64, PhysicsError> {
        Ok(self
            .shape
            .ok_or(PhysicsError::MissingFrameData {
                action: "retrieving semi major axis radius",
                data: "shape",
                frame: self.into(),
            })?
            .semi_major_equatorial_radius_km)
    }

    pub fn flattening(&self) -> Result<f64, PhysicsError> {
        Ok(self
            .shape
            .ok_or(PhysicsError::MissingFrameData {
                action: "retrieving flattening ratio",
                data: "shape",
                frame: self.into(),
            })?
            .flattening())
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
                " (μ = {} km3/s, {})",
                self.mu_km3_s2.unwrap(),
                self.shape.unwrap()
            )?;
        } else if self.is_celestial() {
            write!(f, " (μ = {} km3/s)", self.mu_km3_s2.unwrap())?;
        }
        Ok(())
    }
}

impl fmt::LowerExp for Frame {
    /// Only prints the ephemeris name
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let body_name = match celestial_name_from_id(self.ephemeris_id) {
            Some(name) => name.to_string(),
            None => format!("{}", self.ephemeris_id),
        };
        write!(f, "{body_name}")
    }
}

impl fmt::Octal for Frame {
    /// Only prints the orientation name
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let orientation_name = match orientation_name_from_id(self.orientation_id) {
            Some(name) => name.to_string(),
            None => format!("orientation {}", self.orientation_id),
        };

        write!(f, "{orientation_name}")
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
    use crate::constants::frames::EME2000;

    #[test]
    fn format_frame() {
        assert_eq!(format!("{EME2000}"), "Earth J2000");
        assert_eq!(format!("{EME2000:x}"), "Earth J2000");
        assert_eq!(format!("{EME2000:o}"), "J2000");
        assert_eq!(format!("{EME2000:e}"), "Earth");
    }
}
