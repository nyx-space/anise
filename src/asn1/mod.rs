/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
extern crate der;
extern crate hifitime;
pub mod common;
pub mod context;
pub mod ephemeris;
pub mod lookuptable;
pub mod metadata;
pub mod semver;
pub mod spline;
pub mod splinecoeffs;
pub mod splinekind;
pub mod time;

use self::semver::Semver;
/// The current version of ANISE
pub const ANISE_VERSION: Semver = Semver {
    major: 0,
    minor: 0,
    patch: 1,
};

/// The maximum number of trajectories that can be loaded in a single context
pub const MAX_TRAJECTORIES: usize = 256;
