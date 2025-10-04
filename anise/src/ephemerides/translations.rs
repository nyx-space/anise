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

use super::EphemerisError;
use super::EphemerisPhysicsSnafu;
use crate::almanac::Almanac;
use crate::astro::aberration::stellar_aberration;
use crate::astro::Aberration;
use crate::constants::frames::SSB_J2000;
use crate::constants::SPEED_OF_LIGHT_KM_S;
use crate::hifitime::Epoch;
use crate::math::cartesian::CartesianState;
use crate::math::units::*;
use crate::math::Vector3;
use crate::prelude::Frame;

/// **Limitation:** no translation or rotation may have more than 8 nodes.
pub const MAX_TREE_DEPTH: usize = 8;

impl Almanac {
    /// Returns the Cartesian state of the target frame as seen from the observer frame at the provided epoch, and optionally given the aberration correction.
    ///
    /// # SPICE Compatibility
    /// This function is the SPICE equivalent of spkezr: `spkezr(TARGET_ID, EPOCH_TDB_S, ORIENTATION_ID, ABERRATION, OBSERVER_ID)`
    /// In ANISE, the TARGET_ID and ORIENTATION are provided in the first argument (TARGET_FRAME), as that frame includes BOTH
    /// the target ID and the orientation of that target. The EPOCH_TDB_S is the epoch in the TDB time system, which is computed
    /// in ANISE using Hifitime. THe ABERRATION is computed by providing the optional Aberration flag. Finally, the OBSERVER
    /// argument is replaced by OBSERVER_FRAME: if the OBSERVER_FRAME argument has the same orientation as the TARGET_FRAME, then this call
    /// will return exactly the same data as the spkerz SPICE call.
    ///
    /// # Warning
    /// This function only performs the translation and no rotation whatsoever. Use the `transform` function instead to include rotations.
    ///
    /// # Note
    /// This function performs a recursion of no more than twice the [MAX_TREE_DEPTH].
    ///
    /// # Algorithm
    /// 1.  Find the common ancestor of the `target_frame` and `observer_frame` in the ephemeris tree using `common_ephemeris_path`.
    /// 2.  Initialize the state vectors for both the forward (observer to common ancestor) and backward (target to common ancestor) paths.
    /// 3.  Iteratively traverse the ephemeris tree from the observer and target frames up to the common ancestor, accumulating the state vectors at each step using `translation_parts_to_parent`.
    /// 4.  If aberration corrections are requested, calculate the one-way light time and apply the correction to the target's position.
    /// 5.  The final state is the difference between the backward and forward state vectors.
    pub fn translate(
        &self,
        target_frame: Frame,
        mut observer_frame: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
    ) -> Result<CartesianState, EphemerisError> {
        if observer_frame == target_frame {
            // Both frames match, return this frame's hash (i.e. no need to go higher up).
            return Ok(CartesianState::zero(observer_frame));
        }

        // If there is no frame info, the user hasn't loaded this frame, but might still want to compute a translation.
        if let Ok(obs_frame_info) = self.frame_info(observer_frame) {
            // User has loaded the planetary data for this frame, so let's use that as the to_frame.
            observer_frame = obs_frame_info;
        }

        match ab_corr {
            None => {
                // Geometric case (no aberration correction)
                let (node_count, _path, common_node) =
                    self.common_ephemeris_path(observer_frame, target_frame, epoch)?;

                // The `fwrd` variables store the state of the observer frame relative to the common ancestor.
                let (mut pos_fwrd, mut vel_fwrd, mut frame_fwrd) =
                    if observer_frame.ephem_origin_id_match(common_node) {
                        // Observer is the common ancestor, so state is zero.
                        (Vector3::zeros(), Vector3::zeros(), observer_frame)
                    } else {
                        self.translation_parts_to_parent(observer_frame, epoch)?
                    };

                // The `bwrd` variables store the state of the target frame relative to the common ancestor.
                let (mut pos_bwrd, mut vel_bwrd, mut frame_bwrd) =
                    if target_frame.ephem_origin_id_match(common_node) {
                        // Target is the common ancestor, so state is zero.
                        (Vector3::zeros(), Vector3::zeros(), target_frame)
                    } else {
                        self.translation_parts_to_parent(target_frame, epoch)?
                    };

                // Traverse the ephemeris tree from both the observer and target up to the common ancestor.
                for _ in 0..node_count {
                    if !frame_fwrd.ephem_origin_id_match(common_node) {
                        // Accumulate the state from the current forward frame to its parent.
                        let (cur_pos_fwrd, cur_vel_fwrd, cur_frame_fwrd) =
                            self.translation_parts_to_parent(frame_fwrd, epoch)?;

                        pos_fwrd += cur_pos_fwrd;
                        vel_fwrd += cur_vel_fwrd;
                        frame_fwrd = cur_frame_fwrd;
                    }

                    if !frame_bwrd.ephem_origin_id_match(common_node) {
                        // Accumulate the state from the current backward frame to its parent.
                        let (cur_pos_bwrd, cur_vel_bwrd, cur_frame_bwrd) =
                            self.translation_parts_to_parent(frame_bwrd, epoch)?;

                        pos_bwrd += cur_pos_bwrd;
                        vel_bwrd += cur_vel_bwrd;
                        frame_bwrd = cur_frame_bwrd;
                    }
                }

                // The final state is the difference between the state of the target and the observer, both relative to the common ancestor.
                Ok(CartesianState {
                    radius_km: pos_bwrd - pos_fwrd,
                    velocity_km_s: vel_bwrd - vel_fwrd,
                    epoch,
                    frame: observer_frame.with_orient(target_frame.orientation_id),
                })
            }
            Some(ab_corr) => {
                // Aberration correction case. This is a rewrite of NAIF SPICE's `spkapo`.

                // Find the geometric position of the observer body with respect to the solar system barycenter (SSB).
                let obs_ssb = self.translate(observer_frame, SSB_J2000, epoch, None)?;
                let obs_ssb_pos_km = obs_ssb.radius_km;
                let obs_ssb_vel_km_s = obs_ssb.velocity_km_s;

                // Find the geometric position of the target body with respect to the SSB at the same epoch.
                let tgt_ssb = self.translate(target_frame, SSB_J2000, epoch, None)?;
                let tgt_ssb_pos_km = tgt_ssb.radius_km;
                let tgt_ssb_vel_km_s = tgt_ssb.velocity_km_s;

                // Calculate the initial relative position and velocity.
                let mut rel_pos_km = tgt_ssb_pos_km - obs_ssb_pos_km;
                let mut rel_vel_km_s = tgt_ssb_vel_km_s - obs_ssb_vel_km_s;

                // Compute the initial one-way light time.
                let mut one_way_lt_s = rel_pos_km.norm() / SPEED_OF_LIGHT_KM_S;

                // Iteratively correct for the one-way light time.
                // The number of iterations depends on whether a converged solution is requested.
                let num_it = if ab_corr.converged { 3 } else { 1 };
                let lt_sign = if ab_corr.transmit_mode { 1.0 } else { -1.0 };

                for _ in 0..num_it {
                    // Calculate the light-time corrected epoch.
                    let epoch_lt = epoch + lt_sign * one_way_lt_s * TimeUnit::Second;
                    // Find the position of the target at the corrected epoch.
                    let tgt_ssb = self
                        .translate(target_frame, SSB_J2000, epoch_lt, None)
                        .map_err(|e| EphemerisError::LightTimeCorrection {
                            epoch,
                            epoch_lt,
                            ab_corr,
                            source: Box::new(e),
                        })?;
                    let tgt_ssb_pos_km = tgt_ssb.radius_km;
                    let tgt_ssb_vel_km_s = tgt_ssb.velocity_km_s;
                    // Update the relative position.
                    rel_pos_km = tgt_ssb_pos_km - obs_ssb_pos_km;
                    let r_norm = rel_pos_km.norm();
                    // Update the light-time corrected relative velocity.
                    let geometric_rel_vel = tgt_ssb_vel_km_s - obs_ssb_vel_km_s;
                    if r_norm > 0.0 {
                        let inv_c_r = 1.0 / (SPEED_OF_LIGHT_KM_S * r_norm);
                        let r_dot_v_rel = rel_pos_km.dot(&geometric_rel_vel);
                        let r_dot_v_tgt = rel_pos_km.dot(&tgt_ssb_vel_km_s);
                        // The rate of change of light time.
                        let dlt = (inv_c_r * r_dot_v_rel) / (1.0 - lt_sign * r_dot_v_tgt * inv_c_r);
                        rel_vel_km_s = tgt_ssb_vel_km_s * (1.0 + lt_sign * dlt) - obs_ssb_vel_km_s;
                    } else {
                        rel_vel_km_s = geometric_rel_vel;
                    }
                    // Update the one-way light time for the next iteration.
                    one_way_lt_s = r_norm / SPEED_OF_LIGHT_KM_S;
                }

                // If stellar aberration correction is requested, apply it now.
                if ab_corr.stellar {
                    rel_pos_km = stellar_aberration(rel_pos_km, obs_ssb_vel_km_s, ab_corr)
                        .context(EphemerisPhysicsSnafu {
                            action: "computing stellar aberration",
                        })?;
                }

                Ok(CartesianState {
                    radius_km: rel_pos_km,
                    velocity_km_s: rel_vel_km_s,
                    epoch,
                    frame: observer_frame.with_orient(target_frame.orientation_id),
                })
            }
        }
    }

