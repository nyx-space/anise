/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use crate::{
    astro::PhysicsResult,
    errors::{InvalidRotationSnafu, InvalidStateRotationSnafu, PhysicsError},
    math::{cartesian::CartesianState, Matrix3, Matrix6, Vector3, Vector6},
    prelude::Frame,
    NaifId,
};
use nalgebra::Vector4;
use snafu::ensure;

use super::{r1, r2, r3, Quaternion, Rotation};
use core::fmt;
use core::ops::Mul;

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
        Self {
            rot_mat: r1(angle_rad),
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
        Self {
            rot_mat: r2(angle_rad),
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
        Self {
            rot_mat: r3(angle_rad),
            from,
            to,
            rot_mat_dt: None,
        }
    }

    /// Returns the 6x6 DCM to rotate a state, if the time derivative of this DCM exists.
    pub fn state_dcm(&self) -> Matrix6 {
        let mut full_dcm = Matrix6::zeros();
        for i in 0..6 {
            for j in 0..6 {
                if (i < 3 && j < 3) || (i >= 3 && j >= 3) {
                    full_dcm[(i, j)] = self.rot_mat[(i % 3, j % 3)];
                } else if i >= 3 && j < 3 {
                    full_dcm[(i, j)] = self
                        .rot_mat_dt
                        .map(|dcm_dt| dcm_dt[(i - 3, j)])
                        .unwrap_or(0.0);
                }
            }
        }

        full_dcm
    }

    /// Builds an identity rotation
    pub fn identity(from: i32, to: i32) -> Self {
        let rot_mat = Matrix3::identity();

        Self {
            rot_mat,
            from,
            to,
            rot_mat_dt: None,
        }
    }

    /// Returns whether this rotation is identity, checking first the frames and then the rotation matrix (but ignores its time derivative)
    pub fn is_identity(&self) -> bool {
        self.to == self.from || (self.rot_mat - Matrix3::identity()).norm() < 1e-8
    }

    /// Returns whether the `rot_mat` of this DCM is a valid rotation matrix.
    /// The criteria for validity are:
    /// -- The columns of the matrix are unit vectors, within a specified tolerance.
    /// -- The determinant of the matrix formed by unitizing the columns of the input matrix is 1, within a specified tolerance. This criterion ensures that the columns of the matrix are nearly orthogonal, and that they form a right-handed basis.
    /// [Source: SPICE's rotation.req](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/rotation.html#Validating%20a%20rotation%20matrix)
    pub fn is_valid(&self, unit_tol: f64, det_tol: f64) -> bool {
        for col in self.rot_mat.column_iter() {
            if (col.norm() - 1.0).abs() > unit_tol {
                return false;
            }
        }
        (self.rot_mat.determinant() - 1.0).abs() < det_tol
    }

    /// Multiplies this DCM with another one WITHOUT checking if the frames match.
    pub(crate) fn mul_unchecked(&self, other: Self) -> Self {
        let mut rslt = *self;
        rslt.rot_mat *= other.rot_mat;
        rslt.from = other.from;
        // Make sure to apply the transport theorem.
        if let Some(other_rot_mat_dt) = other.rot_mat_dt {
            if let Some(rot_mat_dt) = self.rot_mat_dt {
                rslt.rot_mat_dt =
                    Some(rot_mat_dt * other.rot_mat + self.rot_mat * other_rot_mat_dt);
            } else {
                rslt.rot_mat_dt = Some(self.rot_mat * other_rot_mat_dt);
            }
        } else if let Some(rot_mat_dt) = self.rot_mat_dt {
            rslt.rot_mat_dt = Some(rot_mat_dt * other.rot_mat);
        }
        rslt
    }

    pub fn transpose(&self) -> Self {
        Self {
            rot_mat: self.rot_mat.transpose(),
            rot_mat_dt: self.rot_mat_dt.map(|rot_mat_dt| rot_mat_dt.transpose()),
            to: self.from,
            from: self.to,
        }
    }
}

impl Mul for DCM {
    type Output = Result<Self, PhysicsError>;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_identity() {
            let mut rslt = rhs;
            rslt.from = rhs.from;
            rslt.to = self.to;
            Ok(rslt)
        } else if rhs.is_identity() {
            let mut rslt = self;
            rslt.from = rhs.from;
            rslt.to = self.to;
            Ok(rslt)
            // Ok(self)
        } else {
            ensure!(
                self.from == rhs.to,
                InvalidRotationSnafu {
                    action: "multiply DCMs",
                    from1: self.from,
                    to1: self.to,
                    from2: rhs.from,
                    to2: rhs.to
                }
            );

            Ok(self.mul_unchecked(rhs))
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
    ///
    ///
    /// let r1 = DCM::r1(FRAC_PI_2, 0, 1);
    ///
    /// // Rotation of the X vector about X, yields X
    /// assert_eq!(r1 * Vector3::x(), Vector3::x());
    /// // Rotation of the Z vector about X by half pi, yields -Y
    /// assert!((r1 * Vector3::z() - Vector3::y()).norm() < f64::EPSILON);
    /// // Rotation of the Y vector about X by half pi, yields Z
    /// assert!((r1 * Vector3::y() + Vector3::z()).norm() < f64::EPSILON);
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
    type Output = Vector6;

    /// Applying the matrix to a vector yields the vector's representation in the new coordinate system.
    fn mul(self, rhs: Vector6) -> Self::Output {
        self.state_dcm() * rhs
    }
}

impl Mul<CartesianState> for DCM {
    type Output = PhysicsResult<CartesianState>;

