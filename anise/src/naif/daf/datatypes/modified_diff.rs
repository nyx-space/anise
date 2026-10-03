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

use crate::errors::{
    DecodingError, InaccessibleBytesSnafu, IntegrityError, MathError, TooFewDoublesSnafu,
};
use crate::math::interpolation::{InterpDecodingSnafu, InterpolationError};
use crate::naif::daf::NAIFSummaryRecord;
use crate::{
    math::Vector3,
    naif::daf::{NAIFDataRecord, NAIFDataSet},
};

// Length of a single modified difference type 1 record.
const MD1_RCRD_LEN: usize = 71;
// Largest difference line dimension (MAXDIM) supported by SPICE, cf. MAXTRM in spke21.c.
const MAXTRM: usize = 25;

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
        nth_record(self.record_data, self.num_records, n)
    }

    fn evaluate<S: NAIFSummaryRecord>(
        &self,
        epoch_et_s: f64,
        summary: &S,
    ) -> Result<Self::StateKind, InterpolationError> {
        let rcrd_idx = record_index(self.epoch_data, summary, epoch_et_s)?;
        let record = self.nth_record(rcrd_idx).context(InterpDecodingSnafu)?;
        record.check_orders_and_nodes()?;
        Ok(record.to_pos_vel(epoch_et_s))
    }

    fn check_integrity(&self) -> Result<(), IntegrityError> {
        check_integrity(
            Self::DATASET_NAME,
            self.record_data,
            self.epoch_data,
            self.epoch_registry,
        )
    }
}

#[derive(PartialEq)]
pub struct ModifiedDiffType21<'a> {
    pub maxdim: usize,
    pub num_records: usize,
    pub epoch_data: &'a [f64],
    pub epoch_registry: &'a [f64],
    pub record_data: &'a [f64],
}

impl fmt::Display for ModifiedDiffType21<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Extended Modified Differences Type 21 from {:E} to {:E} with {} items of dimension {} ({} epoch directories)",
            Epoch::from_et_seconds(*self.epoch_data.first().unwrap_or(&0.0)),
            Epoch::from_et_seconds(*self.epoch_data.last().unwrap_or(&0.0)),
            self.num_records,
            self.maxdim,
            self.epoch_registry.len()
        )
    }
}

/// SPK Type 21 is Type 1 with a per-segment difference line dimension (MAXDIM), cf. spke21.c
impl<'a> NAIFDataSet<'a> for ModifiedDiffType21<'a> {
    type StateKind = (Vector3, Vector3);
    type RecordKind = ModifiedDiffRecord<'a>;
    const DATASET_NAME: &'static str = "Extended Modified Differences Type 21";

    fn from_f64_slice(slice: &'a [f64]) -> Result<Self, DecodingError> {
        ensure!(
            // 1: MAXDIM; 1: Num Records.
            slice.len() >= 2,
            TooFewDoublesSnafu {
                dataset: Self::DATASET_NAME,
                need: 2_usize,
                got: slice.len()
            }
        );
        // Bound MAXDIM on the raw f64, which also rejects NaN.
        let maxdim_f64 = slice[slice.len() - 2];
        if !(1.0..=MAXTRM as f64).contains(&maxdim_f64) {
            return Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: Self::DATASET_NAME,
                    variable: "difference line dimension (MAXDIM)",
                    value: maxdim_f64,
                    reason: "must be between 1 and MAXTRM (25)",
                },
            });
        }
        let maxdim = maxdim_f64 as usize;
        let rcrd_len = 4 * maxdim + 11;
        let num_records = slice[slice.len() - 1] as usize;
        ensure!(
            num_records < slice.len(),
            InaccessibleBytesSnafu {
                start: 0_usize,
                end: num_records,
                size: slice.len()
            }
        );
        let idx = num_records * rcrd_len;
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
            maxdim,
            num_records,
            record_data,
            epoch_data,
            epoch_registry,
        })
    }

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, DecodingError> {
        nth_record(self.record_data, self.num_records, n)
    }

    fn evaluate<S: NAIFSummaryRecord>(
        &self,
        epoch_et_s: f64,
        summary: &S,
    ) -> Result<Self::StateKind, InterpolationError> {
        let rcrd_idx = record_index(self.epoch_data, summary, epoch_et_s)?;
        let record = self.nth_record(rcrd_idx).context(InterpDecodingSnafu)?;
        record.check_orders_and_nodes()?;
        Ok(record.to_pos_vel(epoch_et_s))
    }

    fn check_integrity(&self) -> Result<(), IntegrityError> {
        check_integrity(
            Self::DATASET_NAME,
            self.record_data,
            self.epoch_data,
            self.epoch_registry,
        )
    }
}

