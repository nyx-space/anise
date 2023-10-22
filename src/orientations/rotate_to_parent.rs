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
use crate::constants::frames::EARTH_ECLIPJ2000;
use crate::constants::orientations::{ECLIPJ2000, J2000, J2000_TO_ECLIPJ2000_ANGLE_RAD};
use crate::hifitime::Epoch;
use crate::math::rotation::{r1, r3, DCM};
use crate::naif::daf::datatypes::Type2ChebyshevSet;
use crate::naif::daf::{DAFError, DafDataType, NAIFDataSet};
use crate::orientations::{BPCSnafu, OrientationDataSetSnafu, OrientationInterpolationSnafu};
use crate::prelude::Frame;

impl Almanac {
    /// Returns the direct cosine matrix (DCM) to rotate from the `source` to its parent in the orientation hierarchy at the provided epoch,
    ///
    /// # Example
    /// If the ephemeris stores position interpolation coefficients in kilometer but this function is called with millimeters as a distance unit,
    /// the output vectors will be in mm, mm/s, mm/s^2 respectively.
    ///
    /// # Errors
    /// + As of now, some interpolation types are not supported, and if that were to happen, this would return an error.
    ///
    /// **WARNING:** This function only performs the rotation and no translation whatsoever. Use the `transform_to_parent_from` function instead to include rotations.
    pub fn rotation_to_parent(&self, source: Frame, epoch: Epoch) -> Result<DCM, OrientationError> {
        if source == EARTH_ECLIPJ2000 {
            // The parent of Earth ecliptic J2000 is the J2000 inertial frame.
            return Ok(DCM {
                rot_mat: r1(J2000_TO_ECLIPJ2000_ANGLE_RAD),
                rot_mat_dt: None,
                from: ECLIPJ2000,
                to: J2000,
            });
        }
        // Let's see if this orientation is defined in the loaded BPC files
        match self.bpc_summary_at_epoch(source.orientation_id, epoch) {
            Ok((summary, bpc_no, idx_in_bpc)) => {
                let new_frame = source.with_orient(summary.inertial_frame_id);

                trace!("rotate {source} wrt to {new_frame} @ {epoch:E}");

                // This should not fail because we've fetched the spk_no from above with the spk_summary_at_epoch call.
                let bpc_data = self.bpc_data[bpc_no]
                    .as_ref()
                    .ok_or(OrientationError::Unreachable)?;

                let (ra_dec_w, d_ra_dec_w) = match summary.data_type()? {
                    DafDataType::Type2ChebyshevTriplet => {
                        let data = bpc_data
                            .nth_data::<Type2ChebyshevSet>(idx_in_bpc)
                            .with_context(|_| BPCSnafu {
                                action: "fetching data for interpolation",
                            })?;
                        data.evaluate(epoch, summary)
                            .with_context(|_| OrientationInterpolationSnafu)?
                    }
                    dtype => {
                        return Err(OrientationError::BPC {
                            action: "rotation to parent",
                            source: DAFError::UnsupportedDatatype {
                                dtype,
                                kind: "BPC computations",
                            },
                        })
                    }
                };

                // And build the DCM
                let rot_mat = r3(ra_dec_w[2]) * r1(ra_dec_w[1]) * r3(ra_dec_w[0]);
                let rot_mat_dt = Some(r3(d_ra_dec_w[2]) * r1(d_ra_dec_w[1]) * r3(d_ra_dec_w[0]));
                Ok(DCM {
                    rot_mat,
                    rot_mat_dt,
                    from: source.orientation_id,
                    to: summary.inertial_frame_id,
                })
            }
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