    fn mul(self, rhs: CartesianState) -> Self::Output {
        self * &rhs
    }
}

impl Mul<&CartesianState> for DCM {
    type Output = PhysicsResult<CartesianState>;

    fn mul(self, rhs: &CartesianState) -> Self::Output {
        ensure!(
            self.from == rhs.frame.orientation_id,
            InvalidStateRotationSnafu {
                from: self.from,
                to: self.to,
                state_frame: rhs.frame
            }
        );
        let new_state = self.state_dcm() * rhs.to_cartesian_pos_vel();

        let mut rslt = *rhs;
        rslt.radius_km = new_state.fixed_rows::<3>(0).to_owned().into();
        rslt.velocity_km_s = new_state.fixed_rows::<3>(3).to_owned().into();
        rslt.frame.orientation_id = self.to;

        Ok(rslt)
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

impl fmt::Display for DCM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rotation {:o} -> {:o} (transport theorem = {}){}Derivative: {}",
            Frame::from_orient_ssb(self.from),
            Frame::from_orient_ssb(self.to),
            self.rot_mat_dt.is_some(),
            self.rot_mat,
            match self.rot_mat_dt {
                None => "None".to_string(),
                Some(dcm_dt) => format!("{dcm_dt}"),
            }
        )
    }
}

#[cfg(test)]
mod ut_dcm {
    use crate::math::Matrix3;

    use super::{Vector3, DCM};
    use core::f64::consts::FRAC_PI_2;

    #[test]
    fn test_r1() {
        let r1 = DCM::r1(FRAC_PI_2, 0, 1);

        // Rotation of the X vector about X, yields X
        assert_eq!(r1 * Vector3::x(), Vector3::x());
        // Rotation of the Z vector about X by half pi, yields Y
        assert!((r1 * Vector3::z() - Vector3::y()).norm() < f64::EPSILON);
        // Rotation of the Y vector about X by half pi, yields -Z
        assert!((r1 * Vector3::y() + Vector3::z()).norm() < f64::EPSILON);

        assert!(
            (r1.rot_mat - Matrix3::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0)).norm()
                < f64::EPSILON
        );
    }

    #[test]
    fn test_r2() {
        let r2 = DCM::r2(FRAC_PI_2, 0, 1);

        // Rotation of the Y vector about Y, yields Y
        assert_eq!(r2 * Vector3::y(), Vector3::y());
        // Rotation of the X vector about Y by -half pi, yields Z
        assert!((r2 * Vector3::x() - Vector3::z()).norm() < f64::EPSILON);
        // Rotation of the Z vector about Y by -half pi, yields -X
        assert!((r2 * Vector3::z() + Vector3::x()).norm() < f64::EPSILON);

        // Edge case: Rotation by 0 degrees should yield the original vector
        let r2_zero = DCM::r2(0.0, 0, 1);
        assert!((r2_zero * Vector3::x() - Vector3::x()).norm() < f64::EPSILON);

        assert!(
            (r2.rot_mat - Matrix3::new(0.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0)).norm()
                < f64::EPSILON
        );
    }

    #[test]
    fn test_r3() {
        let r3 = DCM::r3(FRAC_PI_2, 0, 1);

        assert!(r3.is_valid(1e-12, 1e-12));

        // Rotation of the Z vector about Z, yields Z
        assert_eq!(r3 * Vector3::z(), Vector3::z());
        // Rotation of the X vector about Z by -half pi, yields -Y
        assert!((r3 * Vector3::x() + Vector3::y()).norm() < f64::EPSILON);
        // Rotation of the Y vector about Z by -half pi, yields X
        assert!((r3 * Vector3::y() - Vector3::x()).norm() < f64::EPSILON);

        // Edge case: Rotation by 0 degrees should yield the original vector
        let r3_zero = DCM::r3(0.0, 0, 1);
        assert!((r3_zero * Vector3::x() - Vector3::x()).norm() < f64::EPSILON);

        assert!(
            (r3.rot_mat - Matrix3::new(0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0)).norm()
                < f64::EPSILON
        );
    }
}
