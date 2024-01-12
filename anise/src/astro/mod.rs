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

pub mod utils;

pub(crate) mod aberration;
pub use aberration::Aberration;

pub mod orbit;
pub mod orbit_geodetic;

pub type PhysicsResult<T> = Result<T, PhysicsError>;

/// A structure that stores the result of Azimuth, Elevation, and Range calculation.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(get_all, set_all))]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct AzElRange {
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
    pub range_km: f64,
}

#[cfg_attr(feature = "python", pymethods)]
impl AzElRange {
    /// Returns false if the range is less than one millimeter, or any of the angles are NaN.
    pub fn is_valid(&self) -> bool {
        self.azimuth_deg.is_finite() && self.elevation_deg.is_finite() && self.range_km > 1e-6
    }
}
