/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{Covariance, Ephemeris, EphemerisError, EphemerisRecord, LocalFrame, Orbit};
use crate::naif::daf::data_types::DataType;
use crate::naif::daf::DafDataType;
use crate::NaifId;
use nalgebra::Matrix6;
use ndarray::Array2;
use numpy::{PyArray2, PyReadonlyArray2, PyUntypedArrayMethods};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::collections::BTreeMap;

#[pymethods]
impl Ephemeris {
    /// :rtype: str
    #[getter]
    fn get_object_id(&self) -> String {
        self.object_id.clone()
    }

    /// :rtype: str
    #[getter]
    fn get_interpolation(&self) -> String {
        match self.interpolation {
            DataType::Type9LagrangeUnequalStep => "LAGRANGE".to_string(),
            DataType::Type13HermiteUnequalStep => "HERMITE".to_string(),
            _ => unreachable!(),
        }
    }

    /// :rtype: int
    #[getter]
    fn get_degree(&self) -> usize {
        self.degree
    }

    #[new]
    fn py_new(orbit_list: Vec<Orbit>, object_id: String) -> Self {
        let mut state_data = BTreeMap::new();

        for orbit in orbit_list {
            state_data.insert(orbit.epoch, EphemerisRecord { orbit, covar: None });
        }

        Self {
            state_data,
            object_id,
            interpolation: DafDataType::Type13HermiteUnequalStep,
            degree: 7,
        }
    }

    /// Initializes a new Ephemeris from a file path to CCSDS OEM file.
    ///
    /// :type path: str
    /// :rtype: Ephemeris
    #[classmethod]
    #[pyo3(name = "from_ccsds_oem_file", signature=(path))]
    fn py_from_ccsds_oem_file(_cls: Bound<'_, PyType>, path: &str) -> Result<Self, EphemerisError> {
        Self::from_ccsds_oem_file(path)
    }

    /// Initializes a new Ephemeris from a file path to Ansys STK .e file.
    ///
    /// :type path: str
    /// :rtype: Ephemeris
    #[classmethod]
    #[pyo3(name = "from_stk_e_file", signature=(path))]
    fn py_from_stk_e_file(_cls: Bound<'_, PyType>, path: &str) -> Result<Self, EphemerisError> {
        Self::from_stk_e_file(path)
    }

    /// Exports this Ephemeris to CCSDS OEM at the provided path, optionally specifying an originator and/or an object name
    ///
    /// :type path: str
    /// :type originator: str, optional
    /// :type object_name: str, optional
    /// :rtype: None
    #[pyo3(name = "write_ccsds_oem", signature=(path, originator=None, object_name=None))]
    fn py_write_ccsds_oem(
        &self,
        path: &str,
        originator: Option<String>,
        object_name: Option<String>,
    ) -> Result<(), EphemerisError> {
        self.write_ccsds_oem(path, originator, object_name)
    }

    /// Converts this ephemeris to SPICE BSP/SPK file in the provided data type, saved to the provided output_fname.
    ///
    /// :type naif_id: int
    /// :type output_fname: str
    /// :type data_type: DataType
    /// :rtype: None
    #[pyo3(name = "write_spice_bsp")]
    pub fn py_write_spice_bsp(
        &self,
        naif_id: NaifId,
        output_fname: &str,
        data_type: Option<DataType>,
    ) -> Result<(), EphemerisError> {
        self.write_spice_bsp(naif_id, output_fname, data_type)
    }

    /// Returns the number of states
    ///
    /// :rtype: int
    fn len(&self) -> usize {
        self.state_data.len()
    }

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self}@{self:p}")
    }

    fn __iter__(slf: Bound<'_, Self>) -> PyResult<EphemerisIterator> {
        let keys: Vec<hifitime::Epoch> = slf.borrow().state_data.keys().copied().collect();
        Ok(EphemerisIterator {
            ephem: slf.into(),
            keys: keys.into_iter(),
        })
    }

    fn __reversed__(slf: Bound<'_, Self>) -> PyResult<EphemerisIterator> {
        let keys: Vec<hifitime::Epoch> = slf.borrow().state_data.keys().rev().copied().collect();
        Ok(EphemerisIterator {
            ephem: slf.into(),
            keys: keys.into_iter(),
        })
    }
}

#[pymethods]
impl Covariance {
    #[new]
    fn py_new<'py>(covar: PyReadonlyArray2<'py, f64>, local_frame: LocalFrame) -> PyResult<Self> {
        if covar.shape() != [6, 6] {
            return Err(PyErr::new::<PyTypeError, _>("covariance must be 6x6"));
        }

        let matrix = Matrix6::from_row_iterator(covar.as_array().iter().copied());

        Ok(Self {
            matrix,
            local_frame,
        })
    }
    /// Returns the 6x6 DCM to rotate a state. If the time derivative of this DCM is defined, this 6x6 accounts for the transport theorem.
    /// Warning: you MUST manually install numpy to call this function.
    /// :rtype: numpy.ndarray
    #[getter]
    fn get_matrix<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        // Extract data from SMatrix (column-major order, hence the transpose)
        let data: Vec<f64> = self.matrix.transpose().iter().copied().collect();

        // Create an ndarray Array2 (row-major order)
        let state_dcm = Array2::from_shape_vec((6, 6), data).unwrap();

        let pt_state_dcm = PyArray2::<f64>::from_owned_array(py, state_dcm);

        Ok(pt_state_dcm)
    }

    /// :rtype: str
    fn __str__(&self) -> String {
        format!("{self}")
    }

    /// :rtype: str
    fn __repr__(&self) -> String {
        format!("{self}@{self:p}")
    }
}

#[pyclass]
struct EphemerisIterator {
    ephem: Py<Ephemeris>,
    keys: std::vec::IntoIter<hifitime::Epoch>,
}

#[pymethods]
impl EphemerisIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python<'_>) -> Option<EphemerisRecord> {
        if let Some(key) = slf.keys.next() {
            slf.ephem.borrow(py).state_data.get(&key).copied()
        } else {
            None
        }
    }
}
