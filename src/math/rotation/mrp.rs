/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{
    math::{Matrix3, Vector3},
    prelude::AniseError,
    NaifId,
};
use core::f64::EPSILON;

use super::{Quaternion, EPSILON_RAD};

/// Represents the orientation of a rigid body in three-dimensional space using Modified Rodrigues Parameters (MRP).
///
/// Modified Rodrigues Parameters (MRP) are a set of three parameters `s0`, `s1`, and `s2` used for representing
/// the attitude of a rigid body. They are an alternative to quaternions (Euler parameters) and are particularly
/// useful for avoiding singularities and providing a smooth representation of rotations.
///
/// MRPs are related to the quaternion parameters `q` by:
/// s0 = q1 / (1 + q0)
/// s1 = q2 / (1 + q0)
/// s2 = q3 / (1 + q0)
///
/// # Shadow MRP
///
/// For any rotation, there are two sets of MRPs that can represent it: the regular MRPs and the shadow MRPs.
/// The shadow MRP is defined as `-s / (s0^2 + s1^2 + s2^2)`, where `s` is the vector of MRPs `[s0, s1, s2]`.
///
/// # Applications
///
/// In the context of spacecraft attitude determination and control, MRPs are often used because they provide
/// a numerically stable way to represent the attitude of a spacecraft without the singularities that are present
/// with Euler angles and with a minimal set of parameters.
#[derive(Copy, Clone, Debug)]
pub struct MRP {
    pub s0: f64,
    pub s1: f64,
    pub s2: f64,
    pub from: NaifId,
    pub to: NaifId,
}

impl MRP {
    /// Returns the norm of this Euler Parameter as a scalar.
    pub(crate) fn scalar_norm(&self) -> f64 {
        (self.s0 * self.s0 + self.s1 * self.s1 + self.s2 * self.s2).sqrt()
    }

    /// Computes the shadow MRP.
    ///
    /// # Returns
    ///
    /// The shadow MRP as a new instance of `MRP`.
    pub fn shadow(&self) -> Result<Self, AniseError> {
        let s_squared = self.s0 * self.s0 + self.s1 * self.s1 + self.s2 * self.s2;
        if s_squared < EPSILON {
            Err(AniseError::MathError(
                crate::errors::MathErrorKind::DivisionByZero,
            ))
        } else {
            Ok(MRP {
                s0: -self.s0 / s_squared,
                s1: -self.s1 / s_squared,
                s2: -self.s2 / s_squared,
                from: self.from,
                to: self.to,
            })
        }
    }

    /// Returns whether this MRP is singular.
    pub fn is_singular(&self) -> bool {
        self.scalar_norm() < EPSILON
    }

    /// If the norm of this MRP is greater than max_norm then this MRP is set to its shadow set
    pub fn normalize(&mut self, max_norm: f64) {
        if self.scalar_norm() >= max_norm {
            *self = self.shadow().unwrap();
        }
    }

    /// Returns the principal rotation vector and the angle in radians
    ///
    /// # Note
    /// If the MRP is singular, this returns an angle of zero and a vector of zero.
    pub fn prv_angle(&self) -> (Vector3, f64) {
        match Quaternion::try_from(*self) {
            Ok(q) => q.prv_angle(),
            Err(_) => (Vector3::zeros(), 0.0),
        }
    }

    /// Returns the data of this MRP as a vector, simplifies lots of computations
    /// but at the cost of losing frame information.
    pub(crate) fn as_vector(&self) -> Vector3 {
        Vector3::new(self.s0, self.s1, self.s2)
    }

