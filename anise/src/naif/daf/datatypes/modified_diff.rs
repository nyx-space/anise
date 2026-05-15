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
use snafu::{ResultExt, ensure};

use crate::errors::{DecodingError, InaccessibleBytesSnafu, IntegrityError, TooFewDoublesSnafu};
use crate::math::interpolation::{InterpDecodingSnafu, InterpolationError};
use crate::naif::daf::NAIFSummaryRecord;
use crate::{
    math::Vector3,
    naif::daf::{NAIFDataRecord, NAIFDataSet},
};

// Length of a single modified difference type 1 record.
const MD1_RCRD_LEN: usize = 71;

/// Hard upper bound on MAXDIM the Modified-Difference recurrence work
/// arrays accommodate. Type 1 fixes MAXDIM at 15; Type 21 in practice
/// uses up to 25 (e.g. JPL Horizons on-demand SPKs).
pub(crate) const MD_MAXDIM_LIMIT: usize = 25;

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

/// SPK Type 1 is UNDOCUMENTED, so this implementation is a reverse engineering of the original CSPICE code in spke01.c
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

        // NOTE: We do NOT use the epoch registry. Despite the code being strictly identical to the zero-error
        // Hermite registry search, it led here to extremely large interpolation errors.

        // We want the index of the first element that is > epoch.
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
        // SPK Type 1 has a fixed MAXDIM = 15.
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
            15,
        )
    }
}

/// Shared evaluator for SPK Types 1 and 21 (Modified Difference Array
/// family). The math is undocumented outside of CSPICE `spke01.c` /
/// `spke21.c`; this is a Rust translation, verified against JPL
/// reference data. Both record types use the same recurrence — only the
/// per-record array sizes (driven by `MAXDIM`) differ.
///
/// * `epoch` — requested evaluation epoch.
/// * `ref_epoch` — reference epoch stored in the record (seconds, ET).
/// * `nodes` — stepsize-function nodes (length MAXDIM).
/// * `ref_state` — reference state (rx, ry, rz, vx, vy, vz).
/// * `mod_diff_array` — flat (3 × MAXDIM) modified differences.
/// * `kqmax1` — max integration order + 1.
/// * `kq` — per-component (3) integration orders.
/// * `maxdim` — row stride for `mod_diff_array` (15 for Type 1, ≤25 for Type 21).
pub(crate) fn modified_diff_interpolate(
    epoch: Epoch,
    ref_epoch: f64,
    nodes: &[f64],
    ref_state: (f64, f64, f64, f64, f64, f64),
    mod_diff_array: &[f64],
    kqmax1: f64,
    kq: &[f64],
    maxdim: usize,
) -> (Vector3, Vector3) {
    let (ref_x, ref_y, ref_z, ref_vx, ref_vy, ref_vz) = ref_state;
    let delta = epoch.to_et_seconds() - ref_epoch;
    let mut tp = delta;

    let mq2 = kqmax1 - 2.0;

    let mut fc = [0.0_f64; MD_MAXDIM_LIMIT + 1];
    let mut wc = [0.0_f64; MD_MAXDIM_LIMIT];
    let mut w = [0.0_f64; 2 * MD_MAXDIM_LIMIT + 4];

    for j in 0..mq2.max(0.0) as usize {
        fc[j] = tp / nodes[j];
        wc[j] = delta / nodes[j];
        tp = delta + nodes[j];
    }

    for (j, mut_w) in w.iter_mut().enumerate().take(kqmax1 as usize) {
        *mut_w = 1.0 / ((j + 1) as f64);
    }

    let mut ks = kqmax1 as usize - 1;
    for jx in 1..(mq2 + 1.0).max(0.0) as usize {
        for j in 0..jx {
            w[j + ks] = fc[j] * w[j + ks - 1] - wc[j] * w[j + ks];
        }
        ks -= 1;
    }

    let mut pos_km = Vector3::zeros();
    let mut vel_km_s = Vector3::zeros();

    for i in 0..3 {
        let component_order = kq[i] as usize;
        let mut poly_sum = 0.0;
        for j in 0..component_order {
            // Flat (3 × maxdim) array indexed as [i, j] = i*maxdim + j.
            poly_sum += mod_diff_array[i * maxdim + j] * w[j + ks];
        }
        let (refpos, refvel) = match i {
            0 => (ref_x, ref_vx),
            1 => (ref_y, ref_vy),
            2 => (ref_z, ref_vz),
            _ => unreachable!(),
        };
        pos_km[i] = refpos + delta * (refvel + delta * poly_sum);
    }

    if mq2 > 0.0 {
        for j in 1..(mq2 + 1.0) as usize {
            w[j] = fc[j - 1] * w[j - 1] - wc[j - 1] * w[j];
        }
    }
    ks -= 1;

    for i in 0..3 {
        let component_order = kq[i] as usize;
        let mut poly_sum_vel = 0.0;
        for j in 0..component_order {
            poly_sum_vel += mod_diff_array[i * maxdim + j] * w[j + ks];
        }
        let refvel = match i {
            0 => ref_vx,
            1 => ref_vy,
            2 => ref_vz,
            _ => unreachable!(),
        };
        vel_km_s[i] = refvel + delta * poly_sum_vel;
    }

    (pos_km, vel_km_s)
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
            kqmax1: slice[67],
            kq: &slice[68..71],
        }
    }
}

#[cfg(test)]
mod ut_spk1 {
    use crate::{math::Vector3, prelude::*};
    use hifitime::Epoch;

    /// Tests that the high error in the validation is not due to the implementation of the SPK Type 1 algorithm.
    /// Specifically, I test the epoch where I've used CSPICE to transform to the parent. Then I check that ANISE
    /// computes the same thing. It does.
    #[test]
    fn spk1_highest_error() {
        let epoch = Epoch::from_et_seconds(810652114.2299933);

        let almanac = Almanac::default().load("../data/mro.bsp").unwrap();

        let state = almanac
            .translate_to_parent(Frame::from_ephem_j2000(-74), epoch)
            .unwrap();

        let expct_radius_km = Vector3::new(
            1.844_061_319_966_917_4e3,
            -2.619_224_673_328_194e3,
            1.833_017_170_120_489e3,
        );

        let expct_velocity_km_s = Vector3::new(
            -2.644_158_725_448_453e-1,
            -2.051_522_654_915_796_6,
            -2.683_823_516_568_13,
        );

        // Serves as validation that ANISE and SPICE match to machine precision.
        assert_eq!(state.radius_km, expct_radius_km);
        assert_eq!(state.velocity_km_s, expct_velocity_km_s);
    }
}
