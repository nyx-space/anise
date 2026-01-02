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
use crate::errors::PhysicsError;
use crate::math::cartesian::CartesianState;
use crate::math::rotation::{EulerParameter, DCM};
use crate::math::Vector3;
use crate::structure::dataset::DataSetT;
use core::f64::consts::TAU;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
mod python;

mod enc_dec;

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

impl Default for FovShape {
    fn default() -> Self {
        Self::Conical {
            half_angle_deg: 0.0,
        }
    }
}

#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.instrument"))]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Instrument {
    /// The static rotation from the Parent Frame to the instrument Frame.
    /// (How the camera is bolted onto the bus).
    pub mounting_rotation: EulerParameter,

    /// The translation offset from the Parent Frame origin (CoM) to the instrument origin.
    /// (The "Lever Arm").
    pub mounting_translation: Vector3,

    /// The geometric definition of the field of view.
    pub fov: FovShape,
}

#[cfg_attr(feature = "python", pymethods)]
impl Instrument {
    /// Computes the state (orientation + Cartesian state) of the instrument
    /// at a specific instant, given the spacecraft's state.
    ///
    /// NOTE: This call will return an error if the reference frames are not adequate.
    /// Example:
    /// - If the mounting rotation "from" frame does not match in sc_attitude_to_body "to" frame IDs
    pub fn transform_state(
        &self,
        sc_attitude_to_body: EulerParameter,
        mut sc_state: CartesianState,
    ) -> PhysicsResult<(EulerParameter, CartesianState)> {
        let q_inertial_to_instrument = (self.mounting_rotation * sc_attitude_to_body)?;

        let offset_inertial = sc_attitude_to_body.conjugate() * self.mounting_translation;

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
        let (q_i2s, state_s) = self.transform_state(sc_attitude_to_body, sc_state)?;

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
                let angle_off_boresight = vec_instrument.angle(&Vector3::z());

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

    /// Computes the footprint (swath) of the instrument on a target body.
    ///
    /// This function projects the edges of the Field of View onto the provided target ellipsoid.
    ///
    /// # Arguments
    /// * `sc_attitude_inertial_to_body` - The orientation of the spacecraft body relative to Inertial.
    /// * `sc_state` - The inertial state (position/velocity) of the spacecraft.
    /// * `target_state` - The inertial state of the target body center.
    /// * `target_orientation_inertial_to_fixed` - The orientation of the target body frame relative to Inertial.
    /// * `resolution` - The number of points to generate along the FOV boundary.
    ///
    /// # Returns
    /// A vector of `Orbit` objects, each representing a point on the surface of the target
    /// expressed in the `target_frame` (Fixed).
    pub fn compute_footprint(
        &self,
        sc_attitude_to_body: EulerParameter,
        sc_state: CartesianState,
        target_state: CartesianState,
        target_orientation_to_fixed: EulerParameter,
        resolution: usize,
    ) -> PhysicsResult<Vec<CartesianState>> {
        let target_shape = target_state
            .frame
            .shape
            .ok_or(PhysicsError::MissingFrameData {
                action: "retrieving ellipsoid shape",
                data: "shape",
                frame: target_state.frame.into(),
            })?;

        let mut footprint = Vec::with_capacity(resolution);

        // 1. Get Instrument Inertial State (Position & Orientation)
        //    q_i2s: Inertial -> Instrument
        //    pos_s_i: Instrument Position in Inertial
        let (q_i2s, pos_s_i) = self.transform_state(sc_attitude_to_body, sc_state)?;

        // 2. Compute Relative Position in Target Body-Fixed Frame
        //    r_rel_i = Pos_Instrument_Inertial - Pos_Target_Inertial
        let r_rel_i = (pos_s_i - target_state)?.radius_km;

        //    Transform to Target Body-Fixed Frame
        //    r_rel_b = q_i2b * r_rel_i
        let dcm_i2fixed = DCM::from(target_orientation_to_fixed);
        let pos_sensor_fixed = dcm_i2fixed * r_rel_i;

        // 3. Compute Rotation from Instrument to Target Body-Fixed Frame
        //    q_s2fixed = q_i2fixed * q_s2i
        //              = q_i2fixed * q_i2s.conjugate()
        //    Note: We rely on ANISE frame chaining checks, or manual composition if IDs differ.
        let dcm_i2s = DCM::from(q_i2s);
        //    R_s2fixed = R_i2fixed * R_s2i = R_i2fixed * R_i2s^T
        let dcm_s2fixed = (dcm_i2fixed * dcm_i2s.transpose())?;

        // 4. Generate Rays in Instrument Frame (Z-forward)
        let rays_sensor = self.generate_fov_boundary_vectors(resolution);

        // 5. Intersect each ray
        for ray_s in rays_sensor {
            // Rotate ray to Target Fixed Frame
            let ray_fixed = dcm_s2fixed * ray_s;

            // Perform Intersection
            if let Some(surface_point) = target_shape.intersect(pos_sensor_fixed, ray_fixed) {
                // Construct Orbit
                // Velocity on surface is zero in the Fixed frame
                let orbit = CartesianState {
                    radius_km: surface_point,
                    velocity_km_s: Vector3::zeros(),
                    epoch: sc_state.epoch,
                    frame: target_state.frame,
                };
                footprint.push(orbit);
            }
        }

        Ok(footprint)
    }
}

impl Instrument {
    /// Helper to generate unit vectors defining the boundary of the FOV in the Instrument Frame.
    fn generate_fov_boundary_vectors(&self, resolution: usize) -> Vec<Vector3> {
        let mut rays = Vec::with_capacity(resolution);

        match self.fov {
            FovShape::Conical { half_angle_deg } => {
                let half_angle = half_angle_deg.to_radians();
                let (sin_a, cos_a) = half_angle.sin_cos();

                for i in 0..resolution {
                    let phi = (i as f64) * TAU / (resolution as f64);
                    // Standard spherical coordinates (Z is boresight)
                    let v = Vector3::new(sin_a * phi.cos(), sin_a * phi.sin(), cos_a);
                    rays.push(v.normalize());
                }
            }
            FovShape::Rectangular {
                x_half_angle_deg,
                y_half_angle_deg,
            } => {
                let tx = x_half_angle_deg.to_radians().tan();
                let ty = y_half_angle_deg.to_radians().tan();

                // Define 4 corners in the Z=1 plane
                let corners = [
                    Vector3::new(-tx, ty, 1.0),  // Top-Left
                    Vector3::new(tx, ty, 1.0),   // Top-Right
                    Vector3::new(tx, -ty, 1.0),  // Bottom-Right
                    Vector3::new(-tx, -ty, 1.0), // Bottom-Left
                ];

                // Distribute points along the 4 edges
                let points_per_side = resolution / 4;
                // Ensure at least 1 point per side
                let points_per_side = if points_per_side == 0 {
                    1
                } else {
                    points_per_side
                };

                for i in 0..4 {
                    let start = corners[i];
                    let end = corners[(i + 1) % 4];

                    for j in 0..points_per_side {
                        let t = (j as f64) / (points_per_side as f64);
                        // Linear interpolation on the plane, then normalize
                        let v = (start * (1.0 - t) + end * t).normalize();
                        rays.push(v);
                    }
                }
            }
        }
        rays
    }
}

impl DataSetT for Instrument {
    const NAME: &'static str = "Instrument";
}

#[cfg(test)]
mod ut_instrument {
    use super::*;
    use crate::math::rotation::EulerParameter;
    use crate::math::Vector3;
    use crate::prelude::{Epoch, Frame, Orbit};
    use crate::structure::planetocentric::ellipsoid::Ellipsoid;

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

