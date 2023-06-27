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

use super::Quaternion;

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

    /// Try to convert a quaternion into its MRP representation
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
    use core::f64::consts::TAU;

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
}
