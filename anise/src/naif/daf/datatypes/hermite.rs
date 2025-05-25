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

use crate::errors::{DecodingError, IntegrityError, TooFewDoublesSnafu};
use crate::math::interpolation::StridedDataAccess;
use crate::math::interpolation::{hermite_eval, InterpDecodingSnafu, InterpolationError};
use crate::naif::daf::NAIFSummaryRecord;
use crate::{
    math::{cartesian::CartesianState, Vector3},
    naif::daf::{NAIFDataRecord, NAIFDataSet, NAIFRecord},
    DBL_SIZE,
};

use super::posvel::PositionVelocityRecord;

/// Provides strided access to a specific component (e.g., X-position, Y-velocity)
/// within a slice of state data records.
///
/// This accessor treats a portion of a larger `state_data` slice as a sequence of records,
/// each with a defined `record_stride`, and allows access to a specific `component_offset`
/// within each record in that sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentAccessor<'a> {
    data: &'a [f64],         // Reference to the full state_data slice
    start_record_idx: usize, // The index of the first record in `data` that this accessor views
    num_records: usize,      // The number of records in this accessor's view
    component_offset: usize, // Offset within each record to access the specific component (0 for X, 1 for Y, etc.)
    record_stride: usize, // Number of f64s per complete record (e.g., 6 for PositionVelocityRecord)
}

impl<'a> ComponentAccessor<'a> {
    /// Creates a new `ComponentAccessor`.
    ///
    /// # Arguments
    /// * `data`: The underlying slice of f64 data containing all state records.
    /// * `start_record_idx`: The starting record index in `data` for the window this accessor covers.
    /// * `num_records`: The number of records this accessor will expose.
    /// * `component_offset`: The 0-indexed offset of the component within each record
    ///   (e.g., 0 for X, 1 for Y, ..., 5 for Vz in a PositionVelocityRecord).
    /// * `record_stride`: The total number of f64 values that make up one full record.
    pub fn new(
        data: &'a [f64],
        start_record_idx: usize,
        num_records: usize,
        component_offset: usize,
        record_stride: usize,
    ) -> Self {
        // Basic validation: component_offset must be less than record_stride.
        assert!(
            component_offset < record_stride,
            "Component offset must be less than record stride."
        );
        // Further validation: check if the access window is within the bounds of `data`.
        // The last accessed element via this accessor (conceptually) would be for the
        // (num_records - 1)-th record in the window.
        // Its data index would be:
        // (start_record_idx + num_records - 1) * record_stride + component_offset
        if num_records > 0 {
            assert!(
                (start_record_idx + num_records - 1) * record_stride + component_offset
                    < data.len(),
                "ComponentAccessor window exceeds bounds of underlying data slice."
            );
        } else {
            // If num_records is 0, it's an empty accessor, which is fine.
            // start_record_idx can be anything if num_records is 0, but for consistency,
            // let's ensure that start_record_idx * record_stride + component_offset doesn't cause issues
            // if data is also empty. However, data.len() handles empty data slice fine.
            // This case is generally okay.
        }

        Self {
            data,
            start_record_idx,
            num_records,
            component_offset,
            record_stride,
        }
    }
}

impl<'a> StridedDataAccess for ComponentAccessor<'a> {
    /// Returns the number of records (and thus, component values) this accessor covers.
    #[inline]
    fn len(&self) -> usize {
        self.num_records
    }

    /// Returns `true` if this accessor covers zero records.
    #[inline]
    fn is_empty(&self) -> bool {
        self.num_records == 0
    }

    /// Retrieves the component value for the `index`-th record in this accessor's window.
    ///
    /// `index` is 0-based relative to the start of the window defined by `start_record_idx`
    /// and `num_records`.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds (i.e., `index >= self.num_records`).
    #[inline]
    fn get(&self, index: usize) -> f64 {
        if index >= self.num_records {
            panic!(
                "Index out of bounds for ComponentAccessor: index was {}, but len is {}",
                index, self.num_records
            );
        }
        // Calculate the actual index in the underlying `data` slice.
        // This formula correctly navigates to the start of the `index`-th record
        // within the window (which is `start_record_idx + index` in absolute record terms)
        // and then applies the component offset.
        let data_idx = (self.start_record_idx + index) * self.record_stride + self.component_offset;
        self.data[data_idx]
    }
}

