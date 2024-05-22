/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::errors::{InvalidRotationSnafu, PhysicsError};
use crate::math::rotation::EPSILON;
use crate::structure::dataset::DataSetT;
use crate::{math::Vector3, math::Vector4, NaifId};
use core::fmt;
use core::ops::Mul;
use der::{Decode, Encode, Reader, Writer};
use nalgebra::Matrix4x3;
use snafu::ensure;

use super::EPSILON_RAD;

/// Quaternion will always be a unit quaternion in ANISE, cf. EulerParameter.
///
/// In ANISE, Quaternions use exclusively the Hamiltonian convenstion.
pub type Quaternion = EulerParameter;

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
#[derive(Clone, Copy, Debug)]
pub struct EulerParameter {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub from: NaifId,
    pub to: NaifId,
}

impl EulerParameter {
    pub const fn identity(from: NaifId, to: NaifId) -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            from,
            to,
        }
    }

    /// Creates a new Euler Parameter and ensures that it's the short rotation
    pub fn new(w: f64, x: f64, y: f64, z: f64, from: NaifId, to: NaifId) -> Self {
        Self {
            w,
            x,
            y,
            z,
            from,
            to,
        }
        .normalize()
    }

    /// Returns true if the quaternion represents a rotation of zero radians
    pub fn is_zero(&self) -> bool {
        self.w.abs() < EPSILON || (1.0 - self.w.abs()) < EPSILON
    }

    /// Creates an Euler Parameter representing the short way rotation around the X (R1) axis.
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
        .normalize()
    }

    /// Creates an Euler Parameter representing the short way rotation around the Y (R2) axis.
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
        .normalize()
    }

    /// Creates an Euler Parameter representing the short way rotation around the Z (R3) axis.
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
        .normalize()
    }

    /// Returns the norm of this Euler Parameter as a scalar.
    pub(crate) fn scalar_norm(&self) -> f64 {
        (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalize the quaternion.
    pub fn normalize(&self) -> Self {
        let norm = self.scalar_norm();
        let mut me = *self;
        me.w /= norm;
        me.x /= norm;
        me.y /= norm;
        me.z /= norm;
        me
    }

    /// Returns the short way rotation of this quaternion
    pub fn short(&self) -> Self {
        if self.w < 0.0 {
            // TODO: Check that this is correct.
            let mut me = *self;
            me.w *= -1.0;
            me
        } else {
            *self
        }
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

    /// Returns the 4x3 matrix which relates the body angular velocity vector w to the derivative of this Euler Parameter.
    /// dQ/dt = 1/2 [B(Q)] w
    pub fn b_matrix(&self) -> Matrix4x3<f64> {
        Matrix4x3::new(
            -self.x, -self.y, -self.z, self.w, -self.z, self.y, self.z, self.w, -self.x, -self.y,
            self.x, self.w,
        )
    }

    /// Returns the euler parameter derivative for this Euler parameter and body angular velocity vector w.
    /// dQ/dt = 1/2 [B(Q)] w
    pub fn derivative(&self, w: Vector3) -> Self {
        let q = 0.25 * self.b_matrix() * w;

        Self {
            w: q[0],
            x: q[1],
            y: q[2],
            z: q[3],
            from: self.from,
            to: self.to,
        }
    }

    /// Returns the principal line of rotation (a unit vector) and the angle of rotation in radians
    pub fn uvec_angle(&self) -> (Vector3, f64) {
        let half_angle_rad = self.w.acos();
        if half_angle_rad.abs() < EPSILON {
            // Prevent divisions by (near) zero
            (Vector3::zeros(), 2.0 * half_angle_rad)
        } else {
            let prv = Vector3::new(self.x, self.y, self.z) / half_angle_rad.sin();

            (prv / prv.norm(), 2.0 * half_angle_rad)
        }
    }

    /// Returns the principal rotation vector representation of this Euler Parameter
    pub fn prv(&self) -> Vector3 {
        let (uvec, angle) = self.uvec_angle();
        angle * uvec
    }

    /// Returns the data of this Euler Parameter as a vector, simplifies lots of computations
    /// but at the cost of losing frame information.
    pub(crate) fn as_vector(&self) -> Vector4 {
        Vector4::new(self.w, self.x, self.y, self.z)
    }
}

impl Mul for Quaternion {
    type Output = Result<Self, PhysicsError>;

    fn mul(self, rhs: Quaternion) -> Self::Output {
        ensure!(
            self.to == rhs.from,
            InvalidRotationSnafu {
                action: "multiply quaternions",
                from1: self.from,
                to1: self.to,
                from2: rhs.from,
                to2: rhs.to
            }
        );

        let s = self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z;
        let i = self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y;
        let j = self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x;
        let k = self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w;

        let (from, to) = if self.to == rhs.from && self.from == rhs.to {
            // Then we don't change the frames
            (self.from, self.to)
        } else {
            (self.from, rhs.to)
        };

        Ok(Quaternion {
            w: s,
            x: i,
            y: j,
            z: k,
            from,
            to,
        })
    }
}

impl Mul for &Quaternion {
    type Output = Result<Quaternion, PhysicsError>;

    fn mul(self, other: &Quaternion) -> Result<Quaternion, PhysicsError> {
        *self * *other
    }
}

impl Mul<Vector3> for Quaternion {
    type Output = Vector3;

    fn mul(self, rhs: Vector3) -> Self::Output {
        let rhs_q = Self::new(0.0, rhs.x, rhs.y, rhs.z, self.from, self.to);

        let q_rot = ((self.conjugate() * rhs_q).unwrap() * self)
            .unwrap()
            .as_vector();

        Vector3::new(q_rot[1], q_rot[2], q_rot[3])
    }
}

impl PartialEq for Quaternion {
    fn eq(&self, other: &Self) -> bool {
        if self.to == other.to && self.from == other.from {
            if (self.w - other.w).abs() < 1e-12 && (self.w - 1.0).abs() < 1e-12 {
                true
            } else {
                let (self_uvec, self_angle) = self.uvec_angle();
                let (other_uvec, other_angle) = other.uvec_angle();

                (self_angle - other_angle).abs() < EPSILON_RAD
                    && (self_uvec - other_uvec).norm() <= 1e-12
            }
        } else {
            false
        }
    }
}

impl fmt::Display for EulerParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Euler Parameter {} -> {} = [w = {:1.6}, {:1.6}, {:1.6}, {:1.6}]",
            self.from, self.to, self.w, self.x, self.y, self.z
        )
    }
}

