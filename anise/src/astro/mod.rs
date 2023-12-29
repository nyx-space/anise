/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::errors::PhysicsError;

#[cfg(feature = "python")]
use pyo3::prelude::*;
/// Defines the aberration corrections to the state of the target body to account for one-way light time and stellar aberration.
/// **WARNING:** This enum is a placeholder until [https://github.com/anise-toolkit/anise.rs/issues/26] is implemented.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub enum Aberration {
    #[pyo3(name = "NoAberration")]
    None,
}

pub mod orbit;
pub mod orbit_geodetic;

pub type PhysicsResult<T> = Result<T, PhysicsError>;
