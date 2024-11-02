/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{perp_vector, root_mean_squared, root_sum_squared, Vector3};
use crate::{
    astro::PhysicsResult,
    constants::SPEED_OF_LIGHT_KM_S,
    errors::{EpochMismatchSnafu, FrameMismatchSnafu, MathError, PhysicsError},
    prelude::Frame,
};

use core::fmt;
use core::ops::{Add, Neg, Sub};
use hifitime::{Duration, Epoch, TimeUnits};
use nalgebra::Vector6;
use serde_derive::{Deserialize, Serialize};
use snafu::ensure;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Defines a Cartesian state in a given frame at a given epoch in a given time scale. Radius data is expressed in kilometers. Velocity data is expressed in kilometers per second.
/// Regardless of the constructor used, this struct stores all the state information in Cartesian coordinates as these are always non singular.
///
/// Unless noted otherwise, algorithms are from GMAT 2016a [StateConversionUtil.cpp](https://github.com/ChristopherRabotin/GMAT/blob/37201a6290e7f7b941bc98ee973a527a5857104b/src/base/util/StateConversionUtil.cpp).
///
/// :type x_km: float
/// :type y_km: float
/// :type z_km: float
/// :type vx_km_s: float
/// :type vy_km_s: float
/// :type vz_km_s: float
/// :type epoch: Epoch
/// :type frame: Frame
/// :rtype: Orbit
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass(name = "Orbit"))]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
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
    /// Builds a state of zero radius and velocity at zero seconds TDB (01 Jan 2000, midnight TDB) in the provided frame.
    pub fn zero(frame: Frame) -> Self {
        Self {
            radius_km: Vector3::zeros(),
            velocity_km_s: Vector3::zeros(),
            epoch: Epoch::from_tdb_seconds(0.0),
            frame,
        }
    }

    /// Builds a state of zero radius and velocity at the provided epoch in the provided frame.
    pub fn zero_at_epoch(epoch: Epoch, frame: Frame) -> Self {
        Self {
            radius_km: Vector3::zeros(),
            velocity_km_s: Vector3::zeros(),
            epoch,
            frame,
        }
    }

    /// Creates a new Cartesian state in the provided frame at the provided Epoch.
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

    /// Creates a new Cartesian state in the provided frame at the provided Epoch (shortcut to `new`).
    ///
    /// **Units:** km, km, km, km/s, km/s, km/s
    #[allow(clippy::too_many_arguments)]
    pub fn cartesian(
        x_km: f64,
        y_km: f64,
        z_km: f64,
        vx_km_s: f64,
        vy_km_s: f64,
        vz_km_s: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> Self {
        Self::new(x_km, y_km, z_km, vx_km_s, vy_km_s, vz_km_s, epoch, frame)
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

    /// Returns a copy of this state where the position and velocity are set to the input vector whose units must be [km, km, km, km/s, km/s, km/s]
    pub fn with_cartesian_pos_vel(self, pos_vel: Vector6<f64>) -> Self {
        let mut me = self;
        let radius_km = pos_vel.fixed_rows::<3>(0).into_owned();
        let velocity_km_s = pos_vel.fixed_rows::<3>(3).into_owned();
        me.radius_km = radius_km;
        me.velocity_km_s = velocity_km_s;
        me
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
        perp_vector(&self.velocity_km_s, &self.r_hat()) / self.rmag_km()
    }

    /// Adds the other state to this state WITHOUT checking if the frames match.
    pub(crate) fn add_unchecked(&self, other: &Self) -> Self {
        Self {
            radius_km: self.radius_km + other.radius_km,
            velocity_km_s: self.velocity_km_s + other.velocity_km_s,
            epoch: self.epoch,
            frame: self.frame,
        }
    }

    /// Subs the other state to this state WITHOUT checking if the frames match.
    pub(crate) fn sub_unchecked(&self, other: &Self) -> Self {
        Self {
            radius_km: self.radius_km - other.radius_km,
            velocity_km_s: self.velocity_km_s - other.velocity_km_s,
            epoch: self.epoch,
            frame: self.frame,
        }
    }

    /// Adds the provided delta-v (in km/s) to the current velocity vector, mimicking an impulsive maneuver.
    pub fn apply_dv_km_s(&mut self, dv_km_s: Vector3) {
        self.velocity_km_s += dv_km_s;
    }

    /// Copies this orbit after adding the provided delta-v (in km/s) to the velocity vector, mimicking an impulsive maneuver.
    pub fn with_dv_km_s(&self, dv_km_s: Vector3) -> Self {
        let mut me = *self;
        me.apply_dv_km_s(dv_km_s);
        me
    }

    /// Returns True if velocity (dynamics) are defined.
    /// Which may not be the case if [Self] was built with [Self::from_position]
    /// and you only intend to use partial rotations.
    pub fn has_velocity_dynamics(&self) -> bool {
        self.velocity_km_s.norm() > 0.0
    }
}

// Methods shared with Python
#[cfg_attr(feature = "python", pymethods)]
impl CartesianState {
    /// Returns the magnitude of the radius vector in km
    ///
    /// :rtype: float
    pub fn rmag_km(&self) -> f64 {
        self.radius_km.norm()
    }

    /// Returns the magnitude of the velocity vector in km/s
    ///
    /// :rtype: float
    pub fn vmag_km_s(&self) -> f64 {
        self.velocity_km_s.norm()
    }

    /// Returns the distance in kilometers between this state and another state, if both frame match (epoch does not need to match).
    ///
    /// :type other: Orbit
    /// :rtype: float
    pub fn distance_to_km(&self, other: &Self) -> PhysicsResult<f64> {
        ensure!(
            self.frame.ephem_origin_match(other.frame)
                && self.frame.orient_origin_match(other.frame),
            FrameMismatchSnafu {
                action: "computing distance between states",
                frame1: self.frame,
                frame2: other.frame
            }
        );

        Ok(self.distance_to_point_km(&other.radius_km))
    }

    /// Returns the root mean squared (RSS) radius difference between this state and another state, if both frames match (epoch does not need to match)
    ///
    /// :type other: Orbit
    /// :rtype: float
    pub fn rss_radius_km(&self, other: &Self) -> PhysicsResult<f64> {
        ensure!(
            self.frame.ephem_origin_match(other.frame)
                && self.frame.orient_origin_match(other.frame),
            FrameMismatchSnafu {
                action: "computing radius RSS",
                frame1: self.frame,
                frame2: other.frame
            }
        );
        Ok(root_sum_squared(&self.radius_km, &other.radius_km))
    }

    /// Returns the root mean squared (RSS) velocity difference between this state and another state, if both frames match (epoch does not need to match)
    ///
    /// :type other: Orbit
    /// :rtype: float
    pub fn rss_velocity_km_s(&self, other: &Self) -> PhysicsResult<f64> {
        ensure!(
            self.frame.ephem_origin_match(other.frame)
                && self.frame.orient_origin_match(other.frame),
            FrameMismatchSnafu {
                action: "computing velocity RSS",
                frame1: self.frame,
                frame2: other.frame
            }
        );
        Ok(root_sum_squared(&self.velocity_km_s, &other.velocity_km_s))
    }

    /// Returns the root sum squared (RMS) radius difference between this state and another state, if both frames match (epoch does not need to match)
    ///
    /// :type other: Orbit
    /// :rtype: float
    pub fn rms_radius_km(&self, other: &Self) -> PhysicsResult<f64> {
        ensure!(
            self.frame.ephem_origin_match(other.frame)
                && self.frame.orient_origin_match(other.frame),
            FrameMismatchSnafu {
                action: "computing radius RSS",
                frame1: self.frame,
                frame2: other.frame
            }
        );
        Ok(root_mean_squared(&self.radius_km, &other.radius_km))
    }

    /// Returns the root sum squared (RMS) velocity difference between this state and another state, if both frames match (epoch does not need to match)
    ///
    /// :type other: Orbit
    /// :rtype: float
    pub fn rms_velocity_km_s(&self, other: &Self) -> PhysicsResult<f64> {
        ensure!(
            self.frame.ephem_origin_match(other.frame)
                && self.frame.orient_origin_match(other.frame),
            FrameMismatchSnafu {
                action: "computing velocity RSS",
                frame1: self.frame,
                frame2: other.frame
            }
        );
        Ok(root_mean_squared(&self.velocity_km_s, &other.velocity_km_s))
    }

    /// Returns whether this orbit and another are equal within the specified radial and velocity absolute tolerances
    ///
    /// :type other: Orbit
    /// :type radial_tol_km: float
    /// :type velocity_tol_km_s: float
    /// :rtype: bool
    pub fn eq_within(&self, other: &Self, radial_tol_km: f64, velocity_tol_km_s: f64) -> bool {
        self.epoch == other.epoch
            && (self.radius_km.x - other.radius_km.x).abs() < radial_tol_km
            && (self.radius_km.y - other.radius_km.y).abs() < radial_tol_km
            && (self.radius_km.z - other.radius_km.z).abs() < radial_tol_km
            && (self.velocity_km_s.x - other.velocity_km_s.x).abs() < velocity_tol_km_s
            && (self.velocity_km_s.y - other.velocity_km_s.y).abs() < velocity_tol_km_s
            && (self.velocity_km_s.z - other.velocity_km_s.z).abs() < velocity_tol_km_s
            && self.frame.ephem_origin_match(other.frame)
            && self.frame.orient_origin_match(other.frame)
    }

    /// Returns the light time duration between this object and the origin of its reference frame.
    ///
    /// :rtype: Duration
    pub fn light_time(&self) -> Duration {
        (self.radius_km.norm() / SPEED_OF_LIGHT_KM_S).seconds()
    }

    /// Returns the absolute position difference in kilometer between this orbit and another.
    /// Raises an error if the frames do not match (epochs do not need to match).
    ///
    /// :type other: Orbit
    /// :rtype: float
    pub fn abs_pos_diff_km(&self, other: &Self) -> PhysicsResult<f64> {
        ensure!(
            self.frame.ephem_origin_match(other.frame)
                && self.frame.orient_origin_match(other.frame),
            FrameMismatchSnafu {
                action: "computing velocity RSS",
                frame1: self.frame,
                frame2: other.frame
            }
        );

        Ok((self.radius_km - other.radius_km).norm())
    }

    /// Returns the absolute velocity difference in kilometer per second between this orbit and another.
    /// Raises an error if the frames do not match (epochs do not need to match).
    ///
    /// :type other: Orbit
    /// :rtype: float
    pub fn abs_vel_diff_km_s(&self, other: &Self) -> PhysicsResult<f64> {
        ensure!(
            self.frame.ephem_origin_match(other.frame)
                && self.frame.orient_origin_match(other.frame),
            FrameMismatchSnafu {
                action: "computing velocity RSS",
                frame1: self.frame,
                frame2: other.frame
            }
        );

        Ok((self.velocity_km_s - other.velocity_km_s).norm())
    }

    /// Returns the absolute position and velocity differences in km and km/s between this orbit and another.
    /// Raises an error if the frames do not match (epochs do not need to match).
    ///
    /// :type other: Orbit
    /// :rtype: typing.Tuple
    pub fn abs_difference(&self, other: &Self) -> PhysicsResult<(f64, f64)> {
        Ok((self.abs_pos_diff_km(other)?, self.abs_vel_diff_km_s(other)?))
    }

    /// Returns the relative position difference (unitless) between this orbit and another.
    /// This is computed by dividing the absolute difference by the norm of this object's radius vector.
    /// If the radius is zero, this function raises a math error.
    /// Raises an error if the frames do not match or  (epochs do not need to match).
    ///
    /// :type other: Orbit
    /// :rtype: float
    pub fn rel_pos_diff(&self, other: &Self) -> PhysicsResult<f64> {
        if self.rmag_km() <= f64::EPSILON {
            return Err(PhysicsError::AppliedMath {
                source: MathError::DivisionByZero {
                    action: "computing relative position difference",
                },
            });
        }

        Ok(self.abs_pos_diff_km(other)? / self.rmag_km())
    }

    /// Returns the absolute velocity difference in kilometer per second between this orbit and another.
    /// Raises an error if the frames do not match (epochs do not need to match).
    ///
    /// :type other: Orbit
    /// :rtype: float
    pub fn rel_vel_diff(&self, other: &Self) -> PhysicsResult<f64> {
        if self.vmag_km_s() <= f64::EPSILON {
            return Err(PhysicsError::AppliedMath {
                source: MathError::DivisionByZero {
                    action: "computing relative velocity difference",
                },
            });
        }

        Ok(self.abs_vel_diff_km_s(other)? / self.vmag_km_s())
    }

    /// Returns the relative difference between this orbit and another for the position and velocity, respectively the first and second return values.
    /// Both return values are UNITLESS because the relative difference is computed as the absolute difference divided by the rmag and vmag of this object.
    /// Raises an error if the frames do not match, if the position is zero or the velocity is zero.
    ///
    /// :type other: Orbit
    /// :rtype: typing.Tuple
    pub fn rel_difference(&self, other: &Self) -> PhysicsResult<(f64, f64)> {
        Ok((self.rel_pos_diff(other)?, self.rel_vel_diff(other)?))
    }
}

impl Add for CartesianState {
    type Output = Result<CartesianState, PhysicsError>;

    /// Adds one state to another. This will return an error if the epochs or frames are different.
    fn add(self, other: CartesianState) -> Self::Output {
        ensure!(
            self.epoch == other.epoch,
            EpochMismatchSnafu {
                action: "adding states",
                epoch1: self.epoch,
                epoch2: other.epoch
            }
        );

        ensure!(
            self.frame.ephemeris_id == other.frame.ephemeris_id,
            FrameMismatchSnafu {
                action: "adding states",
                frame1: self.frame,
                frame2: other.frame
            }
        );

        Ok(self.add_unchecked(&other))
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

impl Sub for CartesianState {
    type Output = Result<CartesianState, PhysicsError>;

    /// Adds one state to another. This will return an error if the epochs or frames are different.
    fn sub(self, other: CartesianState) -> Self::Output {
        ensure!(
            self.epoch == other.epoch,
            EpochMismatchSnafu {
                action: "subtracting states",
                epoch1: self.epoch,
                epoch2: other.epoch
            }
        );

        ensure!(
            self.frame.ephemeris_id == other.frame.ephemeris_id,
            FrameMismatchSnafu {
                action: "subtracting states",
                frame1: self.frame,
                frame2: other.frame
            }
        );

        Ok(self.sub_unchecked(&other))
    }
}

impl Neg for CartesianState {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let mut me = self;
        me.radius_km = -me.radius_km;
        me.velocity_km_s = -me.velocity_km_s;
        me
    }
}

#[allow(clippy::format_in_format_args)]
impl fmt::Display for CartesianState {
    // Prints as Cartesian in floating point with units
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let decimals = f.precision().unwrap_or(6);
        write!(
            f,
            "[{:x}] {}\tposition = [{}, {}, {}] km\tvelocity = [{}, {}, {}] km/s",
            self.frame,
            self.epoch,
            format!("{:.*}", decimals, self.radius_km.x),
            format!("{:.*}", decimals, self.radius_km.y),
            format!("{:.*}", decimals, self.radius_km.z),
            format!("{:.*}", decimals, self.velocity_km_s.x),
            format!("{:.*}", decimals, self.velocity_km_s.y),
            format!("{:.*}", decimals, self.velocity_km_s.z)
        )
    }
}

#[allow(clippy::format_in_format_args)]
impl fmt::LowerExp for CartesianState {
    // Prints as Cartesian in scientific notation with units
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let decimals = f.precision().unwrap_or(6);
        write!(
            f,
            "[{:x}] {}\tposition = [{}, {}, {}] km\tvelocity = [{}, {}, {}] km/s",
            self.frame,
            self.epoch,
            format!("{:.*e}", decimals, self.radius_km.x),
            format!("{:.*e}", decimals, self.radius_km.y),
            format!("{:.*e}", decimals, self.radius_km.z),
            format!("{:.*e}", decimals, self.velocity_km_s.x),
            format!("{:.*e}", decimals, self.velocity_km_s.y),
            format!("{:.*e}", decimals, self.velocity_km_s.z)
        )
    }
}