impl Default for EulerParameter {
    fn default() -> Self {
        Self::identity(0, 0)
    }
}

impl<'a> Decode<'a> for EulerParameter {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let from = decoder.decode()?;
        let to = decoder.decode()?;
        let w = decoder.decode()?;
        let x = decoder.decode()?;
        let y = decoder.decode()?;
        let z = decoder.decode()?;

        Ok(Self {
            w,
            x,
            y,
            z,
            from,
            to,
        })
    }
}

impl Encode for EulerParameter {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.from.encoded_len()?
            + self.to.encoded_len()?
            + self.w.encoded_len()?
            + self.x.encoded_len()?
            + self.y.encoded_len()?
            + self.z.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.from.encode(encoder)?;
        self.to.encode(encoder)?;
        self.w.encode(encoder)?;
        self.x.encode(encoder)?;
        self.y.encode(encoder)?;
        self.z.encode(encoder)
    }
}

impl DataSetT for EulerParameter {
    const NAME: &'static str = "euler parameter";
}

#[cfg(test)]
mod ut_quaternion {
    use crate::math::{
        rotation::{generate_angles, vec3_eq, DCM},
        Vector4,
    };

    use super::{EulerParameter, Quaternion, Vector3, EPSILON};
    use core::f64::consts::{FRAC_PI_2, PI};

    #[test]
    fn test_quat_frames() {
        // Ensure that we cannot compose two rotations when the frames don't match.
        // We are using arbitrary numbers for the frames

        for angle in generate_angles() {
            for (i, q) in [
                Quaternion::about_x(angle, 0, 1),
                Quaternion::about_y(angle, 0, 1),
                Quaternion::about_z(angle, 0, 1),
            ]
            .iter()
            .enumerate()
            {
                assert!((q * q).is_err());
                assert!((q * &q.conjugate()).is_ok());
                assert_eq!(
                    (q * &q.conjugate()).unwrap(),
                    Quaternion::identity(0, 1),
                    "axis {i} and {angle}"
                );
                // Check that the PRV is entirely in the appropriate direction
                let prv = q.prv();

                // The i-th index should be equal to the angle input
                assert!((prv[i] - angle).abs() < EPSILON, "{} with {angle}", prv[i]);
                // The overall norm should be PI, i.e. all other components are zero.
                assert!(
                    (prv.norm() - angle.abs()).abs() < EPSILON,
                    "{prv} with {angle}"
                );
            }
        }
    }

