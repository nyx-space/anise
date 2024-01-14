/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::fmt::Display;

use crate::errors::PhysicsError;

use hifitime::Epoch;

#[cfg(feature = "python")]
use pyo3::exceptions::PyTypeError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::pyclass::CompareOp;

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
    pub epoch: Epoch,
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

    /// Initializes a new AzElRange instance
    #[cfg(feature = "python")]
    #[new]
    pub fn py_new(epoch: Epoch, azimuth_deg: f64, elevation_deg: f64, range_km: f64) -> Self {
        Self {
            epoch,
            azimuth_deg,
            elevation_deg,
            range_km,
        }
    }

    #[cfg(feature = "python")]
    fn __str__(&self) -> String {
        format!("{self}")
    }

    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        format!("{self} (@{self:p})")
    }

    #[cfg(feature = "python")]
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> Result<bool, PyErr> {
        match op {
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            _ => Err(PyErr::new::<PyTypeError, _>(format!(
                "{op:?} not available"
            ))),
        }
    }
}

impl Display for AzElRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: az.: {:.6} deg    el.: {:.6} deg    range: {:.6} km",
            self.epoch, self.azimuth_deg, self.elevation_deg, self.range_km
        )
    }
}
