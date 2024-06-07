/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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
use crate::{
    almanac::{MAX_PLANETARY_DATA, MAX_SPACECRAFT_DATA},
    math::rotation::Quaternion,
};

/// The current version of ANISE
pub const ANISE_VERSION: Semver = Semver {
    major: 0,
    minor: 4,
    patch: 0,
};

/// Spacecraft Data Set allow mapping an ID and/or name to spacecraft data, optionally including mass, drag, SRP, an inertia information
pub type SpacecraftDataSet = DataSet<SpacecraftData, MAX_SPACECRAFT_DATA>;
/// Planetary Data Set allow mapping an ID and/or name to planetary data, optionally including shape information and rotation information
pub type PlanetaryDataSet = DataSet<PlanetaryData, MAX_PLANETARY_DATA>;
/// Euler Parameter Data Set allow mapping an ID and/or name to a time invariant Quaternion
pub type EulerParameterDataSet = DataSet<Quaternion, MAX_PLANETARY_DATA>;