    fn mock_target_frame(id: i32, shape: Ellipsoid) -> Frame {
        Frame {
            orientation_id: id,
            ephemeris_id: id,
            mu_km3_s2: None,
            shape: Some(shape),
        }
    }

    #[test]
    fn test_fov_conical_simple() {
        // SETUP: Simple Conical Instrument looking down +Z
        // Frames: 0=Inertial, 1=Body.
        let instrument = Instrument {
            mounting_rotation: EulerParameter::identity(1, 1),
            mounting_translation: Vector3::zeros(),
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
        // SETUP: Rectangular Instrument (Wide Width, Narrow Height)
        // X_Half = 20 deg, Y_Half = 5 deg.
        let instrument = Instrument {
            mounting_rotation: EulerParameter::identity(1, 1),
            mounting_translation: Vector3::zeros(),
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
            fov: FovShape::Conical {
                half_angle_deg: 5.0,
            },
        };

        let sc_att = EulerParameter::identity(1, 0);
        let sc_state = state_at_pos(0, sc_pos_inertial);

        // 1. Verify Inertial State Calculation
        // Expected Pos: SC Pos (1000,0,0) + Lever Arm (0,0,1) = (1000, 0, 1).
        // Note: SC attitude is identity, so Body frame == Inertial frame.
        let (q_inertial_inst, state_inst) = instrument.transform_state(sc_att, sc_state).unwrap();
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

    #[test]
    fn test_footprint_nadir_spherical_conical() {
        // SETUP:
        // Target: Sphere R=6000 km at Origin.
        // Spacecraft: at (0, 0, 10000) km (4000 km altitude).
        // Attitude: Rotated 180 deg about X.
        //           SC Body +Z points to Inertial -Z (Nadir).
        // Instrument: Conical 10 deg FOV, Boresight +Z.

        let r_planet = 6000.0;
        let shape = Ellipsoid::from_sphere(r_planet);
        let target_frame = mock_target_frame(0, shape);

        let instrument = Instrument {
            mounting_rotation: EulerParameter::identity(1, 1),
            mounting_translation: Vector3::zeros(),
            fov: FovShape::Conical {
                half_angle_deg: 10.0,
            },
        };

        // SC Attitude: 180 deg about X so Z_body -> -Z_inertial
        let sc_att = EulerParameter::about_x(core::f64::consts::PI, 1, 0);
        let sc_state = state_at_pos(0, Vector3::new(0.0, 0.0, 10000.0));
        let mut target_state = state_at_origin(0);
        target_state.frame = target_frame;
        let target_orient = EulerParameter::identity(0, 10); // Target fixed aligned with Inertial

        // ACT
        let footprint = instrument
            .compute_footprint(
                sc_att,
                sc_state,
                target_state,
                target_orient,
                36, // resolution
            )
            .expect("Footprint computation failed");

        // ASSERT
        assert_eq!(footprint.len(), 36);

        for orbit in &footprint {
            let pos = orbit.radius_km;

            // 1. Check Surface Intersection: Norm should be R_planet
            assert!((pos.norm() - r_planet).abs() < 1e-6, "Point not on surface");

            // 2. Check Geometry:
            // Since we are looking straight down (Nadir) at Z=10000,
            // and FOV is 10 deg, the "ring" on the surface should have constant Z.
            // (Strictly speaking, it's a small circle of latitude).
            // We can check if the angle from Z-axis matches expectations,
            // but simply checking that Z is positive and constant is a good smoke test.
            assert!(pos.z > 0.0);
            assert!(
                (pos.z - footprint[0].radius_km.z).abs() < 1e-6,
                "Ring is not flat in Z"
            );
        }
    }

    #[test]
    fn test_footprint_rectangular_rotation() {
        // SETUP:
        // Similar to above, but with Rectangular FOV and rotated Instrument.
        // Instrument is rotated 90 deg about Z relative to Bus.
        // So Instrument "Width" (X) aligns with Bus Y.

        let r_planet = 6000.0;
        let shape = Ellipsoid::from_sphere(r_planet);
        let target_frame = mock_target_frame(0, shape);

        let instrument = Instrument {
            // Rotated 90 deg about Z: Inst X -> Body Y
            mounting_rotation: EulerParameter::about_z(core::f64::consts::FRAC_PI_2, 2, 1),
            mounting_translation: Vector3::zeros(),
            fov: FovShape::Rectangular {
                x_half_angle_deg: 20.0, // Wide
                y_half_angle_deg: 5.0,  // Narrow
            },
        };

        let sc_att = EulerParameter::about_x(core::f64::consts::PI, 1, 0); // Nadir Pointing
        let sc_state = state_at_pos(0, Vector3::new(0.0, 0.0, 10000.0));
        let mut target_state = state_at_origin(0);
        target_state.frame = target_frame;
        let target_orient = EulerParameter::identity(0, 10);

        // ACT
        // Resolution 40 -> 10 points per side
        let footprint = instrument
            .compute_footprint(sc_att, sc_state, target_state, target_orient, 40)
            .expect("Computation failed");

        // ASSERT
        assert_eq!(footprint.len(), 40);

        // Check Surface Intersection
        for orbit in &footprint {
            assert!((orbit.rmag_km() - r_planet).abs() < 1e-6);
        }

        // Check Orientation on Surface
        // Since Inst X (Wide, 20deg) is aligned with Body Y (and Inertial -Y),
        // The footprint should be wider in Y than in X.
        let max_y = footprint
            .iter()
            .map(|o| o.radius_km.y.abs())
            .fold(0.0, f64::max);
        let max_x = footprint
            .iter()
            .map(|o| o.radius_km.x.abs())
            .fold(0.0, f64::max);

        assert!(
            max_y > max_x,
            "Footprint should be wider in Y due to instrument rotation"
        );
    }

    #[test]
    fn test_footprint_miss() {
        // SETUP:
        // Spacecraft looking AWAY from the planet.
        // SC at (0, 0, 10000). Attitude Identity (Z body -> Z inertial).
        // Planet at (0, 0, 0).
        // Sensor looks at +Z (away from planet at origin).

        let shape = Ellipsoid::from_sphere(6000.0);
        let target_frame = mock_target_frame(0, shape);

        let instrument = Instrument {
            mounting_rotation: EulerParameter::identity(1, 1),
            mounting_translation: Vector3::zeros(),
            fov: FovShape::Conical {
                half_angle_deg: 10.0,
            },
        };

        let sc_att = EulerParameter::identity(1, 0); // Z -> Z (Away)
        let sc_state = state_at_pos(0, Vector3::new(0.0, 0.0, 10000.0));
        let mut target_state = state_at_origin(0);
        target_state.frame = target_frame;

        let footprint = instrument
            .compute_footprint(
                sc_att,
                sc_state,
                target_state,
                EulerParameter::identity(0, 10),
                10,
            )
            .unwrap();

        // ASSERT
        assert_eq!(
            footprint.len(),
            0,
            "Should return empty footprint when looking away"
        );
    }
}
