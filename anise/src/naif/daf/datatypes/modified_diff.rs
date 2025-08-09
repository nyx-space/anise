/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt;
use hifitime::Epoch;
use snafu::{ensure, ResultExt};

use crate::errors::{DecodingError, InaccessibleBytesSnafu, IntegrityError, TooFewDoublesSnafu};
use crate::math::interpolation::{InterpDecodingSnafu, InterpolationError};
use crate::naif::daf::NAIFSummaryRecord;
use crate::{
    math::Vector3,
    naif::daf::{NAIFDataRecord, NAIFDataSet},
};

// Length of a single modified difference type 1 record.
const MD1_RCRD_LEN: usize = 71;

#[derive(PartialEq)]
pub struct ModifiedDiffType1<'a> {
    pub num_records: usize,
    pub epoch_data: &'a [f64],
    pub epoch_registry: &'a [f64],
    pub record_data: &'a [f64],
}

impl fmt::Display for ModifiedDiffType1<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Modified Differences Type 1 from {:E} to {:E} with {} items ({} epoch directories)",
            Epoch::from_et_seconds(*self.epoch_data.first().unwrap_or(&0.0)),
            Epoch::from_et_seconds(*self.epoch_data.last().unwrap_or(&0.0)),
            self.num_records,
            self.epoch_registry.len()
        )
    }
}

