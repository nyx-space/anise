/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

//! SPK Type 21 — Extended Modified Difference Array.
//!
//! Identical math to SPK Type 1 (see `modified_diff.rs`), except the
//! per-record array sizes (`nodes`, `mod_diff_array`) are determined by
//! a per-segment `MAXDIM` parameter rather than hard-coded at 15.
//!
//! Segment layout (in order):
//!   * N records, each `11 + 4*MAXDIM` doubles
//!   * `N` reference epochs
//!   * optional epoch directory (every 100 epochs)
//!   * 1 double `MAXDIM`
//!   * 1 double `N`
//!
//! Record layout:
//!   * `[0]`                           reference epoch
//!   * `[1 ..= MAXDIM]`                stepsize-function nodes
//!   * `[MAXDIM+1 .. MAXDIM+1+6]`      reference state (px, vx, py, vy, pz, vz)
//!   * `[MAXDIM+7 .. MAXDIM+7+3*MD]`   mod-diff array (3 × MAXDIM)
//!   * `[end-4]`                       kqmax1 (max integration order + 1)
//!   * `[end-3 ..= end-1]`             kq (per-component integration orders)

use core::fmt;
use hifitime::Epoch;
use snafu::{ResultExt, ensure};

use crate::errors::{DecodingError, InaccessibleBytesSnafu, IntegrityError, TooFewDoublesSnafu};
use crate::math::interpolation::{InterpDecodingSnafu, InterpolationError};
use crate::naif::daf::NAIFSummaryRecord;
use crate::naif::daf::datatypes::modified_diff::{MD_MAXDIM_LIMIT, modified_diff_interpolate};
use crate::{
    math::Vector3,
    naif::daf::{NAIFDataRecord, NAIFDataSet},
};

#[derive(PartialEq)]
pub struct ModifiedDiffType21<'a> {
    pub num_records: usize,
    pub maxdim: usize,
    pub record_size: usize,
    pub epoch_data: &'a [f64],
    pub epoch_registry: &'a [f64],
    pub record_data: &'a [f64],
}

impl fmt::Display for ModifiedDiffType21<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Extended Modified Differences Type 21 from {:E} to {:E} with {} items (MAXDIM={}, {} epoch directories)",
            Epoch::from_et_seconds(*self.epoch_data.first().unwrap_or(&0.0)),
            Epoch::from_et_seconds(*self.epoch_data.last().unwrap_or(&0.0)),
            self.num_records,
            self.maxdim,
            self.epoch_registry.len()
        )
    }
}

