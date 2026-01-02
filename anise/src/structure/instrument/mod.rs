/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::astro::PhysicsResult;
use crate::math::cartesian::CartesianState;
use crate::math::rotation::{EulerParameter, DCM};
use crate::math::Vector3;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
mod python;

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.instrument"))]
pub enum FovShape {
    /// Circular Field of View (e.g., simple antenna, laser)
    Conical { half_angle_deg: f64 },
    /// Rectangular Field of View (e.g., camera sensors)
    ///
    /// **Convention:** Assumes the instrument frame is defined such that:
    /// * +Z is the Boresight
    /// * +X is the "Width" direction
    /// * +Y is the "Height" direction
    Rectangular {
        x_half_angle_deg: f64,
        y_half_angle_deg: f64,
    },
}

#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.instrument"))]
#[derive(Copy, Clone, Debug)]
pub struct Instrument {
    /// The static rotation from the Parent Frame to the instrument Frame.
    /// (How the camera is bolted onto the bus).
    pub mounting_rotation: EulerParameter,

    /// The translation offset from the Parent Frame origin (CoM) to the instrument origin.
    /// (The "Lever Arm").
    pub mounting_translation: Vector3,

    /// The primary look direction in the instrument Frame.
    /// For Rectangular FOVs, this is assumed to be +Z.
    pub boresight_axis: Vector3,

    /// The geometric definition of the field of view.
    pub fov: FovShape,
}

impl Instrument {
    /// Computes the inertial state (orientation + Cartesian state) of the instrument
    /// at a specific instant, given the spacecraft's state.
    ///
    /// NOTE: This call will return an error if the reference frames are not adequate.
    /// Example:
    /// - If the mounting rotation "from" frame does not match in sc_attitude_to_body "to" frame IDs
    pub fn inertial_state(
        &self,
        sc_attitude_to_body: EulerParameter,
        mut sc_state: CartesianState,
    ) -> PhysicsResult<(EulerParameter, CartesianState)> {
        let q_inertial_to_instrument = (self.mounting_rotation * sc_attitude_to_body)?;

        let dcm_body_to_inertial = DCM::from(sc_attitude_to_body.conjugate());
        let offset_inertial = dcm_body_to_inertial * self.mounting_translation;

        // Usurp the sc_state as the position of the instrument in the inertial frame
        sc_state.radius_km += offset_inertial;

        Ok((q_inertial_to_instrument, sc_state))
    }

    /// Calculates the angular margin to the FOV boundary in radians.
    ///
    /// This is a continuous function suitable for event detection (root finding).
    /// * `> 0.0`: Target is INSIDE.
    /// * `< 0.0`: Target is OUTSIDE.
    /// * `= 0.0`: Target is ON THE BOUNDARY.
    ///
    /// NOTE: This call will return an error if the reference frames are not adequate.
    /// Example:
    /// - If the mounting rotation "from" frame does not match in sc_attitude_to_body "to" frame IDs
    /// - If the target state frame ID is not identical to the instrument's inertial state given the sc_attitude Euler Parameter.
    pub fn fov_margin_deg(
        &self,
        sc_attitude_to_body: EulerParameter,
        sc_state: CartesianState,
        target_state: CartesianState,
    ) -> PhysicsResult<f64> {
        // 1. Get the instrument Frame State
        let (q_i2s, state_s) = self.inertial_state(sc_attitude_to_body, sc_state)?;

        // 2. Compute the vector to the target in the instrument Frame
        let vec_inertial = (target_state - state_s)?.radius_km;

        // Robustness: If target is coincident with instrument, margin is undefined (or handle gracefully)
        if vec_inertial.norm() < 1e-9 {
            return Ok(-1.0); // Fail-safe: consider "inside the camera" as obscured/invalid
        }

        let dcm_i2s = DCM::from(q_i2s);
        let vec_instrument = dcm_i2s * vec_inertial;

        match self.fov {
            FovShape::Conical { half_angle_deg } => {
                let half_angle = half_angle_deg.to_radians();

                // Angle between the target vector and the defined boresight axis
                let angle_off_boresight = vec_instrument.angle(&self.boresight_axis);

                // Margin = Limit - Current
                Ok((half_angle - angle_off_boresight).to_degrees())
            }
            FovShape::Rectangular {
                x_half_angle_deg,
                y_half_angle_deg,
            } => {
                // Assumes instrument +Z is boresight.
                // Project into XZ and YZ planes.

                // Angle in the X-Z plane (Width)
                let angle_x = vec_instrument.x.atan2(vec_instrument.z).abs();

                // Angle in the Y-Z plane (Height)
                let angle_y = vec_instrument.y.atan2(vec_instrument.z).abs();

                let margin_x = x_half_angle_deg.to_radians() - angle_x;
                let margin_y = y_half_angle_deg.to_radians() - angle_y;

                // For the target to be "In FOV", BOTH margins must be positive.
                // Therefore, the smallest margin dictates the boundary crossing.
                Ok((margin_x.min(margin_y)).to_degrees())
            }
        }
    }

