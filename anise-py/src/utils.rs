/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::path::PathBuf;

use anise::naif::kpl::parser::{convert_fk as convert_fk_rs, convert_tpc as convert_tpc_rs};
use anise::structure::dataset::DataSetError;
use anise::structure::planetocentric::ellipsoid::Ellipsoid;
use pyo3::prelude::*;

pub(crate) fn register_utils(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let sm = PyModule::new(py, "_anise.utils")?;
    sm.add_class::<Ellipsoid>()?;
    sm.add_function(wrap_pyfunction!(convert_fk, sm)?)?;
    sm.add_function(wrap_pyfunction!(convert_tpc, sm)?)?;
    parent_module.add_submodule(sm)?;
    Ok(())
}

/// Converts a KPL/FK file, that defines frame constants like fixed rotations, and frame name to ID mappings into the EulerParameterDataSet equivalent ANISE file.
/// KPL/FK files must be converted into "PCA" (Planetary Constant ANISE) files before being loaded into ANISE.
#[pyfunction]
fn convert_fk(
    fk_file_path: String,
    anise_output_path: String,
    show_comments: Option<bool>,
    overwrite: Option<bool>,
) -> Result<(), DataSetError> {
    let dataset = convert_fk_rs(fk_file_path, show_comments.unwrap_or(false))?;

    dataset.save_as(
        &PathBuf::from(anise_output_path),
        overwrite.unwrap_or(false),
    )?;

    Ok(())
}

/// Converts two KPL/TPC files, one defining the planetary constants as text, and the other defining the gravity parameters, into the PlanetaryDataSet equivalent ANISE file.
/// KPL/TPC files must be converted into "PCA" (Planetary Constant ANISE) files before being loaded into ANISE.
#[pyfunction]
fn convert_tpc(
    pck_file_path: String,
    gm_file_path: String,
    anise_output_path: String,
    overwrite: Option<bool>,
) -> Result<(), DataSetError> {
    let dataset = convert_tpc_rs(pck_file_path, gm_file_path)?;

    dataset.save_as(
        &PathBuf::from(anise_output_path),
        overwrite.unwrap_or(false),
    )?;

    Ok(())
}
