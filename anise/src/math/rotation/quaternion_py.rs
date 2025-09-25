/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{EulerParameter, DCM};
use crate::NaifId;

use nalgebra::Vector3;
use ndarray::{Array1, Array2};
use numpy::{PyArray1, PyArray2, PyReadonlyArray1, PyUntypedArrayMethods};
use pyo3::basic::CompareOp;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyType;

#[pymethods]
impl EulerParameter {
    /// Initializes a new unit quaternion (or Euler Parameter), normalized to the shortest rotation.
    #[new]
    #[pyo3(signature=(w, x, y, z, from_id, to_id))]
    fn py_new(w: f64, x: f64, y: f64, z: f64, from_id: NaifId, to_id: NaifId) -> Self {
        Self::new(w, x, y, z, from_id, to_id)
    }

    /// Creates an Euler Parameter representing the short way rotation about the X (R1) axis
    ///
    /// :type angle_rad: float
    /// :type from_id: int
    /// :type to_id: int
    /// :rtype: Quaternion
    #[classmethod]
    #[pyo3(name="about_x", signature=(angle_rad, from_id, to_id))]
    fn py_about_x(
        _cls: &Bound<'_, PyType>,
        angle_rad: f64,
        from_id: NaifId,
        to_id: NaifId,
    ) -> Self {
        Self::about_x(angle_rad, from_id, to_id)
    }

    /// Creates an Euler Parameter representing the short way rotation about the Y (R2) axis
    ///
    /// :type angle_rad: float
    /// :type from_id: int
    /// :type to_id: int
    /// :rtype: Quaternion
    #[classmethod]
    #[pyo3(name="about_y", signature=(angle_rad, from_id, to_id))]
    fn py_about_y(
        _cls: &Bound<'_, PyType>,
        angle_rad: f64,
        from_id: NaifId,
        to_id: NaifId,
    ) -> Self {
        Self::about_y(angle_rad, from_id, to_id)
    }

    /// Creates an Euler Parameter representing the short way rotation about the Z (R3) axis
    ///
    /// :type angle_rad: float
    /// :type from_id: int
    /// :type to_id: int
    /// :rtype: Quaternion
    #[classmethod]
    #[pyo3(name="about_z", signature=(angle_rad, from_id, to_id))]
    fn py_about_z(
        _cls: &Bound<'_, PyType>,
        angle_rad: f64,
        from_id: NaifId,
        to_id: NaifId,
    ) -> Self {
        Self::about_z(angle_rad, from_id, to_id)
    }

    /// Returns the euler parameter derivative for this EP and the body angular velocity vector w
    /// dQ/dt = 1/2 [B(Q)] omega_rad_s
    ///
    /// :type omega_rad_s: np.array
    /// :rtype: Quaternion
    #[pyo3(name="derivative", signature=(omega_rad_s))]
    fn py_derivative<'py>(&self, omega_rad_s: PyReadonlyArray1<'py, f64>) -> PyResult<Self> {
        if omega_rad_s.shape() != [3] {
            return Err(PyErr::new::<PyTypeError, _>(
                "angular velocity vector omega must be 1x3",
            ));
        }

        let omega = Vector3::from_row_iterator(omega_rad_s.as_array().iter().copied());
        Ok(self.derivative(omega))
    }

    /// Returns the 4x3 matrix which relates the body angular velocity vector w to the derivative of this Euler Parameter.
    /// dQ/dt = 1/2 [B(Q)] w
    ///
    /// :rtype: np.array
    #[pyo3(name = "b_matrix")]
    fn py_b_matrix<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray2<f64>> {
        // Extract data from SMatrix (column-major order, hence the transpose)
        let b: Vec<f64> = self.b_matrix().transpose().iter().copied().collect();

        let b_mat = Array2::from_shape_vec((4, 3), b).unwrap();

        PyArray2::<f64>::from_owned_array(py, b_mat)
    }

    /// Returns the principal line of rotation (a unit vector) and the angle of rotation in radians
    ///
    /// :rtype: tuple
    #[pyo3(name = "uvec_angle_rad")]
    fn py_uvec_angle_rad<'py>(&self, py: Python<'py>) -> (Bound<'py, PyArray1<f64>>, f64) {
        let (uvec, angle) = self.uvec_angle_rad();

        let data: Vec<f64> = uvec.iter().copied().collect();

        let vec = Array1::from_shape_vec((3,), data).unwrap();
        (PyArray1::<f64>::from_owned_array(py, vec), angle)
    }

    /// Returns the principal rotation vector representation of this Euler Parameter
    /// :rtype: np.array
    #[pyo3(name = "prv")]
    fn py_prv<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        let data: Vec<f64> = self.prv().iter().copied().collect();

        let vec = Array1::from_shape_vec((3,), data).unwrap();
        PyArray1::<f64>::from_owned_array(py, vec)
    }

    /// Returns the data of this EP as a vector.
    /// :rtype: np.array
    #[pyo3(name = "as_vector")]
    fn py_as_vector<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        let data: Vec<f64> = self.as_vector().iter().copied().collect();

        let vec = Array1::from_shape_vec((3,), data).unwrap();
        PyArray1::<f64>::from_owned_array(py, vec)
    }

    /// Convert this quaterion to a DCM
    /// :rtype: DCM
    fn to_dcm(&self) -> DCM {
        DCM::from(*self)
    }

    /// :rtype: float
    #[getter]
    fn get_w(&self) -> f64 {
        self.w
    }
    /// :type w: float
    #[setter]
    fn set_w(&mut self, w: f64) {
        self.w = w;
    }
    /// :rtype: float
    #[getter]
    fn get_x(&self) -> f64 {
        self.x
    }
    /// :type x: float
    #[setter]
    fn set_x(&mut self, x: f64) {
        self.x = x;
    }
    /// :rtype: float
    #[getter]
    fn get_y(&self) -> f64 {
        self.y
    }
    /// :type y: float
    #[setter]
    fn set_y(&mut self, y: f64) {
        self.y = y;
    }
    /// :rtype: float
    #[getter]
    fn get_z(&self) -> f64 {
        self.z
    }
    /// :type z: float
    #[setter]
    fn set_z(&mut self, z: f64) {
        self.z = z;
    }
    /// :rtype: int
    #[getter]
    fn get_to_id(&self) -> NaifId {
        self.to
    }
    /// :type to_id: int
    #[setter]
    fn set_to_id(&mut self, to_id: NaifId) {
        self.to = to_id;
    }
    /// :rtype: int
    #[getter]
    fn get_from_id(&self) -> NaifId {
        self.from
    }
    /// :type from_id: int
    #[setter]
    fn set_from_id(&mut self, from_id: NaifId) {
        self.from = from_id;
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
