/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use snafu::ResultExt;

use super::EphemerisError;
use super::UnderlyingPhysicsSnafu;
use crate::almanac::Almanac;
use crate::astro::Aberration;
use crate::hifitime::Epoch;
use crate::math::cartesian::CartesianState;
use crate::math::units::*;
use crate::math::Vector3;
use crate::prelude::{Frame, FrameTrait};

/// **Limitation:** no translation or rotation may have more than 8 nodes.
pub const MAX_TREE_DEPTH: usize = 8;

impl<'a> Almanac<'a> {
    /// Returns the position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`.
    ///
    /// **WARNING:** This function only performs the translation and no rotation whatsoever. Use the `transform_from_to` function instead to include rotations.
    ///
    /// Note: this function performs a recursion of no more than twice the [MAX_TREE_DEPTH].
    pub fn translate_from_to(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        ab_corr: Aberration,
        length_unit: LengthUnit,
        time_unit: TimeUnit,
    ) -> Result<CartesianState, EphemerisError> {
        if from_frame == to_frame {
            // Both frames match, return this frame's hash (i.e. no need to go higher up).
            return Ok(CartesianState::zero(from_frame));
        }

        let (node_count, path, common_node) =
            self.common_ephemeris_path(from_frame, to_frame, epoch)?;

        // The fwrd variables are the states from the `from frame` to the common node
        let (mut pos_fwrd, mut vel_fwrd, mut acc_fwrd, mut frame_fwrd) =
            if from_frame.ephem_origin_id_match(common_node) {
                (
                    Vector3::zeros(),
                    Vector3::zeros(),
                    Vector3::zeros(),
                    from_frame,
                )
            } else {
                self.translate_to_parent(from_frame, epoch, ab_corr, length_unit, time_unit)?
            };

        // The bwrd variables are the states from the `to frame` back to the common node
        let (mut pos_bwrd, mut vel_bwrd, mut acc_bwrd, mut frame_bwrd) =
            if to_frame.ephem_origin_id_match(common_node) {
                (
                    Vector3::zeros(),
                    Vector3::zeros(),
                    Vector3::zeros(),
                    to_frame,
                )
            } else {
                self.translate_to_parent(to_frame, epoch, ab_corr, length_unit, time_unit)?
            };

        for cur_node_hash in path.iter().take(node_count) {
            if !frame_fwrd.ephem_origin_id_match(common_node) {
                let (cur_pos_fwrd, cur_vel_fwrd, cur_acc_fwrd, cur_frame_fwrd) =
                    self.translate_to_parent(frame_fwrd, epoch, ab_corr, length_unit, time_unit)?;

                pos_fwrd += cur_pos_fwrd;
                vel_fwrd += cur_vel_fwrd;
                acc_fwrd += cur_acc_fwrd;
                frame_fwrd = cur_frame_fwrd;
            }

            if !frame_bwrd.ephem_origin_id_match(common_node) {
                let (cur_pos_bwrd, cur_vel_bwrd, cur_acc_bwrd, cur_frame_bwrd) =
                    self.translate_to_parent(frame_bwrd, epoch, ab_corr, length_unit, time_unit)?;

                pos_bwrd += cur_pos_bwrd;
                vel_bwrd += cur_vel_bwrd;
                acc_bwrd += cur_acc_bwrd;
                frame_bwrd = cur_frame_bwrd;
            }

            // We know this exist, so we can safely unwrap it
            if cur_node_hash.unwrap() == common_node {
                break;
            }
        }

        Ok(CartesianState {
            radius_km: pos_fwrd - pos_bwrd,
            velocity_km_s: vel_fwrd - vel_bwrd,
            acceleration_km_s2: Some(acc_fwrd - acc_bwrd),
            epoch,
            frame: to_frame,
        })
    }

    /// Returns the position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in km, the velocity in km/s, and the acceleration in km/s^2.
    pub fn translate_from_to_km_s(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        ab_corr: Aberration,
    ) -> Result<CartesianState, EphemerisError> {
        self.translate_from_to(
            from_frame,
            to_frame,
            epoch,
            ab_corr,
            LengthUnit::Kilometer,
            TimeUnit::Second,
        )
    }

    /// Returns the position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in m, the velocity in m/s, and the acceleration in m/s^2.
    pub fn translate_from_to_m_s(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        ab_corr: Aberration,
    ) -> Result<CartesianState, EphemerisError> {
        self.translate_from_to(
            from_frame,
            to_frame,
            epoch,
            ab_corr,
            LengthUnit::Meter,
            TimeUnit::Second,
        )
    }

    /// Returns the geometric position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in km, the velocity in km/s, and the acceleration in km/s^2.
    pub fn translate_from_to_km_s_geometric(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
    ) -> Result<CartesianState, EphemerisError> {
        self.translate_from_to(
            from_frame,
            to_frame,
            epoch,
            Aberration::None,
            LengthUnit::Kilometer,
            TimeUnit::Second,
        )
    }

    /// Returns the geometric position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in m, the velocity in m/s, and the acceleration in m/s^2.
    pub fn translate_from_to_m_s_geometric(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
    ) -> Result<CartesianState, EphemerisError> {
        self.translate_from_to(
            from_frame,
            to_frame,
            epoch,
            Aberration::None,
            LengthUnit::Meter,
            TimeUnit::Second,
        )
    }

    /// Translates a state with its origin (`to_frame`) and given its units (distance_unit, time_unit), returns that state with respect to the requested frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_state_to` function instead to include rotations.
    #[allow(clippy::too_many_arguments)]
    pub fn translate_state_to(
        &self,
        position: Vector3,
        velocity: Vector3,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        ab_corr: Aberration,
        distance_unit: LengthUnit,
        time_unit: TimeUnit,
    ) -> Result<CartesianState, EphemerisError> {
        // Compute the frame translation
        let frame_state = self.translate_from_to(
            from_frame,
            to_frame,
            epoch,
            ab_corr,
            distance_unit,
            time_unit,
        )?;

        let dist_unit_factor = LengthUnit::Kilometer.from_meters() * distance_unit.to_meters();
        let time_unit_factor = time_unit.in_seconds();

        let input_state = CartesianState {
            radius_km: position * dist_unit_factor,
            velocity_km_s: velocity * dist_unit_factor / time_unit_factor,
            acceleration_km_s2: None,
            epoch,
            frame: from_frame,
        };

        (input_state + frame_state).with_context(|_| UnderlyingPhysicsSnafu {})
    }
}