/// SPK Type 21 is UNDOCUMENTED outside of CSPICE source `spke21.c`. This
/// implementation follows the same reverse-engineered math as Type 1
/// (`modified_diff.rs`), with the record size and array spans driven by
/// the per-segment `MAXDIM` value.
impl<'a> NAIFDataSet<'a> for ModifiedDiffType21<'a> {
    type StateKind = (Vector3, Vector3);
    type RecordKind = ModifiedDiffRecord21<'a>;
    const DATASET_NAME: &'static str = "Extended Modified Differences Type 21";

    fn from_f64_slice(slice: &'a [f64]) -> Result<Self, DecodingError> {
        ensure!(
            slice.len() >= 2,
            TooFewDoublesSnafu {
                dataset: Self::DATASET_NAME,
                need: 2_usize,
                got: slice.len()
            }
        );
        let num_records = slice[slice.len() - 1] as usize;
        let maxdim = slice[slice.len() - 2] as usize;
        ensure!(
            maxdim > 0 && maxdim <= MD_MAXDIM_LIMIT,
            TooFewDoublesSnafu {
                dataset: Self::DATASET_NAME,
                need: MD_MAXDIM_LIMIT,
                got: maxdim
            }
        );
        let record_size = 11 + 4 * maxdim;
        let total_records_bytes = num_records * record_size;
        ensure!(
            total_records_bytes + num_records + 2 <= slice.len(),
            InaccessibleBytesSnafu {
                start: 0_usize,
                end: total_records_bytes + num_records + 2,
                size: slice.len(),
            }
        );
        let record_data = &slice[..total_records_bytes];
        let epoch_data = &slice[total_records_bytes..total_records_bytes + num_records];
        // Everything between epoch_data and the trailing (maxdim, num_records)
        // pair is the optional epoch directory.
        let epoch_registry = &slice[total_records_bytes + num_records..slice.len() - 2];

        Ok(Self {
            num_records,
            maxdim,
            record_size,
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
        let start = n * self.record_size;
        let end = start + self.record_size;
        let slice = self.record_data.get(start..end).ok_or(
            DecodingError::InaccessibleBytes {
                start,
                end,
                size: self.record_data.len(),
            },
        )?;
        Ok(ModifiedDiffRecord21::from_slice_f64_maxdim(slice, self.maxdim))
    }

    fn evaluate<S: NAIFSummaryRecord>(
        &self,
        epoch: Epoch,
        _: &S,
    ) -> Result<Self::StateKind, InterpolationError> {
        if self.epoch_data.is_empty() {
            return Err(InterpolationError::MissingInterpolationData { epoch });
        }
        let last_epoch = *self
            .epoch_data
            .last()
            .ok_or(InterpolationError::MissingInterpolationData { epoch })?;
        if epoch.to_et_seconds() < self.epoch_data[0] - 1e-2
            || epoch.to_et_seconds() > last_epoch + 1e-2
        {
            return Err(InterpolationError::NoInterpolationData {
                req: epoch,
                start: Epoch::from_et_seconds(self.epoch_data[0]),
                end: Epoch::from_et_seconds(last_epoch),
            });
        }

        // First record whose end-epoch is >= requested epoch.
        let rcrd_idx = self
            .epoch_data
            .partition_point(|&epoch_et| epoch_et <= epoch.to_et_seconds());

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
        for val in self.epoch_data {
            if !val.is_finite() {
                return Err(IntegrityError::SubNormal {
                    dataset: Self::DATASET_NAME,
                    variable: "one of the epoch data",
                });
            }
        }
        for val in self.epoch_registry {
            if !val.is_finite() {
                return Err(IntegrityError::SubNormal {
                    dataset: Self::DATASET_NAME,
                    variable: "one of the epoch registry",
                });
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct ModifiedDiffRecord21<'a> {
    pub maxdim: usize,
    pub ref_epoch: f64,
    pub nodes: &'a [f64],
    pub ref_x_km: f64,
    pub ref_y_km: f64,
    pub ref_z_km: f64,
    pub ref_vx_km_s: f64,
    pub ref_vy_km_s: f64,
    pub ref_vz_km_s: f64,
    /// Flat (3 × MAXDIM) modified-difference array.
    pub mod_diff_array: &'a [f64],
    pub kqmax1: f64,
    pub kq: &'a [f64],
}

impl<'a> ModifiedDiffRecord21<'a> {
    pub fn from_slice_f64_maxdim(slice: &'a [f64], maxdim: usize) -> Self {
        let ref_state_start = 1 + maxdim;
        let mod_diff_start = ref_state_start + 6;
        let mod_diff_end = mod_diff_start + 3 * maxdim;
        let kqmax1_idx = mod_diff_end;
        let kq_start = kqmax1_idx + 1;
        Self {
            maxdim,
            ref_epoch: slice[0],
            nodes: &slice[1..1 + maxdim],
            ref_x_km: slice[ref_state_start],
            ref_vx_km_s: slice[ref_state_start + 1],
            ref_y_km: slice[ref_state_start + 2],
            ref_vy_km_s: slice[ref_state_start + 3],
            ref_z_km: slice[ref_state_start + 4],
            ref_vz_km_s: slice[ref_state_start + 5],
            mod_diff_array: &slice[mod_diff_start..mod_diff_end],
            kqmax1: slice[kqmax1_idx],
            kq: &slice[kq_start..kq_start + 3],
        }
    }

    pub fn to_pos_vel(&self, epoch: Epoch) -> (Vector3, Vector3) {
        // Same recurrence as SPK Type 1 — shared helper, parameterised
        // by per-segment MAXDIM (15 for Type 1, ≤ 25 for Type 21).
        modified_diff_interpolate(
            epoch,
            self.ref_epoch,
            self.nodes,
            (
                self.ref_x_km,
                self.ref_y_km,
                self.ref_z_km,
                self.ref_vx_km_s,
                self.ref_vy_km_s,
                self.ref_vz_km_s,
            ),
            self.mod_diff_array,
            self.kqmax1,
            self.kq,
            self.maxdim,
        )
    }
}

impl<'a> fmt::Display for ModifiedDiffRecord21<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<'a> NAIFDataRecord<'a> for ModifiedDiffRecord21<'a> {
    fn from_slice_f64(slice: &'a [f64]) -> Self {
        // MAXDIM is needed to slice the record; callers should use
        // `from_slice_f64_maxdim` instead. As a fallback, assume the
        // common MAXDIM=25.
        Self::from_slice_f64_maxdim(slice, 25)
    }
}
