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
    math::{Matrix3, Matrix6, Vector3, Vector6},
    prelude::AniseError,
    NaifId,
};
use nalgebra::Vector4;

use core::ops::Mul;

use super::{Quaternion, Rotation};

#[derive(Copy, Clone, Debug, Default)]
pub struct DCM {
    /// The rotation matrix itself
    pub rot_mat: Matrix3,
    /// The time derivative of the rotation matrix
    pub rot_mat_dt: Option<Matrix3>,
    /// The source frame
    pub from: NaifId,
    /// The destination frame
    pub to: NaifId,
}

impl Rotation for DCM {}

impl DCM {
    /// Returns a rotation matrix for a rotation about the X axis.
    ///
    /// Source: `euler1` function from Baslisk
    /// # Arguments
    ///
    /// * `angle_rad` - The angle of rotation in radians.
    ///
    pub fn r1(angle_rad: f64, from: NaifId, to: NaifId) -> Self {
        let (s, c) = angle_rad.sin_cos();
        let rot_mat = Matrix3::new(1.0, 0.0, 0.0, 0.0, c, s, 0.0, -s, c);
        Self {
            rot_mat,
            from,
            to,
            rot_mat_dt: None,
        }
    }

    /// Returns a rotation matrix for a rotation about the Y axis.
    ///
    /// Source: `euler2` function from Basilisk
    /// # Arguments
    ///
    /// * `angle` - The angle of rotation in radians.
    ///
    pub fn r2(angle_rad: f64, from: NaifId, to: NaifId) -> Self {
        let (s, c) = angle_rad.sin_cos();
        let rot_mat = Matrix3::new(c, 0.0, -s, 0.0, 1.0, 0.0, s, 0.0, c);
        Self {
            rot_mat,
            from,
            to,
            rot_mat_dt: None,
        }
    }

    /// Returns a rotation matrix for a rotation about the Z axis.
    ///
    /// Source: `euler3` function from Basilisk
    /// # Arguments
    ///
    /// * `angle_rad` - The angle of rotation in radians.
    ///
    pub fn r3(angle_rad: f64, from: NaifId, to: NaifId) -> Self {
        let (s, c) = angle_rad.sin_cos();
        let rot_mat = Matrix3::new(c, s, 0.0, -s, c, 0.0, 0.0, 0.0, 1.0);
        Self {
            rot_mat,
            from,
            to,
            rot_mat_dt: None,
        }
    }

    /// Returns the 6x6 DCM to rotate a state, if the time derivative of this DCM exists.
    pub fn state_dcm(&self) -> Result<Matrix6, AniseError> {
        match self.rot_mat_dt {
            Some(mat_dt) => {
                let mut full_dcm = Matrix6::zeros();
                for i in 0..6 {
                    for j in 0..6 {
                        if (i < 3 && j < 3) || (i >= 3 && j >= 3) {
                            full_dcm[(i, j)] = self.rot_mat[(i % 3, j % 3)];
                        } else if i >= 3 && j < 3 {
                            full_dcm[(i, j)] = mat_dt[(i - 3, j)];
                        }
                    }
                }

                Ok(full_dcm)
            }
            None => Err(AniseError::ItemNotFound),
        }
    }

    pub fn identity(from: i32, to: i32) -> Self {
        let rot_mat = Matrix3::identity();

        Self {
            rot_mat,
            from,
            to,
            rot_mat_dt: None,
        }
    }
}

impl Mul<Vector3> for DCM {
    type Output = Vector3;

