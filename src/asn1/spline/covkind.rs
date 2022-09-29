/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{Decode, Encode, Reader, Writer};

use crate::DBL_SIZE;

use super::statekind::StateKind;

/// Covariance Kind defines what kind of covariance is stored in the spline, if at all.
/// Under the hood, this works exactly like [StateKind] since the CovKind structure has a single field `data` which is a StateKind.
///
/// # Storage requirements and field ordering
/// Covariance information requires more data than just a state since it includes both the covariance and the variance between different elements.
/// In ANISE, this is stored as an upper triangular matrix.
///
/// ## Position variance storage
///
/// The order of the data is as follows:
/// 1. cov_x_x
/// 2. cov_y_x
/// 3. cov_y_y
/// 4. cov_z_x
/// 5. cov_z_y
/// 6. cov_z_z
///
/// Hence, if the covariance is interpolated with a degree 6, then the position covariance of a single spline is stored as a contiguous octet array of 288 octets:
///
/// | field | length | start octet | end octet
/// | -- | -- | -- | -- |
/// | cov_x_x | 6*8 = 48 | 0 | 47
/// | cov_y_x | 6*8 = 48 | 48 | 95
/// | cov_y_y | 48 | 96 | 143
/// | cov_z_x | 48 | 144 | 191
/// | cov_z_y | 48 | 192 | 239
/// | cov_z_z | 48 | 240 | 287
///
/// ### Example
/// Storing the position and velocity covariance, interpolated as a 6 degree polynomial will require **6** fields of **6 * 8 = 48* octets each, leading to **288 octets per spline**.
///
/// ## Position and velocity variance storage
///
/// It is not possible to store the velocity variance without also storing the position variance. If we've missed a use case where this is relevant, please open an issue.
///
/// Additional fields for the velocity variance.
///
/// + cov_vx_x
/// + cov_vx_y
/// + cov_vx_z
/// + cov_vx_vx
///  
/// + cov_vy_x
/// + cov_vy_y
/// + cov_vy_z
/// + cov_vy_vx
/// + cov_vy_vy
///
/// + cov_vz_x
/// + cov_vz_y
/// + cov_vz_z
/// + cov_vz_vx
/// + cov_vz_vy
/// + cov_vz_vz
///
/// ### Example
/// Storing the position and velocity covariance, interpolated as a 6 degree polynomial will require **6 + 15 = 21** fields of **6 * 8 = 48* octets each, leading to **1008 octets per spline**.
///
/// ## Position, velocity, and acceleration variance storage
///
/// We also don't know of a use case where one would need to store the variance of the acceleration, but it's supported because the support is relatively easy.
/// **Warning:** this will add 7+8+9 = 24 fields, each storing one 64-bit floating point number _per interpolation degree_.
///
/// + cov_ax_x
/// + cov_ax_y
/// + cov_ax_z
/// + cov_ax_vx
/// + cov_ax_vy
/// + cov_ax_vz
/// + cov_ax_ax
///
/// + cov_ay_x
/// + cov_ay_y
/// + cov_ay_z
/// + cov_ay_vx
/// + cov_ay_vy
/// + cov_ay_vz
/// + cov_ay_ax
/// + cov_ay_ay
///
/// + cov_az_x
/// + cov_az_y
/// + cov_az_z
/// + cov_az_vx
/// + cov_az_vy
/// + cov_az_vz
/// + cov_az_ax
/// + cov_az_ay
/// + cov_az_az
///
/// ### Example
/// Storing the full covariance of position, velocity, and acceleration, interpolated as a 6 degree polynomial will require **6 + 15 + 24 = 45** fields of **6 * 8 = 48* octets each, leading to **2160 octets per spline**.

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct CovKind {
    pub(crate) data: StateKind,
}

impl CovKind {
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length in octets required to store this covariance information
    pub const fn len(&self) -> usize {
        let num_items = match self.data {
            StateKind::None => 0,
            StateKind::Position { degree } => degree * 6,
            StateKind::PositionVelocity { degree } => degree * (6 + 15),
            StateKind::PositionVelocityAcceleration { degree } => degree * (6 + 15 + 21),
        };
        DBL_SIZE * (num_items as usize)
    }

    /// Returns the interpolation degree
    pub const fn degree(&self) -> u8 {
        match &self.data {
            StateKind::None => 0,
            StateKind::Position { degree } => *degree,
            StateKind::PositionVelocity { degree } => *degree,
            StateKind::PositionVelocityAcceleration { degree } => *degree,
        }
    }
}

impl Default for CovKind {
    fn default() -> Self {
        Self {
            data: StateKind::None,
        }
    }
}

/// Allows conversion of the CovKind into a u8 with the following mapping.
impl From<CovKind> for u16 {
    fn from(kind: CovKind) -> Self {
        u16::from(kind.data)
    }
}

impl From<&CovKind> for u16 {
    fn from(kind: &CovKind) -> Self {
        u16::from(*kind)
    }
}

/// Allows conversion of a u8 into a CovKind.
impl From<u16> for CovKind {
    fn from(val: u16) -> Self {
        Self {
            data: StateKind::from(val),
        }
    }
}

impl Encode for CovKind {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let converted: u16 = self.into();
        converted.encoded_len()
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let converted: u16 = self.into();
        converted.encode(encoder)
    }
}

impl<'a> Decode<'a> for CovKind {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let converted: u16 = decoder.decode()?;
        Ok(Self::from(converted))
    }
}
