/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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
fn anise(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    register_time_module(m)?;
    astro::register_astro(m)?;
    utils::register_utils(m)?;
    m.add_class::<Almanac>()?;
    m.add_class::<Aberration>()?;
    m.add_class::<MetaAlmanac>()?;
    m.add_class::<MetaFile>()?;
    Ok(())
}

/// Reexport hifitime as anise.time
fn register_time_module(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let sm = PyModule::new_bound(parent_module.py(), "time")?;

    sm.add_class::<Epoch>()?;
    sm.add_class::<TimeScale>()?;
    sm.add_class::<TimeSeries>()?;
    sm.add_class::<Duration>()?;
    sm.add_class::<Unit>()?;
    sm.add_class::<LatestLeapSeconds>()?;
    sm.add_class::<LeapSecondsFile>()?;
    sm.add_class::<Ut1Provider>()?;

    Python::with_gil(|py| {
        py_run!(py, sm, "import sys; sys.modules['anise.time'] = sm");
    });
    parent_module.add_submodule(&sm)?;
    Ok(())
}