    #[test]
    fn test_quat_start_end_frames() {
        for angle in generate_angles() {
            let q1 = Quaternion::about_x(angle, 0, 1);
            let (uvec_q1, _angle_rad) = q1.uvec_angle();
            let q2 = Quaternion::about_x(angle, 1, 2);

            let q1_to_q2 = (q1 * q2).unwrap();
            assert_eq!(q1_to_q2.from, 0, "{angle}");
            assert_eq!(q1_to_q2.to, 2, "{angle}");

            let (uvec, angle_rad) = q1_to_q2.uvec_angle();

            if uvec.norm() > EPSILON {
                if !(-PI..=PI).contains(&angle) {
                    assert_eq!(uvec, -uvec_q1, "{angle}");
                } else {
                    assert_eq!(uvec, uvec_q1, "{angle}");
                    let cmp_angle = (2.0 * angle).abs();
                    assert!(
                        (angle_rad - cmp_angle).abs() < 1e-12,
                        "got: {angle_rad}\twant: {cmp_angle} (orig: {angle})"
                    );
                }
            }

            // Check the conjugate

            let q2_to_q1 = (q2.conjugate() * q1.conjugate()).unwrap();
            assert_eq!(q2_to_q1.from, 2, "{angle}");
            assert_eq!(q2_to_q1.to, 0, "{angle}");

            let (uvec, _angle_rad) = q2_to_q1.uvec_angle();
            if uvec.norm() > EPSILON {
                if (-PI..=PI).contains(&angle) {
                    assert_eq!(uvec, -uvec_q1, "{angle}");
                } else {
                    assert_eq!(uvec, uvec_q1, "{angle}");
                }
            }
        }
    }

    #[test]
    fn test_zero() {
        let z = EulerParameter::identity(0, 1);
        assert!(z.is_zero());
        // Test that the identity DCM matches.
        let c = DCM::identity(0, 1);
        let q = Quaternion::from(c);
        assert_eq!(c, q.into());
    }

    #[test]
    fn test_derivative_zero_angular_velocity() {
        let euler_params = EulerParameter::identity(0, 1);
        let w = Vector3::new(0.0, 0.0, 0.0);
        let derivative = euler_params.derivative(w);

        // With zero angular velocity, the derivative should be zero
        assert!(derivative.is_zero());
    }

    #[test]
    fn test_dcm_recip() {
        // Test the reciprocity with DCMs
        for angle in generate_angles() {
            let c_x = DCM::r1(angle, 0, 1);
            let q_x = Quaternion::about_x(angle, 0, 1);

            println!("{q_x} for {:.2} deg", angle.to_degrees());

            // Check that rotating X by anything around R1 returns the same regardless of whether we're using the DCM or EP representation
            vec3_eq(
                DCM::from(q_x) * Vector3::x(),
                c_x * Vector3::x(),
                format!("X on {}", angle.to_degrees()),
            );

            vec3_eq(
                q_x * Vector3::x(),
                c_x * Vector3::x(),
                format!("X on {}", angle.to_degrees()),
            );

            // Idem around Y
            vec3_eq(
                q_x * Vector3::y(),
                c_x * Vector3::y(),
                format!("Y on {}", angle.to_degrees()),
            );

            // Idem around Z
            vec3_eq(
                DCM::from(q_x) * Vector3::z(),
                c_x * Vector3::z(),
                format!("Z on {}", angle.to_degrees()),
            );
        }
    }

    #[test]
    fn test_single_axis_rotations() {
        let q_x = Quaternion::about_x(FRAC_PI_2, 0, 1);
        // Check the components
        assert!(
            (q_x.as_vector() - Vector4::new(0.5_f64.sqrt(), 0.5_f64.sqrt(), 0.0, 0.0)).norm()
                < EPSILON
        );
        assert_eq!(q_x * Vector3::x(), Vector3::x());
        // Check that rotating Y by PI /2 about X returns -Z
        let d = DCM::from(q_x);
        assert_eq!(d * Vector3::y(), q_x * Vector3::y());
        assert!((d * Vector3::y() - -Vector3::z()).norm() < 1e-12);
        assert!((q_x * Vector3::y() - -Vector3::z()).norm() < 1e-12);

        let q_y = Quaternion::about_y(FRAC_PI_2, 0, 1);
        assert_eq!(q_y * Vector3::y(), Vector3::y());
        let d = DCM::from(q_y);
        assert_eq!(d * Vector3::z(), q_y * Vector3::z());
        assert!((d * Vector3::x() - Vector3::z()).norm() < 1e-12);
        assert!((q_y * Vector3::x() - Vector3::z()).norm() < 1e-12);

        let q_z = Quaternion::about_z(FRAC_PI_2, 0, 1);
        assert_eq!(q_z * Vector3::z(), Vector3::z());
        let d = DCM::from(q_z);
        assert_eq!(d * Vector3::x(), q_z * Vector3::x());
    }

    // TODO: Add useful tests

    use der::{Decode, Encode};

    #[test]
    fn ep_encdec_min_repr() {
        // A minimal representation of a planetary constant.
        let repr = EulerParameter {
            from: -123,
            to: 345,
            w: 0.1,
            x: 0.2,
            y: 0.2,
            z: 0.2,
        }
        .normalize();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = EulerParameter::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }
}