    /// Applying the matrix to a vector yields the vector's representation relative to the rotated coordinate system.
    ///
    /// # Example
    ///
    /// ```
    /// use anise::math::Vector3;
    /// use anise::math::rotation::DCM;
    /// use core::f64::consts::FRAC_PI_2;
    /// use core::f64::EPSILON;
    ///
    /// let r1 = DCM::r1(FRAC_PI_2, 0, 1);
    ///
    /// // Rotation of the X vector about X, yields X
    /// assert_eq!(r1 * Vector3::x(), Vector3::x());
    /// // Rotation of the Z vector about X by half pi, yields -Y
    /// assert!((r1 * Vector3::z() - Vector3::y()).norm() < EPSILON);
    /// // Rotation of the Y vector about X by half pi, yields Z
    /// assert!((r1 * Vector3::y() + Vector3::z()).norm() < EPSILON);
    /// ```
    ///
    /// # Warnings
    ///
    /// + No frame checks are done when multiplying by a vector
    /// + As a Vector3, this is assumed to be only position, and so the transport theorem is not applied.
    ///
    fn mul(self, rhs: Vector3) -> Self::Output {
        self.rot_mat * rhs
    }
}

impl Mul<Vector6> for DCM {
    type Output = Result<Vector6, AniseError>;

    /// Applying the matrix to a vector yields the vector's representation in the new coordinate system.
    fn mul(self, rhs: Vector6) -> Self::Output {
        Ok(self.state_dcm()? * rhs)
    }
}

impl From<DCM> for Quaternion {
    /// Convert from a DCM into its quaternion representation
    ///
    /// # Warning
    /// If this DCM has a time derivative, it will be lost in the conversion.
    ///
    /// # Failure cases
    /// This conversion cannot fail.
    fn from(dcm: DCM) -> Self {
        // From Basilisk's `C2EP` function
        let c = dcm.rot_mat;
        let tr = c.trace();
        let b2 = Vector4::new(
            (1.0 + tr) / 4.0,
            (1.0 + 2.0 * c[(0, 0)] - tr) / 4.0,
            (1.0 + 2.0 * c[(1, 1)] - tr) / 4.0,
            (1.0 + 2.0 * c[(2, 2)] - tr) / 4.0,
        );
        let (w, x, y, z) = match b2.imax() {
            0 => (
                b2[0].sqrt(),
                (c[(1, 2)] - c[(2, 1)]) / 4.0 / b2[0],
                (c[(2, 0)] - c[(0, 2)]) / 4.0 / b2[0],
                (c[(0, 1)] - c[(1, 0)]) / 4.0 / b2[0],
            ),
            1 => {
                let mut x = b2[1].sqrt();
                let mut w = (c[(1, 2)] - c[(2, 1)]) / 4.0 / b2[1];
                if w < 0.0 {
                    w = -w;
                    x = -x;
                }

                let y = (c[(0, 1)] + c[(1, 0)]) / 4.0 / x;
                let z = (c[(2, 0)] + c[(0, 2)]) / 4.0 / x;

                (w, x, y, z)
            }
            2 => {
                let mut y = b2[2].sqrt();
                let mut w = (c[(2, 0)] - c[(0, 2)]) / 4.0 / b2[2];
                if w < 0.0 {
                    w = -w;
                    y = -y;
                }

                let x = (c[(0, 1)] + c[(1, 0)]) / 4.0 / y;
                let z = (c[(1, 2)] + c[(2, 1)]) / 4.0 / y;

                (w, x, y, z)
            }
            3 => {
                let mut z = b2[3].sqrt();
                let mut w = (c[(0, 1)] - c[(1, 0)]) / 4.0 / b2[3];
                if w < 0.0 {
                    z = -z;
                    w = -w;
                }

                let x = (c[(2, 0)] + c[(0, 2)]) / 4.0 / z;
                let y = (c[(1, 2)] + c[(2, 1)]) / 4.0 / z;

                (w, x, y, z)
            }
            _ => unreachable!(),
        };

        Quaternion::new(w, x, y, z, dcm.from, dcm.to)
    }
}

