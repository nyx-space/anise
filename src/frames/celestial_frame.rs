/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::HashType;

use super::{Frame, FrameTrait};
use core::fmt::{Display, Formatter};

/// Defines a Celestial Frame kind, which is a Frame that also defines a standard gravitational parameter
pub trait CelestialFrameTrait: FrameTrait {
    /// Returns the standard gravitational parameter of this frame (consider switching to UOM for this)
    fn mu_km3_s2(&self) -> f64;
}

/// A CelestialFrame is a frame whose equatorial and semi major radii are defined.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CelestialFrame {
    pub frame: Frame,
    pub mu_km3_s2: f64,
}

impl FrameTrait for CelestialFrame {
    fn ephemeris_hash(&self) -> HashType {
        self.frame.ephemeris_hash()
    }

    fn orientation_hash(&self) -> HashType {
        self.frame.orientation_hash()
    }
}

impl CelestialFrameTrait for CelestialFrame {
    fn mu_km3_s2(&self) -> f64 {
        self.mu_km3_s2
    }
}

impl Display for CelestialFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} (μ = {} km3/s)", self.frame, self.mu_km3_s2())
    }
}
