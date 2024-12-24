/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::NaifId;

use super::DCM;

use nalgebra::Matrix3;
use ndarray::Array2;
use numpy::{PyArray2, PyReadonlyArray2, PyUntypedArrayMethods};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyType;

#[pymethods]
impl DCM {
    #[new]
    #[pyo3(signature=(np_rot_mat, from_id, to_id, np_rot_mat_dt=None))]
    pub fn py_new<'py>(
        np_rot_mat: PyReadonlyArray2<'py, f64>,
        from_id: NaifId,
        to_id: NaifId,
        np_rot_mat_dt: Option<PyReadonlyArray2<'py, f64>>,
    ) -> PyResult<Self> {
        if np_rot_mat.shape() != [3, 3] {
            return Err(PyErr::new::<PyTypeError, _>("rotation matrix must be 3x3"));
        }

        let rot_mat = Matrix3::from_row_iterator(np_rot_mat.as_array().iter().copied());

        let rot_mat_dt = if let Some(np_rot_mat_dt) = np_rot_mat_dt {
            if np_rot_mat_dt.shape() != [3, 3] {
                return Err(PyErr::new::<PyTypeError, _>(
                    "rotation matrix time derivative must be 3x3",
                ));
            }
            Some(Matrix3::from_row_iterator(
                np_rot_mat_dt.as_array().iter().copied(),
            ))
        } else {
            None
        };

        Ok(Self {
            rot_mat,
            rot_mat_dt,
            from: from_id,
            to: to_id,
        })
    }

    /// Returns a rotation matrix for a rotation about the X axis.
    ///
    /// Source: `euler1` function from Baslisk
    ///
    /// :type angle_rad: float
    /// :type from_id: int
    /// :type to_id: int
    /// :rtype: DCM
    #[classmethod]
    pub fn from_r1(
        _cls: &Bound<'_, PyType>,
        angle_rad: f64,
        from_id: NaifId,
        to_id: NaifId,
    ) -> Self {
        Self::r1(angle_rad, from_id, to_id)
    }

    /// Returns a rotation matrix for a rotation about the Y axis.
    ///
    /// Source: `euler2` function from Basilisk
    ///
    /// :type angle_rad: float
    /// :type from_id: int
    /// :type to_id: int
    /// :rtype: DCM
    #[classmethod]
    pub fn from_r2(
        _cls: &Bound<'_, PyType>,
        angle_rad: f64,
        from_id: NaifId,
        to_id: NaifId,
    ) -> Self {
        Self::r2(angle_rad, from_id, to_id)
    }

    /// Returns a rotation matrix for a rotation about the Z axis.
    ///
    /// Source: `euler3` function from Basilisk
    ///
    /// :type angle_rad: float
    /// :type from_id: int
    /// :type to_id: int
    /// :rtype: DCM
    #[classmethod]
    pub fn from_r3(
        _cls: &Bound<'_, PyType>,
        angle_rad: f64,
        from_id: NaifId,
        to_id: NaifId,
    ) -> Self {
        Self::r3(angle_rad, from_id, to_id)
    }

    /// Builds an identity rotation.
    ///
    /// :type from_id: int
    /// :type to_id: int
    /// :rtype: DCM
    #[classmethod]
    pub fn from_identity(_cls: &Bound<'_, PyType>, from_id: i32, to_id: i32) -> Self {
        Self::identity(from_id, to_id)
    }

    /// :rtype: numpy.array
    #[getter]
    fn get_rot_mat<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        // Extract data from SMatrix (column-major order, hence the transpose)
        let data: Vec<f64> = self.rot_mat.transpose().iter().copied().collect();

        // Create an ndarray Array2 (row-major order)
        let rot_mat = Array2::from_shape_vec((3, 3), data).unwrap();

        let py_rot_mat = PyArray2::<f64>::from_owned_array(py, rot_mat);

        Ok(py_rot_mat)
    }

    /// :rtype: numpy.array
    #[getter]
    fn get_rot_mat_dt<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyArray2<f64>>>> {
        if self.rot_mat_dt.is_none() {
            return Ok(None);
        }
        // Extract data from SMatrix (column-major order, hence the transpose)
        let data: Vec<f64> = self
            .rot_mat_dt
            .unwrap()
            .transpose()
            .iter()
            .copied()
            .collect();

        // Create an ndarray Array2 (row-major order)
        let rot_mat_dt = Array2::from_shape_vec((3, 3), data).unwrap();

        let py_rot_mat_dt = PyArray2::<f64>::from_owned_array(py, rot_mat_dt);

        Ok(Some(py_rot_mat_dt))
    }

    /// :rtype: int
    #[getter]
    fn get_from_id(&self) -> PyResult<NaifId> {
        Ok(self.from)
    }

    /// :rtype: int
    #[getter]
    fn get_to_id(&self) -> PyResult<NaifId> {
        Ok(self.to)
    }

    /// Returns the 6x6 DCM to rotate a state. If the time derivative of this DCM is defined, this 6x6 accounts for the transport theorem.
    /// Warning: you MUST manually install numpy to call this function.
    /// :rtype: numpy.array
    fn get_state_dcm<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        // Extract data from SMatrix (column-major order, hence the transpose)
        let data: Vec<f64> = self.state_dcm().transpose().iter().copied().collect();

        // Create an ndarray Array2 (row-major order)
        let state_dcm = Array2::from_shape_vec((6, 6), data).unwrap();

        let pt_state_dcm = PyArray2::<f64>::from_owned_array(py, state_dcm);

        Ok(pt_state_dcm)
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
}
