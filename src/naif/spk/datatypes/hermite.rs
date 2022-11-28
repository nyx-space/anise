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

use crate::math::interpolation::{hermite_eval, MAX_SAMPLES};
use crate::{
    math::{cartesian::CartesianState, Vector3},
    naif::daf::{NAIFDataRecord, NAIFDataSet, NAIFRecord},
    prelude::AniseError,
    DBL_SIZE,
};

use super::posvel::PositionVelocityRecord;

pub struct HermiteSetType12<'a> {
    pub first_state_epoch: Epoch,
    pub step_size: Duration,
    pub window_size: usize,
    pub num_records: usize,
    pub record_data: &'a [f64],
}

impl<'a> fmt::Display for HermiteSetType12<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hermite Type 12: start: {:E}\tstep: {}\twindow size: {}\tnum records: {}\tlen data: {}",
            self.first_state_epoch,
            self.step_size,
            self.window_size,
            self.num_records,
            self.record_data.len()
        )
    }
}

impl<'a> NAIFDataSet<'a> for HermiteSetType12<'a> {
    type StateKind = CartesianState;
    type RecordKind = PositionVelocityRecord;

    fn from_slice_f64(slice: &'a [f64]) -> Self {
        // For this kind of record, the metadata is stored at the very end of the dataset, so we need to read that first.
        let first_state_epoch = Epoch::from_et_seconds(slice[slice.len() - 4]);
        let step_size = slice[slice.len() - 3].seconds();
        let window_size = slice[slice.len() - 2] as usize;
        let num_records = slice[slice.len() - 1] as usize;

        Self {
            first_state_epoch,
            step_size,
            window_size,
            num_records,
            record_data: &slice[0..slice.len() - 4],
        }
    }

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, AniseError> {
        let rcrd_len = self.record_data.len() / self.num_records;
        Ok(Self::RecordKind::from_slice_f64(
            self.record_data
                .get(n * rcrd_len..(n + 1) * rcrd_len)
                .ok_or(AniseError::MalformedData((n + 1) * rcrd_len))?,
        ))
    }

    fn evaluate(
        &self,
        _epoch: Epoch,
        _start_epoch: Epoch,
    ) -> Result<CartesianState, crate::prelude::AniseError> {
        todo!("https://github.com/anise-toolkit/anise.rs/issues/14")
    }
}

pub struct HermiteSetType13<'a> {
    /// Number of samples to use to build the interpolation
    pub samples: usize,
    /// Total number of records stored in this data
    pub num_records: usize,
    /// State date used for the interpolation
    pub state_data: &'a [f64],
    /// Epochs of each of the state data, must be of the same length as state_data. ANISE expects this to be ordered chronologically!
    pub epoch_data: &'a [f64],
    /// Epoch registry to reduce the search space in epoch data.
    pub epoch_registry: &'a [f64],
}

impl<'a> HermiteSetType13<'a> {
    pub fn degree(&self) -> usize {
        2 * self.samples - 1
    }
}

impl<'a> fmt::Display for HermiteSetType13<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hermite Type 13 from {:E} to {:E} with degree {} ({} items, {} epoch directories)",
            Epoch::from_et_seconds(*self.epoch_data.first().unwrap()),
            Epoch::from_et_seconds(*self.epoch_data.last().unwrap()),
            self.degree(),
            self.epoch_data.len(),
            self.epoch_registry.len()
        )
    }
}

