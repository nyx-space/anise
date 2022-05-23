/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

/// A Frame uniquely defined by its ephemeris center and orientation.
///
/// # Notes
/// 1. If a frame defines a gravity parameter μ (mu), then it it considered a celestial object.
/// 2. If a frame defines an equatorial radius, a semi major radius, and a flattening ratio, then
/// is considered a geoid.
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
    pub fn is_celestial(&self) -> bool {
        self.mu_km3s.is_some()
    }

    /// Returns whether this frame is a geoid frame
    pub fn is_geoid(&self) -> bool {
        self.is_celestial()
            && self.equatorial_radius.is_some()
            && self.semi_major_radius.is_some()
            && self.flattening.is_some()
    }
}
