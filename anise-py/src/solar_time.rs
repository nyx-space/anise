use pyo3::prelude::*;
use pyo3::types::PyFloat;
use hifitime_py::{Epoch as PyEpoch}; // Assuming hifitime_py::Epoch is the PyO3 wrapper for hifitime::Epoch
use anise::almanac::solar_time::{et2lst as rs_et2lst, lst2et as rs_lst2et};
use anise::errors::SpiceError; // To map errors
use anise::NaifId; // For lst2et body_id
use hifitime::Epoch; // For rs_lst2et reference_et_for_date_estimation

// Helper to convert SpiceError to PyErr
fn to_py_err(err: SpiceError) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(format!("{}", err))
}

#[pyfunction]
#[pyo3(name = "et2lst")]
fn py_et2lst(py_epoch: &PyEpoch, longitude_deg: f64) -> PyResult<f64> {
    let rust_epoch: Epoch = py_epoch.into(); // Convert from PyEpoch to hifitime::Epoch
    rs_et2lst(rust_epoch, longitude_deg).map_err(to_py_err)
}

#[pyfunction]
#[pyo3(name = "lst2et")]
fn py_lst2et(
    lst_seconds: f64,
    longitude_deg: f64,
    body_id: NaifId,
    py_reference_et: &PyEpoch,
) -> PyResult<PyEpoch> {
    let rust_reference_et: Epoch = py_reference_et.into();
    rs_lst2et(lst_seconds, longitude_deg, body_id, rust_reference_et)
        .map(|rust_epoch| PyEpoch::from(rust_epoch)) // Convert hifitime::Epoch back to PyEpoch
        .map_err(to_py_err)
}

pub(crate) fn solar_time_module(py: Python, parent_module: &PyModule) -> PyResult<()> {
    let child_module = PyModule::new(py, "solar_time")?;
    child_module.add_function(wrap_pyfunction!(py_et2lst, child_module)?)?;
    child_module.add_function(wrap_pyfunction!(py_lst2et, child_module)?)?;
    parent_module.add_submodule(child_module)?;
    Ok(())
}
