/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::fmt::Display;

use crate::errors::PhysicsError;
use crate::frames::Frame;

use hifitime::{Duration, Epoch};

#[cfg(feature = "python")]
use pyo3::exceptions::PyTypeError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::pyclass::CompareOp;

pub mod utils;

pub(crate) mod aberration;
pub use aberration::Aberration;

pub(crate) mod occultation;
pub use occultation::Occultation;

pub mod orbit;
pub mod orbit_geodetic;

pub type PhysicsResult<T> = Result<T, PhysicsError>;

/// A structure that stores the result of Azimuth, Elevation, Range, Range rate calculation.
///
/// :type epoch: Epoch
/// :type azimuth_deg: float
/// :type elevation_deg: float
/// :type range_km: float
/// :type range_rate_km_s: float
/// :type obstructed_by: Frame, optional
/// :rtype: AzElRange
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct AzElRange {
    pub epoch: Epoch,
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
    pub range_km: f64,
    pub range_rate_km_s: f64,
    pub obstructed_by: Option<Frame>,
    pub light_time: Duration,
}

#[cfg_attr(feature = "python", pymethods)]
impl AzElRange {
    /// Returns false if the range is less than one millimeter, or any of the angles are NaN.
    ///
    /// :rtype: bool
    pub fn is_valid(&self) -> bool {
        self.azimuth_deg.is_finite() && self.elevation_deg.is_finite() && self.range_km > 1e-6
    }

    /// Returns whether there is an obstruction.
    ///
    /// :rtype: bool
    pub const fn is_obstructed(&self) -> bool {
        self.obstructed_by.is_some()
    }
}

#[cfg_attr(feature = "python", pymethods)]
#[cfg(feature = "python")]
impl AzElRange {
    /// Initializes a new AzElRange instance
    #[new]
    pub fn py_new(
        epoch: Epoch,
        azimuth_deg: f64,
        elevation_deg: f64,
        range_km: f64,
        range_rate_km_s: f64,
        obstructed_by: Option<Frame>,
    ) -> Self {
        use crate::constants::SPEED_OF_LIGHT_KM_S;
        use hifitime::TimeUnits;

        Self {
            epoch,
            azimuth_deg,
            elevation_deg,
            range_km,
            range_rate_km_s,
            obstructed_by,
            light_time: (range_km / SPEED_OF_LIGHT_KM_S).seconds(),
        }
    }

    /// :rtype: Epoch
    #[getter]
    fn get_epoch(&self) -> PyResult<Epoch> {
        Ok(self.epoch)
    }
    /// :type epoch: Epoch
    #[setter]
    fn set_epoch(&mut self, epoch: Epoch) -> PyResult<()> {
        self.epoch = epoch;
        Ok(())
    }

    /// :rtype: float
    #[getter]
    fn get_azimuth_deg(&self) -> PyResult<f64> {
        Ok(self.azimuth_deg)
    }
    /// :type azimuth_deg: f64
    #[setter]
    fn set_azimuth_deg(&mut self, azimuth_deg: f64) -> PyResult<()> {
        self.azimuth_deg = azimuth_deg;
        Ok(())
    }

    /// :rtype: float
    #[getter]
    fn get_elevation_deg(&self) -> PyResult<f64> {
        Ok(self.elevation_deg)
    }
    /// :type elevation_deg: f64
    #[setter]
    fn set_elevation_deg(&mut self, elevation_deg: f64) -> PyResult<()> {
        self.elevation_deg = elevation_deg;
        Ok(())
    }

    /// :rtype: float
    #[getter]
    fn get_range_km(&self) -> PyResult<f64> {
        Ok(self.range_km)
    }
    /// :type range_km: f64
    #[setter]
    fn set_range_km(&mut self, range_km: f64) -> PyResult<()> {
        use crate::constants::SPEED_OF_LIGHT_KM_S;
        use hifitime::TimeUnits;

        self.range_km = range_km;
        self.light_time = (range_km / SPEED_OF_LIGHT_KM_S).seconds();
        Ok(())
    }

    /// :rtype: float
    #[getter]
    fn get_range_rate_km_s(&self) -> PyResult<f64> {
        Ok(self.range_rate_km_s)
    }
    /// :type range_rate_km_s: f64
    #[setter]
    fn set_range_rate_km_s(&mut self, range_rate_km_s: f64) -> PyResult<()> {
        self.range_rate_km_s = range_rate_km_s;
        Ok(())
    }

    /// :rtype: Frame
    #[getter]
    fn get_obstructed_by(&self) -> PyResult<Option<Frame>> {
        Ok(self.obstructed_by)
    }
    /// :type obstructed_by: Frame
    #[setter]
    fn set_obstructed_by(&mut self, obstructed_by: Option<Frame>) -> PyResult<()> {
        self.obstructed_by = obstructed_by;
        Ok(())
    }

    /// :rtype: Duration
    #[getter]
    fn get_light_time(&self) -> PyResult<Duration> {
        Ok(self.light_time)
    }

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self} (@{self:p})")
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> Result<bool, PyErr> {
        match op {
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            _ => Err(PyErr::new::<PyTypeError, _>(format!(
                "{op:?} not available"
            ))),
        }
    }

    /// Allows for pickling the object
    ///
    /// :rtype: typing.Tuple
    fn __getnewargs__(&self) -> Result<(Epoch, f64, f64, f64, f64, Option<Frame>), PyErr> {
        Ok((
            self.epoch,
            self.azimuth_deg,
            self.elevation_deg,
            self.range_km,
            self.range_rate_km_s,
            self.obstructed_by,
        ))
    }
}

impl Display for AzElRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let obs = match self.obstructed_by {
            None => "none".to_string(),
            Some(frame) => format!("{frame:e}"),
        };
        write!(
            f,
            "{}: az.: {:.6} deg    el.: {:.6} deg    range: {:.6} km    range-rate: {:.6} km/s    obstruction: {}",
            self.epoch, self.azimuth_deg, self.elevation_deg, self.range_km, self.range_rate_km_s, obs
        )
    }
}
