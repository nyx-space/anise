/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

/**
 * This module only contains the serialization and deserialization components of ANISE.
 * All other computations are at a higher level module.
 */
pub mod dataset;
pub mod lookuptable;
pub mod metadata;
pub mod planetocentric;
pub mod semver;
pub mod spacecraft;

use self::semver::Semver;
/// The current version of ANISE
pub const ANISE_VERSION: Semver = Semver {
    major: 0,
    minor: 0,
    patch: 1,
};

/// The maximum number of trajectories that can be loaded in a single context
pub const MAX_TRAJECTORIES: usize = 31;