#[cfg(test)]
mod cartesian_state_ut {

    use hifitime::{Duration, Epoch, TimeUnits};

    use crate::constants::frames::{EARTH_J2000, VENUS_J2000};
    use crate::errors::PhysicsError;
    use crate::math::Vector6;

    use super::CartesianState;

    #[test]
    fn add_wrong_epoch() {
        let e = Epoch::now().unwrap();
        let e2 = e + 1.seconds();
        let frame = EARTH_J2000;
        let s1 = CartesianState::new(10.0, 20.0, 30.0, 1.0, 2.0, 2.0, e, frame);
        let s2 = CartesianState::new(10.0, 20.0, 30.0, 1.0, 2.0, 2.0, e2, frame);

        assert_eq!(
            s1 + s2,
            Err(PhysicsError::EpochMismatch {
                action: "adding states",
                epoch1: e,
                epoch2: e2,
            })
        )
    }

    #[test]
    fn add_wrong_frame() {
        let e = Epoch::now().unwrap();
        let frame = EARTH_J2000;
        let frame2 = VENUS_J2000;
        let s1 = CartesianState::new(10.0, 20.0, 30.0, 1.0, 2.0, 2.0, e, frame);
        let s2 = CartesianState::new(10.0, 20.0, 30.0, 1.0, 2.0, 2.0, e, frame2);

        assert_eq!(
            s1 + s2,
            Err(PhysicsError::FrameMismatch {
                action: "adding states",
                frame1: frame.into(),
                frame2: frame2.into(),
            })
        )
    }

