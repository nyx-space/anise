/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{celestial_frame::GravityParam, CelestialFrameTrait, FrameTrait};
use crate::astro::Frame;
use crate::HashType;
use core::fmt::{Display, Formatter};
use uom::si::{f64::*, length::kilometer, volume_rate::cubic_kilometer_per_second};

/// Defines a Celestial Frame kind, which is a Frame that also defines a standard gravitational parameter
trait GeodeticFrameTrait: CelestialFrameTrait {
    /// Equatorial radius in kilometers
    fn equatorial_radius(&self) -> Length;
    /// Semi major radius in kilometers
    fn semi_major_radius(&self) -> Length;
    /// Flattening coefficient (unit less)
    fn flattening(&self) -> f64;
}

/// A Frame uniquely defined by its ephemeris center and orientation.
///
/// # Notes
/// 1. If a frame defines a gravity parameter μ (mu), then it it considered a celestial object.
/// 2. If a frame defines an equatorial radius, a semi major radius, and a flattening ratio, then
/// is considered a geoid.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GeodeticFrame {
    pub frame: Frame,
    pub mu_km3s: f64,
    pub equatorial_radius_km: f64,
    pub semi_major_radius_km: f64,
    pub flattening: f64,
}

impl GeodeticFrame {
    pub const fn ephem_origin_hash_match(&self, other_hash: HashType) -> bool {
        self.frame.ephem_origin_hash_match(other_hash)
    }

    pub const fn orient_origin_hash_match(&self, other_hash: HashType) -> bool {
        self.frame.orient_origin_hash_match(other_hash)
    }

    pub const fn ephem_origin_match(&self, other: Self) -> bool {
        self.frame.ephem_origin_match(other.frame)
    }

    pub const fn orient_origin_match(&self, other: Self) -> bool {
        self.frame.orient_origin_match(other.frame)
    }
}

impl FrameTrait for GeodeticFrame {
    fn ephemeris_hash(&self) -> HashType {
        self.frame.ephemeris_hash
    }

    fn orientation_hash(&self) -> HashType {
        self.frame.orientation_hash
    }
}

impl CelestialFrameTrait for GeodeticFrame {
    fn mu(&self) -> GravityParam {
        GravityParam::new::<cubic_kilometer_per_second>(self.mu_km3s)
    }
}

impl GeodeticFrameTrait for GeodeticFrame {
    fn equatorial_radius(&self) -> Length {
        Length::new::<kilometer>(self.equatorial_radius_km)
    }

    fn semi_major_radius(&self) -> Length {
        Length::new::<kilometer>(self.semi_major_radius_km)
    }

    fn flattening(&self) -> f64 {
        todo!()
    }
}

impl Display for GeodeticFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.frame)?;
        write!(f, " (μ = {} km3/s", self.mu_km3s)?;

        write!(
            f,
            ", eq. radius = {} km, sm axis = {} km, f = {}",
            self.equatorial_radius_km, self.semi_major_radius_km, self.flattening
        )?;

        write!(f, ")")
    }
}

#[allow(clippy::from_over_into)]
impl Into<Frame> for GeodeticFrame {
    /// Lossy operation to convert FrameDetail into a Frame.
    ///
    /// This will cause the LOSS of the constants stored in the frame detail.
    fn into(self) -> Frame {
        self.frame
    }
}
