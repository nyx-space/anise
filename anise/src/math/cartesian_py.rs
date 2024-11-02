/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

// This file contains Python specific helper functions that don't fit anywhere else.

use super::cartesian::CartesianState;
use crate::prelude::Frame;
use hifitime::Epoch;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::PyType;

#[pymethods]
impl CartesianState {
    /// Creates a new Cartesian state in the provided frame at the provided Epoch.
    ///
    /// **Units:** km, km, km, km/s, km/s, km/s
    ///
    /// :type x_km: float
    /// :type y_km: float
    /// :type z_km: float
    /// :type vx_km_s: float
    /// :type vy_km_s: float
    /// :type vz_km_s: float
    /// :type epoch: Epoch
    /// :type frame: Frame
    /// :rtype: Orbit
    #[allow(clippy::too_many_arguments)]
    #[classmethod]
    pub fn from_cartesian(
        _cls: &Bound<'_, PyType>,
        x_km: f64,
        y_km: f64,
        z_km: f64,
        vx_km_s: f64,
        vy_km_s: f64,
        vz_km_s: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> Self {
        Self::new(x_km, y_km, z_km, vx_km_s, vy_km_s, vz_km_s, epoch, frame)
    }

    /// Creates a new Cartesian state in the provided frame at the provided Epoch (calls from_cartesian).
    ///
    /// **Units:** km, km, km, km/s, km/s, km/s
    #[allow(clippy::too_many_arguments)]
    #[new]
    pub fn py_new(
        x_km: f64,
        y_km: f64,
        z_km: f64,
        vx_km_s: f64,
        vy_km_s: f64,
        vz_km_s: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> Self {
        Self::new(x_km, y_km, z_km, vx_km_s, vy_km_s, vz_km_s, epoch, frame)
    }

    /// :rtype: float
    #[getter]
    fn get_x_km(&self) -> PyResult<f64> {
        Ok(self.radius_km[0])
    }
    /// :type x_km: float
    /// :rtype: None
    #[setter]
    fn set_x_km(&mut self, x_km: f64) -> PyResult<()> {
        self.radius_km[0] = x_km;
        Ok(())
    }

    /// :rtype: float
    #[getter]
    fn get_y_km(&self) -> PyResult<f64> {
        Ok(self.radius_km[1])
    }
    /// :type y_km: float
    /// :rtype: None
    #[setter]
    fn set_y_km(&mut self, y_km: f64) -> PyResult<()> {
        self.radius_km[1] = y_km;
        Ok(())
    }

    /// :rtype: float
    #[getter]
    fn get_z_km(&self) -> PyResult<f64> {
        Ok(self.radius_km[2])
    }
    /// :type z_km: float
    /// :rtype: None
    #[setter]
    fn set_z_km(&mut self, z_km: f64) -> PyResult<()> {
        self.radius_km[2] = z_km;
        Ok(())
    }

    /// :rtype: float
    #[getter]
    fn get_vx_km_s(&self) -> PyResult<f64> {
        Ok(self.velocity_km_s[0])
    }
    /// :type vx_km_s: float
    /// :rtype: None
    #[setter]
    fn set_vx_km_s(&mut self, vx_km_s: f64) -> PyResult<()> {
        self.velocity_km_s[0] = vx_km_s;
        Ok(())
    }

    /// :rtype: float
    #[getter]
    fn get_vy_km_s(&self) -> PyResult<f64> {
        Ok(self.velocity_km_s[1])
    }
    /// :type vy_km_s: float
    /// :rtype: None
    #[setter]
    fn set_vy_km_s(&mut self, vy_km_s: f64) -> PyResult<()> {
        self.velocity_km_s[1] = vy_km_s;
        Ok(())
    }

    /// :rtype: float
    #[getter]
    fn get_vz_km_s(&self) -> PyResult<f64> {
        Ok(self.velocity_km_s[2])
    }
    /// :type vz_km_s: float
    /// :rtype: None
    #[setter]
    fn set_vz_km(&mut self, vz_km_s: f64) -> PyResult<()> {
        self.velocity_km_s[2] = vz_km_s;
        Ok(())
    }

    /// :rtype: Epoch
    #[getter]
    fn get_epoch(&self) -> PyResult<Epoch> {
        Ok(self.epoch)
    }

    /// :rtype: Frame
    #[getter]
    fn get_frame(&self) -> PyResult<Frame> {
        Ok(self.frame)
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

    #[allow(clippy::type_complexity)]
    /// :rtype: typing.Tuple
    fn __getnewargs__(&self) -> Result<(f64, f64, f64, f64, f64, f64, Epoch, Frame), PyErr> {
        Ok((
            self.radius_km[0],
            self.radius_km[1],
            self.radius_km[2],
            self.velocity_km_s[0],
            self.velocity_km_s[1],
            self.velocity_km_s[2],
            self.epoch,
            self.frame,
        ))
    }
}
