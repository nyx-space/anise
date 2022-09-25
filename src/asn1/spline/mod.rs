/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

mod covkind;
pub use covkind::*;
mod evenness;
pub use evenness::*;
mod meta;
pub use meta::*;
mod splines;
pub use splines::*;
mod statekind;
pub use statekind::*;

/// The fields that can be queried for spline data.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
