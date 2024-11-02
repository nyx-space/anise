/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{
    planetary::{PlanetaryDataError, PlanetaryDataSetSnafu},
    Almanac,
};
use crate::prelude::Frame;
use pyo3::prelude::*;
use snafu::prelude::*;

#[pymethods]
impl Almanac {
    /// Returns the frame information (gravitational param, shape) as defined in this Almanac from an empty frame
    /// :type uid: Frame
    /// :rtype: Frame
    pub fn frame_info(&self, uid: Frame) -> Result<Frame, PlanetaryDataError> {
        Ok(self
            .planetary_data
            .get_by_id(uid.ephemeris_id)
            .context(PlanetaryDataSetSnafu {
                action: "fetching frame by its UID via ephemeris_id",
            })?
            .to_frame(uid.into()))
    }
}