    /// Returns the 3x3 matrix which relates the body angular velocity vector w to the derivative of this MRP.
    /// dQ/dt = 1/4 [B(Q)] w
    pub fn b_matrix(&self) -> Matrix3 {
        let mut b = Matrix3::zeros();
        let s2 = self.scalar_norm();
        let q = self.as_vector();

        b[(0, 0)] = 1.0 - s2 + 2.0 * q[0] * q[0];
        b[(0, 1)] = 2.0 * (q[0] * q[1] - q[2]);
        b[(0, 2)] = 2.0 * (q[0] * q[2] + q[1]);
        b[(1, 0)] = 2.0 * (q[1] * q[0] + q[2]);
        b[(1, 1)] = 1.0 - s2 + 2.0 * q[1] * q[1];
        b[(1, 2)] = 2.0 * (q[1] * q[2] - q[0]);
        b[(2, 0)] = 2.0 * (q[2] * q[0] - q[1]);
        b[(2, 1)] = 2.0 * (q[2] * q[1] + q[0]);
        b[(2, 2)] = 1.0 - s2 + 2.0 * q[2] * q[2];

        b
    }

    /// Returns the MRP derivative for this MRP and body angular velocity vector w.
    /// dQ/dt = 1/4 [B(Q)] w
    pub fn derivative(&self, w: Vector3) -> MRP {
        let s = 0.25 * self.b_matrix() * w;

        MRP {
            s0: s[0],
            s1: s[1],
            s2: s[2],
            from: self.from,
            to: self.to,
        }
    }
}

impl PartialEq for MRP {
    fn eq(&self, other: &Self) -> bool {
        let (self_prv, self_angle) = self.prv_angle();
        let (other_prv, other_angle) = other.prv_angle();
        (self_angle - other_angle).abs() < EPSILON_RAD
            && self_prv.dot(&other_prv).acos() < EPSILON_RAD
    }
}

impl TryFrom<Quaternion> for MRP {
    type Error = AniseError;

    /// Try to convert a quaternion into its MRP representation
    ///
    /// # Failure cases
    /// + A zero rotation, as the associated MRP is singular
    fn try_from(q: Quaternion) -> Result<Self, Self::Error> {
        if (1.0 + q.w).abs() < EPSILON {
            Err(AniseError::MathError(
                crate::errors::MathErrorKind::DivisionByZero,
            ))
        } else {
            Ok(Self {
                from: q.from,
                to: q.to,
                s0: q.x / (1.0 + q.w),
                s1: q.y / (1.0 + q.w),
                s2: q.z / (1.0 + q.w),
            })
        }
    }
}

impl TryFrom<MRP> for Quaternion {
    type Error = AniseError;

    /// Try to convert from an MRP into its quaternion representation
    ///
    /// # Failure cases
    /// + A rotation of +/- tau, as the associated MRP is singular
    fn try_from(s: MRP) -> Result<Self, Self::Error> {
        let qm = s.scalar_norm();
        let ps = 1.0 + qm * qm;
        Ok(Quaternion {
            w: (1.0 - qm * qm) / ps,
            x: 2.0 * s.s0 / ps,
            y: 2.0 * s.s1 / ps,
            z: 2.0 * s.s2 / ps,
            from: s.from,
            to: s.to,
        })
    }
}

#[cfg(test)]
mod ut_mrp {
    use super::{Quaternion, MRP};
    use core::f64::consts::{PI, TAU};

    #[test]
    fn test_singular() {
        let q = Quaternion::about_x(TAU, 0, 1);

        assert!(MRP::try_from(q).is_err());

        let q = Quaternion::about_x(-TAU, 0, 1);

        assert!(MRP::try_from(q).is_err());

        let s = MRP {
            s0: 0.0,
            s1: 0.0,
            s2: 0.0,
            from: 0,
            to: 1,
        };

        assert!(s.is_singular());

        assert!(s.shadow().is_err());
    }

    #[test]
    fn test_shadow_set() {
        let m = MRP::try_from(Quaternion::about_y(PI, 0, 1)).unwrap();
        let shadow_m = m.shadow().unwrap();
        assert_eq!(shadow_m.shadow().unwrap(), m);
    }

    #[test]
    fn test_reciprocity() {
        let q = Quaternion::about_x(PI, 0, 1);

        let m = MRP::try_from(q).unwrap();

        let q_back = Quaternion::try_from(m).unwrap();

        // TODO: Redefine equality to be within a very small angle.
        assert_eq!(q_back, q);
    }
}
