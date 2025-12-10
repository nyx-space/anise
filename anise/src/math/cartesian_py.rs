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
use ndarray::Array1;
use numpy::{PyReadonlyArray1, PyArray1};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyTuple, PyType};

fn from_npy_slice(pos_vel_na: &[f64], epoch: Epoch, frame: Frame) -> PyResult<CartesianState> {
    if pos_vel_na.len() != 6 {
        return Err(PyValueError::new_err(format!(
            "Expected a numpy array of size 6, got {}",
            pos_vel_na.len()
        )));
    }
    Ok(CartesianState::new(
        pos_vel_na[0],
        pos_vel_na[1],
        pos_vel_na[2],
        pos_vel_na[3],
        pos_vel_na[4],
        pos_vel_na[5],
        epoch,
        frame,
    ))
}

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

    /// Creates a new Cartesian state from a numpy array, an epoch, and a frame.
    ///
    /// **Units:** km, km, km, km/s, km/s, km/s
    #[classmethod]
    pub fn from_cartesian_npy(
        _cls: &Bound<'_, PyType>,
        pos_vel: PyReadonlyArray1<f64>,
        epoch: Epoch,
        frame: Frame,
    ) -> PyResult<Self> {
        from_npy_slice(pos_vel.as_slice()?, epoch, frame)
    }

    /// Creates a new Cartesian state in the provided frame at the provided Epoch (calls from_cartesian).
    ///
    /// **Units:** km, km, km, km/s, km/s, km/s
    #[allow(clippy::too_many_arguments)]
    #[new]
    pub fn py_new(args: &Bound<'_, PyTuple>) -> PyResult<Self> {
        if args.len() == 8 {
            let x_km: f64 = args.get_item(0)?.extract()?;
            let y_km: f64 = args.get_item(1)?.extract()?;
            let z_km: f64 = args.get_item(2)?.extract()?;
            let vx_km_s: f64 = args.get_item(3)?.extract()?;
            let vy_km_s: f64 = args.get_item(4)?.extract()?;
            let vz_km_s: f64 = args.get_item(5)?.extract()?;
            let epoch: Epoch = args.get_item(6)?.extract()?;
            let frame: Frame = args.get_item(7)?.extract()?;
            Ok(Self::new(
                x_km, y_km, z_km, vx_km_s, vy_km_s, vz_km_s, epoch, frame,
            ))
        } else if args.len() == 3 {
            let pos_vel: PyReadonlyArray1<f64> = args.get_item(0)?.extract()?;
            let epoch: Epoch = args.get_item(1)?.extract()?;
            let frame: Frame = args.get_item(2)?.extract()?;
            from_npy_slice(pos_vel.as_slice()?, epoch, frame)
        } else {
            Err(PyTypeError::new_err(
                "Orbit constructor takes either 6 floats, an epoch, and a frame, or a 6-element numpy array, an epoch, and a frame.",
            ))
        }
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

    /// Returns this state as a Cartesian vector of size 6 in [km, km, km, km/s, km/s, km/s]
    ///
    /// Note that the time is **not** returned in the vector.
    /// :rtype: numpy.array
    fn cartesian_pos_vel<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let data: Vec<f64> = self.to_cartesian_pos_vel().iter().copied().collect();

        let state = Array1::from_iter(data);

        Ok(PyArray1::<f64>::from_owned_array(py, state))
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
