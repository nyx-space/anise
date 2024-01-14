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
use anise::constants::usual_planetary_constants::MEAN_EARTH_ANGULAR_VELOCITY_DEG_S;
use anise::constants::usual_planetary_constants::MEAN_LUNA_ANGULAR_VELOCITY_DEG_S;
use anise::constants::SPEED_OF_LIGHT_KM_S;
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
    sm.add_class::<AzElRange>()?;

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

#[pyclass]
#[pyo3(module = "anise.astro.constants")]
struct UsualConstants {}

#[pymethods]
impl UsualConstants {
    #[classattr]
    /// Mean angular velocity of the Earth in deg/s
    /// Source: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016 (confirmed by https://hpiers.obspm.fr/eop-pc/models/constants.html)
    const MEAN_EARTH_ANGULAR_VELOCITY_DEG_S: f64 = MEAN_EARTH_ANGULAR_VELOCITY_DEG_S;
    #[classattr]
    /// Mean angular velocity of the Moon in deg/s, computed from hifitime:
    /// ```py
    /// >>> luna_period = Unit.Day*27+Unit.Hour*7+Unit.Minute*43+Unit.Second*12
    /// >>> tau/luna_period.to_seconds()
    /// 2.661698975163682e-06
    /// ```
    /// Source: https://www.britannica.com/science/month#ref225844 via https://en.wikipedia.org/w/index.php?title=Lunar_day&oldid=1180701337
    const MEAN_LUNA_ANGULAR_VELOCITY_DEG_S: f64 = MEAN_LUNA_ANGULAR_VELOCITY_DEG_S;
    #[classattr]
    /// Speed of light in kilometers per second (km/s)
    const SPEED_OF_LIGHT_KM_S: f64 = SPEED_OF_LIGHT_KM_S;
}

pub(crate) fn register_constants(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let sm = PyModule::new(py, "_anise.astro.constants")?;
    sm.add_class::<Frames>()?;
    sm.add_class::<UsualConstants>()?;

    py_run!(
        py,
        sm,
        "import sys; sys.modules['anise.astro.constants'] = sm"
    );
    parent_module.add_submodule(sm)?;
    Ok(())
}
