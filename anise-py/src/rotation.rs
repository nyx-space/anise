/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::math::rotation::{Quaternion, DCM};
use pyo3::prelude::*;

#[pymodule]
pub(crate) fn rotation(_py: Python, sm: &Bound<PyModule>) -> PyResult<()> {
    sm.add_class::<DCM>()?;
    sm.add_class::<Quaternion>()?;

    Ok(())
}