    /// Checks if a target is visible within the Field of View.
    pub fn is_target_in_fov(
        &self,
        sc_attitude_inertial_to_body: EulerParameter,
        sc_state: CartesianState,
        target_state: CartesianState,
    ) -> PhysicsResult<bool> {
        Ok(self.fov_margin_deg(sc_attitude_inertial_to_body, sc_state, target_state)? >= 0.0)
    }
}

#[cfg(test)]
mod ut_instrument {
    use super::*;
    use crate::math::rotation::EulerParameter;
    use crate::math::Vector3;
    use crate::prelude::{Epoch, Frame, Orbit};

    // Helper to create a dummy state at the origin
    fn state_at_origin(frame_id: i32) -> Orbit {
        CartesianState {
            epoch: Epoch::from_tdb_seconds(0.0),
            frame: Frame::from_orient_ssb(frame_id),
            radius_km: Vector3::zeros(),
            velocity_km_s: Vector3::zeros(),
        }
    }

    // Helper to create a dummy state at a specific position
    fn state_at_pos(frame_id: i32, pos: Vector3) -> Orbit {
        CartesianState {
            epoch: Epoch::from_tdb_seconds(0.0),
            frame: Frame::from_orient_ssb(frame_id),
            radius_km: pos,
            velocity_km_s: Vector3::zeros(),
        }
    }

    #[test]
    fn test_fov_conical_simple() {
        // SETUP: Simple Conical Instrument looking down +Z
        // Frames: 0=Inertial, 1=Body.
        let instrument = Instrument {
            mounting_rotation: EulerParameter::identity(1, 1), // Identity rotation
            mounting_translation: Vector3::zeros(),            // No offset
            boresight_axis: Vector3::z(),
            fov: FovShape::Conical {
                half_angle_deg: 10.0,
            },
        };

        let sc_att = EulerParameter::identity(1, 0);
        let sc_state = state_at_origin(0);

        // CASE 1: Target straight ahead (+Z) -> Should be INSIDE
        // Margin should be roughly 10 deg (Target is at 0, Limit is 10)
        let target_pos_inside = Vector3::new(0.0, 0.0, 100.0);
        let target_state = state_at_pos(0, target_pos_inside);

        let margin = instrument
            .fov_margin_deg(sc_att, sc_state, target_state)
            .unwrap();

        assert!(margin > 0.0);
        assert!((margin - 10.0).abs() < 1e-6);
        assert!(instrument
            .is_target_in_fov(sc_att, sc_state, target_state)
            .unwrap());

        // CASE 2: Target at 90 deg (+X) -> Should be OUTSIDE
        // Margin should be 10 - 90 = -80 deg
        let target_pos_outside = Vector3::new(100.0, 0.0, 0.0);
        let target_state_out = state_at_pos(0, target_pos_outside);

        let margin_out = instrument
            .fov_margin_deg(sc_att, sc_state, target_state_out)
            .unwrap();
        assert!(margin_out < 0.0);
        assert!((margin_out - -80.0).abs() < 1e-6);
        assert!(!instrument
            .is_target_in_fov(sc_att, sc_state, target_state_out)
            .unwrap());
    }