    #[test]
    fn add_nominal() {
        let e = Epoch::now().unwrap();
        let frame = EARTH_J2000;
        let s1 = CartesianState::new(10.0, 20.0, 30.0, 1.0, 2.0, 2.0, e, frame);
        let s2 = CartesianState::new(10.0, 20.0, 30.0, 1.0, 2.0, 2.0, e, frame);
        let s3 = CartesianState::new(20.0, 40.0, 60.0, 2.0, 4.0, 4.0, e, frame);

        assert_eq!(s1 + s2, Ok(s3));

        assert_eq!(format!("{s1}"), format!("[Earth J2000] {e}\tposition = [10.000000, 20.000000, 30.000000] km\tvelocity = [1.000000, 2.000000, 2.000000] km/s"));
        assert_eq!(format!("{s1:e}"), format!("[Earth J2000] {e}\tposition = [1.000000e1, 2.000000e1, 3.000000e1] km\tvelocity = [1.000000e0, 2.000000e0, 2.000000e0] km/s"));
    }

    #[test]
    fn distance() {
        let e = Epoch::now().unwrap();
        let frame = EARTH_J2000;
        let s1 = CartesianState::new(10.0, 20.0, 30.0, 1.0, 2.0, 2.0, e, frame);
        let s2 = CartesianState::new(10.0, 20.0, 30.0, 1.0, 2.0, 2.0, e, frame);

        assert!(s1.distance_to_km(&s2).unwrap().abs() < f64::EPSILON);

        let as_vec6 = Vector6::new(10.0, 20.0, 30.0, 1.0, 2.0, 2.0);
        assert_eq!(s1.to_cartesian_pos_vel(), as_vec6);

        assert_eq!(
            CartesianState::from_cartesian_pos_vel(as_vec6, e, frame),
            s1
        );
    }

    #[test]
    fn zeros() {
        let e = Epoch::now().unwrap();
        let frame = EARTH_J2000;
        let s = CartesianState::zero(frame);

        // We cannot call the orbital momentum magnitude if the radius is zero.
        assert!(s.hmag().is_err());

        let s = CartesianState::zero_at_epoch(e, frame);
        assert!(s.hmag().is_err());

        assert_eq!(s.light_time(), Duration::ZERO);
    }

    #[test]
    fn test_serde() {
        let e = Epoch::now().unwrap();
        let frame = EARTH_J2000;
        let state = CartesianState::new(10.0, 20.0, 30.0, 1.0, 2.0, 2.0, e, frame);

        let serialized = serde_yaml::to_string(&state).unwrap();
        let rtn: CartesianState = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(rtn, state);
    }
}
