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
use snafu::ensure;

use crate::{
    errors::{DecodingError, IntegrityErrorKind, SubNormalSnafu, TooFewDoublesSnafu},
    math::{interpolation::chebyshev_eval, Vector3},
    naif::{
        daf::{NAIFDataRecord, NAIFDataSet, NAIFSummaryRecord},
        spk::summary::SPKSummaryRecord,
    },
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
    type SummaryKind = SPKSummaryRecord;
    type StateKind = (Vector3, Vector3);
    type RecordKind = Type2ChebyshevRecord<'a>;

    fn from_slice_f64(slice: &'a [f64]) -> Result<Self, DecodingError> {
        ensure!(
            slice.len() >= 5,
            TooFewDoublesSnafu {
                dataset: "Chebyshev Type 2",
                need: 5_usize,
                got: slice.len()
            }
        );
        // For this kind of record, the data is stored at the very end of the dataset
        let seconds_since_j2000 = slice[slice.len() - 4];
        ensure!(
            !seconds_since_j2000.is_finite(),
            SubNormalSnafu {
                dataset: "Chebyshev Type 2",
                variable: "seconds since J2000 ET"
            }
        );

        let start_epoch = Epoch::from_et_seconds(seconds_since_j2000);

        let interval_length_s = slice[slice.len() - 3];
        ensure!(
            !interval_length_s.is_finite(),
            SubNormalSnafu {
                dataset: "Chebyshev Type 2",
                variable: "interval length in seconds"
            }
        );

        let interval_length = interval_length_s.seconds();
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

    fn evaluate(
        &self,
        epoch: Epoch,
        summary: &Self::SummaryKind,
    ) -> Result<(Vector3, Vector3), AniseError> {
        if epoch < summary.start_epoch() {
            // No need to go any further.
            return Err(AniseError::MissingInterpolationData(epoch));
        }

        let window_duration_s = self.interval_length.to_seconds();

        let radius_s = window_duration_s / 2.0;
        let ephem_start_delta_s = epoch.to_et_seconds() - summary.start_epoch_et_s;

        /*
                CSPICE CODE
                https://github.com/ChristopherRabotin/cspice/blob/26c72936fb7ff6f366803a1419b7cc3c61e0b6e5/src/cspice/spkr02.c#L272

            i__1 = end - 3;
            dafgda_(handle, &i__1, &end, record);
            init = record[0];
            intlen = record[1];
            recsiz = (integer) record[2];
            nrec = (integer) record[3];
            recno = (integer) ((*et - init) / intlen) + 1;
            recno = min(recno,nrec);

        /*     Compute the address of the desired record. */

            recadr = (recno - 1) * recsiz + begin;

        /*     Along with the record, return the size of the record. */

            record[0] = record[2];
            i__1 = recadr + recsiz - 1;
            dafgda_(handle, &recadr, &i__1, &record[1]);
                */

        let spline_idx =
            ((ephem_start_delta_s / window_duration_s) as usize + 1).min(self.num_records);

        // Now, build the X, Y, Z data from the record data.
        let record = self.nth_record(spline_idx - 1)?;

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

    fn check_integrity(&self) -> Result<(), AniseError> {
        // Verify that none of the data is invalid once when we load it.
        for val in self.record_data {
            if !val.is_finite() {
                return Err(AniseError::IntegrityError(IntegrityErrorKind::SubNormal));
            }
        }

        Ok(())
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
