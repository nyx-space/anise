/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use super::Almanac;
use snafu::prelude::*;

use crate::{
    prelude::{Frame, FrameUid},
    structure::{dataset::DataSetError, PlanetaryDataSet},
};

#[derive(Debug, Snafu, PartialEq)]
#[snafu(visibility(pub(crate)))]
pub enum PlanetaryDataError {
    #[snafu(display("when {action} {source}"))]
    PlanetaryDataSet {
        action: &'static str,
        source: DataSetError,
    },
}

impl Almanac {
    /// Given the frame UID (or something that can be transformed into it), attempt to retrieve the full frame information, if that frame is loaded
    pub fn frame_from_uid<U: Into<FrameUid>>(&self, uid: U) -> Result<Frame, PlanetaryDataError> {
        let uid = uid.into();
        Ok(self
            .planetary_data
            .get_by_id(uid.ephemeris_id)
            .with_context(|_| PlanetaryDataSetSnafu {
                action: "fetching frame by its UID via ephemeris_id",
            })?
            .to_frame(uid))
    }

    /// Loads the provided planetary data into a clone of this original Almanac.
    pub fn with_planetary_data(&self, planetary_data: PlanetaryDataSet) -> Self {
        let mut me = self.clone();
        me.planetary_data = planetary_data;
        me
    }
}
