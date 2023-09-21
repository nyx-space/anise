/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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

use self::{
    dataset::DataSet, planetocentric::PlanetaryData, semver::Semver, spacecraft::SpacecraftData,
};
use crate::almanac::{MAX_PLANETARY_DATA, MAX_SPACECRAFT_DATA};

/// The current version of ANISE
pub const ANISE_VERSION: Semver = Semver {
    major: 0,
    minor: 0,
    patch: 1,
};

pub type SpacecraftDataSet<'a> = DataSet<'a, SpacecraftData<'a>, MAX_SPACECRAFT_DATA>;
pub type PlanetaryDataSet<'a> = DataSet<'a, PlanetaryData, MAX_PLANETARY_DATA>;
