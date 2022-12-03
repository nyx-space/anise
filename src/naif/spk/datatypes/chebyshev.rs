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
use log::error;

use crate::{
    math::{interpolation::chebyshev_eval, Vector3},
    naif::daf::{NAIFDataRecord, NAIFDataSet},
    prelude::AniseError,
};

pub struct Type2ChebyshevSet<'a> {
    pub init_epoch: Epoch,
    pub interval_length: Duration,
    pub rsize: usize,
    pub num_records: usize,
    pub record_data: &'a [f64],
}

impl<'a> Type2ChebyshevSet<'a> {
    pub fn degree(&self) -> usize {
        (self.rsize - 2) / 3 - 1
    }
}

impl<'a> fmt::Display for Type2ChebyshevSet<'a> {
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

impl<'a> NAIFDataSet<'a> for Type2ChebyshevSet<'a> {
    // At this stage, we don't know the frame of what we're interpolating!
    type StateKind = (Vector3, Vector3);
    type RecordKind = Type2ChebyshevRecord<'a>;

    fn from_slice_f64(slice: &'a [f64]) -> Result<Self, AniseError> {
        if slice.len() < 5 {
            error!(
                "Cannot build a Type 2 Chebyshev set from only {} items",
                slice.len()
            );
            return Err(AniseError::MalformedData(5));
        }
        // For this kind of record, the data is stored at the very end of the dataset
        let start_epoch = Epoch::from_et_seconds(slice[slice.len() - 4]);
        let interval_length = slice[slice.len() - 3].seconds();
        let rsize = slice[slice.len() - 2] as usize;
        let num_records = slice[slice.len() - 1] as usize;

        Ok(Self {
            init_epoch: start_epoch,
            interval_length,
            rsize,
            num_records,
            record_data: &slice[0..slice.len() - 4],
        })
    }

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, AniseError> {
        Ok(Self::RecordKind::from_slice_f64(
            self.record_data
                .get(n * self.rsize..(n + 1) * self.rsize)
                .ok_or(AniseError::MalformedData((n + 1) * self.rsize))?,
        ))
    }

    fn evaluate(&self, epoch: Epoch, start_epoch: Epoch) -> Result<(Vector3, Vector3), AniseError> {
        let window_duration_s = self.interval_length.to_seconds();

        let radius_s = window_duration_s / 2.0;
        let ephem_start_delta = epoch - start_epoch;
        let ephem_start_delta_s = ephem_start_delta.to_seconds();

        if ephem_start_delta_s < 0.0 {
            return Err(AniseError::MissingInterpolationData(epoch));
        }

        // In seconds
        let spline_idx = (ephem_start_delta_s / window_duration_s).round() as usize;

        // Now, build the X, Y, Z data from the record data.
        let record = self.nth_record(spline_idx)?;

        let normalized_time = (epoch.to_et_seconds() - record.midpoint_et_s) / radius_s;

        let mut pos = Vector3::zeros();
        let mut vel = Vector3::zeros();

        for (cno, coeffs) in [record.x_coeffs, record.y_coeffs, record.z_coeffs]
            .iter()
            .enumerate()
        {
            let (val, deriv) =
                chebyshev_eval(normalized_time, coeffs, radius_s, epoch, self.degree())?;
            pos[cno] = val;
            vel[cno] = deriv;
        }

        Ok((pos, vel))
    }
}

pub struct Type2ChebyshevRecord<'a> {
    pub midpoint_et_s: f64,
    pub radius: Duration,
    pub x_coeffs: &'a [f64],
    pub y_coeffs: &'a [f64],
    pub z_coeffs: &'a [f64],
}

impl<'a> Type2ChebyshevRecord<'a> {
    pub fn midpoint_epoch(&self) -> Epoch {
        Epoch::from_et_seconds(self.midpoint_et_s)
    }
}

impl<'a> fmt::Display for Type2ChebyshevRecord<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "start: {:E}\tend: {:E}\nx: {:?}\ny: {:?}\nz: {:?}",
            self.midpoint_epoch() - self.radius,
            self.midpoint_epoch() + self.radius,
            self.x_coeffs,
            self.y_coeffs,
            self.z_coeffs
        )
    }
}

impl<'a> NAIFDataRecord<'a> for Type2ChebyshevRecord<'a> {
    fn from_slice_f64(slice: &'a [f64]) -> Self {
        let num_coeffs = (slice.len() - 2) / 3;
        let end_x_idx = num_coeffs + 2;
        let end_y_idx = 2 * num_coeffs + 2;
        Self {
            midpoint_et_s: slice[0],
            radius: slice[1].seconds(),
            x_coeffs: &slice[2..end_x_idx],
            y_coeffs: &slice[end_x_idx..end_y_idx],
            z_coeffs: &slice[end_y_idx..],
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
