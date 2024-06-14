/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use snafu::ensure;

use crate::{
    errors::{DivisionByZeroSnafu, InvalidRotationSnafu, MathError, PhysicsError},
    math::{Matrix3, Vector3},
    NaifId,
};

use core::ops::Mul;

use super::{Quaternion, Rotation};

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

impl Rotation for MRP {}

impl MRP {
    /// Creates a new normalized MRP
    pub fn new(s0: f64, s1: f64, s2: f64, from: NaifId, to: NaifId) -> Self {
        Self {
            s0,
            s1,
            s2,
            from,
            to,
        }
        .normalize()
    }

    /// Returns the norm of this MRP as a scalar.
    pub(crate) fn scalar_norm(&self) -> f64 {
        (self.s0 * self.s0 + self.s1 * self.s1 + self.s2 * self.s2).sqrt()
    }

    /// Returns the square of the norm of this MRP as a scalar.
    pub(crate) fn norm_squared(&self) -> f64 {
        self.s0 * self.s0 + self.s1 * self.s1 + self.s2 * self.s2
    }

    /// Computes the shadow MRP.
    ///
    /// # Returns
    ///
    /// The shadow MRP as a new instance of `MRP`.
    pub fn shadow(&self) -> Result<Self, MathError> {
        ensure!(
            !self.is_singular(),
            DivisionByZeroSnafu {
                action: "cannot compute shadow MRP of a singular MRP"
            }
        );

        let s_squared = self.s0 * self.s0 + self.s1 * self.s1 + self.s2 * self.s2;
        Ok(MRP {
            s0: -self.s0 / s_squared,
            s1: -self.s1 / s_squared,
            s2: -self.s2 / s_squared,
            from: self.from,
            to: self.to,
        })
    }

    /// Returns whether this MRP is singular.
    pub fn is_singular(&self) -> bool {
        self.scalar_norm() < f64::EPSILON
    }

    /// If the norm of this MRP is greater than max_norm then this MRP is set to its shadow set
    pub fn normalize(&self) -> Self {
        if self.scalar_norm() > 1.0 {
            self.shadow().unwrap()
        } else {
            *self
        }
    }

    /// Returns the principal line of rotation (a unit vector) and the angle of rotation in radians
    ///
    /// # Note
    /// If the MRP is singular, this returns an angle of zero and a vector of zero.
    pub fn uvec_angle(&self) -> (Vector3, f64) {
        Quaternion::from(*self).uvec_angle()
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
    pub fn diff_eq(&self, w: Vector3) -> MRP {
        let s = 0.25 * self.b_matrix() * w;

        MRP {
            s0: s[0],
            s1: s[1],
            s2: s[2],
            from: self.from,
            to: self.to,
        }
    }

    /// Returns the relative MRP between self and the rhs MRP.
    pub fn relative_to(&self, rhs: &Self) -> Result<Self, PhysicsError> {
        ensure!(
            self.from == rhs.from,
            InvalidRotationSnafu {
                action: "compute relative MRP",
                from1: self.from,
                to1: self.to,
                from2: rhs.from,
                to2: rhs.to
            }
        );

        // Using the same notation as in Eq. 3.153 in Schaub and Junkins, 3rd edition
        let s_prime = self;
        let s_dprime = rhs;
        let denom = 1.0
            + s_prime.norm_squared() * s_dprime.norm_squared()
            + 2.0 * s_prime.as_vector().dot(&s_dprime.as_vector());
        let num1 = (1.0 - s_prime.norm_squared()) * s_dprime.as_vector();
        let num2 = -(1.0 - s_dprime.norm_squared()) * s_prime.as_vector();
        let num3 = 2.0 * s_dprime.as_vector().cross(&s_prime.as_vector());

        let sigma = (num1 + num2 + num3) / denom;
        Ok(Self::new(sigma[0], sigma[1], sigma[2], rhs.from, self.to))
    }
}

impl PartialEq for MRP {
    /// Equality between two MRPs is whether the frames match and both representations nearly match, or the shadow set of one nearly matches that of the other.
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from
            && self.to == other.to
            && self.is_singular() == other.is_singular()
            && ((self.as_vector() - other.as_vector()).norm() < 1e-12
                || (self.as_vector() - other.shadow().unwrap().as_vector()).norm() < 1e-12)
    }
}

impl Mul for MRP {
    type Output = Result<MRP, PhysicsError>;

    fn mul(self, rhs: Self) -> Self::Output {
        ensure!(
            self.to == rhs.from,
            InvalidRotationSnafu {
                action: "compose MRPs",
                from1: self.from,
                to1: self.to,
                from2: rhs.from,
                to2: rhs.to
            }
        );

        // Using the same notation as in Eq. 3.152 in Schaub and Junkins, 3rd edition
        let s_prime = self;
        let s_dprime = rhs;
        let denom = 1.0 + s_prime.norm_squared() * s_dprime.norm_squared()
            - 2.0 * s_prime.as_vector().dot(&s_dprime.as_vector());
        let num1 = (1.0 - s_prime.norm_squared()) * s_dprime.as_vector();
        let num2 = (1.0 - s_dprime.norm_squared()) * s_prime.as_vector();
        let num3 = -2.0 * s_dprime.as_vector().cross(&s_prime.as_vector());

        let sigma = (num1 + num2 + num3) / denom;
        Ok(Self::new(sigma[0], sigma[1], sigma[2], self.from, rhs.to))
    }
}

