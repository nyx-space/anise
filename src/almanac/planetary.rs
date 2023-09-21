use crate::{
    prelude::{Frame, FrameUid},
    structure::dataset::DataSetError,
};

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

#[derive(Debug, Snafu, PartialEq)]
#[snafu(visibility(pub(crate)))]
pub enum PlanetaryDataError {
    #[snafu(display("when {action} {source}"))]
    PlanetaryDataSet {
        action: &'static str,
        source: DataSetError,
    },
}

impl<'a: 'b, 'b> Almanac<'a> {
    /// Given the frame UID, attempt to retrieve the full frame information, if that frame is loaded
    pub fn frame_from_uid(&self, uid: FrameUid) -> Result<Frame, PlanetaryDataError> {
        Ok(self
            .planetary_data
            .get_by_id(uid.ephemeris_id)
            .with_context(|_| PlanetaryDataSetSnafu {
                action: "fetching frame by its UID via ephemeris_id",
            })?
            .to_frame(uid))
    }

    /// Given the frame, attempt to retrieve the full frame information, if that frame is loaded
    pub fn frame_fetch(&self, frame: Frame) -> Result<Frame, PlanetaryDataError> {
        self.frame_from_uid(frame.into())
    }
}