impl<'a> NAIFDataSet<'a> for ModifiedDiffType1<'a> {
    type StateKind = (Vector3, Vector3);
    type RecordKind = ModifiedDiffRecord<'a>;
    const DATASET_NAME: &'static str = "Modified Differences Type 1";

    fn from_f64_slice(slice: &'a [f64]) -> Result<Self, DecodingError> {
        ensure!(
            // 1: Epoch; 1: Num Records; 71: length of a single record.
            slice.len() >= 2 + MD1_RCRD_LEN,
            TooFewDoublesSnafu {
                dataset: Self::DATASET_NAME,
                need: 2 + MD1_RCRD_LEN,
                got: slice.len()
            }
        );
        let num_records = slice[slice.len() - 1] as usize;
        ensure!(
            num_records < slice.len(),
            InaccessibleBytesSnafu {
                start: 0_usize,
                end: num_records,
                size: slice.len()
            }
        );
        let idx = num_records * MD1_RCRD_LEN;
        ensure!(
            idx + num_records <= slice.len() - 2,
            InaccessibleBytesSnafu {
                start: 0_usize,
                end: idx + num_records + 2,
                size: slice.len(),
            }
        );
        let record_data = &slice[..idx];
        let epoch_data = &slice[idx..idx + num_records];
        let epoch_registry = &slice[idx + num_records..slice.len() - 2];

        Ok(Self {
            num_records,
            record_data,
            epoch_data,
            epoch_registry,
        })
    }

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, DecodingError> {
        if self.num_records == 0 {
            return Err(DecodingError::InaccessibleBytes {
                start: n,
                end: n + 1,
                size: 0,
            });
        }
        let rcrd_len = self.record_data.len() / self.num_records;
        Ok(Self::RecordKind::from_slice_f64(
            self.record_data
                .get(n * rcrd_len..(n + 1) * rcrd_len)
                .ok_or(DecodingError::InaccessibleBytes {
                    start: n * rcrd_len,
                    end: (n + 1) * rcrd_len,
                    size: self.record_data.len(),
                })?,
        ))
    }

    fn evaluate<S: NAIFSummaryRecord>(
        &self,
        epoch: Epoch,
        _: &S,
    ) -> Result<Self::StateKind, InterpolationError> {
        // Start by doing a binary search on the epoch registry to limit the search space in the total number of epochs.
        if self.epoch_data.is_empty() {
            return Err(InterpolationError::MissingInterpolationData { epoch });
        }
        // Check that we even have interpolation data for that time
        if epoch.to_et_seconds() < self.epoch_data[0] - 1e-2
            || epoch.to_et_seconds() > *self.epoch_data.last().unwrap() + 1e-2
        {
            return Err(InterpolationError::NoInterpolationData {
                req: epoch,
                start: Epoch::from_et_seconds(self.epoch_data[0]),
                end: Epoch::from_et_seconds(*self.epoch_data.last().unwrap()),
            });
        }

        // Search through a reduced data slice if available
        let (search_data_slice, slice_offset) = if self.epoch_registry.is_empty() {
            // No registry, search the entire epoch_data
            (self.epoch_data, 0)
        } else {
            // Use epoch_registry to narrow down the search space.
            // dir_idx is the index of the first registry epoch such that epoch_registry[dir_idx] >= et_target.
            let dir_idx = self
                .epoch_registry
                .partition_point(|&reg_epoch| reg_epoch < epoch.to_et_seconds());

            let sub_array_start_idx = if dir_idx == 0 {
                // et_target <= self.epoch_registry[0] (i.e., et_target is before or at the first directory epoch, E_100).
                // Search in the first block of epoch_data (indices 0-99, or up to num_records-1).
                0
            } else {
                // self.epoch_registry[dir_idx - 1] < et_target.
                // The block of 100 epochs in epoch_data starts with the epoch corresponding to
                // the (dir_idx-1)-th entry in epoch_registry. This is E_(dir_idx * 100).
                // Its 0-based index in epoch_data is (dir_idx * 100) - 1.
                (dir_idx * 100) - 1
            };

            // The block is at most 100 records long, or fewer if at the end of epoch_data.
            // Ensure end index does not exceed total number of records.
            let sub_array_end_idx = (sub_array_start_idx + 99).min(self.num_records - 1);

            // It's possible num_records is small enough that sub_array_start_idx is already past sub_array_end_idx if not careful,
            // however, epoch_registry is non-empty only if num_records >= 100 (approx), so sub_array_start_idx should be valid.
            // The slice must be valid, e.g. start <= end.
            (
                &self.epoch_data[sub_array_start_idx..=sub_array_end_idx.max(sub_array_start_idx)],
                sub_array_start_idx,
            )
        };

        // We want the index of the first element that is > epoch.
        let local_idx =
            search_data_slice.partition_point(|&epoch_et| epoch_et <= epoch.to_et_seconds());

        // The record we need is the one right before this index.
        // If local_idx is 0, it means the target epoch is before the first element in the slice;
        // saturating_sub(1) correctly keeps the index at 0, and your slicing logic,
        // which includes the last record of the previous directory, will handle this.
        let rcrd_idx = local_idx.saturating_sub(1) + slice_offset;

        let record = self.nth_record(rcrd_idx).context(InterpDecodingSnafu)?;

        Ok(record.to_pos_vel(epoch))
    }

    fn check_integrity(&self) -> Result<(), IntegrityError> {
        for val in self.record_data {
            if !val.is_finite() {
                return Err(IntegrityError::SubNormal {
                    dataset: Self::DATASET_NAME,
                    variable: "one of the record data",
                });
            }
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Default, Debug)]
#[repr(C)]
pub struct ModifiedDiffRecord<'a> {
    /// Reference epoch at the start of the record
    pub ref_epoch: f64,
    /// Vector of interpolation nodes
    pub nodes: &'a [f64],
    /// Reference position, in km
    pub ref_x_km: f64,
    /// Reference position, in km
    pub ref_y_km: f64,
    /// Reference position, in km
    pub ref_z_km: f64,
    /// Reference velocity, in km/s
    pub ref_vx_km_s: f64,
    /// Reference velocity, in km/s
    pub ref_vy_km_s: f64,
    /// Reference velocity, in km/s
    pub ref_vz_km_s: f64,
    /// Effectively a matrix (x, y, z) containing the core coefficients that define the trajectory's deviation from linear motion
    pub mod_diff_array: &'a [f64],
    // Max integration order plus 1
    pub kqmax1: f64,
    // Integration order array for each component
    pub kq: &'a [f64],
}