/// A container to provide easy access to Hermite interpolation data points.
#[derive(Debug, Clone, PartialEq)]
pub struct HermiteInterpolationData<'a> {
    /// Full slice of state data, where each record is contiguous (e.g., [X, Y, Z, Vx, Vy, Vz]).
    state_data: &'a [f64],
    /// Full slice of epoch data corresponding to each record in `state_data`.
    epoch_data: &'a [f64],
    /// Index of the first record in `state_data` and `epoch_data` to be included in this interpolation window.
    first_record_idx: usize,
    /// The number of records from `first_record_idx` to be included in this interpolation window.
    num_records_in_window: usize,
}

impl<'a> HermiteInterpolationData<'a> {
    /// Creates a new `HermiteInterpolationData` instance.
    ///
    /// # Arguments
    ///
    /// * `state_data`: A slice of `f64` containing all state vectors.
    /// * `epoch_data`: A slice of `f64` containing all epochs corresponding to each state vector.
    /// * `first_record_idx`: The starting record index within `state_data` and `epoch_data` for this interpolation window.
    /// * `num_records_in_window`: The number of records to include in this window.
    pub fn new(
        state_data: &'a [f64],
        epoch_data: &'a [f64],
        first_record_idx: usize,
        num_records_in_window: usize,
    ) -> Self {
        // Bounds for epoch_data are implicitly checked by the slice operation in epochs().
        // Bounds for state_data are implicitly checked by ComponentAccessor's constructor.
        Self {
            state_data,
            epoch_data,
            first_record_idx,
            num_records_in_window,
        }
    }

    /// Returns a slice of the epochs for the interpolation window.
    /// The bounds are checked by `evaluate` before this struct is created.
    pub fn epochs(&self) -> &'a [f64] {
        &self.epoch_data[self.first_record_idx..self.first_record_idx + self.num_records_in_window]
    }

    /// Stride of a complete PositionVelocityRecord in f64 units.
    const RECORD_STRIDE: usize = PositionVelocityRecord::SIZE / DBL_SIZE; // This is 6

    /// Returns a `ComponentAccessor` for the X components of position vectors.
    pub fn x_accessor(&self) -> ComponentAccessor<'a> {
        ComponentAccessor::new(
            self.state_data,
            self.first_record_idx,
            self.num_records_in_window,
            0, // component_offset for X
            Self::RECORD_STRIDE,
        )
    }

    /// Returns a `ComponentAccessor` for the Y components of position vectors.
    pub fn y_accessor(&self) -> ComponentAccessor<'a> {
        ComponentAccessor::new(
            self.state_data,
            self.first_record_idx,
            self.num_records_in_window,
            1, // component_offset for Y
            Self::RECORD_STRIDE,
        )
    }

    /// Returns a `ComponentAccessor` for the Z components of position vectors.
    pub fn z_accessor(&self) -> ComponentAccessor<'a> {
        ComponentAccessor::new(
            self.state_data,
            self.first_record_idx,
            self.num_records_in_window,
            2, // component_offset for Z
            Self::RECORD_STRIDE,
        )
    }

    /// Returns a `ComponentAccessor` for the X components of velocity vectors.
    pub fn vx_accessor(&self) -> ComponentAccessor<'a> {
        ComponentAccessor::new(
            self.state_data,
            self.first_record_idx,
            self.num_records_in_window,
            3, // component_offset for Vx
            Self::RECORD_STRIDE,
        )
    }

    /// Returns a `ComponentAccessor` for the Y components of velocity vectors.
    pub fn vy_accessor(&self) -> ComponentAccessor<'a> {
        ComponentAccessor::new(
            self.state_data,
            self.first_record_idx,
            self.num_records_in_window,
            4, // component_offset for Vy
            Self::RECORD_STRIDE,
        )
    }

    /// Returns a `ComponentAccessor` for the Z components of velocity vectors.
    pub fn vz_accessor(&self) -> ComponentAccessor<'a> {
        ComponentAccessor::new(
            self.state_data,
            self.first_record_idx,
            self.num_records_in_window,
            5, // component_offset for Vz
            Self::RECORD_STRIDE,
        )
    }
}

