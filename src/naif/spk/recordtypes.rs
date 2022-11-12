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
use hifitime::{Duration, Epoch, TimeUnits};

use crate::naif::daf::{NAIFDataRecord, NAIFDataSet};

pub struct ChebyshevSetType2Type3<'a> {
    pub init_epoch: Epoch,
    pub interval_length: Duration,
    pub rsize: usize,
    pub num_records: usize,
    pub record_data: &'a [f64],
}

impl<'a> fmt::Display for ChebyshevSetType2Type3<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "start: {:E}\tlength: {}\trsize: {}\tnum_records: {}\tlen data: {}",
            self.init_epoch,
            self.interval_length,
            self.rsize,
            self.num_records,
            self.record_data.len()
        )
    }
}

impl<'a> NAIFDataSet<'a> for ChebyshevSetType2Type3<'a> {
    type RecordKind = Type2ChebyshevRecord<'a>;

    fn from_slice_f64(slice: &'a [f64]) -> Self {
        // For this kind of record, the data is stored at the very end of the dataset
        let init_epoch = Epoch::from_et_seconds(slice[slice.len() - 4]);
        let interval_length = slice[slice.len() - 3].seconds();
        let rsize = slice[slice.len() - 2] as usize;
        let num_records = slice[slice.len() - 1] as usize;

        Self {
            init_epoch,
            interval_length,
            rsize,
            num_records,
            record_data: &slice[0..slice.len() - 4],
        }
    }

    fn nth_record(&self, n: usize) -> Self::RecordKind {
        let rcrd_len = self.record_data.len() / self.num_records;
        Self::RecordKind::from_slice_f64(&self.record_data[n * rcrd_len..(n + 1) * rcrd_len])
    }
}

pub struct Type2ChebyshevRecord<'a> {
    pub midpoint: Epoch,
    pub radius: Duration,
    pub x_coeffs: &'a [f64],
    pub y_coeffs: &'a [f64],
    pub z_coeffs: &'a [f64],
}

impl<'a> fmt::Display for Type2ChebyshevRecord<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "start: {:E}\tend: {:E}\nx: {:?}\ny: {:?}\nz: {:?}",
            self.midpoint - self.radius,
            self.midpoint + self.radius,
            self.x_coeffs,
            self.y_coeffs,
            self.z_coeffs
        )
    }
}

impl<'a> NAIFDataRecord<'a> for Type2ChebyshevRecord<'a> {
    fn from_slice_f64(slice: &'a [f64]) -> Self {
        let num_coeffs = (slice.len() - 2) / 3;
        Self {
            midpoint: Epoch::from_et_seconds(slice[0]),
            radius: slice[1].seconds(),
            x_coeffs: &slice[2..num_coeffs],
            y_coeffs: &slice[2 + num_coeffs..num_coeffs * 2],
            z_coeffs: &slice[2 + num_coeffs * 2..],
        }
    }
}

pub struct Type3ChebyshevRecord<'a> {
    pub midpoint: Epoch,
    pub radius: Duration,
    pub x_coeffs: &'a [f64],
    pub y_coeffs: &'a [f64],
    pub z_coeffs: &'a [f64],
    pub vx_coeffs: &'a [f64],
    pub vy_coeffs: &'a [f64],
    pub vz_coeffs: &'a [f64],
}

impl<'a> fmt::Display for Type3ChebyshevRecord<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "start: {}\tend: {}\nx:  {:?}\ny:  {:?}\nz:  {:?}\nvx: {:?}\nvy: {:?}\nvz: {:?}",
            self.midpoint - self.radius,
            self.midpoint + self.radius,
            self.x_coeffs,
            self.y_coeffs,
            self.z_coeffs,
            self.vx_coeffs,
            self.vy_coeffs,
            self.vz_coeffs
        )
    }
}

impl<'a> NAIFDataRecord<'a> for Type3ChebyshevRecord<'a> {
    fn from_slice_f64(slice: &'a [f64]) -> Self {
        let num_coeffs = (slice.len() - 2) / 6;
        Self {
            midpoint: Epoch::from_et_seconds(slice[0]),
            radius: slice[1].seconds(),
            x_coeffs: &slice[2..num_coeffs],
            y_coeffs: &slice[2 + num_coeffs..num_coeffs * 2],
            z_coeffs: &slice[2 + num_coeffs * 2..num_coeffs * 3],
            vx_coeffs: &slice[2 + num_coeffs * 3..num_coeffs * 4],
            vy_coeffs: &slice[2 + num_coeffs * 4..num_coeffs * 5],
            vz_coeffs: &slice[2 + num_coeffs * 5..],
        }
    }
}

#[derive(Debug)]
pub struct PositionVelocityRecord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

impl fmt::Display for PositionVelocityRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<'a> NAIFDataRecord<'a> for PositionVelocityRecord {
    fn from_slice_f64(slice: &'a [f64]) -> Self {
        Self {
            x: slice[0],
            y: slice[1],
            z: slice[2],
            vx: slice[3],
            vy: slice[4],
            vz: slice[5],
        }
    }
}
