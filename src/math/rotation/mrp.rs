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
    /// + A zero rotation, as the associated MRP is singular
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
