/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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
    errors::{DecodingError, IntegrityError, TooFewDoublesSnafu},
    math::{cartesian::CartesianState, interpolation::InterpolationError, Vector3},
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

impl<'a> fmt::Display for LagrangeSetType8<'a> {
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
    type StateKind = CartesianState;
    type RecordKind = PositionVelocityRecord;
    const DATASET_NAME: &'static str = "Lagrange Type 8";

    fn from_slice_f64(slice: &'a [f64]) -> Result<Self, DecodingError> {
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
                    dataset: "Hermite Type 12",
                    variable: "step size in seconds",
                },
            });
        }

        let step_size = step_size_s.seconds();
        let degree = slice[slice.len() - 2] as usize;
        let num_records = slice[slice.len() - 1] as usize;

        Ok(Self {
            first_state_epoch,
            step_size,
            degree,
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
        todo!("https://github.com/anise-toolkit/anise.rs/issues/12")
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

impl<'a> fmt::Display for LagrangeSetType9<'a> {
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

    fn from_slice_f64(slice: &'a [f64]) -> Result<Self, DecodingError> {
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
        _epoch: Epoch,
        _: &S,
    ) -> Result<Self::StateKind, InterpolationError> {
        todo!("https://github.com/anise-toolkit/anise.rs/issues/13")
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
