/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::ops::Add;

use super::{perpv, Vector3};
use crate::{
    errors::{EpochMismatchSnafu, FrameMismatchSnafu, PhysicsError},
    prelude::Frame,
};
use hifitime::Epoch;
use nalgebra::Vector6;
use snafu::ensure;

/// Defines a Cartesian state in a given frame at a given epoch in a given time scale. Radius data is expressed in kilometers. Velocity data is expressed in kilometers per second.
/// Regardless of the constructor used, this struct stores all the state information in Cartesian coordinates as these are always non singular.
///
/// Unless noted otherwise, algorithms are from GMAT 2016a [StateConversionUtil.cpp](https://github.com/ChristopherRabotin/GMAT/blob/37201a6290e7f7b941bc98ee973a527a5857104b/src/base/util/StateConversionUtil.cpp).
#[derive(Copy, Clone, Debug)]
pub struct CartesianState {
    /// Position radius in kilometers
    pub radius_km: Vector3,
    /// Velocity in kilometers per second
    pub velocity_km_s: Vector3,
    /// Epoch with time scale at which this is valid.
    pub epoch: Epoch,
    /// Frame in which this Cartesian state lives.
    pub frame: Frame,
}

impl CartesianState {
    pub fn zero(frame: Frame) -> Self {
        Self {
            radius_km: Vector3::zeros(),
            velocity_km_s: Vector3::zeros(),
            epoch: Epoch::from_tdb_seconds(0.0),
            frame,
        }
    }

    pub fn zero_as_epoch(epoch: Epoch, frame: Frame) -> Self {
        Self {
            radius_km: Vector3::zeros(),
            velocity_km_s: Vector3::zeros(),
            epoch,
            frame,
        }
    }

    /// Creates a new Cartesian state in the provided frame at the provided Epoch, and does not set its acceleration.
    ///
    /// **Units:** km, km, km, km/s, km/s, km/s
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x_km: f64,
        y_km: f64,
        z_km: f64,
        vx_km_s: f64,
        vy_km_s: f64,
        vz_km_s: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> Self {
        Self {
            radius_km: Vector3::new(x_km, y_km, z_km),
            velocity_km_s: Vector3::new(vx_km_s, vy_km_s, vz_km_s),
            epoch,
            frame,
        }
    }

    /// Creates a new Cartesian in the provided frame at the provided Epoch in time with 0.0 velocity.
    ///
    /// **Units:** km, km, km
    pub fn from_position(x_km: f64, y_km: f64, z_km: f64, epoch: Epoch, frame: Frame) -> Self {
        Self::new(x_km, y_km, z_km, 0.0, 0.0, 0.0, epoch, frame)
    }

    /// Creates a new Cartesian around in the provided frame from the borrowed state vector
    ///
    /// The state vector **must** be x, y, z, vx, vy, vz. This function is a shortcut to `cartesian`
    /// and as such it has the same unit requirements.
    ///
    /// **Units:** position data must be in kilometers, velocity data must be in kilometers per second.
    pub fn from_cartesian_pos_vel(pos_vel: Vector6<f64>, epoch: Epoch, frame: Frame) -> Self {
        Self::new(
            pos_vel[0], pos_vel[1], pos_vel[2], pos_vel[3], pos_vel[4], pos_vel[5], epoch, frame,
        )
    }

    /// Returns the magnitude of the radius vector in km
    pub fn rmag_km(&self) -> f64 {
        self.radius_km.norm()
    }

    /// Returns the magnitude of the velocity vector in km/s
    pub fn vmag_km_s(&self) -> f64 {
        self.velocity_km_s.norm()
    }

    /// Returns a copy of the state with a new radius
    pub fn with_radius_km(self, new_radius_km: Vector3) -> Self {
        let mut me = self;
        me.radius_km = new_radius_km;
        me
    }

    /// Returns a copy of the state with a new radius
    pub fn with_velocity_km_s(self, new_velocity_km_s: Vector3) -> Self {
        let mut me = self;
        me.velocity_km_s = new_velocity_km_s;
        me
    }

    /// Returns this state as a Cartesian Vector6 in [km, km, km, km/s, km/s, km/s]
    ///
    /// Note that the time is **not** returned in the vector.
    pub fn to_cartesian_pos_vel(self) -> Vector6<f64> {
        Vector6::from_iterator(
            self.radius_km
                .iter()
                .chain(self.velocity_km_s.iter())
                .cloned(),
        )
    }

    /// Returns the distance in kilometers between this state and another state.
    /// Will **panic** is the frames are different
    pub fn distance_to(&self, other: &Self) -> f64 {
        assert_eq!(
            self.frame, other.frame,
            "cannot compute the distance between two states in different frames"
        );
        self.distance_to_point_km(&other.radius_km)
    }

    /// Returns the distance in kilometers between this state and a point assumed to be in the same frame.
    pub fn distance_to_point_km(&self, other_km: &Vector3) -> f64 {
        (self.radius_km - other_km).norm()
    }

    /// Returns the unit vector in the direction of the state radius
    pub fn r_hat(&self) -> Vector3 {
        self.radius_km / self.rmag_km()
    }

    /// Returns the unit vector in the direction of the state velocity
    pub fn v_hat(&self) -> Vector3 {
        perpv(&self.velocity_km_s, &self.r_hat()) / self.rmag_km()
    }

    /// Returns whether this orbit and another are equal within the specified radial and velocity absolute tolerances
    pub fn eq_within(&self, other: &Self, radial_tol_km: f64, velocity_tol_km_s: f64) -> bool {
        self.epoch == other.epoch
            && (self.radius_km.x - other.radius_km.x).abs() < radial_tol_km
            && (self.radius_km.y - other.radius_km.y).abs() < radial_tol_km
            && (self.radius_km.z - other.radius_km.z).abs() < radial_tol_km
            && (self.velocity_km_s.x - other.velocity_km_s.x).abs() < velocity_tol_km_s
            && (self.velocity_km_s.y - other.velocity_km_s.y).abs() < velocity_tol_km_s
            && (self.velocity_km_s.z - other.velocity_km_s.z).abs() < velocity_tol_km_s
            && self.frame == other.frame
    }
}

impl Add for CartesianState {
    type Output = Result<CartesianState, PhysicsError>;

    /// Adds one state to another. This will return an error if the epochs or frames are different.
    fn add(self, other: CartesianState) -> Self::Output {
        ensure!(
            self.epoch == other.epoch,
            EpochMismatchSnafu {
                action: "translating states",
                epoch1: self.epoch,
                epoch2: other.epoch
            }
        );

        ensure!(
            self.frame == other.frame,
            FrameMismatchSnafu {
                action: "translating states",
                frame1: self.frame,
                frame2: other.frame
            }
        );

        Ok(CartesianState {
            radius_km: self.radius_km + other.radius_km,
            velocity_km_s: self.velocity_km_s + other.velocity_km_s,
            epoch: self.epoch,
            frame: self.frame,
        })
    }
}

impl PartialEq for CartesianState {
    /// Two states are equal if their position are equal within one centimeter and their velocities within one centimeter per second.
    fn eq(&self, other: &Self) -> bool {
        let radial_tol = 1e-5; // centimeter
        let velocity_tol = 1e-5; // centimeter per second
        self.eq_within(other, radial_tol, velocity_tol)
    }
}
