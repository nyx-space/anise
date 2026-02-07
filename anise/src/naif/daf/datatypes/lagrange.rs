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
use hifitime::{Duration, Epoch, TimeUnits};
use snafu::{ensure, ResultExt};

use crate::{
    errors::{DecodingError, IntegrityError, TooFewDoublesSnafu},
    math::{
        interpolation::{lagrange_eval, InterpDecodingSnafu, InterpolationError, MAX_SAMPLES},
        Vector3,
    },
    naif::daf::{NAIFDataRecord, NAIFDataSet, NAIFRecord, NAIFSummaryRecord},
    DBL_SIZE,
};

use super::posvel::PositionVelocityRecord;

#[derive(PartialEq)]
pub struct LagrangeSetType8<'a> {
    pub first_state_epoch: Epoch,
    pub step_size: Duration,
    pub degree: usize,
    pub num_records: usize,
    pub record_data: &'a [f64],
}

impl fmt::Display for LagrangeSetType8<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lagrange Type 8: start: {:E}\tstep: {}\twindow size: {}\tnum records: {}\tlen data: {}",
            self.first_state_epoch,
            self.step_size,
            self.degree,
            self.num_records,
            self.record_data.len()
        )
    }
}

impl<'a> NAIFDataSet<'a> for LagrangeSetType8<'a> {
    type StateKind = (Vector3, Vector3);
    type RecordKind = PositionVelocityRecord;
    const DATASET_NAME: &'static str = "Lagrange Type 8";

    fn from_f64_slice(slice: &'a [f64]) -> Result<Self, DecodingError> {
        ensure!(
            slice.len() >= 5,
            TooFewDoublesSnafu {
                dataset: Self::DATASET_NAME,
                need: 5_usize,
                got: slice.len()
            }
        );

        // For this kind of record, the metadata is stored at the very end of the dataset, so we need to read that first.
        let seconds_since_j2000 = slice[slice.len() - 4];
        if !seconds_since_j2000.is_finite() {
            return Err(DecodingError::Integrity {
                source: IntegrityError::SubNormal {
                    dataset: Self::DATASET_NAME,
                    variable: "seconds since J2000 ET",
                },
            });
        }

        let first_state_epoch = Epoch::from_et_seconds(seconds_since_j2000);
        let step_size_s = slice[slice.len() - 3];
        if !step_size_s.is_finite() {
            return Err(DecodingError::Integrity {
                source: IntegrityError::SubNormal {
                    dataset: Self::DATASET_NAME,
                    variable: "step size in seconds",
                },
            });
        }

        let step_size = step_size_s.seconds();
        let degree = slice[slice.len() - 2] as usize;
        let num_records = slice[slice.len() - 1] as usize;

        let record_data = &slice[0..slice.len() - 4];
        ensure!(
            record_data.len() == 6 * num_records,
            TooFewDoublesSnafu {
                dataset: Self::DATASET_NAME,
                need: 6 * num_records,
                got: record_data.len(),
            }
        );

        Ok(Self {
            first_state_epoch,
            step_size,
            degree,
            num_records,
            record_data,
        })
    }

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, DecodingError> {
        let rcrd_len = 6;
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
        let et = epoch.to_et_seconds();
        let t0 = self.first_state_epoch.to_et_seconds();
        let h = self.step_size.to_seconds();

        if h.abs() < f64::EPSILON {
            return Err(InterpolationError::CorruptedData {
                what: "step size is zero",
            });
        }

        // Find the index such that t0 + idx * h <= et < t0 + (idx + 1) * h
        let idx_f = (et - t0) / h;

        // Exact match check
        if (idx_f - idx_f.round()).abs() < 1e-12 {
            let idx = idx_f.round() as isize;
            if idx >= 0 && idx < self.num_records as isize {
                return Ok(self
                    .nth_record(idx as usize)
                    .context(InterpDecodingSnafu)?
                    .to_pos_vel());
            }
        }

        let group_size = self.degree + 1;
        let idx = idx_f.floor() as isize;

        // Selection logic from SPICE: centered as closely as possible.
        // For N points, if target is in [t_i, t_{i+1}], we use i - (N-1)/2 as the first index.
        let first_idx = (idx - ((group_size as isize - 1) / 2))
            .max(0)
            .min((self.num_records as isize - group_size as isize).max(0));

        let last_idx = (first_idx + group_size as isize).min(self.num_records as isize);
        let actual_group_size = (last_idx - first_idx) as usize;

        // Statically allocated arrays of the maximum number of samples
        let mut epochs = [0.0; MAX_SAMPLES];
        let mut xs = [0.0; MAX_SAMPLES];
        let mut ys = [0.0; MAX_SAMPLES];
        let mut zs = [0.0; MAX_SAMPLES];
        let mut vxs = [0.0; MAX_SAMPLES];
        let mut vys = [0.0; MAX_SAMPLES];
        let mut vzs = [0.0; MAX_SAMPLES];

        for (cno, cur_idx) in (first_idx..last_idx).enumerate() {
            let record = self
                .nth_record(cur_idx as usize)
                .context(InterpDecodingSnafu)?;
            xs[cno] = record.x_km;
            ys[cno] = record.y_km;
            zs[cno] = record.z_km;
            vxs[cno] = record.vx_km_s;
            vys[cno] = record.vy_km_s;
            vzs[cno] = record.vz_km_s;
            epochs[cno] = t0 + (cur_idx as f64) * h;
        }

        let (x_km, _) = lagrange_eval(&epochs[..actual_group_size], &xs[..actual_group_size], et)?;
        let (y_km, _) = lagrange_eval(&epochs[..actual_group_size], &ys[..actual_group_size], et)?;
        let (z_km, _) = lagrange_eval(&epochs[..actual_group_size], &zs[..actual_group_size], et)?;
        let (vx_km_s, _) =
            lagrange_eval(&epochs[..actual_group_size], &vxs[..actual_group_size], et)?;
        let (vy_km_s, _) =
            lagrange_eval(&epochs[..actual_group_size], &vys[..actual_group_size], et)?;
        let (vz_km_s, _) =
            lagrange_eval(&epochs[..actual_group_size], &vzs[..actual_group_size], et)?;

        Ok((
            Vector3::new(x_km, y_km, z_km),
            Vector3::new(vx_km_s, vy_km_s, vz_km_s),
        ))
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

#[derive(PartialEq)]
pub struct LagrangeSetType9<'a> {
    pub degree: usize,
    pub num_records: usize,
    pub state_data: &'a [f64],
    pub epoch_data: &'a [f64],
    pub epoch_registry: &'a [f64],
}

