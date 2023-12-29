/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::structure::planetocentric::ellipsoid::Ellipsoid;
use pyo3::prelude::*;
use pyo3::py_run;

use anise::astro::orbit::Orbit;
use anise::constants::frames::*;
use anise::frames::Frame;

pub(crate) fn register_astro(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let sm = PyModule::new(py, "_anise.astro")?;
    sm.add_class::<Ellipsoid>()?;
    sm.add_class::<Frame>()?;
    sm.add_class::<Orbit>()?;

    register_constants(py, sm)?;

    py_run!(py, sm, "import sys; sys.modules['anise.astro'] = sm");
    parent_module.add_submodule(sm)?;
    Ok(())
}

#[pyclass]
#[pyo3(module = "anise.astro.constants")]
struct Frames {}

#[pymethods]
impl Frames {
    #[classattr]
    const SSB_J2000: Frame = SSB_J2000;
    #[classattr]
    const MERCURY_J2000: Frame = MERCURY_J2000;
    #[classattr]
    const VENUS_J2000: Frame = VENUS_J2000;
    #[classattr]
    const EARTH_MOON_BARYCENTER_J2000: Frame = EARTH_MOON_BARYCENTER_J2000;
    #[classattr]
    const MARS_BARYCENTER_J2000: Frame = MARS_BARYCENTER_J2000;
    #[classattr]
    const JUPITER_BARYCENTER_J2000: Frame = JUPITER_BARYCENTER_J2000;
    #[classattr]
    const SATURN_BARYCENTER_J2000: Frame = SATURN_BARYCENTER_J2000;
    #[classattr]
    const URANUS_BARYCENTER_J2000: Frame = URANUS_BARYCENTER_J2000;
    #[classattr]
    const NEPTUNE_BARYCENTER_J2000: Frame = NEPTUNE_BARYCENTER_J2000;
    #[classattr]
    const PLUTO_BARYCENTER_J2000: Frame = PLUTO_BARYCENTER_J2000;
    #[classattr]
    const SUN_J2000: Frame = SUN_J2000;
    #[classattr]
    const LUNA_J2000: Frame = LUNA_J2000;
    #[classattr]
    const EARTH_J2000: Frame = EARTH_J2000;
    #[classattr]
    const EME2000: Frame = EME2000;
    #[classattr]
    const EARTH_ECLIPJ2000: Frame = EARTH_ECLIPJ2000;
    #[classattr]
    const IAU_MERCURY_FRAME: Frame = IAU_MERCURY_FRAME;
    #[classattr]
    const IAU_VENUS_FRAME: Frame = IAU_VENUS_FRAME;
    #[classattr]
    const IAU_EARTH_FRAME: Frame = IAU_EARTH_FRAME;
    #[classattr]
    const IAU_MARS_FRAME: Frame = IAU_MARS_FRAME;
    #[classattr]
    const IAU_JUPITER_FRAME: Frame = IAU_JUPITER_FRAME;
    #[classattr]
    const IAU_SATURN_FRAME: Frame = IAU_SATURN_FRAME;
    #[classattr]
    const IAU_NEPTUNE_FRAME: Frame = IAU_NEPTUNE_FRAME;
    #[classattr]
    const IAU_URANUS_FRAME: Frame = IAU_URANUS_FRAME;
    #[classattr]
    const EARTH_ITRF93: Frame = EARTH_ITRF93;
}

pub(crate) fn register_constants(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let sm = PyModule::new(py, "_anise.astro.constants")?;
    sm.add_class::<Frames>()?;

    py_run!(
        py,
        sm,
        "import sys; sys.modules['anise.astro.constants'] = sm"
    );
    parent_module.add_submodule(sm)?;
    Ok(())
}