impl TryFrom<Quaternion> for MRP {
    type Error = MathError;

    /// Try to convert a quaternion into its MRP representation
    ///
    /// # Failure cases
    /// + A zero rotation, as the associated MRP is singular
    fn try_from(q: Quaternion) -> Result<Self, Self::Error> {
        ensure!(
            (1.0 + q.w).abs() >= f64::EPSILON,
            DivisionByZeroSnafu {
                action: "quaternion represents a zero rotation, which is a singular MRP"
            }
        );

        let s = Self {
            from: q.from,
            to: q.to,
            s0: q.x / (1.0 + q.w),
            s1: q.y / (1.0 + q.w),
            s2: q.z / (1.0 + q.w),
        }
        .normalize();
        // We don't ever want to deal with singular MRPs, so check once more
        ensure!(
            !s.is_singular(),
            DivisionByZeroSnafu {
                action: "MRP from quaternion is singular"
            }
        );
        Ok(s)
    }
}

impl From<MRP> for Quaternion {
    /// Convert from an MRP into its quaternion representation
    fn from(s: MRP) -> Self {
        let qm = s.scalar_norm();
        let ps = 1.0 + qm * qm;
        Quaternion {
            w: (1.0 - qm * qm) / ps,
            x: 2.0 * s.s0 / ps,
            y: 2.0 * s.s1 / ps,
            z: 2.0 * s.s2 / ps,
            from: s.from,
            to: s.to,
        }
        .normalize()
    }
}

#[cfg(test)]
mod ut_mrp {
    use crate::math::rotation::generate_angles;

    use super::{Quaternion, MRP};
    use core::f64::consts::{FRAC_PI_2, PI, TAU};

    #[test]
    fn test_singular() {
        let q = Quaternion::about_x(TAU, 0, 1);
        assert!(MRP::try_from(q).is_err());

        let q = Quaternion::about_x(-TAU, 0, 1);
        assert!(MRP::try_from(q).is_err());

        let q = Quaternion::about_y(TAU, 0, 1);
        assert!(MRP::try_from(q).is_err());

        let q = Quaternion::about_y(-TAU, 0, 1);
        assert!(MRP::try_from(q).is_err());

        let q = Quaternion::about_z(TAU, 0, 1);
        assert!(MRP::try_from(q).is_err());

        let q = Quaternion::about_z(-TAU, 0, 1);
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

        let (uvec, angle) = s.uvec_angle();
        assert_eq!(uvec.norm(), 0.0);
        assert_eq!(angle, 0.0);
    }

    #[test]
    fn test_shadow_set_recip() {
        for angle in generate_angles() {
            let q = Quaternion::about_z(angle, 0, 1);
            if let Ok(m) = MRP::try_from(q) {
                let shadow_m = m.shadow().unwrap();
                assert_eq!(shadow_m.shadow().unwrap(), m);
            }

            let q = Quaternion::about_y(angle, 0, 1);
            if let Ok(m) = MRP::try_from(q) {
                let shadow_m = m.shadow().unwrap();
                assert_eq!(shadow_m.shadow().unwrap(), m);
            }

            let q = Quaternion::about_x(angle, 0, 1);
            if let Ok(m) = MRP::try_from(q) {
                let shadow_m = m.shadow().unwrap();
                assert_eq!(shadow_m.shadow().unwrap(), m);
            }
        }
    }

    #[test]
    fn test_quat_recip() {
        // NOTE: MRPs are always the short way rotation, so we enforce the short way rotation of the EPs as well.
        for angle in generate_angles() {
            let q = Quaternion::about_x(angle, 0, 1).short();
            if let Ok(m) = MRP::try_from(q) {
                let q_back = Quaternion::from(m);
                assert_eq!(q_back, q, "X fail with {angle}");
            }

            let q = Quaternion::about_y(angle, 0, 1).short();
            if let Ok(m) = MRP::try_from(q) {
                let q_back = Quaternion::from(m);
                assert_eq!(q_back, q, "Y fail with {angle}");
            }

            let q = Quaternion::about_z(angle, 0, 1).short();
            if let Ok(m) = MRP::try_from(q) {
                let q_back = Quaternion::from(m);
                assert_eq!(q_back, q, "Z fail with {angle}");
            }
        }
    }

    #[test]
    fn test_composition() {
        let m_x0: MRP = Quaternion::about_x(FRAC_PI_2, 0, 1).try_into().unwrap();
        let m_x1: MRP = Quaternion::about_x(FRAC_PI_2, 1, 2).try_into().unwrap();
        let m_x: MRP = Quaternion::about_x(PI, 0, 2).try_into().unwrap();

        assert_eq!((m_x0 * m_x1).unwrap(), m_x);
        // Check that we can compute the relative rotation
        let mx_rel_x0 = m_x.relative_to(&m_x0).unwrap();
        let rel = Quaternion::about_x(-FRAC_PI_2, 0, 2);

        assert_eq!(rel, mx_rel_x0.into());
        // Also check that if two quaternions are equal, then their MRPs should also be equal
        let rel_mrp: MRP = rel.try_into().unwrap();
        assert_eq!(rel_mrp, mx_rel_x0);
    }
}
