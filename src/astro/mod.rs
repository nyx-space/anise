/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

/// Defines the aberration corrections to the state of the target body to account for one-way light time and stellar aberration.
/// **WARNING:** This enum is a placeholder until [https://github.com/anise-toolkit/anise.rs/issues/26] is implemented.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Aberration {
    None,
}

pub mod celestial_frame;
pub mod frame;
pub mod geodetic_frame;
pub mod orbit;
pub mod orbit_geodetic;

pub use celestial_frame::CelestialFrameTrait;
pub use frame::{Frame, FrameTrait};
pub use geodetic_frame::{GeodeticFrame, GeodeticFrameTrait};
