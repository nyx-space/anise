/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

/// The fields that can be queried for spline data.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Field {
    MidPoint,
    Duration,
    X,
    Y,
    Z,
    Vx,
    Vy,
    Vz,
    Ax,
    Ay,
    Az,
    CovXX,
    CovYZ,
    CovYY,
    CovZX,
    CovZY,
    CovZZ,
    CovVxX,
    CovVxY,
    CovVxZ,
    CovVxVx,
    CovVyX,
    CovVyY,
    CovVyZ,
    CovVyVx,
    CovVyVy,
    CovVzX,
    CovVzY,
    CovVzZ,
    CovVzVx,
    CovVzVy,
    CovVzVz,
    CovAxX,
    CovAxY,
    CovAxZ,
    CovAxVx,
    CovAxVy,
    CovAxVz,
    CovAxAx,
    CovAyX,
    CovAyY,
    CovAyZ,
    CovAyVx,
    CovAyVy,
    CovAyVz,
    CovAyAx,
    CovAyAy,
    CovAzX,
    CovAzY,
    CovAzZ,
    CovAzVx,
    CovAzVy,
    CovAzVz,
    CovAzAx,
    CovAzAy,
    CovAzAz,
}

impl Field {
    pub const fn is_position(&self) -> bool {
        matches!(self, Self::X | Self::Y | Self::Z)
    }

    pub const fn is_velocity(&self) -> bool {
        matches!(self, Self::Vx | Self::Vy | Self::Vz)
    }

    pub const fn is_acceleration(&self) -> bool {
        matches!(self, Self::Ax | Self::Ay | Self::Az)
    }

    pub const fn is_covariance(&self) -> bool {
        matches!(
            self,
            Self::CovXX
                | Self::CovYZ
                | Self::CovYY
                | Self::CovZX
                | Self::CovZY
                | Self::CovZZ
                | Self::CovVxX
                | Self::CovVxY
                | Self::CovVxZ
                | Self::CovVxVx
                | Self::CovVyX
                | Self::CovVyY
                | Self::CovVyZ
                | Self::CovVyVx
                | Self::CovVyVy
                | Self::CovVzX
                | Self::CovVzY
                | Self::CovVzZ
                | Self::CovVzVx
                | Self::CovVzVy
                | Self::CovVzVz
                | Self::CovAxX
                | Self::CovAxY
                | Self::CovAxZ
                | Self::CovAxVx
                | Self::CovAxVy
                | Self::CovAxVz
                | Self::CovAxAx
                | Self::CovAyX
                | Self::CovAyY
                | Self::CovAyZ
                | Self::CovAyVx
                | Self::CovAyVy
                | Self::CovAyVz
                | Self::CovAyAx
                | Self::CovAyAy
                | Self::CovAzX
                | Self::CovAzY
                | Self::CovAzZ
                | Self::CovAzVx
                | Self::CovAzVy
                | Self::CovAzVz
                | Self::CovAzAx
                | Self::CovAzAy
                | Self::CovAzAz
        )
    }
}
