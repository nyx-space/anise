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
use crate::NaifId;
use crate::naif::daf::data_types::DataType;
use nalgebra::Matrix6;
use ndarray::Array2;
use numpy::{PyArray2, PyReadonlyArray2, PyUntypedArrayMethods};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;

fn interpolation_name(interpolation: DataType) -> String {
    match interpolation {
        DataType::Type9LagrangeUnequalStep => "LAGRANGE".to_string(),
        DataType::Type13HermiteUnequalStep | DataType::Type12HermiteEqualStep => {
            "HERMITE".to_string()
        }
        _ => unreachable!(),
    }
}

#[pymethods]
impl Ephemeris {
    /// :rtype: str
    fn get_object_id(&self) -> String {
        self.object_id.clone()
    }

    /// :type object_id: str
    fn set_object_id(&mut self, object_id: String) {
        self.object_id = object_id;
    }

    /// :rtype: str
    #[getter]
    fn get_interpolation(&self) -> Result<String, EphemerisError> {
        Ok(interpolation_name(self.interpolation()?))
    }

    /// :type interp: str
    #[setter(interpolation)]
    fn py_set_interpolation(&mut self, interp: &str) -> Result<(), PyErr> {
        match interp.to_lowercase().as_str() {
            "lagrange" => {
                self.set_interpolation(DataType::Type9LagrangeUnequalStep);
                Ok(())
            }
            "hermite" => {
                self.set_interpolation(DataType::Type13HermiteUnequalStep);
                Ok(())
            }
            _ => Err(PyValueError::new_err(
                "interpolation must be Hermite or Lagrange",
            )),
        }
    }

    /// :rtype: int
    #[getter]
    fn get_degree(&self) -> Result<usize, EphemerisError> {
        self.degree()
    }

    /// :type degree: int
    #[setter(degree)]
    fn py_set_degree(&mut self, degree: usize) -> Result<(), PyErr> {
        if degree < 1 {
            Err(PyValueError::new_err("degree must be strictly positive"))
        } else {
            self.set_degree(degree).map_err(PyErr::from)?;
            Ok(())
        }
    }

    #[new]
    fn py_new(orbit_list: Vec<Orbit>, object_id: String) -> Self {
        let mut ephem = Self::new(object_id);

        for orbit in orbit_list {
            ephem.insert(EphemerisRecord { orbit, covar: None });
        }

        ephem
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

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self}@{self:p}")
    }

    fn __iter__(slf: Bound<'_, Self>) -> PyResult<EphemerisIterator> {
        let mut records = slf
            .borrow()
            .segments
            .iter()
            .flat_map(|segment| segment.state_data.values().copied())
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.orbit.epoch);
        Ok(EphemerisIterator {
            records: records.into_iter(),
        })
    }

    /// Returns the interpolation method for the segment valid at the provided epoch.
    ///
    /// :type epoch: Epoch
    /// :rtype: str
    #[pyo3(name = "interpolation_at")]
    fn py_interpolation_at(&self, epoch: hifitime::Epoch) -> Result<String, EphemerisError> {
        Ok(interpolation_name(self.interpolation_at(epoch)?))
    }

    /// Returns the interpolation degree for the segment valid at the provided epoch.
    ///
    /// :type epoch: Epoch
    /// :rtype: int
    #[pyo3(name = "degree_at")]
    fn py_degree_at(&self, epoch: hifitime::Epoch) -> Result<usize, EphemerisError> {
        self.degree_at(epoch)
    }

    fn __reversed__(slf: Bound<'_, Self>) -> PyResult<EphemerisIterator> {
        let mut records = slf
            .borrow()
            .segments
            .iter()
            .flat_map(|segment| segment.state_data.values().copied())
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.orbit.epoch);
        records.reverse();
        Ok(EphemerisIterator {
            records: records.into_iter(),
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
        let state_dcm =
            Array2::from_shape_vec((6, 6), data).expect("6x6 matrix always has 36 elements");

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
    records: std::vec::IntoIter<EphemerisRecord>,
}

#[pymethods]
impl EphemerisIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<EphemerisRecord> {
        slf.records.next()
    }
}
