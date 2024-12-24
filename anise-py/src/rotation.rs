/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::math::rotation::DCM;
use pyo3::prelude::*;
use pyo3::py_run;

pub(crate) fn register_rotation(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let sm = PyModule::new(parent_module.py(), "rotation")?;
    sm.add_class::<DCM>()?;

    Python::with_gil(|py| {
        py_run!(py, sm, "import sys; sys.modules['anise.rotation'] = sm");
    });

    parent_module.add_submodule(&sm)?;
    Ok(())
}
