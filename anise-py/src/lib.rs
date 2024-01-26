/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use ::anise::almanac::metaload::{MetaAlmanac, MetaFile};
use ::anise::almanac::Almanac;
use ::anise::astro::Aberration;
use hifitime::leap_seconds::{LatestLeapSeconds, LeapSecondsFile};
use hifitime::prelude::*;
use hifitime::ut1::Ut1Provider;

use pyo3::prelude::*;
use pyo3::py_run;

mod astro;
mod constants;
mod utils;

/// A Python module implemented in Rust.
#[pymodule]
fn anise(py: Python, m: &PyModule) -> PyResult<()> {
    register_time_module(py, m)?;
    astro::register_astro(py, m)?;
    utils::register_utils(py, m)?;
    m.add_class::<Almanac>()?;
    m.add_class::<Aberration>()?;
    m.add_class::<MetaAlmanac>()?;
    m.add_class::<MetaFile>()?;
    Ok(())
}

/// Reexport hifitime as anise.time
fn register_time_module(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    pyo3_log::init();
    let sm = PyModule::new(py, "_anise.time")?;

    sm.add_class::<Epoch>()?;
    sm.add_class::<TimeScale>()?;
    sm.add_class::<TimeSeries>()?;
    sm.add_class::<Duration>()?;
    sm.add_class::<Unit>()?;
    sm.add_class::<LatestLeapSeconds>()?;
    sm.add_class::<LeapSecondsFile>()?;
    sm.add_class::<Ut1Provider>()?;

    py_run!(py, sm, "import sys; sys.modules['anise.time'] = sm");
    parent_module.add_submodule(sm)?;
    Ok(())
}
