/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{prelude::AniseError, NaifId};
use core::f64::EPSILON;
use core::ops::Mul;
use nalgebra::Matrix4x3;

pub use core::f64::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, PI, TAU};

/// Quaternion will always be a unit quaternion in ANISE, cf. [EulerParameters].
pub type Quaternion = EulerParameters;

/// Represents the orientation of a rigid body in three-dimensional space using Euler parameters.
///
/// Euler parameters, also known as unit quaternions, are a set of four parameters `b0`, `b1`, `b2`, and `b3`.
/// For clarity, in ANISE, these are denoted `w`, `x`, `y`, `z`.
/// They are an extension of the concept of using Euler angles for representing orientations and are
/// particularly useful because they avoid gimbal lock and are more compact than rotation matrices.
///
/// # Definitions
///
/// Euler parameters are defined in terms of the axis of rotation and the angle of rotation. If a body
/// rotates by an angle `θ` about an axis defined by the unit vector `e = [e1, e2, e3]`, the Euler parameters
/// are defined as:
///
/// b0 = cos(θ / 2)
/// b1 = e1 * sin(θ / 2)
/// b2 = e2 * sin(θ / 2)
/// b3 = e3 * sin(θ / 2)
///
/// These parameters have the property that `b0^2 + b1^2 + b2^2 + b3^2 = 1`, which means they represent
/// a rotation in SO(3) and can be used to interpolate rotations smoothly.
///
/// # Applications
///
/// In the context of spacecraft mechanics, Euler parameters are often used because they provide a
/// numerically stable way to represent the attitude of a spacecraft without the singularities that
/// are present with Euler angles.
///
/// # Usage
/// Importantly, ANISE prevents the composition of two Euler Parameters if the frames do not match.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EulerParameters {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub from: NaifId,
    pub to: NaifId,
}

impl EulerParameters {
    pub fn zero(from: NaifId, to: NaifId) -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            from,
            to,
        }
    }

    /// Returns true if the quaternion represents a rotation of zero radians
    pub fn is_zero(&self) -> bool {
        (1.0 - self.w.abs()) < EPSILON
    }

    pub fn about_x(angle_rad: f64, from: NaifId, to: NaifId) -> Self {
        let (s_theta, c_theta) = (angle_rad / 2.0).sin_cos();

        Self {
            w: c_theta,
            x: s_theta,
            y: 0.0,
            z: 0.0,
            from,
            to,
        }
    }

    pub fn about_y(angle_rad: f64, from: NaifId, to: NaifId) -> Self {
        let (s_theta, c_theta) = (angle_rad / 2.0).sin_cos();

        Self {
            w: c_theta,
            x: 0.0,
            y: s_theta,
            z: 0.0,
            from,
            to,
        }
    }

    pub fn about_z(angle_rad: f64, from: NaifId, to: NaifId) -> Self {
        let (s_theta, c_theta) = (angle_rad / 2.0).sin_cos();

        Self {
            w: c_theta,
            x: 0.0,
            y: 0.0,
            z: s_theta,
            from,
            to,
        }
    }

    /// Returns the norm of this Euler Parameter as a scalar.
    pub(crate) fn scalar_norm(&self) -> f64 {
        (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalize the quaternion.
    pub fn normalize(&mut self) {
        let norm = self.scalar_norm();
        self.w /= norm;
        self.x /= norm;
        self.y /= norm;
        self.z /= norm;
    }

    /// Compute the conjugate of the quaternion.
    ///
    /// # Note
    /// Because Euler Parameters are unit quaternions, the inverse and the conjugate are identical.
    pub fn conjugate(&self) -> Self {
        Self {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
            from: self.to,
            to: self.from,
        }
    }

    /// Returns the 4x3 matrix which relates the body angular velocity vector w to the derivative of Euler parameter vector Q.
    /// dQ/dt = 1/2 [B(Q)] w
    pub fn derivative(&self) -> Matrix4x3<f64> {
        Matrix4x3::new(
            -self.x, -self.y, -self.z, self.w, -self.z, self.y, self.z, self.w, -self.x, -self.y,
            self.x, self.w,
        )
    }
}

impl Mul for Quaternion {
    type Output = Result<Quaternion, AniseError>;

    fn mul(self, other: Quaternion) -> Result<Quaternion, AniseError> {
        if self.to != other.from {
            Err(AniseError::IncompatibleRotation {
                from: other.from,
                to: self.to,
            })
        } else {
            let s = self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z;
            let i = self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y;
            let j = self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x;
            let k = self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w;

            Ok(Quaternion {
                w: s,
                x: i,
                y: j,
                z: k,
                from: self.from,
                to: other.to,
            })
        }
    }
}

#[cfg(test)]
mod ut_quaternion {
    use super::{Quaternion, PI};
    #[test]
    fn test_quat_invalid() {
        // Ensure that we cannot compose two rotations when the frames don't match.
        // We are using arbitrary numbers for the frames
        let q1 = Quaternion::about_x(PI, 0, 1);

        assert!((q1 * q1).is_err());
        assert!((q1 * q1.conjugate()).is_ok());
        assert_eq!((q1 * q1.conjugate()).unwrap(), Quaternion::zero(0, 0));
    }
}
