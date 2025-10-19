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
use ::anise::analysis::event::{Event, EventArc, EventDetails, EventEdge};
use ::anise::analysis::prelude::OrbitalElement;
use ::anise::analysis::python::{
    PyFrameSpec, PyOrthogonalFrame, PyScalarExpr, PyStateSpec, PyVectorExpr,
};
use ::anise::analysis::report::ReportScalars;
use ::anise::analysis::specs::Plane;
use ::anise::astro::Aberration;
use ::anise::structure::dataset::location_dhall::PyLocationDataSet;
use ::anise::structure::dataset::location_dhall::{LocationDhallSet, LocationDhallSetEntry};
use hifitime::leap_seconds::{LatestLeapSeconds, LeapSecondsFile};
use hifitime::python::{PyDurationError, PyHifitimeError, PyParsingError};
use hifitime::ut1::Ut1Provider;
use hifitime::{prelude::*, MonthName, Polynomial};

use pyo3::{prelude::*, wrap_pyfunction, wrap_pymodule};

mod astro;
mod bin;
mod constants;
mod rotation;
mod utils;

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "_anise")]
fn anise(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    m.add_wrapped(wrap_pymodule!(time))?;
    m.add_wrapped(wrap_pymodule!(analysis))?;
    m.add_wrapped(wrap_pymodule!(astro::astro))?;
    m.add_wrapped(wrap_pymodule!(constants::constants))?;
    m.add_wrapped(wrap_pymodule!(utils::utils))?;
    m.add_wrapped(wrap_pymodule!(rotation::rotation))?;
    m.add_wrapped(wrap_pyfunction!(bin::exec_gui))?;

    m.add_class::<Almanac>()?;
    m.add_class::<Aberration>()?;
    m.add_class::<MetaAlmanac>()?;
    m.add_class::<MetaFile>()?;
    m.add_class::<LocationDhallSet>()?;
    m.add_class::<LocationDhallSetEntry>()?;
    m.add_class::<PyLocationDataSet>()?;
    Ok(())
}

#[pymodule]
fn analysis(_py: Python, sm: &Bound<PyModule>) -> PyResult<()> {
    sm.add_class::<PyStateSpec>()?;
    sm.add_class::<PyOrthogonalFrame>()?;
    sm.add_class::<PyVectorExpr>()?;
    sm.add_class::<PyScalarExpr>()?;
    sm.add_class::<PyFrameSpec>()?;
    sm.add_class::<OrbitalElement>()?;
    sm.add_class::<Plane>()?;
    sm.add_class::<Event>()?;
    sm.add_class::<EventDetails>()?;
    sm.add_class::<EventEdge>()?;
    sm.add_class::<EventArc>()?;
    sm.add_class::<ReportScalars>()?;
    Ok(())
}

/// Reexport hifitime as anise.time
#[pymodule]
fn time(_py: Python, sm: &Bound<PyModule>) -> PyResult<()> {
    sm.add_class::<Epoch>()?;
    sm.add_class::<TimeScale>()?;
    sm.add_class::<TimeSeries>()?;
    sm.add_class::<Duration>()?;
    sm.add_class::<Unit>()?;
    sm.add_class::<LatestLeapSeconds>()?;
    sm.add_class::<LeapSecondsFile>()?;
    sm.add_class::<Ut1Provider>()?;
    sm.add_class::<MonthName>()?;
    sm.add_class::<PyHifitimeError>()?;
    sm.add_class::<PyDurationError>()?;
    sm.add_class::<PyParsingError>()?;
    sm.add_class::<Polynomial>()?;
    sm.add_class::<Weekday>()?;
    Ok(())
}
