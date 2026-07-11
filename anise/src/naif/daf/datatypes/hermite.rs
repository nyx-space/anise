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
use snafu::{ResultExt, ensure};

use crate::errors::{DecodingError, IntegrityError, TooFewDoublesSnafu};
use crate::math::interpolation::{
    InterpDecodingSnafu, InterpolationError, MAX_SAMPLES, hermite_eval,
};
use crate::naif::daf::NAIFSummaryRecord;
use crate::{
    DBL_SIZE,
    math::Vector3,
    naif::daf::{NAIFDataRecord, NAIFDataSet, NAIFRecord},
};

use super::posvel::PositionVelocityRecord;

#[derive(PartialEq)]
pub struct HermiteSetType12<'a> {
    pub first_state_epoch: Epoch,
    pub step_size: Duration,
    pub samples: usize,
    pub num_records: usize,
    pub record_data: &'a [f64],
}

impl fmt::Display for HermiteSetType12<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hermite Type 12: start: {:E}\tstep: {}\tsamples: {}\tnum records: {}\tlen data: {}",
            self.first_state_epoch,
            self.step_size,
            self.samples,
            self.num_records,
            self.record_data.len()
        )
    }
}

impl<'a> NAIFDataSet<'a> for HermiteSetType12<'a> {
    type StateKind = (Vector3, Vector3);
    type RecordKind = PositionVelocityRecord;
    const DATASET_NAME: &'static str = "Hermite Type 12";

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
        // NOTE: The Type 12 and 13 specify that the windows size minus one is stored!
        let samples = slice[slice.len() - 2] as usize + 1;
        if samples > MAX_SAMPLES {
            return Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: Self::DATASET_NAME,
                    variable: "number of interpolation samples",
                    value: samples as f64,
                    reason: "must be less than or equal to MAX_SAMPLES (32)",
                },
            });
        }
        let num_records_f64 = slice[slice.len() - 1];
        if !num_records_f64.is_finite() || num_records_f64 <= 0.0 {
            return Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: Self::DATASET_NAME,
                    variable: "number of records",
                    value: num_records_f64,
                    reason: "must be a finite value greater than zero",
                },
            });
        }
        let num_records = num_records_f64 as usize;

        // Type 12 stores equal-step states, so the record area is exactly six doubles per
        // record with no per-state epochs. Without this the divide in nth_record yields a
        // record length below six and the PositionVelocityRecord decoder indexes past the
        // slice. Matches the Type 8 Lagrange (equal-step) decoder.
        let record_data = &slice[0..slice.len() - 4];
        let need = num_records.saturating_mul(6);
        ensure!(
            record_data.len() == need,
            TooFewDoublesSnafu {
                dataset: Self::DATASET_NAME,
                need,
                got: record_data.len(),
            }
        );

        Ok(Self {
            first_state_epoch,
            step_size,
            samples,
            num_records,
            record_data,
        })
    }

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, DecodingError> {
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
        summary: &S,
    ) -> Result<Self::StateKind, InterpolationError> {
        if epoch < summary.start_epoch() - 1e-7.seconds()
            || epoch > summary.end_epoch() + 1e-7.seconds()
        {
            return Err(InterpolationError::NoInterpolationData {
                req: epoch,
                start: summary.start_epoch(),
                end: summary.end_epoch(),
            });
        }

        let delta_t_s = (epoch - self.first_state_epoch).to_seconds();
        let step_size_s = self.step_size.to_seconds();
        let float_index = delta_t_s / step_size_s;

        let mut first_idx = if self.samples.is_multiple_of(2) {
            // Even window size
            let i = float_index.floor() as usize;
            i.saturating_sub(self.samples / 2 - 1)
        } else {
            // Odd window size
            let nearest_i = float_index.round() as usize;
            nearest_i.saturating_sub((self.samples - 1) / 2)
        };

        // Ensure we don't go past the end of the records
        if first_idx + self.samples > self.num_records {
            first_idx = self.num_records.saturating_sub(self.samples);
        }

        // Statically allocated arrays of the maximum number of samples
        let mut epochs = [0.0; MAX_SAMPLES];
        let mut xs = [0.0; MAX_SAMPLES];
        let mut ys = [0.0; MAX_SAMPLES];
        let mut zs = [0.0; MAX_SAMPLES];
        let mut vxs = [0.0; MAX_SAMPLES];
        let mut vys = [0.0; MAX_SAMPLES];
        let mut vzs = [0.0; MAX_SAMPLES];
        for (cno, idx) in (first_idx..first_idx + self.samples).enumerate() {
            let record = self.nth_record(idx).context(InterpDecodingSnafu)?;
            xs[cno] = record.x_km;
            ys[cno] = record.y_km;
            zs[cno] = record.z_km;
            vxs[cno] = record.vx_km_s;
            vys[cno] = record.vy_km_s;
            vzs[cno] = record.vz_km_s;
            epochs[cno] = (self.first_state_epoch + (idx as f64) * self.step_size).to_et_seconds();
        }

        // Build the interpolation polynomials making sure to limit the slices to exactly the number of items we actually used
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

        Ok((pos_km, vel_km_s))
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

#[derive(Default, PartialEq)]
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

impl HermiteSetType13<'_> {
    pub fn degree(&self) -> usize {
        2 * self.samples - 1
    }
}