impl<'a> NAIFDataSet<'a> for HermiteSetType13<'a> {
    type StateKind = (Vector3, Vector3);
    type RecordKind = PositionVelocityRecord;

    fn from_slice_f64(slice: &'a [f64]) -> Self {
        // For this kind of record, the metadata is stored at the very end of the dataset
        let num_records = slice[slice.len() - 1] as usize;
        let samples = slice[slice.len() - 2] as usize;
        // NOTE: The ::SIZE returns the C representation memory size of this, but we only want the number of doubles.
        let state_data_end_idx = PositionVelocityRecord::SIZE / DBL_SIZE * num_records;
        let state_data = slice.get(0..state_data_end_idx).unwrap();
        let epoch_data_end_idx = state_data_end_idx + num_records;
        let epoch_data = slice.get(state_data_end_idx..epoch_data_end_idx).unwrap();
        // And the epoch directory is whatever remains minus the metadata
        let epoch_registry = slice.get(epoch_data_end_idx..slice.len() - 2).unwrap();

        Self {
            samples,
            num_records,
            state_data,
            epoch_data,
            epoch_registry,
        }
    }

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, AniseError> {
        let rcrd_len = self.state_data.len() / self.num_records;
        Ok(Self::RecordKind::from_slice_f64(
            self.state_data
                .get(n * rcrd_len..(n + 1) * rcrd_len)
                .ok_or(AniseError::MalformedData((n + 1) * rcrd_len))?,
        ))
    }

    fn evaluate(
        &self,
        epoch: Epoch,
        _start_epoch: Epoch,
    ) -> Result<Self::StateKind, crate::prelude::AniseError> {
        // Start by doing a binary search on the epoch registry to limit the search space in the total number of epochs.
        // TODO: use the epoch registry to reduce the search space
        // Check that we even have interpolation data for that time
        if epoch.to_et_seconds() < self.epoch_data[0]
            || epoch.to_et_seconds() > *self.epoch_data.last().unwrap()
        {
            return Err(AniseError::MissingInterpolationData(epoch));
        }
        // Now, perform a binary search on the epochs themselves.
        match self.epoch_data.binary_search_by(|epoch_et| {
            epoch_et.partial_cmp(&epoch.to_et_seconds()).expect(
                "ANISE internal error: epochs in Hermite data or provided is NaN or Infinite",
            )
        }) {
            Ok(idx) => {
                // Oh wow, this state actually exists, no interpolation needed!
                Ok(self.nth_record(idx)?.to_pos_vel())
            }
            Err(idx) => {
                // We didn't find it, so let's build an interpolation here.

                // Check that we won't be fetching out of the window.
                let (first_idx, last_idx) = if idx < self.samples / 2 {
                    // Uh oh, we don't have enough states, so let's bound it to the valid state data
                    (0, self.samples)
                } else if (self.samples % 2 == 0 && idx + self.samples / 2 + 1 > self.num_records)
                    || (self.samples % 2 == 1 && idx + self.samples / 2 > self.num_records)
                {
                    (self.num_records - self.samples - 1, self.samples - 1)
                } else {
                    dbg!(idx - (self.samples - 1) / 2, idx + (self.samples - 1) / 2)
                };

                // Statically allocated arrays of the maximum number of samples
                let mut epochs = [0.0; MAX_SAMPLES];
                let mut xs = [0.0; MAX_SAMPLES];
                let mut ys = [0.0; MAX_SAMPLES];
                let mut zs = [0.0; MAX_SAMPLES];
                let mut vxs = [0.0; MAX_SAMPLES];
                let mut vys = [0.0; MAX_SAMPLES];
                let mut vzs = [0.0; MAX_SAMPLES];
                for (cno, idx) in (first_idx..=last_idx).enumerate() {
                    let record = self.nth_record(idx)?;
                    xs[cno] = record.x_km;
                    ys[cno] = record.y_km;
                    zs[cno] = record.z_km;
                    vxs[cno] = record.vx_km_s;
                    vys[cno] = record.vy_km_s;
                    vzs[cno] = record.vz_km_s;
                    epochs[cno] = self.epoch_data[idx];
                }

                // TODO: Build a container that uses the underlying data and provides an index into it.

                // Build the interpolation polynomials making sure to limit the slices to exactly the number of items we actually used
                // The other ones are zeros, which would cause the interpolation function to fail.
                let (x_km, vx_km_s) = hermite_eval(
                    &epochs[..self.samples],
                    &xs[..self.samples],
                    &vxs[..self.samples],
                    epoch.to_et_seconds(),
                )?;

                let (y_km, vy_km_s) = hermite_eval(
                    &epochs[..self.samples],
                    &ys[..self.samples],
                    &vys[..self.samples],
                    epoch.to_et_seconds(),
                )?;

                let (z_km, vz_km_s) = hermite_eval(
                    &epochs[..self.samples],
                    &zs[..self.samples],
                    &vzs[..self.samples],
                    epoch.to_et_seconds(),
                )?;

                // And build the result
                let pos_km = Vector3::new(x_km, y_km, z_km);
                let vel_km_s = Vector3::new(vx_km_s, vy_km_s, vz_km_s);

                dbg!(pos_km, vel_km_s);

                Ok((pos_km, vel_km_s))
            }
        }
    }
}