#[derive(PartialEq)]
pub struct HermiteSetType12<'a> {
    pub first_state_epoch: Epoch,
    pub step_size: Duration,
    pub window_size: usize,
    pub num_records: usize,
    pub record_data: &'a [f64],
}

impl fmt::Display for HermiteSetType12<'_> {
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
        let window_size = slice[slice.len() - 2] as usize;
        let num_records = slice[slice.len() - 1] as usize;

        Ok(Self {
            first_state_epoch,
            step_size,
            window_size,
            num_records,
            record_data: &slice[0..slice.len() - 4],
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
        _epoch: Epoch,
        _: &S,
    ) -> Result<CartesianState, InterpolationError> {
        Err(InterpolationError::UnimplementedType {
            dataset: Self::DATASET_NAME,
            issue: 14,
        })
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
        // TODO: use the epoch registry to reduce the search space
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
        // Now, perform a binary search on the epochs themselves.
        match self.epoch_data.binary_search_by(|epoch_et| {
            epoch_et
                .partial_cmp(&epoch.to_et_seconds())
                .expect("epochs in Hermite data is now NaN or infinite but was not before")
        }) {
            Ok(idx) => {
                // Oh wow, this state actually exists, no interpolation needed!
                Ok(self
                    .nth_record(idx)
                    .context(InterpDecodingSnafu)?
                    .to_pos_vel())
            }
            Err(idx) => {
                // We didn't find it, so let's build an interpolation here.
                // `idx` is the index of the first epoch GREATER than the requested epoch.
                // `self.samples` (from HermiteSetType13) is the DESIRED number of points for interpolation.

                // Determine the starting index (`first_idx`) for our window of points.
                // Try to center the window around `idx`.
                let mut first_idx = idx.saturating_sub(self.samples / 2);

                // Adjust `first_idx` if the window [first_idx, first_idx + self.samples) would extend
                // beyond the total number of records (`self.num_records`).
                // If so, shift `first_idx` to the left so the window ends at `self.num_records`.
                // Ensure `first_idx` does not become negative (guaranteed by saturating_sub).
                if first_idx + self.samples > self.num_records {
                    first_idx = self.num_records.saturating_sub(self.samples);
                }
                // After this, if self.num_records < self.samples, first_idx will be 0.

                // Determine the actual number of samples available for this interpolation window.
                // This can be less than `self.samples` if `self.num_records` is small or
                // if `self.num_records < self.samples`.
                let actual_samples_for_window = (self.num_records - first_idx).min(self.samples);

                // Create the data provider for the interpolation window.
                // The `samples` field within `interp_window_data` will be `actual_samples_for_window`.
                let interp_window_data = HermiteInterpolationData::new(
                    self.state_data,
                    self.epoch_data,
                    first_idx,
                    actual_samples_for_window,
                );

                // Build the interpolation polynomials using data from HermiteInterpolationData.
                // The accessor methods return ComponentAccessor instances.
                let epochs_slice = interp_window_data.epochs();

                let (x_km, vx_km_s) = hermite_eval(
                    epochs_slice,
                    &interp_window_data.x_accessor(),
                    &interp_window_data.vx_accessor(),
                    epoch.to_et_seconds(),
                )?;

                let (y_km, vy_km_s) = hermite_eval(
                    epochs_slice,
                    &interp_window_data.y_accessor(),
                    &interp_window_data.vy_accessor(),
                    epoch.to_et_seconds(),
                )?;

                let (z_km, vz_km_s) = hermite_eval(
                    epochs_slice,
                    &interp_window_data.z_accessor(),
                    &interp_window_data.vz_accessor(),
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
    use crate::math::Vector3;
    use crate::naif::spk::summary::SPKSummaryRecord;
    use hifitime::Epoch;

    // Helper function to create HermiteSetType13 instances for tests
    // Making it 'static for simplicity in tests by leaking the data.
    fn create_test_hermite_set(
        state_vecs: Vec<[f64; 6]>,
        epoch_secs: Vec<f64>,
        interpolation_samples: usize,
        epoch_registry_secs: Option<Vec<f64>>,
    ) -> HermiteSetType13<'static> {
        let num_records = state_vecs.len();
        assert_eq!(
            num_records,
            epoch_secs.len(),
            "Mismatch between state vector count and epoch count"
        );

        let mut state_data_flat: Vec<f64> = Vec::new();
        for sv in state_vecs {
            state_data_flat.extend_from_slice(&sv);
        }

        let mut full_slice_data = state_data_flat;
        full_slice_data.extend_from_slice(&epoch_secs);

        if let Some(reg_secs) = epoch_registry_secs {
            full_slice_data.extend_from_slice(&reg_secs);
        }
        // else, epoch_registry slice will be empty if get is called with start == end

        let samples_stored = (interpolation_samples.max(1) - 1) as f64; // Max(1) because stored as samples-1
        full_slice_data.push(samples_stored);
        full_slice_data.push(num_records as f64);

        let leaked_slice: &'static [f64] = Box::leak(full_slice_data.into_boxed_slice());
        HermiteSetType13::from_f64_slice(leaked_slice).unwrap()
    }

    const EPSILON: f64 = 1e-9;

    #[test]
    fn evaluate_exact_epoch_match() {
        let states = vec![
            [1.0, 2.0, 3.0, 0.1, 0.2, 0.3],
            [4.0, 5.0, 6.0, 0.4, 0.5, 0.6],
            [7.0, 8.0, 9.0, 0.7, 0.8, 0.9],
        ];
        let epochs = vec![0.0, 10.0, 20.0];
        let hermite_set = create_test_hermite_set(states.clone(), epochs.clone(), 2, None);

        let target_epoch = Epoch::from_et_seconds(10.0);
        let dummy_summary = SPKSummaryRecord::default();

        match hermite_set.evaluate(target_epoch, &dummy_summary) {
            Ok((pos, vel)) => {
                assert_eq!(pos.x, states[1][0]);
                assert_eq!(pos.y, states[1][1]);
                assert_eq!(pos.z, states[1][2]);
                assert_eq!(vel.x, states[1][3]);
                assert_eq!(vel.y, states[1][4]);
                assert_eq!(vel.z, states[1][5]);
            }
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn evaluate_basic_interpolation_with_velocity() {
        // Record 1: (0,0,0) v=(1,0,0) at t=0
        // Record 2: (10,0,0) v=(1,0,0) at t=10
        // Interpolate at t=5.
        // Position: p(s) = p0*h00(s) + p1*h01(s) + v0*h10(s)*dt + v1*h11(s)*dt
        // Velocity: v(s) = p0*h00'(s)/dt + p1*h01'(s)/dt + v0*h10'(s) + v1*h11'(s)
        // For s=0.5 (midpoint):
        // h00(0.5)=0.5, h01(0.5)=0.5
        // h10(0.5)=0.25, h11(0.5)=-0.25
        // dt = 10.0
        // Expected pos.x = 0*0.5 + 10*0.5 + 1*0.25*10 + 1*(-0.25)*10 = 0 + 5 + 2.5 - 2.5 = 5.0
        // Expected vel.x: v(s) = ( (p1-p0)/dt * h00_prime_normalized(s) + ... )
        // h00'(s) = 6s^2-6s => h00'(0.5) = 6*0.25 - 3 = 1.5 - 3 = -1.5
        // h01'(s) = -6s^2+6s => h01'(0.5) = -1.5 + 3 = 1.5
        // h10'(s) = 3s^2-4s+1 => h10'(0.5) = 3*0.25 - 2 + 1 = 0.75 - 2 + 1 = -0.25
        // h11'(s) = 3s^2-2s   => h11'(0.5) = 3*0.25 - 1 = 0.75 - 1 = -0.25
        // vel.x = ( (10-0)/10 * (-1.5) + (0-10)/10 * (1.5) ) -> this is not how hermite_eval calculates velocity.
        // hermite_eval returns v(t) from derivative of p(t) polynomial.
        // v(t) = (p0/dt)*h00'(s) + (p1/dt)*h01'(s) + v0*h10'(s) + v1*h11'(s)
        // vel.x = (0/10)*(-1.5) + (10/10)*(1.5) + 1*(-0.25) + 1*(-0.25) = 0 + 1.5 - 0.25 - 0.25 = 1.0
        let states = vec![
            [0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            [10.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        ];
        let epochs = vec![0.0, 10.0];
        let hermite_set = create_test_hermite_set(states, epochs, 2, None);

        let target_epoch = Epoch::from_et_seconds(5.0);
        let dummy_summary = SPKSummaryRecord::default();

        match hermite_set.evaluate(target_epoch, &dummy_summary) {
            Ok((pos, vel)) => {
                assert!((pos.x - 5.0).abs() < EPSILON, "pos.x: {}", pos.x);
                assert!((pos.y - 0.0).abs() < EPSILON, "pos.y: {}", pos.y);
                assert!((pos.z - 0.0).abs() < EPSILON, "pos.z: {}", pos.z);
                assert!((vel.x - 1.0).abs() < EPSILON, "vel.x: {}", vel.x);
                assert!((vel.y - 0.0).abs() < EPSILON, "vel.y: {}", vel.y);
                assert!((vel.z - 0.0).abs() < EPSILON, "vel.z: {}", vel.z);
            }
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn evaluate_near_beginning() {
        let states = vec![
            [1.0, 1.0, 1.0, 0.1, 0.1, 0.1],
            [2.0, 2.0, 2.0, 0.2, 0.2, 0.2],
            [3.0, 3.0, 3.0, 0.3, 0.3, 0.3],
            [4.0, 4.0, 4.0, 0.4, 0.4, 0.4],
        ];
        let epochs = vec![0.0, 10.0, 20.0, 30.0];
        // Request 4 samples, have 4 records.
        let hermite_set = create_test_hermite_set(states.clone(), epochs.clone(), 4, None);

        // Target epoch is 1.0, very close to the first epoch (0.0).
        // first_idx should be 0, actual_samples_for_window should be 4.
        // HermiteInterpolationData will use records at index 0, 1, 2, 3.
        let target_epoch = Epoch::from_et_seconds(1.0);
        let dummy_summary = SPKSummaryRecord::default();

        match hermite_set.evaluate(target_epoch, &dummy_summary) {
            Ok((pos, vel)) => {
                // Check that results are finite and somewhat plausible (close to first record)
                assert!(pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite());
                assert!(vel.x.is_finite() && vel.y.is_finite() && vel.z.is_finite());
                // Position should be close to states[0][0..3]
                assert!((pos.x - states[0][0]).abs() < 0.5, "pos.x: {}", pos.x);
                // Looser check
            }
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

    #[test]
    fn evaluate_near_end() {
        let states = vec![
            [1.0, 1.0, 1.0, 0.1, 0.1, 0.1],
            [2.0, 2.0, 2.0, 0.2, 0.2, 0.2],
            [3.0, 3.0, 3.0, 0.3, 0.3, 0.3],
            [4.0, 4.0, 4.0, 0.4, 0.4, 0.4],
        ];
        let epochs = vec![0.0, 10.0, 20.0, 30.0];
        let hermite_set = create_test_hermite_set(states.clone(), epochs.clone(), 4, None);

        // Target epoch 29.0, close to last epoch (30.0)
        // first_idx should be 0 (since 4 samples from 4 records means all are used).
        // HermiteInterpolationData will use records at index 0,1,2,3.
        let target_epoch = Epoch::from_et_seconds(29.0);
        let dummy_summary = SPKSummaryRecord::default();

        match hermite_set.evaluate(target_epoch, &dummy_summary) {
            Ok((pos, vel)) => {
                assert!(pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite());
                assert!(vel.x.is_finite() && vel.y.is_finite() && vel.z.is_finite());
                // Position should be close to states[3][0..3]
                assert!((pos.x - states[3][0]).abs() < 0.5, "pos.x: {}", pos.x);
                // Looser check
            }
            Err(e) => panic!("Evaluation failed: {:?}", e),
        }
    }

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
}
