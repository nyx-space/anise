/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use crate::math::rotation::EulerParameter;
use crate::math::Vector3;
use crate::NaifId;

#[derive(Clone, Debug, PartialEq)]
pub enum FovShape {
    /// Circular Field of View (e.g., simple antenna, laser)
    Conical { half_angle_deg: f64 },
    /// Rectangular Field of View (e.g., camera sensors)
    Rectangular {
        x_half_angle_deg: f64,
        y_half_angle_deg: f64,
    },
}

#[derive(Clone, Debug)]
pub struct Instrument {
    /// The Frame this sensor is mechanically attached to.
    /// Usually the Spacecraft Body Frame ID.
    pub parent_frame: NaifId,

    /// The static rotation from the Parent Frame to the Sensor Frame.
    /// (How the camera is bolted onto the bus).
    pub mounting_rotation: EulerParameter,

    /// The translation offset from the Parent Frame origin (CoM) to the Sensor origin.
    /// (The "Lever Arm").
    pub mounting_translation: Vector3,

    /// The primary look direction in the Sensor Frame.
    /// Default is usually +Z, but good to make explicit.
    pub boresight_axis: Vector3,

    /// The geometric definition of the field of view.
    pub fov: FovShape,
}

impl Instrument {
    /// Helper to get the total rotation from Inertial to Sensor at a given time.
    ///
    /// # Arguments
    /// * `q_inertial_to_parent`: The attitude of the spacecraft body from your dataset.
    pub fn inertial_orientation(&self, q_inertial_to_parent: EulerParameter) -> EulerParameter {
        // q_inertial_to_sensor = q_parent_to_sensor * q_inertial_to_parent
        (self.mounting_rotation * q_inertial_to_parent).unwrap()
    }
}
