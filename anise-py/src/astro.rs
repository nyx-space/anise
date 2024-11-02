/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::astro::AzElRange;
use anise::astro::Occultation;
use anise::structure::planetocentric::ellipsoid::Ellipsoid;
use pyo3::prelude::*;
use pyo3::py_run;

use anise::astro::orbit::Orbit;
use anise::frames::Frame;

use super::constants::register_constants;

pub(crate) fn register_astro(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let sm = PyModule::new_bound(parent_module.py(), "astro")?;
    sm.add_class::<Ellipsoid>()?;
    sm.add_class::<Frame>()?;
    sm.add_class::<Orbit>()?;
    sm.add_class::<AzElRange>()?;
    sm.add_class::<Occultation>()?;

    register_constants(&sm)?;

    Python::with_gil(|py| {
        py_run!(py, sm, "import sys; sys.modules['anise.astro'] = sm");
    });

    parent_module.add_submodule(&sm)?;
    Ok(())
}
