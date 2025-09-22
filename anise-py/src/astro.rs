/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::astro::{AzElRange, Location, Occultation, TerrainMask};
use anise::frames::FrameUid;
use anise::structure::planetocentric::ellipsoid::Ellipsoid;
use pyo3::prelude::*;

use anise::astro::orbit::Orbit;
use anise::frames::Frame;
use pyo3::wrap_pymodule;

#[pymodule]
pub(crate) fn astro(_py: Python, sm: &Bound<'_, PyModule>) -> PyResult<()> {
    sm.add_class::<Ellipsoid>()?;
    sm.add_class::<Frame>()?;
    sm.add_class::<FrameUid>()?;
    sm.add_class::<Orbit>()?;
    sm.add_class::<AzElRange>()?;
    sm.add_class::<Occultation>()?;
    sm.add_class::<Location>()?;
    sm.add_class::<TerrainMask>()?;

    // Also add the constants as a submodule to astro for backward compatibility
    sm.add_wrapped(wrap_pymodule!(crate::constants::constants))?;

    Ok(())
}