impl fmt::Display for HermiteSetType13<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hermite Type 13 from {:E} to {:E} with degree {} ({} items, {} epoch directories)",
            Epoch::from_et_seconds(*self.epoch_data.first().unwrap_or(&0.0)),
            Epoch::from_et_seconds(*self.epoch_data.last().unwrap_or(&0.0)),
            self.degree(),
            self.epoch_data.len(),
            self.epoch_registry.len()
        )
    }
}

impl<'a> NAIFDataSet<'a> for HermiteSetType13<'a> {
    type StateKind = (Vector3, Vector3);
    type RecordKind = PositionVelocityRecord;
    const DATASET_NAME: &'static str = "Hermite Type 13";

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
        let num_records_f64 = slice[slice.len() - 1];
        if !num_records_f64.is_finite() {
            return Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: Self::DATASET_NAME,
                    variable: "number of records",
                    value: num_records_f64,
                    reason: "must be a finite value",
                },
            });
        }
        let num_records = num_records_f64 as usize;

        // NOTE: The Type 12 and 13 specify that the windows size minus one is stored!
        let num_samples_f64 = slice[slice.len() - 2];
        if !num_samples_f64.is_finite() {
            return Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: Self::DATASET_NAME,
                    variable: "number of interpolation samples",
                    value: num_samples_f64,
                    reason: "must be a finite value",
                },
            });
        }

        let samples = num_samples_f64 as usize + 1;
        if samples > MAX_SAMPLES {
            return Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: Self::DATASET_NAME,
                    variable: "number of interpolation samples",
                    value: samples as f64,
                    reason: "must be less than or equal to MAX_SAMPLES (32)",
                },
            });
        }
        // A non-empty segment must carry at least a full interpolation window of states.
        // With fewer, evaluate() recentres the window with `last_idx - 2 * num_left`, which
        // underflows and panics when the segment is queried at an interior epoch.
        if num_records > 0 && num_records < samples {
            return Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: Self::DATASET_NAME,
                    variable: "number of records",
                    value: num_records as f64,
                    reason: "must be at least the interpolation window size (samples)",
                },
            });
        }
        // NOTE: The ::SIZE returns the C representation memory size of this, but we only want the number of doubles.
        let state_data_end_idx = PositionVelocityRecord::SIZE / DBL_SIZE * num_records;
        let state_data =
            slice
                .get(0..state_data_end_idx)
                .ok_or(DecodingError::InaccessibleBytes {
                    start: 0,
                    end: state_data_end_idx,
                    size: slice.len(),
                })?;
        let epoch_data_end_idx = state_data_end_idx + num_records;
        let epoch_data = slice.get(state_data_end_idx..epoch_data_end_idx).ok_or(
            DecodingError::InaccessibleBytes {
                start: state_data_end_idx,
                end: epoch_data_end_idx,
                size: slice.len(),
            },
        )?;
        // And the epoch directory is whatever remains minus the metadata
        let epoch_registry = slice.get(epoch_data_end_idx..slice.len() - 2).ok_or(
            DecodingError::InaccessibleBytes {
                start: epoch_data_end_idx,
                end: slice.len() - 2,
                size: slice.len(),
            },
        )?;

        Ok(Self {
            samples,
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
        let last_epoch = *self
            .epoch_data
            .last()
            .ok_or(InterpolationError::MissingInterpolationData { epoch })?;
        if epoch.to_et_seconds() < self.epoch_data[0] - 1e-7
            || epoch.to_et_seconds() > last_epoch + 1e-7
        {
            return Err(InterpolationError::NoInterpolationData {
                req: epoch,
                start: Epoch::from_et_seconds(self.epoch_data[0]),
                end: Epoch::from_et_seconds(last_epoch),
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

            // The directory index is derived from the untrusted epoch registry, whose length
            // and values need not agree with num_records. If it lands past the epoch data the
            // slice below would panic, so treat an out-of-range directory as corrupt data.
            if sub_array_start_idx >= self.num_records {
                return Err(InterpolationError::CorruptedData {
                    what: "epoch directory index is out of bounds of the epoch data",
                });
            }

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

        match search_data_slice.binary_search_by(|epoch_et| {
            epoch_et
                .partial_cmp(&epoch.to_et_seconds())
                .expect("epochs in Hermite data is now NaN or infinite but was not before")
        }) {
            Ok(idx) => {
                // Oh wow, this state actually exists, no interpolation needed!
                Ok(self
                    .nth_record(idx + slice_offset)
                    .context(InterpDecodingSnafu)?
                    .to_pos_vel())
            }
            Err(idx) => {
                // We didn't find et_target exactly. `idx` is the insertion point in `search_data_slice`.
                // Convert `idx` (local insertion point) to an absolute index in `self.epoch_data`.
                let absolute_insertion_idx = idx + slice_offset;
                let num_left = self.samples / 2;

                // Ensure that we aren't fetching out of the window
                let mut first_idx = absolute_insertion_idx.saturating_sub(num_left);
                let last_idx = self.num_records.min(first_idx + self.samples);

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
mod hermite_ut {
    use crate::{
        errors::{DecodingError, IntegrityError},
        naif::daf::NAIFDataSet,
    };

    use super::HermiteSetType13;

    #[test]
    fn too_small() {
        if HermiteSetType13::from_f64_slice(&[0.1, 0.2])
            != Err(DecodingError::TooFewDoubles {
                dataset: "Hermite Type 13",
                got: 2,
                need: 3,
            })
        {
            panic!("test failure");
        }
    }

    #[test]
    fn invalid_data() {
        // Two metadata, one state, one epoch
        let zeros = [0.0_f64; 2 * 7 + 2];

        let mut invalid_num_records = zeros;
        invalid_num_records[zeros.len() - 1] = f64::INFINITY;
        match HermiteSetType13::from_f64_slice(&invalid_num_records) {
            Ok(_) => panic!("test failed on invalid num records"),
            Err(e) => {
                assert_eq!(
                    e,
                    DecodingError::Integrity {
                        source: IntegrityError::InvalidValue {
                            dataset: "Hermite Type 13",
                            variable: "number of records",
                            value: f64::INFINITY,
                            reason: "must be a finite value",
                        },
                    }
                );
            }
        }

        let mut invalid_num_samples = zeros;
        invalid_num_samples[zeros.len() - 2] = f64::INFINITY;
        match HermiteSetType13::from_f64_slice(&invalid_num_samples) {
            Ok(_) => panic!("test failed on invalid num samples"),
            Err(e) => {
                assert_eq!(
                    e,
                    DecodingError::Integrity {
                        source: IntegrityError::InvalidValue {
                            dataset: "Hermite Type 13",
                            variable: "number of interpolation samples",
                            value: f64::INFINITY,
                            reason: "must be a finite value",
                        },
                    }
                );
            }
        }

        let mut invalid_epoch = zeros;
        invalid_epoch[zeros.len() - 3] = f64::INFINITY;

        let dataset = HermiteSetType13::from_f64_slice(&invalid_epoch).unwrap();
        match dataset.check_integrity() {
            Ok(_) => panic!("test failed on invalid interval_length"),
            Err(e) => {
                assert_eq!(
                    e,
                    IntegrityError::SubNormal {
                        dataset: "Hermite Type 13",
                        variable: "one of the epoch registry data",
                    },
                );
            }
        }

        let mut invalid_record = zeros;
        invalid_record[0] = f64::INFINITY;
        // Force the number of records to be one, otherwise everything is considered the epoch registry
        invalid_record[zeros.len() - 1] = 1.0;

        let dataset = HermiteSetType13::from_f64_slice(&invalid_record).unwrap();
        match dataset.check_integrity() {
            Ok(_) => panic!("test failed on invalid interval_length"),
            Err(e) => {
                assert_eq!(
                    e,
                    IntegrityError::SubNormal {
                        dataset: "Hermite Type 13",
                        variable: "one of the state data",
                    },
                );
            }
        }
    }

    #[test]
    fn rejects_window_larger_than_records_type13() {
        // num_records = 4 but the declared window is samples = 7. evaluate() would recentre the
        // window with `last_idx - 2 * num_left` and underflow when queried at an interior epoch,
        // so the segment must be rejected at decode time.
        let mut slice = vec![0.0_f64; 30];
        slice[28] = 6.0; // window size - 1 => samples = 7
        slice[29] = 4.0; // num_records
        match HermiteSetType13::from_f64_slice(&slice) {
            Ok(_) => panic!("a window larger than the record count must be rejected"),
            Err(e) => assert_eq!(
                e,
                DecodingError::Integrity {
                    source: IntegrityError::InvalidValue {
                        dataset: "Hermite Type 13",
                        variable: "number of records",
                        value: 4.0,
                        reason: "must be at least the interpolation window size (samples)",
                    },
                }
            ),
        }
    }

    #[test]
    fn type13_registry_index_out_of_bounds() {
        use super::HermiteSetType13;
        use crate::math::interpolation::InterpolationError;
        use crate::naif::spk::summary::SPKSummaryRecord;
        use hifitime::Epoch;

        // Two records (12 state doubles + 2 epochs), one epoch-registry entry, window size 2.
        // The registry entry sits below an in-range query epoch, so the directory search yields
        // start index 99, which points past the 2-element epoch data.
        let mut slice = vec![0.0_f64; 17];
        slice[12] = 0.0; // epoch_data[0]
        slice[13] = 100.0; // epoch_data[1]
        slice[14] = -1000.0; // single registry entry, below the query epoch
        slice[15] = 1.0; // samples - 1 => window size 2
        slice[16] = 2.0; // num_records
        let set = HermiteSetType13::from_f64_slice(&slice).unwrap();
        let summary = SPKSummaryRecord::default();
        match set.evaluate(Epoch::from_et_seconds(50.0), &summary) {
            Ok(_) => panic!("an out-of-range epoch directory must be rejected"),
            Err(e) => assert_eq!(
                e,
                InterpolationError::CorruptedData {
                    what: "epoch directory index is out of bounds of the epoch data",
                }
            ),
        }
    }

    #[test]
    fn type12_zero_records() {
        use super::HermiteSetType12;

        // A Type 12 segment whose trailing metadata declares num_records = 0.
        // nth_record divides record_data.len() by num_records during evaluate, so
        // this must be rejected at decode time rather than panicking later.
        let mut slice = vec![0.0_f64; 10];
        let n = slice.len();
        slice[n - 4] = 0.0; // first state epoch
        slice[n - 3] = 10.0; // step size
        slice[n - 2] = 3.0; // window size - 1 => samples = 4
        slice[n - 1] = 0.0; // num_records = 0

        if HermiteSetType12::from_f64_slice(&slice)
            != Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: "Hermite Type 12",
                    variable: "number of records",
                    value: 0.0,
                    reason: "must be a finite value greater than zero",
                },
            })
        {
            panic!("Type 12 with zero records should be rejected at decode time");
        }
    }

    #[test]
    fn type12_short_record_data() {
        use super::HermiteSetType12;

        // record_data holds six doubles but num_records = 5, so nth_record would compute a
        // record length of one and the PositionVelocityRecord decoder would index past it.
        // Reject the mismatch at decode time.
        let mut slice = vec![0.0_f64; 10];
        let n = slice.len();
        slice[n - 4] = 0.0; // first state epoch
        slice[n - 3] = 1.0; // step size
        slice[n - 2] = 0.0; // window size - 1 => samples = 1
        slice[n - 1] = 5.0; // num_records = 5

        if HermiteSetType12::from_f64_slice(&slice)
            != Err(DecodingError::TooFewDoubles {
                dataset: "Hermite Type 12",
                need: 30,
                got: 6,
            })
        {
            panic!("Type 12 with record data shorter than num_records must be rejected");
        }
    }

    #[test]
    fn test_hermite_type12() {
        use super::HermiteSetType12;
        use crate::naif::spk::summary::SPKSummaryRecord;
        use hifitime::Epoch;

        // Construct a synthetic HermiteSetType12
        let num_records = 10;
        let samples = 4; // Window size 4, degree 7
        let step_size_s = 10.0;
        let first_epoch_s = 100.0;

        let mut record_data = Vec::with_capacity(num_records * 6);
        for i in 0..num_records {
            let t = first_epoch_s + (i as f64) * step_size_s;
            // Linear motion: x = t, y = 2*t, z = 3*t
            record_data.push(t); // x
            record_data.push(2.0 * t); // y
            record_data.push(3.0 * t); // z
            record_data.push(1.0); // vx
            record_data.push(2.0); // vy
            record_data.push(3.0); // vz
        }

        // Add metadata at the end
        let mut slice_data = record_data.clone();
        slice_data.push(first_epoch_s);
        slice_data.push(step_size_s);
        slice_data.push((samples - 1) as f64); // stores window size - 1
        slice_data.push(num_records as f64);

        let dataset = HermiteSetType12::from_f64_slice(&slice_data).unwrap();
        assert_eq!(dataset.samples, samples);
        assert_eq!(dataset.num_records, num_records);

        let mut summary = SPKSummaryRecord::default();
        summary.start_epoch_et_s = first_epoch_s;
        summary.end_epoch_et_s = first_epoch_s + (num_records as f64 - 1.0) * step_size_s;

        // Test exact match
        let epoch = Epoch::from_et_seconds(120.0);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        assert!((result.0.x - 120.0).abs() < 1e-12);
        assert!((result.0.y - 240.0).abs() < 1e-12);
        assert!((result.1.x - 1.0).abs() < 1e-12);

        // Test interpolation
        let epoch = Epoch::from_et_seconds(125.0);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        assert!((result.0.x - 125.0).abs() < 1e-12);
        assert!((result.0.y - 250.0).abs() < 1e-12);
        assert!((result.1.x - 1.0).abs() < 1e-12);

        // Test boundary case: near start
        let epoch = Epoch::from_et_seconds(105.0);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        assert!((result.0.x - 105.0).abs() < 1e-12);

        // Test boundary case: near end
        let epoch = Epoch::from_et_seconds(185.0);
        let result = dataset.evaluate(epoch, &summary).unwrap();
        assert!((result.0.x - 185.0).abs() < 1e-12);
    }
}
