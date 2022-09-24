/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{Decode, Encode, Reader, Tag, Writer};
use hifitime::Duration;

use super::splinespacing::SplineSpacing;
pub struct SplineMeta {
    pub spacing: SplineSpacing,
}

/*
    + Move degree here
    + Specify state kind: None, Position, PositionVelocity, PositionVelocityAcceleration and later MRP, MRPRates, etc.
    + Specify cov kind: None, etc. idem
    + Encode those as single u8 each.
    + All Spline data has both the start epoch of the spline and the duration: this will be 11 and 10 octets each! Hopefully that isn't too large.
    + If it is too large, if spline space is set to evenly spaced, then remove the duration ==> that means the first entry should be duration and not epoch
        => it's OK to remove the first item or the last, weird to remove any other one.
    + For the index, consider only storing the centuries as i16. Then, for a given time T, check the century. to get the first mini-segment?
    And then store N epochs a u64 offset in naoseconds from that century? The only issue: how to fetch the N-th mini-segment?
        => Maybe, in the time index, store the century and total length in bytes of what's encoded for that century? That should work but it'll be a pain to create especially for an interpolation overlapping two centuries.
    + ALSO! Shouldn't the window duration for evenly spaced splines be a single u64 of nanoseconds? Same size as f64, but more precise, and can have up to 4 centuries: not bad!
*/
