/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::fmt::{Display, Formatter};

use crate::constants::celestial_bodies::hash_celestial_name;
use crate::constants::orientations::hash_orientation_name;

/// A Frame uniquely defined by its ephemeris center and orientation.
///
/// # Notes
/// 1. If a frame defines a gravity parameter μ (mu), then it it considered a celestial object.
/// 2. If a frame defines an equatorial radius, a semi major radius, and a flattening ratio, then
/// is considered a geoid.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Frame {
    pub ephemeris_hash: u32,
    pub orientation_hash: u32,
    pub mu_km3s: Option<f64>,
    pub equatorial_radius: Option<f64>,
    pub semi_major_radius: Option<f64>,
    pub flattening: Option<f64>,
}

impl Frame {
    /// Returns whether this frame is a celestial frame
    pub const fn is_celestial(&self) -> bool {
        self.mu_km3s.is_some()
    }

    /// Returns whether this frame is a geoid frame
    pub const fn is_geoid(&self) -> bool {
        self.is_celestial()
            && self.equatorial_radius.is_some()
            && self.semi_major_radius.is_some()
            && self.flattening.is_some()
    }

    pub const fn ephem_origin_match(&self, other: u32) -> bool {
        self.ephemeris_hash == other
    }

    pub const fn orient_origin_match(&self, other: u32) -> bool {
        self.orientation_hash == other
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let body_name = match hash_celestial_name(self.ephemeris_hash) {
            Some(name) => name.to_string(),
            None => format!("body {}", self.ephemeris_hash),
        };

        let orientation_name = match hash_orientation_name(self.orientation_hash) {
            Some(name) => name.to_string(),
            None => format!("orientation {}", self.orientation_hash),
        };

        write!(f, "{body_name} {orientation_name}")?;
        if self.is_celestial() {
            write!(f, " (μ = {} km3/s", self.mu_km3s.unwrap())?;
        }
        if self.is_geoid() {
            write!(
                f,
                ", eq. radius = {} km, sm axis = {} km, f = {})",
                self.equatorial_radius.unwrap(),
                self.semi_major_radius.unwrap(),
                self.flattening.unwrap()
            )
        } else {
            write!(f, ")")
        }
    }
}
