/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt;
use zerocopy::FromBytes;

use crate::naif::daf::{NAIFDataRecord, NAIFRecord};

#[derive(Copy, Clone, Default, FromBytes, Debug)]
pub struct PositionVelocityRecord {
    pub x_km: f64,
    pub y_km: f64,
    pub z_km: f64,
    pub vx_km_s: f64,
    pub vy_km_s: f64,
    pub vz_km_s: f64,
}

impl fmt::Display for PositionVelocityRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl NAIFRecord for PositionVelocityRecord {}

impl<'a> NAIFDataRecord<'a> for PositionVelocityRecord {
    fn from_slice_f64(slice: &'a [f64]) -> Self {
        Self {
            x_km: slice[0],
            y_km: slice[1],
            z_km: slice[2],
            vx_km_s: slice[3],
            vy_km_s: slice[4],
            vz_km_s: slice[5],
        }
    }
}