    #[test]
    fn test_fov_rectangular_aspect() {
        // SETUP: Rectangular Sensor (Wide Width, Narrow Height)
        // X_Half = 20 deg, Y_Half = 5 deg.
        let instrument = Instrument {
            mounting_rotation: EulerParameter::identity(1, 1),
            mounting_translation: Vector3::zeros(),
            boresight_axis: Vector3::z(),
            fov: FovShape::Rectangular {
                x_half_angle_deg: 20.0,
                y_half_angle_deg: 5.0,
            },
        };

        let sc_att = EulerParameter::identity(1, 0);
        let sc_state = state_at_origin(0);

        // CASE 1: Target is at 10 deg azimuth (X), 0 deg elevation (Y).
        // X Margin: 20 - 10 = +10.
        // Y Margin: 5 - 0 = +5.
        // Result: INSIDE. Min Margin = +5.
        let angle_rad = 10.0_f64.to_radians();
        let target_vec = Vector3::new(angle_rad.sin(), 0.0, angle_rad.cos());
        let target_state = state_at_pos(0, target_vec * 100.0);

        let margin = instrument
            .fov_margin_deg(sc_att, sc_state, target_state)
            .unwrap();
        assert!((margin - 5.0).abs() < 1.0); // Rough check on degrees logic
        assert!(instrument
            .is_target_in_fov(sc_att, sc_state, target_state)
            .unwrap());

        // CASE 2: Target is at 0 deg azimuth (X), 10 deg elevation (Y).
        // X Margin: 20 - 0 = +20.
        // Y Margin: 5 - 10 = -5.
        // Result: OUTSIDE. Min Margin = -5.
        let angle_rad = 10.0_f64.to_radians();
        let target_vec = Vector3::new(0.0, angle_rad.sin(), angle_rad.cos());
        let target_state_out = state_at_pos(0, target_vec * 100.0);

        let margin_out = instrument
            .fov_margin_deg(sc_att, sc_state, target_state_out)
            .unwrap();
        assert!(margin_out < 0.0);
        assert!(!instrument
            .is_target_in_fov(sc_att, sc_state, target_state_out)
            .unwrap());
    }

    #[test]
    fn test_offset_and_rotation() {
        // SETUP: Complex Geometry
        // Spacecraft is at (1000, 0, 0) km Inertial.
        // Instrument is offset by (0, 0, 1) km in Body Frame (Lever Arm).
        // Instrument is rotated 90 deg about Y relative to Body (Looks +X Body).
        // Body is aligned with Inertial.

        let sc_pos_inertial = Vector3::new(1000.0, 0.0, 0.0);
        let lever_arm_body = Vector3::new(0.0, 0.0, 1.0);

        // Rotation Body->Instrument: 90 deg about Y.
        // Body X maps to Instrument Z (Boresight).
        let mounting_rot = EulerParameter::about_y(90.0_f64.to_radians(), 2, 1);

        let instrument = Instrument {
            mounting_rotation: mounting_rot,
            mounting_translation: lever_arm_body,
            boresight_axis: Vector3::z(), // Instrument frame boresight
            fov: FovShape::Conical {
                half_angle_deg: 5.0,
            },
        };

        let sc_att = EulerParameter::identity(1, 0);
        let sc_state = state_at_pos(0, sc_pos_inertial);

        // 1. Verify Inertial State Calculation
        // Expected Pos: SC Pos (1000,0,0) + Lever Arm (0,0,1) = (1000, 0, 1).
        // Note: SC attitude is identity, so Body frame == Inertial frame.
        let (q_inertial_inst, state_inst) = instrument.inertial_state(sc_att, sc_state).unwrap();
        println!("{q_inertial_inst}");

        let diff_pos = state_inst.radius_km - Vector3::new(1000.0, 0.0, 1.0);
        assert!(
            diff_pos.norm() < 1e-6,
            "Instrument position calculation incorrect"
        );

        // 2. Verify Visibility
        // Target is at (2000, 0, 1).
        // Vector from Instrument: (2000,0,1) - (1000,0,1) = (1000, 0, 0).
        // In Body Frame (Identity SC att), this is +X.
        // The Instrument is rotated 90 deg Y, so it looks down Body +X.
        // Therefore, this target should be perfectly Boresighted.

        let target_state = state_at_pos(0, Vector3::new(2000.0, 0.0, 1.0));

        let margin = instrument
            .fov_margin_deg(sc_att, sc_state, target_state)
            .unwrap();
        assert!(
            (margin - 5.0).abs() < 1e-6,
            "Target should be on boresight (margin = half-angle)"
        );
        assert!(instrument
            .is_target_in_fov(sc_att, sc_state, target_state)
            .unwrap());
    }

    #[test]
    fn test_frame_mismatch_error() {
        // Ensure error is returned if frames don't chain
        let instrument = Instrument {
            mounting_rotation: EulerParameter::identity(1, 2), // Body(1) -> Inst(2)
            mounting_translation: Vector3::zeros(),
            boresight_axis: Vector3::z(),
            fov: FovShape::Conical {
                half_angle_deg: 1.0,
            },
        };

        // SC Attitude is Inertial(0) -> OtherFrame(5).
        // This should fail because '5' != '1' (Mounting Base).
        let sc_att = EulerParameter::identity(5, 0);
        let sc_state = state_at_origin(0);
        let target_state = state_at_origin(0);

        let result = instrument.fov_margin_deg(sc_att, sc_state, target_state);
        assert!(result.is_err());
    }
}
