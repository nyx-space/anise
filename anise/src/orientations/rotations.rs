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
    ///
    /// # Algorithm
    /// 1.  Find the common ancestor of the `from_frame` and `to_frame` in the orientation tree using `common_orientation_path`.
    /// 2.  Initialize the DCMs for both the forward (from to common ancestor) and backward (to to common ancestor) paths.
    /// 3.  Iteratively traverse the orientation tree from the `from` and `to` frames up to the common ancestor, composing the DCMs at each step using `rotation_to_parent`.
    /// 4.  The final DCM is the composition of the forward and backward DCMs.
    pub fn rotate(
        &self,
        from_frame: Frame,
        mut to_frame: Frame,
        epoch: Epoch,
    ) -> Result<DCM, OrientationError> {
        // If there is no frame info, the user hasn't loaded this frame, but might still want to compute a translation.
        if let Ok(to_frame_info) = self.frame_info(to_frame) {
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

        // The `dcm_fwrd` variable stores the rotation from the `from_frame` to the common ancestor.
        let mut dcm_fwrd = if from_frame.orient_origin_id_match(common_node) {
            // The from_frame is the common ancestor, so the rotation is identity.
            DCM::identity(common_node, common_node)
        } else {
            self.rotation_to_parent(from_frame, epoch)?
        };

        // The `dcm_bwrd` variable stores the rotation from the `to_frame` to the common ancestor.
        let mut dcm_bwrd = if to_frame.orient_origin_id_match(common_node) {
            // The to_frame is the common ancestor, so the rotation is identity.
            DCM::identity(common_node, common_node)
        } else {
            self.rotation_to_parent(to_frame, epoch)?.transpose()
        };

        // Traverse the orientation tree from both the `from` and `to` frames up to the common ancestor.
        for cur_node_id in path.iter().take(node_count) {
            let next_parent = cur_node_id.unwrap();
            if next_parent == J2000 {
                // The parent rotation of J2000 is itself, so we can skip this.
                continue;
            }

            // Get the rotation from the current node to its parent.
            let cur_dcm = self.rotation_to_parent(Frame::from_orient_ssb(next_parent), epoch)?;

            // Compose the rotations.
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
                // This should be unreachable if the path finding logic is correct.
                return Err(OrientationError::Unreachable);
            }

            if next_parent == common_node {
                // We have reached the common ancestor, so we can stop.
                break;
            }
        }

        // Combine the forward and backward rotations to get the final rotation.
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

    /// Returns the angular velocity vector in rad/s of the from_frame wtr to the to_frame.
    ///
    /// This can be used to compute the angular velocity of the Earth ITRF93 frame with respect to the J2000 frame for example.
    pub fn angular_velocity_rad_s(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
    ) -> Result<Vector3, OrientationError> {
        let dcm = self.rotate(from_frame, to_frame, epoch)?;

        if let Some(omega_rad_s) = dcm.angular_velocity_rad_s() {
            Ok(omega_rad_s)
        } else {
            Err(OrientationError::OrientationPhysics {
                source: crate::errors::PhysicsError::DCMMissingDerivative {
                    action: "computing the angular velocity",
                },
            })
        }
    }

    /// Returns the angular velocity vector in rad/s of the from_frame wtr to the J2000 frame.
    pub fn angular_velocity_wtr_j2000_rad_s(
        &self,
        from_frame: Frame,
        epoch: Epoch,
    ) -> Result<Vector3, OrientationError> {
        self.angular_velocity_rad_s(from_frame, from_frame.with_orient(J2000), epoch)
    }

    /// Returns the angular velocity vector in deg/s of the from_frame wtr to the to_frame.
    ///
    /// This can be used to compute the angular velocity of the Earth ITRF93 frame with respect to the J2000 frame for example.
    pub fn angular_velocity_deg_s(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
    ) -> Result<Vector3, OrientationError> {
        let dcm = self.rotate(from_frame, to_frame, epoch)?;

        if let Some(omega_deg_s) = dcm.angular_velocity_deg_s() {
            Ok(omega_deg_s)
        } else {
            Err(OrientationError::OrientationPhysics {
                source: crate::errors::PhysicsError::DCMMissingDerivative {
                    action: "computing the angular velocity",
                },
            })
        }
    }

    /// Returns the angular velocity vector in deg/s of the from_frame wtr to the J2000 frame.
    pub fn angular_velocity_wtr_j2000_deg_s(
        &self,
        from_frame: Frame,
        epoch: Epoch,
    ) -> Result<Vector3, OrientationError> {
        self.angular_velocity_deg_s(from_frame, from_frame.with_orient(J2000), epoch)
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