impl fmt::Display for LagrangeSetType9<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lagrange Type 9 from {:E} to {:E} with degree {} ({} items, {} epoch directories)",
            Epoch::from_et_seconds(*self.epoch_data.first().unwrap()),
            Epoch::from_et_seconds(*self.epoch_data.last().unwrap()),
            self.degree,
            self.epoch_data.len(),
            self.epoch_registry.len()
        )
    }
}

impl<'a> NAIFDataSet<'a> for LagrangeSetType9<'a> {
    type StateKind = (Vector3, Vector3);
    type RecordKind = PositionVelocityRecord;
    const DATASET_NAME: &'static str = "Lagrange Type 9";

    fn from_f64_slice(slice: &'a [f64]) -> Result<Self, DecodingError> {
        ensure!(
            slice.len() >= 3,
            TooFewDoublesSnafu {
                dataset: Self::DATASET_NAME,
                need: 3_usize,
                got: slice.len()
            }
        );

        // For this kind of record, the metadata is stored at the very end of the dataset
        let num_records = slice[slice.len() - 1] as usize;
        let degree = slice[slice.len() - 2] as usize;
        // NOTE: The ::SIZE returns the C representation memory size of this, but we only want the number of doubles.
        let state_data_end_idx = PositionVelocityRecord::SIZE / DBL_SIZE * num_records;
        let state_data = slice.get(0..state_data_end_idx).unwrap();
        let epoch_data_end_idx = state_data_end_idx + num_records;
        let epoch_data = slice.get(state_data_end_idx..epoch_data_end_idx).unwrap();
        // And the epoch directory is whatever remains minus the metadata
        let epoch_registry = slice.get(epoch_data_end_idx..slice.len() - 2).unwrap();

        Ok(Self {
            degree,
            num_records,
            state_data,
            epoch_data,
            epoch_registry,
        })
    }

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, DecodingError> {
        let rcrd_len = self.state_data.len() / self.num_records;
        Ok(Self::RecordKind::from_slice_f64(
            self.state_data
                .get(n * rcrd_len..(n + 1) * rcrd_len)
                .ok_or(DecodingError::InaccessibleBytes {
                    start: n * rcrd_len,
                    end: (n + 1) * rcrd_len,
                    size: self.state_data.len(),
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
        if epoch.to_et_seconds() < self.epoch_data[0] - 1e-7
            || epoch.to_et_seconds() > *self.epoch_data.last().unwrap() + 1e-7
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

        // Now, perform a binary search on the epochs themselves.
        match search_data_slice.binary_search_by(|epoch_et| {
            epoch_et
                .partial_cmp(&epoch.to_et_seconds())
                .expect("epochs in Lagrange data is now NaN or infinite but was not before")
        }) {
            Ok(idx) => {
                // Oh wow, this state actually exists, no interpolation needed!
                Ok(self
                    .nth_record(idx + slice_offset)
                    .context(InterpDecodingSnafu)?
                    .to_pos_vel())
            }
            Err(idx) => {
                // We didn't find it, so let's build an interpolation here.
                let absolute_insertion_idx = idx + slice_offset;
                let group_size = self.degree + 1;
                let num_left = group_size / 2;

                // Ensure that we aren't fetching out of the window
                let mut first_idx = absolute_insertion_idx.saturating_sub(num_left);
                let last_idx = self.num_records.min(first_idx + group_size);

                // Check that we have enough samples
                if last_idx == self.num_records {
                    first_idx = last_idx - 2 * num_left;
                }

                // Statically allocated arrays of the maximum number of samples
                let mut epochs = [0.0; MAX_SAMPLES];
                let mut xs = [0.0; MAX_SAMPLES];
                let mut ys = [0.0; MAX_SAMPLES];
                let mut zs = [0.0; MAX_SAMPLES];
                let mut vxs = [0.0; MAX_SAMPLES];
                let mut vys = [0.0; MAX_SAMPLES];
                let mut vzs = [0.0; MAX_SAMPLES];

                for (cno, idx) in (first_idx..last_idx).enumerate() {
                    let record = self.nth_record(idx).context(InterpDecodingSnafu)?;
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
                let (x_km, _) = lagrange_eval(
                    &epochs[..group_size],
                    &xs[..group_size],
                    epoch.to_et_seconds(),
                )?;

                let (y_km, _) = lagrange_eval(
                    &epochs[..group_size],
                    &ys[..group_size],
                    epoch.to_et_seconds(),
                )?;

                let (z_km, _) = lagrange_eval(
                    &epochs[..group_size],
                    &zs[..group_size],
                    epoch.to_et_seconds(),
                )?;

                let (vx_km_s, _) = lagrange_eval(
                    &epochs[..group_size],
                    &vxs[..group_size],
                    epoch.to_et_seconds(),
                )?;

                let (vy_km_s, _) = lagrange_eval(
                    &epochs[..group_size],
                    &vys[..group_size],
                    epoch.to_et_seconds(),
                )?;

                let (vz_km_s, _) = lagrange_eval(
                    &epochs[..group_size],
                    &vzs[..group_size],
                    epoch.to_et_seconds(),
                )?;

                // And build the result
                let pos_km = Vector3::new(x_km, y_km, z_km);
                let vel_km_s = Vector3::new(vx_km_s, vy_km_s, vz_km_s);

                Ok((pos_km, vel_km_s))
            }
        }
    }

    fn check_integrity(&self) -> Result<(), IntegrityError> {
        // Verify that none of the data is invalid once when we load it.
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
                    variable: "one of the epoch registry data",
                });
            }
        }

        for val in self.state_data {
            if !val.is_finite() {
                return Err(IntegrityError::SubNormal {
                    dataset: Self::DATASET_NAME,
                    variable: "one of the state data",
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod ut_lagrange {
    use super::*;
    use crate::naif::spk::summary::SPKSummaryRecord;
    use hifitime::Epoch;

    #[test]
    fn test_lagrange_type8() {
        let num_records = 100;
        let degree = 3;
        let h = 60.0;
        let t0 = Epoch::from_et_seconds(0.0);

        let mut record_data = Vec::with_capacity(num_records * 6);
        for i in 0..num_records {
            let t = (i as f64) * h;
            // Quadratic motion: x = t^2 + t + 1, vx = 2t + 1
            record_data.push(t * t + t + 1.0); // x
            record_data.push(0.0); // y
            record_data.push(0.0); // z
            record_data.push(2.0 * t + 1.0); // vx
            record_data.push(0.0); // vy
            record_data.push(0.0); // vz
        }

        let dataset = LagrangeSetType8 {
            first_state_epoch: t0,
            step_size: h.seconds(),
            degree,
            num_records,
            record_data: &record_data,
        };

        let summary = SPKSummaryRecord::default();

        // Test exact match
        let epoch = Epoch::from_et_seconds(60.0);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        assert!((result.0.x - (60.0 * 60.0 + 60.0 + 1.0)).abs() < 1e-12);
        assert!((result.1.x - (2.0 * 60.0 + 1.0)).abs() < 1e-12);

        // Test interpolation (mid-point of an interval)
        let epoch = Epoch::from_et_seconds(90.0);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        // Since motion is quadratic and degree is 3, Lagrange should be exact.
        assert!((result.0.x - (90.0 * 90.0 + 90.0 + 1.0)).abs() < 1e-12);
        assert!((result.1.x - (2.0 * 90.0 + 1.0)).abs() < 1e-12);

        // Test near start
        let epoch = Epoch::from_et_seconds(10.0);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        assert!((result.0.x - (10.0 * 10.0 + 10.0 + 1.0)).abs() < 1e-12);

        // Test near end
        let epoch = Epoch::from_et_seconds((num_records as f64 - 1.5) * h);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        let et = (num_records as f64 - 1.5) * h;
        assert!((result.0.x - (et * et + et + 1.0)).abs() < 1e-12);
    }

    #[test]
    fn test_lagrange_optimization() {
        // Construct a synthetic LagrangeSetType9
        let num_records = 250;
        let degree = 1; // Linear interpolation
        let mut epoch_data = Vec::with_capacity(num_records);
        let mut state_data = Vec::with_capacity(num_records * 6);
        let mut epoch_registry = Vec::new();

        for i in 0..num_records {
            let t = i as f64;
            epoch_data.push(t);
            // Linear motion: x = t, y = 2*t, z = 3*t
            state_data.push(t); // x
            state_data.push(2.0 * t); // y
            state_data.push(3.0 * t); // z
            state_data.push(1.0); // vx
            state_data.push(2.0); // vy
            state_data.push(3.0); // vz

            // Build registry every 100 records (indices 99, 199, ...)
            if (i + 1) % 100 == 0 {
                epoch_registry.push(t);
            }
        }

        let dataset = LagrangeSetType9 {
            degree,
            num_records,
            state_data: &state_data,
            epoch_data: &epoch_data,
            epoch_registry: &epoch_registry,
        };

        let summary = SPKSummaryRecord::default();

        // Test exact match
        let epoch = Epoch::from_et_seconds(10.0);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        assert_eq!(result.0.x, 10.0);

        // Test interpolation
        let epoch = Epoch::from_et_seconds(10.5);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        assert_eq!(result.0.x, 10.5);
        assert_eq!(result.0.y, 21.0);

        // Test boundary of registry block

        // Target: 99.5.
        // partition_point(|x| x < 99.5): returns 1.
        // sub_array_start_idx = 99.
        // Slice: 99..=198.
        // 99.5 is > epoch_data[99] (99.0).
        let epoch = Epoch::from_et_seconds(99.5);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        assert_eq!(result.0.x, 99.5);

        // Target: 100.5.
        // partition_point(|x| x < 100.5): 1.
        // Same slice. 100.5 is in 99..=198.
        let epoch = Epoch::from_et_seconds(100.5);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        assert_eq!(result.0.x, 100.5);

        // Target: 200.5
        // partition_point(|x| x < 200.5): 2.
        // dir_idx = 2.
        // start = 199.
        // Slice 199..=249.
        let epoch = Epoch::from_et_seconds(200.5);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        assert_eq!(result.0.x, 200.5);
    }
}
