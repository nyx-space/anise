/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::astro::AzElRange;
use anise::structure::planetocentric::ellipsoid::Ellipsoid;
use pyo3::prelude::*;
use pyo3::py_run;

use anise::astro::orbit::Orbit;
use anise::frames::Frame;

use super::constants::register_constants;

pub(crate) fn register_astro(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let sm = PyModule::new(py, "_anise.astro")?;
    sm.add_class::<Ellipsoid>()?;
    sm.add_class::<Frame>()?;
    sm.add_class::<Orbit>()?;
    sm.add_class::<AzElRange>()?;

    register_constants(py, sm)?;

    py_run!(py, sm, "import sys; sys.modules['anise.astro'] = sm");
    parent_module.add_submodule(sm)?;
    Ok(())
}