impl From<Quaternion> for DCM {
    /// Returns the direction cosine matrix in terms of the provided euler parameter
    fn from(q: Quaternion) -> Self {
        let q = q.normalize();
        let q0 = q.w;
        let q1 = q.x;
        let q2 = q.y;
        let q3 = q.z;
        let mut c = Matrix3::zeros();
        c[(0, 0)] = q0 * q0 + q1 * q1 - q2 * q2 - q3 * q3;
        c[(0, 1)] = 2.0 * (q1 * q2 + q0 * q3);
        c[(0, 2)] = 2.0 * (q1 * q3 - q0 * q2);
        c[(1, 0)] = 2.0 * (q1 * q2 - q0 * q3);
        c[(1, 1)] = q0 * q0 - q1 * q1 + q2 * q2 - q3 * q3;
        c[(1, 2)] = 2.0 * (q2 * q3 + q0 * q1);
        c[(2, 0)] = 2.0 * (q1 * q3 + q0 * q2);
        c[(2, 1)] = 2.0 * (q2 * q3 - q0 * q1);
        c[(2, 2)] = q0 * q0 - q1 * q1 - q2 * q2 + q3 * q3;

        Self {
            rot_mat: c,
            rot_mat_dt: None,
            from: q.from,
            to: q.to,
        }
    }
}

impl PartialEq for DCM {
    fn eq(&self, other: &Self) -> bool {
        if (self.rot_mat_dt.is_none() && other.rot_mat_dt.is_some())
            || (self.rot_mat_dt.is_some() && other.rot_mat_dt.is_none())
        {
            false
        } else {
            let rot_mat_match = (self.rot_mat - other.rot_mat).norm() < 1e-1;

            let dt_match = if let Some(self_dt) = self.rot_mat_dt {
                (self_dt - other.rot_mat_dt.unwrap()).norm() < 1e-5
            } else {
                true
            };

            self.from == other.from && self.to == other.to && rot_mat_match && dt_match
        }
    }
}

#[cfg(test)]
mod ut_dcm {
    use crate::math::Matrix3;

    use super::{Vector3, DCM};
    use core::f64::consts::FRAC_PI_2;
    use core::f64::EPSILON;

    #[test]
    fn test_r1() {
        let r1 = DCM::r1(FRAC_PI_2, 0, 1);

        // Rotation of the X vector about X, yields X
        assert_eq!(r1 * Vector3::x(), Vector3::x());
        // Rotation of the Z vector about X by half pi, yields Y
        assert!((r1 * Vector3::z() - Vector3::y()).norm() < EPSILON);
        // Rotation of the Y vector about X by half pi, yields -Z
        assert!((r1 * Vector3::y() + Vector3::z()).norm() < EPSILON);

        assert!(
            (r1.rot_mat - Matrix3::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0)).norm()
                < EPSILON
        );
    }

    #[test]
    fn test_r2() {
        let r2 = DCM::r2(FRAC_PI_2, 0, 1);

        // Rotation of the Y vector about Y, yields Y
        assert_eq!(r2 * Vector3::y(), Vector3::y());
        // Rotation of the X vector about Y by -half pi, yields Z
        assert!((r2 * Vector3::x() - Vector3::z()).norm() < EPSILON);
        // Rotation of the Z vector about Y by -half pi, yields -X
        assert!((r2 * Vector3::z() + Vector3::x()).norm() < EPSILON);

        // Edge case: Rotation by 0 degrees should yield the original vector
        let r2_zero = DCM::r2(0.0, 0, 1);
        assert!((r2_zero * Vector3::x() - Vector3::x()).norm() < EPSILON);

        assert!(
            (r2.rot_mat - Matrix3::new(0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0)).norm()
                < EPSILON
        );
    }

    #[test]
    fn test_r3() {
        let r3 = DCM::r3(FRAC_PI_2, 0, 1);

        // Rotation of the Z vector about Z, yields Z
        assert_eq!(r3 * Vector3::z(), Vector3::z());
        // Rotation of the X vector about Z by -half pi, yields -Y
        assert!((r3 * Vector3::x() + Vector3::y()).norm() < EPSILON);
        // Rotation of the Y vector about Z by -half pi, yields X
        assert!((r3 * Vector3::y() - Vector3::x()).norm() < EPSILON);

        // Edge case: Rotation by 0 degrees should yield the original vector
        let r3_zero = DCM::r3(0.0, 0, 1);
        assert!((r3_zero * Vector3::x() - Vector3::x()).norm() < EPSILON);

        assert!(
            (r3.rot_mat - Matrix3::new(0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0)).norm()
                < EPSILON
        );
    }
}
