/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use snafu::ResultExt;

use super::{OrientationError, OrientationPhysicsSnafu};
use crate::almanac::Almanac;
use crate::constants::orientations::{
    ECLIPJ2000, FRAME_BIAS_DEPSBI_ARCSEC, FRAME_BIAS_DPSIBI_ARCSEC, FRAME_BIAS_DRA0_ARCSEC, ICRS,
    J2000, J2000_TO_ECLIPJ2000_ANGLE_RAD,
};
use crate::constants::ARCSEC_TO_RAD;
use crate::hifitime::Epoch;
use crate::math::rotation::{r1, r1_dot, r2, r3, r3_dot, DCM};
use crate::naif::daf::datatypes::Type2ChebyshevSet;
use crate::naif::daf::{DAFError, DafDataType, NAIFDataSet, NAIFSummaryRecord};
use crate::orientations::{BPCSnafu, OrientationInterpolationSnafu};
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
    pub fn rotation_to_parent(
        &self,
        source: Frame,
        mut epoch: Epoch,
    ) -> Result<DCM, OrientationError> {
        // Compute the frame at the requested frozen epoch if set.
        if let Some(frozen_epoch) = source.frozen_epoch {
            epoch = frozen_epoch;
        }

        if source.orient_origin_id_match(J2000) {
            // The parent of Earth ecliptic J2000 is the J2000 inertial frame.
            return Ok(DCM::identity(J2000, J2000));
        } else if source.orient_origin_id_match(ECLIPJ2000) {
            // The parent of Earth ecliptic J2000 is the J2000 inertial frame.
            return Ok(DCM {
                rot_mat: r1(J2000_TO_ECLIPJ2000_ANGLE_RAD),
                rot_mat_dt: None,
                from: J2000,
                to: ECLIPJ2000,
            });
        } else if source.orient_origin_id_match(ICRS) {
            // SOFA iauBi00 / iauBp00 frame bias matrix.
            // Reference: IERS Conventions 2010 (TN36) eq. 5.18,
            // USNO Circular 179 eq. 3.4, SOFA iauBp00.c.
            let dra0_rad = FRAME_BIAS_DRA0_ARCSEC * ARCSEC_TO_RAD;
            let dpsibi_rad = FRAME_BIAS_DPSIBI_ARCSEC * ARCSEC_TO_RAD;
            let depsbi_rad = FRAME_BIAS_DEPSBI_ARCSEC * ARCSEC_TO_RAD;

            // B = R1(-depsbi) * R2(dpsibi * sin(EPS0)) * R3(dra0)
            let rot_mat = r1(-depsbi_rad)
                * r2(dpsibi_rad * J2000_TO_ECLIPJ2000_ANGLE_RAD.sin())
                * r3(dra0_rad);

            return Ok(DCM {
                rot_mat,
                rot_mat_dt: None,
                from: J2000,
                to: ICRS,
            });
        }

        // Let's see if this orientation is defined in the loaded BPC files
        match self.bpc_summary_at_epoch(source.orientation_id, epoch) {
            Ok((summary, bpc_no, daf_idx, idx_in_bpc)) => {
                let new_frame = source.with_orient(summary.inertial_frame_id);

                // This should not fail because we've fetched the bpc_no from above with the bpc_summary_at_epoch call.
                let (_, bpc_data) = self
                    .bpc_data
                    .get_index(bpc_no)
                    .ok_or(OrientationError::Unreachable)?;

                // Compute the angles and their rates
                let (ra_dec_w, d_ra_dec_w) = match summary.data_type()? {
                    DafDataType::Type2ChebyshevTriplet => {
                        let data = bpc_data
                            .nth_data::<Type2ChebyshevSet>(daf_idx, idx_in_bpc)
                            .context(BPCSnafu {
                                action: "fetching data for interpolation",
                            })?;
                        data.evaluate(epoch, summary)
                            .context(OrientationInterpolationSnafu)?
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
                let twist_rad = ra_dec_w[2];
                let dec_rad = ra_dec_w[1];
                let ra_rad = ra_dec_w[0];

                let twist_dot_rad = d_ra_dec_w[2];
                let dec_dot_rad = d_ra_dec_w[1];
                let ra_dot_rad = d_ra_dec_w[0];

                let rot_mat = r3(twist_rad) * r1(dec_rad) * r3(ra_rad);

                let rot_mat_dt = if source.force_inertial {
                    None
                } else {
                    Some(
                        twist_dot_rad * r3_dot(twist_rad) * r1(dec_rad) * r3(ra_rad)
                            + dec_dot_rad * r3(twist_rad) * r1_dot(dec_rad) * r3(ra_rad)
                            + ra_dot_rad * r3(twist_rad) * r1(dec_rad) * r3_dot(ra_rad),
                    )
                };

                Ok(DCM {
                    rot_mat,
                    rot_mat_dt,
                    from: summary.inertial_frame_id,
                    to: source.orientation_id,
                })
            }
            Err(_) => {
                // Not available as a BPC, so let's see if there's planetary data for it.
                for data in self.planetary_data.values().rev() {
                    if let Ok(planetary_data) = data.get_by_id(source.orientation_id) {
                        // Fetch the parent info
                        let system_data = match data.get_by_id(planetary_data.parent_id) {
                            Ok(parent) => parent,
                            Err(_) => planetary_data,
                        };

                        return planetary_data
                            .rotation_to_parent(epoch, &system_data, source.force_inertial)
                            .context(OrientationPhysicsSnafu);
                    }
                }

                // Finally, let's see if it's in the loaded Euler Parameters.
                // We can call `into` because EPs can be converted directly into DCMs.
                Ok(self.euler_param_from_id(source.orientation_id)?.into())
            }
        }
    }
}
