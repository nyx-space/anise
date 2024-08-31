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

use super::OrientationError;
use super::OrientationPhysicsSnafu;
use crate::almanac::Almanac;
use crate::constants::orientations::J2000;
use crate::hifitime::Epoch;
use crate::math::cartesian::CartesianState;
use crate::math::rotation::DCM;
use crate::math::units::*;
use crate::math::Vector3;
use crate::prelude::Frame;

impl Almanac {
    /// Returns the 6x6 DCM needed to rotation the `from_frame` to the `to_frame`.
    ///
    /// # Warning
    /// This function only performs the rotation and no translation whatsoever. Use the `transform_from_to` function instead to include rotations.
    ///
    /// # Note
    /// This function performs a recursion of no more than twice the MAX_TREE_DEPTH.
    pub fn rotate(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
    ) -> Result<DCM, OrientationError> {
        let mut to_frame: Frame = to_frame;

        // If there is no frame info, the user hasn't loaded this frame, but might still want to compute a translation.
        if let Ok(to_frame_info) = self.frame_from_uid(to_frame) {
            // User has loaded the planetary data for this frame, so let's use that as the to_frame.
            to_frame = to_frame_info;
        }

        if from_frame.orient_origin_match(to_frame) {
            // Both frames match, return this frame's hash (i.e. no need to go higher up).
            return Ok(DCM::identity(
                from_frame.orientation_id,
                to_frame.orientation_id,
            ));
        }

        let (node_count, path, common_node) =
            self.common_orientation_path(from_frame, to_frame, epoch)?;

        // The fwrd variables are the states from the `from frame` to the common node
        let mut dcm_fwrd = if from_frame.orient_origin_id_match(common_node) {
            DCM::identity(common_node, common_node)
        } else {
            self.rotation_to_parent(from_frame, epoch)?
        };

        // The bwrd variables are the states from the `to frame` back to the common node
        let mut dcm_bwrd = if to_frame.orient_origin_id_match(common_node) {
            DCM::identity(common_node, common_node)
        } else {
            self.rotation_to_parent(to_frame, epoch)?.transpose()
        };

        for cur_node_id in path.iter().take(node_count) {
            let next_parent = cur_node_id.unwrap();
            if next_parent == J2000 {
                // The parent rotation of J2000 is itself, so we can skip this.
                continue;
            }

            let cur_dcm = self.rotation_to_parent(Frame::from_orient_ssb(next_parent), epoch)?;

            if dcm_fwrd.from == cur_dcm.from {
                dcm_fwrd = (cur_dcm * dcm_fwrd.transpose()).context(OrientationPhysicsSnafu)?;
            } else if dcm_fwrd.from == cur_dcm.to {
                dcm_fwrd = (dcm_fwrd * cur_dcm)
                    .context(OrientationPhysicsSnafu)?
                    .transpose();
            } else if dcm_bwrd.to == cur_dcm.from {
                dcm_bwrd = (cur_dcm * dcm_bwrd).context(OrientationPhysicsSnafu)?;
            } else if dcm_bwrd.to == cur_dcm.to {
                dcm_bwrd = (dcm_bwrd.transpose() * cur_dcm).context(OrientationPhysicsSnafu)?;
            } else {
                return Err(OrientationError::Unreachable);
            }

            if next_parent == common_node {
                break;
            }
        }

        if dcm_fwrd.from == dcm_bwrd.from {
            (dcm_bwrd * dcm_fwrd.transpose()).context(OrientationPhysicsSnafu)
        } else if dcm_fwrd.from == dcm_bwrd.to {
            Ok((dcm_fwrd * dcm_bwrd)
                .context(OrientationPhysicsSnafu)?
                .transpose())
        } else if dcm_fwrd.to == dcm_bwrd.to {
            Ok((dcm_fwrd.transpose() * dcm_bwrd)
                .context(OrientationPhysicsSnafu)?
                .transpose())
        } else {
            (dcm_bwrd * dcm_fwrd).context(OrientationPhysicsSnafu)
        }
    }

    /// Rotates the provided Cartesian state into the requested observer frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_to` function instead to include rotations.
    #[allow(clippy::too_many_arguments)]
    pub fn rotate_to(
        &self,
        state: CartesianState,
        observer_frame: Frame,
    ) -> Result<CartesianState, OrientationError> {
        let dcm = self.rotate(state.frame, observer_frame, state.epoch)?;

        (dcm * state).context(OrientationPhysicsSnafu {})
    }

    /// Rotates a state with its origin (`to_frame`) and given its units (distance_unit, time_unit), returns that state with respect to the requested frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_state_to` function instead to include rotations.
    #[allow(clippy::too_many_arguments)]
    pub fn rotate_state_to(
        &self,
        position: Vector3,
        velocity: Vector3,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        distance_unit: LengthUnit,
        time_unit: TimeUnit,
    ) -> Result<CartesianState, OrientationError> {
        // Compute the frame translation
        let dcm = self.rotate(from_frame, to_frame, epoch)?;

        let dist_unit_factor = LengthUnit::Kilometer.from_meters() * distance_unit.to_meters();
        let time_unit_factor = time_unit.in_seconds();

        let input_state = CartesianState {
            radius_km: position * dist_unit_factor,
            velocity_km_s: velocity * dist_unit_factor / time_unit_factor,
            epoch,
            frame: from_frame,
        };

        (dcm * input_state).context(OrientationPhysicsSnafu {})
    }
}
