/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use log::trace;
use snafu::ResultExt;

use super::{OrientationError, OrientationPhysicsSnafu};
use crate::almanac::Almanac;
use crate::hifitime::Epoch;
use crate::math::rotation::DCM;
use crate::orientations::OrientationDataSetSnafu;
use crate::prelude::Frame;

impl<'a> Almanac {
    /// Returns the position vector and velocity vector of the `source` with respect to its parent in the ephemeris at the provided epoch,
    /// and in the provided distance and time units.
    ///
    /// # Example
    /// If the ephemeris stores position interpolation coefficients in kilometer but this function is called with millimeters as a distance unit,
    /// the output vectors will be in mm, mm/s, mm/s^2 respectively.
    ///
    /// # Errors
    /// + As of now, some interpolation types are not supported, and if that were to happen, this would return an error.
    ///
    /// **WARNING:** This function only performs the translation and no rotation whatsoever. Use the `transform_to_parent_from` function instead to include rotations.
    pub fn rotation_to_parent(&self, source: Frame, epoch: Epoch) -> Result<DCM, OrientationError> {
        // Let's see if this orientation is defined in the loaded BPC files
        match self.bpc_summary_at_epoch(source.orientation_id, epoch) {
            Ok((_summary, _bpc_no, _idx_in_bpc)) => todo!("BPC not yet supported"),
            Err(_) => {
                trace!("query {source} wrt to its parent @ {epoch:E} using planetary data");
                // Not available as a BPC, so let's see if there's planetary data for it.
                let planetary_data = self
                    .planetary_data
                    .get_by_id(source.orientation_id)
                    .with_context(|_| OrientationDataSetSnafu)?;
                // Fetch the parent info
                let system_data = match self.planetary_data.get_by_id(planetary_data.parent_id) {
                    Ok(parent) => parent,
                    Err(_) => planetary_data,
                };

                planetary_data
                    .rotation_to_parent(epoch, &system_data)
                    .with_context(|_| OrientationPhysicsSnafu)
            }
        }
    }
}