fn nth_record<'a>(
    record_data: &'a [f64],
    num_records: usize,
    n: usize,
) -> Result<ModifiedDiffRecord<'a>, DecodingError> {
    if num_records == 0 {
        return Err(DecodingError::InaccessibleBytes {
            start: n,
            end: n + 1,
            size: 0,
        });
    }
    let rcrd_len = record_data.len() / num_records;
    Ok(ModifiedDiffRecord::from_slice_f64(
        record_data.get(n * rcrd_len..(n + 1) * rcrd_len).ok_or(
            DecodingError::InaccessibleBytes {
                start: n * rcrd_len,
                end: (n + 1) * rcrd_len,
                size: record_data.len(),
            },
        )?,
    ))
}

fn record_index<S: NAIFSummaryRecord>(
    epoch_data: &[f64],
    summary: &S,
    epoch_et_s: f64,
) -> Result<usize, InterpolationError> {
    if epoch_data.is_empty() {
        return Err(InterpolationError::MissingInterpolationData {
            epoch: Epoch::from_et_seconds(epoch_et_s),
        });
    }
    // Each epoch ends its record, so the segment starts before the first epoch.
    if epoch_et_s < summary.start_epoch_et_s().next_down()
        || epoch_et_s > summary.end_epoch_et_s().next_up()
    {
        return Err(InterpolationError::NoInterpolationData {
            req: Epoch::from_et_seconds(epoch_et_s),
            start: summary.start_epoch(),
            end: summary.end_epoch(),
        });
    }

    // NOTE: We do NOT use the epoch registry. Despite the code being strictly identical to the zero-error
    // Hermite registry search, it led here to extremely large interpolation errors.

    // Like SPKR01, use the first record that ends at or after the epoch, or the last record.
    Ok(epoch_data
        .partition_point(|&epoch_et| epoch_et < epoch_et_s)
        .min(epoch_data.len() - 1))
}