impl<'a> ModifiedDiffRecord<'a> {
    pub fn to_pos_vel(&self, epoch: Epoch) -> (Vector3, Vector3) {
        //  Set up for the computation of the various differences.
        let delta = epoch.to_et_seconds() - self.ref_epoch; // Time delta from reference epoch
        let mut tp = delta;

        // The maximum degree of the polynomials we might need to evaluate.
        // mq2 is the number of coefficients for the recurrence relation.
        let mq2 = self.kqmax1 - 2.0;

        // Initialize lists for the recurrence relation coefficients.
        let mut fc = [0.0; 14];
        let mut wc = [0.0; 13];

        for j in 0..mq2 as usize {
            fc[j] = tp / self.nodes[j];
            wc[j] = delta / self.nodes[j];
            tp = delta + self.nodes[j];
        }

        // 3. Compute the W(k) terms for position interpolation.
        let mut w = [0.0; 17];

        // Initialize the first set of W terms with reciprocals.
        for (j, mut_w) in w.iter_mut().enumerate().take(self.kqmax1 as usize) {
            *mut_w = 1.0 / ((j + 1) as f64);
        }

        // This is the core recurrence relation. It builds the values of the
        // position basis polynomials evaluated at the time `delta`.
        let mut ks = self.kqmax1 as usize - 1;
        for jx in 1..(mq2 + 1.0).max(0.0) as usize {
            for j in 0..jx {
                w[j + ks] = fc[j] * w[j + ks - 1] - wc[j] * w[j + ks];
            }
            ks -= 1;
        }

        // 4. Perform position interpolation.
        let mut pos_km = Vector3::zeros();
        let mut vel_km_s = Vector3::zeros();

        for i in 0..3 {
            let component_order = self.kq[i] as usize;
            let mut poly_sum = 0.0;

            for j in 0..component_order {
                // Access dt value from the flat record array.
                // The index is equivalent to dt[i, j] in a 3x15 reshaped array.
                // The dt data block starts at record index 22.
                let dt_idx = i * 15 + j;
                poly_sum += self.mod_diff_array[dt_idx] * w[j + ks]
            }

            let (refpos, refvel) = match i {
                0 => (self.ref_x_km, self.ref_vx_km_s),
                1 => (self.ref_y_km, self.ref_vy_km_s),
                2 => (self.ref_z_km, self.ref_vz_km_s),
                _ => unreachable!(),
            };

            pos_km[i] = refpos + delta * (refvel + delta * poly_sum)
        }

        // 5. Compute the W(k) terms for velocity interpolation.
        if mq2 > 0.0 {
            for j in 1..(mq2 + 1.0) as usize {
                w[j] = fc[j - 1] * w[j - 1] - wc[j - 1] * w[j];
            }
        }
        ks -= 1;

        // 6. Perform velocity interpolation.
        for i in 0..3 {
            let component_order = self.kq[i] as usize;
            let mut poly_sum_vel = 0.0;

            for j in 0..component_order {
                // The index into the flat dt block is the same as for position.
                let dt_idx = i * 15 + j;
                poly_sum_vel += self.mod_diff_array[dt_idx] * w[j + ks];
            }

            let refvel = match i {
                0 => self.ref_vx_km_s,
                1 => self.ref_vy_km_s,
                2 => self.ref_vz_km_s,
                _ => unreachable!(),
            };
            vel_km_s[i] = refvel + delta * poly_sum_vel;
        }

        (pos_km, vel_km_s)
    }
}

impl<'a> fmt::Display for ModifiedDiffRecord<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

// impl<'a> NAIFRecord for ModifiedDiffRecord<'a> {}

impl<'a> NAIFDataRecord<'a> for ModifiedDiffRecord<'a> {
    fn from_slice_f64(slice: &'a [f64]) -> Self {
        Self {
            ref_epoch: slice[0],
            nodes: &slice[1..16],
            ref_x_km: slice[16],
            ref_y_km: slice[18],
            ref_z_km: slice[20],
            ref_vx_km_s: slice[17],
            ref_vy_km_s: slice[19],
            ref_vz_km_s: slice[21],
            mod_diff_array: &slice[22..67],
            kqmax1: slice[67], // TODO: Check this is an integer
            kq: &slice[68..71],
        }
    }
}
