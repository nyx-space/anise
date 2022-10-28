/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::FrameTrait;
use uom::si::f64::*;

pub type GravityParam = VolumeRate;

/// Defines a Celestial Frame kind, which is a Frame that also defines a standard gravitational parameter
pub trait CelestialFrameTrait: FrameTrait {
    /// Returns the standard gravitational parameter of this frame
    fn mu(&self) -> GravityParam;
}
