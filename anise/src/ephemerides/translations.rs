/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use snafu::ResultExt;

use super::EphemerisError;
use super::EphemerisPhysicsSnafu;
use crate::almanac::Almanac;
use crate::astro::Aberration;
use crate::hifitime::Epoch;
use crate::math::cartesian::CartesianState;
use crate::math::units::*;
use crate::math::Vector3;
use crate::prelude::Frame;

/// **Limitation:** no translation or rotation may have more than 8 nodes.
pub const MAX_TREE_DEPTH: usize = 8;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python", pymethods)]
impl Almanac {
    /// Returns the Cartesian state needed to translate the `from_frame` to the `to_frame`.
    ///
    /// # Warning
    /// This function only performs the translation and no rotation whatsoever. Use the `transform_from_to` function instead to include rotations.
    ///
    /// # Note
    /// This function performs a recursion of no more than twice the [MAX_TREE_DEPTH].
    pub fn translate_from_to(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
    ) -> Result<CartesianState, EphemerisError> {
        let mut to_frame: Frame = to_frame;

        // If there is no frame info, the user hasn't loaded this frame, but might still want to compute a translation.
        if let Ok(to_frame_info) = self.frame_from_uid(to_frame) {
            // User has loaded the planetary data for this frame, so let's use that as the to_frame.
            to_frame = to_frame_info;
        }

        if from_frame == to_frame {
            // Both frames match, return this frame's hash (i.e. no need to go higher up).
            return Ok(CartesianState::zero(from_frame));
        }

        let (node_count, path, common_node) =
            self.common_ephemeris_path(from_frame, to_frame, epoch)?;

        // The fwrd variables are the states from the `from frame` to the common node
        let (mut pos_fwrd, mut vel_fwrd, mut frame_fwrd) =
            if from_frame.ephem_origin_id_match(common_node) {
                (Vector3::zeros(), Vector3::zeros(), from_frame)
            } else {
                self.translation_parts_to_parent(from_frame, epoch, ab_corr)?
            };

        // The bwrd variables are the states from the `to frame` back to the common node
        let (mut pos_bwrd, mut vel_bwrd, mut frame_bwrd) =
            if to_frame.ephem_origin_id_match(common_node) {
                (Vector3::zeros(), Vector3::zeros(), to_frame)
            } else {
                self.translation_parts_to_parent(to_frame, epoch, ab_corr)?
            };

        for cur_node_id in path.iter().take(node_count) {
            if !frame_fwrd.ephem_origin_id_match(common_node) {
                let (cur_pos_fwrd, cur_vel_fwrd, cur_frame_fwrd) =
                    self.translation_parts_to_parent(frame_fwrd, epoch, ab_corr)?;

                pos_fwrd += cur_pos_fwrd;
                vel_fwrd += cur_vel_fwrd;
                frame_fwrd = cur_frame_fwrd;
            }

            if !frame_bwrd.ephem_origin_id_match(common_node) {
                let (cur_pos_bwrd, cur_vel_bwrd, cur_frame_bwrd) =
                    self.translation_parts_to_parent(frame_bwrd, epoch, ab_corr)?;

                pos_bwrd += cur_pos_bwrd;
                vel_bwrd += cur_vel_bwrd;
                frame_bwrd = cur_frame_bwrd;
            }

            // We know this exist, so we can safely unwrap it
            if cur_node_id.unwrap() == common_node {
                break;
            }
        }

        Ok(CartesianState {
            radius_km: pos_fwrd - pos_bwrd,
            velocity_km_s: vel_fwrd - vel_bwrd,
            epoch,
            frame: to_frame.with_orient(from_frame.orientation_id),
        })
    }

    /// Returns the geometric position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in km, the velocity in km/s, and the acceleration in km/s^2.
    pub fn translate_from_to_geometric(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
    ) -> Result<CartesianState, EphemerisError> {
        self.translate_from_to(from_frame, to_frame, epoch, Aberration::NONE)
    }

    /// Translates the provided Cartesian state into the requested frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the [transform_to] function instead to include rotations.
    #[allow(clippy::too_many_arguments)]
    pub fn translate_to(
        &self,
        state: CartesianState,
        to_frame: Frame,
        ab_corr: Option<Aberration>,
    ) -> Result<CartesianState, EphemerisError> {
        let frame_state = self.translate_from_to(state.frame, to_frame, state.epoch, ab_corr)?;

        Ok(state.add_unchecked(&frame_state))
    }
}

impl Almanac {
    /// Translates a state with its origin (`to_frame`) and given its units (distance_unit, time_unit), returns that state with respect to the requested frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the [transform_state_to] function instead to include rotations.
    #[allow(clippy::too_many_arguments)]
    pub fn translate_state_to(
        &self,
        position: Vector3,
        velocity: Vector3,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
        distance_unit: LengthUnit,
        time_unit: TimeUnit,
    ) -> Result<CartesianState, EphemerisError> {
        // Compute the frame translation
        let frame_state = self.translate_from_to(from_frame, to_frame, epoch, ab_corr)?;

        let dist_unit_factor = LengthUnit::Kilometer.from_meters() * distance_unit.to_meters();
        let time_unit_factor = time_unit.in_seconds();

        let input_state = CartesianState {
            radius_km: position * dist_unit_factor,
            velocity_km_s: velocity * dist_unit_factor / time_unit_factor,
            epoch,
            frame: from_frame,
        };

        (input_state + frame_state).with_context(|_| EphemerisPhysicsSnafu {
            action: "translating states (likely a bug!)",
        })
    }
}