fn check_integrity(
    dataset: &'static str,
    record_data: &[f64],
    epoch_data: &[f64],
    epoch_registry: &[f64],
) -> Result<(), IntegrityError> {
    for val in record_data {
        if !val.is_finite() {
            return Err(IntegrityError::SubNormal {
                dataset,
                variable: "one of the record data",
            });
        }
    }

    for val in epoch_data {
        if !val.is_finite() {
            return Err(IntegrityError::SubNormal {
                dataset,
                variable: "one of the epoch data",
            });
        }
    }

    for val in epoch_registry {
        if !val.is_finite() {
            return Err(IntegrityError::SubNormal {
                dataset,
                variable: "one of the epoch registry",
            });
        }
    }
    Ok(())
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
    fn check_orders_and_nodes(&self) -> Result<(), InterpolationError> {
        // kqmax1 and the per-component integration orders (kq) are read straight from the
        // file and drive the indexing into the fixed-size work buffers (fc, wc, w) and the
        // 3 x MAXDIM difference array in to_pos_vel. Reject any record whose orders fall outside
        // those bounds so a crafted segment cannot index past them.
        // Check the orders on the raw f64 values rather than casting to usize first: a NaN
        // or negative value would saturate to 0 on cast and slip past the kq check.
        let maxdim = self.nodes.len() as f64;
        if !(2.0..=maxdim + 1.0).contains(&self.kqmax1) {
            return Err(InterpolationError::CorruptedData {
                what: "modified difference kqmax1 outside the supported range (2..=MAXDIM+1)",
            });
        }
        if self
            .kq
            .iter()
            .any(|&order| !(1.0..=maxdim).contains(&order))
        {
            return Err(InterpolationError::CorruptedData {
                what: "modified difference integration order (kq) outside the supported range (1..=MAXDIM)",
            });
        }

        // to_pos_vel divides by the first `kqmax1 - 2` interpolation nodes in the recurrence
        // relation, and the nodes are read verbatim from the file. A zero node yields a
        // non-finite state instead of an error. Every other interpolator rejects its divisor
        // (hermite/lagrange reject duplicate abscissae, chebyshev a zero radius, the equal-step
        // decoders a zero step size), so reject a zero node here too.
        let touched_nodes = (self.kqmax1 - 2.0).max(0.0) as usize;
        if self
            .nodes
            .iter()
            .take(touched_nodes)
            .any(|node| node.abs() < f64::EPSILON)
        {
            return Err(InterpolationError::InterpMath {
                source: MathError::DivisionByZero {
                    action: "modified difference interpolation node is zero",
                },
            });
        }
        Ok(())
    }

    pub fn to_pos_vel(&self, epoch_et_s: f64) -> (Vector3, Vector3) {
        let maxdim = self.nodes.len();

        //  Set up for the computation of the various differences.
        let delta = epoch_et_s - self.ref_epoch; // Time delta from reference epoch
        let mut tp = delta;

        // The maximum degree of the polynomials we might need to evaluate.
        // mq2 is the number of coefficients for the recurrence relation.
        let mq2 = self.kqmax1 - 2.0;

        // Initialize lists for the recurrence relation coefficients.
        let mut fc = [0.0; MAXTRM];
        let mut wc = [0.0; MAXTRM - 1];

        for j in 0..mq2.max(0.0) as usize {
            fc[j] = tp / self.nodes[j];
            wc[j] = delta / self.nodes[j];
            tp = delta + self.nodes[j];
        }

        // 3. Compute the W(k) terms for position interpolation.
        let mut w = [0.0; MAXTRM + 2];

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
                // The index is equivalent to dt[i, j] in a 3 x MAXDIM reshaped array.
                let dt_idx = i * maxdim + j;
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
                let dt_idx = i * maxdim + j;
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
        // A record holds 4 * MAXDIM + 11 doubles, and Type 1 records always have MAXDIM = 15.
        let maxdim = (slice.len() - 11) / 4;
        let kqmax1_idx = 4 * maxdim + 7;
        Self {
            ref_epoch: slice[0],
            nodes: &slice[1..=maxdim],
            ref_x_km: slice[maxdim + 1],
            ref_y_km: slice[maxdim + 3],
            ref_z_km: slice[maxdim + 5],
            ref_vx_km_s: slice[maxdim + 2],
            ref_vy_km_s: slice[maxdim + 4],
            ref_vz_km_s: slice[maxdim + 6],
            mod_diff_array: &slice[maxdim + 7..kqmax1_idx],
            kqmax1: slice[kqmax1_idx],
            kq: &slice[kqmax1_idx + 1..kqmax1_idx + 4],
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
            .translate_to_parent(Frame::from_ephem_icrs(-74), epoch)
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

    /// Each Type 1 epoch ends its record, so the segment's final epoch falls in the last record.
    #[test]
    fn spk1_last_epoch_is_evaluated() {
        let almanac = Almanac::default().load("../data/mro.bsp").unwrap();
        let epoch = Epoch::from_et_seconds(812411100.0);

        assert!(
            almanac
                .translate_to_parent(Frame::from_ephem_icrs(-74), epoch)
                .is_ok()
        );
    }

    /// A crafted Type 1 segment whose kqmax1 / kq orders exceed the work-buffer sizes must be
    /// rejected at evaluation rather than indexing past the fixed fc/wc/w buffers in to_pos_vel.
    #[test]
    fn spk1_out_of_range_orders_are_rejected() {
        use super::ModifiedDiffType1;
        use crate::math::interpolation::InterpolationError;
        use crate::naif::daf::NAIFDataSet;
        use crate::naif::spk::summary::SPKSummaryRecord;

        // One record (71 doubles) + one epoch + two trailing metadata doubles.
        let build = |kqmax1: f64, kq0: f64| {
            let mut slice = [0.0_f64; 74];
            // Avoid division by zero on the interpolation nodes (slice[1..16]).
            for n in slice.iter_mut().take(16).skip(1) {
                *n = 1.0;
            }
            slice[67] = kqmax1; // kqmax1
            slice[68] = kq0; // kq[0]
            slice[69] = 1.0;
            slice[70] = 1.0;
            slice[71] = 0.0; // single epoch at 0 ET seconds
            slice[73] = 1.0; // num_records
            slice
        };

        let summary = SPKSummaryRecord::default();

        let oversized_kqmax1 = build(100.0, 1.0);
        let set = ModifiedDiffType1::from_f64_slice(&oversized_kqmax1).unwrap();
        assert!(matches!(
            set.evaluate(0.0, &summary),
            Err(InterpolationError::CorruptedData { .. })
        ));

        let oversized_kq = build(2.0, 100.0);
        let set = ModifiedDiffType1::from_f64_slice(&oversized_kq).unwrap();
        assert!(matches!(
            set.evaluate(0.0, &summary),
            Err(InterpolationError::CorruptedData { .. })
        ));
    }

    /// A crafted Type 1 segment with a zero interpolation node must be rejected rather than
    /// dividing by it in to_pos_vel and returning a non-finite state.
    #[test]
    fn spk1_zero_node_is_rejected() {
        use super::ModifiedDiffType1;
        use crate::naif::daf::NAIFDataSet;
        use crate::naif::spk::summary::SPKSummaryRecord;

        // One record (71 doubles) + one epoch + two trailing metadata doubles.
        // kqmax1 = 3 means the recurrence divides by the first node (slice[1]).
        let build = |node0: f64| {
            let mut slice = [0.0_f64; 74];
            for n in slice.iter_mut().take(16).skip(1) {
                *n = 1.0;
            }
            slice[1] = node0; // first interpolation node
            slice[67] = 3.0; // kqmax1
            slice[68] = 1.0; // kq[0]
            slice[69] = 1.0; // kq[1]
            slice[70] = 1.0; // kq[2]
            slice[71] = 0.0; // single epoch at 0 ET seconds
            slice[73] = 1.0; // num_records
            slice
        };

        let summary = SPKSummaryRecord::default();

        // A zero first node is divided by in the recurrence, so it must be rejected.
        let zero_node = build(0.0);
        let set = ModifiedDiffType1::from_f64_slice(&zero_node).unwrap();
        assert!(set.evaluate(0.0, &summary).is_err());

        // A non-zero node evaluates without error.
        let valid = build(1.0);
        let set = ModifiedDiffType1::from_f64_slice(&valid).unwrap();
        assert!(set.evaluate(0.0, &summary).is_ok());
    }
}

#[cfg(test)]
mod ut_spk21 {
    use super::ModifiedDiffType21;
    use crate::errors::DecodingError;
    use crate::naif::daf::NAIFDataSet;
    use crate::naif::spk::summary::SPKSummaryRecord;

    /// SPICE rejects a MAXDIM above 25, which would also overflow the work buffers.
    #[test]
    fn spk21_out_of_range_maxdim_is_rejected() {
        // One record of dimension 26 (115 doubles) + one epoch + MAXDIM + num_records.
        let mut slice = [0.0_f64; 118];
        slice[116] = 26.0; // MAXDIM
        slice[117] = 1.0; // num_records
        assert!(matches!(
            ModifiedDiffType21::from_f64_slice(&slice),
            Err(DecodingError::Integrity { .. })
        ));

        slice[116] = 0.0;
        assert!(matches!(
            ModifiedDiffType21::from_f64_slice(&slice),
            Err(DecodingError::Integrity { .. })
        ));
    }

    /// kqmax1 is the highest order plus one, so a full-order record reaches MAXDIM + 1.
    #[test]
    fn spk21_full_order_record_is_evaluated() {
        // One record of dimension 25 (111 doubles) + one epoch + MAXDIM + num_records.
        let mut slice = [0.0_f64; 114];
        for node in slice.iter_mut().take(26).skip(1) {
            *node = 1.0;
        }
        slice[107] = 26.0; // kqmax1
        slice[108..111].fill(25.0); // kq
        slice[112] = 25.0; // MAXDIM
        slice[113] = 1.0; // num_records

        let set = ModifiedDiffType21::from_f64_slice(&slice).unwrap();
        assert!(set.evaluate(0.0, &SPKSummaryRecord::default()).is_ok());
    }
}
