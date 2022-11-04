/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{celestial_frame::CelestialFrame, CelestialFrameTrait, FrameTrait};
use crate::astro::Frame;
use crate::HashType;
use core::fmt::{Display, Formatter};

/// Defines a Celestial Frame kind, which is a Frame that also defines a standard gravitational parameter
pub trait GeodeticFrameTrait: CelestialFrameTrait {
    /// Equatorial radius in kilometers
    fn equatorial_radius_km(&self) -> f64;
    /// Semi major radius in kilometers
    fn semi_major_radius_km(&self) -> f64;
    /// Flattening coefficient (unit less)
    fn flattening(&self) -> f64;
    /// Returns true if this is a body fixed frame
    fn is_body_fixed(&self) -> bool;
}

/// A GeodeticFrame is a Celestial Frame whose equatorial and semi major radii are defined.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GeodeticFrame {
    pub celestial_frame: CelestialFrame,
    pub equatorial_radius_km: f64,
    pub semi_major_radius_km: f64,
    pub flattening: f64,
    pub is_body_fixed: bool,
}

impl FrameTrait for GeodeticFrame {
    fn ephemeris_hash(&self) -> HashType {
        self.celestial_frame.ephemeris_hash()
    }

    fn orientation_hash(&self) -> HashType {
        self.celestial_frame.orientation_hash()
    }
}

impl CelestialFrameTrait for GeodeticFrame {
    fn mu_km3_s2(&self) -> f64 {
        self.celestial_frame.mu_km3_s2()
    }
}

impl GeodeticFrameTrait for GeodeticFrame {
    fn equatorial_radius_km(&self) -> f64 {
        self.equatorial_radius_km
    }

    fn semi_major_radius_km(&self) -> f64 {
        self.semi_major_radius_km
    }

    fn flattening(&self) -> f64 {
        self.flattening
    }

    fn is_body_fixed(&self) -> bool {
        self.is_body_fixed
    }
}

impl Display for GeodeticFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.celestial_frame.frame)?;
        write!(f, " (μ = {} km3/s", self.mu_km3_s2())?;

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
        self.celestial_frame.frame
    }
}
