/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use pyo3::prelude::*;

use crate::math::rotation::EulerParameter;

use super::{FovShape, Instrument};
use nalgebra::Vector3;
use ndarray::Array1;
use numpy::{PyArray1, PyReadonlyArray1};
use pyo3::exceptions::PyValueError;

fn to_numpy_array<'py>(v: Vector3<f64>, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
    let data: Vec<f64> = v.transpose().iter().copied().collect();

    let arr = Array1::from_shape_vec((3,), data).unwrap();

    PyArray1::<f64>::from_owned_array(py, arr)
}

// Helper to safely convert numpy array to Vector3
fn to_vec3<'py>(arr: PyReadonlyArray1<'py, f64>, name: &str) -> PyResult<Vector3<f64>> {
    let view = arr.as_array();
    if view.len() != 3 {
        return Err(PyValueError::new_err(format!(
            "{} must be a length-3 vector, got length {}",
            name,
            view.len()
        )));
    }
    // This is safe because we checked the length
    Ok(Vector3::new(view[0], view[1], view[2]))
}

#[pymethods]
impl Instrument {
    #[new]
    fn py_new<'py>(
        q_to_i: EulerParameter,
        offset: PyReadonlyArray1<'py, f64>,
        fov: FovShape,
    ) -> PyResult<Instrument> {
        let translation = to_vec3(offset, "mounting_translation")?;

        Ok(Self {
            q_to_i,
            offset_i: translation,
            fov,
        })
    }

    // getters
    #[getter]
    fn get_fov(&self) -> FovShape {
        self.fov
    }
    #[getter]
    fn get_q_to_i(&self) -> EulerParameter {
        self.q_to_i
    }
    #[getter]
    fn get_offset_i<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        to_numpy_array(self.offset_i, py)
    }
    // setters

    #[setter]
    fn set_offset_i<'py>(&mut self, offset_i: PyReadonlyArray1<'py, f64>) -> PyResult<()> {
        self.offset_i = to_vec3(offset_i, "mounting_translation")?;
        Ok(())
    }

    #[setter]
    fn set_q_to_i(&mut self, q_to_i: EulerParameter) {
        self.q_to_i = q_to_i;
    }

    #[setter]
    fn set_fov(&mut self, fov: FovShape) {
        self.fov = fov;
    }
}