    /// Returns the geometric position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in km, the velocity in km/s, and the acceleration in km/s^2.
    pub fn translate_geometric(
        &self,
        target_frame: Frame,
        observer_frame: Frame,
        epoch: Epoch,
    ) -> Result<CartesianState, EphemerisError> {
        self.translate(target_frame, observer_frame, epoch, Aberration::NONE)
    }

    /// Translates the provided Cartesian state into the requested observer frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_to` function instead to include rotations.
    pub fn translate_to(
        &self,
        state: CartesianState,
        mut observer_frame: Frame,
        ab_corr: Option<Aberration>,
    ) -> Result<CartesianState, EphemerisError> {
        let frame_state = self.translate(state.frame, observer_frame, state.epoch, ab_corr)?;
        let mut new_state = state.add_unchecked(&frame_state);

        // If there is no frame info, the user hasn't loaded this frame, but might still want to compute a translation.
        if let Ok(obs_frame_info) = self.frame_info(observer_frame) {
            // User has loaded the planetary data for this frame, so let's use that as the to_frame.
            observer_frame = obs_frame_info;
        }
        new_state.frame = observer_frame.with_orient(state.frame.orientation_id);
        Ok(new_state)
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
        observer_frame: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
        distance_unit: LengthUnit,
        time_unit: TimeUnit,
    ) -> Result<CartesianState, EphemerisError> {
        let dist_unit_factor = LengthUnit::Kilometer.from_meters() * distance_unit.to_meters();
        let time_unit_factor = time_unit.in_seconds();

        let state = CartesianState {
            radius_km: position * dist_unit_factor,
            velocity_km_s: velocity * dist_unit_factor / time_unit_factor,
            epoch,
            frame: from_frame,
        };

        self.translate_to(state, observer_frame, ab_corr)
    }
}
