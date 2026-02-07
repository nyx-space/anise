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

use crate::{
    errors::{DecodingError, IntegrityError, TooFewDoublesSnafu},
    math::{
        interpolation::{chebyshev_eval_poly, InterpDecodingSnafu, InterpolationError},
        Vector3,
    },
    naif::daf::datatypes::chebyshev3::Type3ChebyshevRecord,
    naif::daf::{NAIFDataRecord, NAIFDataSet, NAIFSummaryRecord},
};

#[derive(PartialEq, Debug)]
pub struct ChebyshevSetType14<'a> {
    pub degree: usize,
    pub num_records: usize,
    pub packet_data: &'a [f64],
    pub epoch_data: &'a [f64],
    pub packet_directory: &'a [f64],
    pub epoch_directory: &'a [f64],
}

impl fmt::Display for ChebyshevSetType14<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Chebyshev Type 14: degree: {}\tnum_records: {}\tlen data: {}",
            self.degree,
            self.num_records,
            self.packet_data.len()
        )
    }
}

impl<'a> NAIFDataSet<'a> for ChebyshevSetType14<'a> {
    type StateKind = (Vector3, Vector3);
    type RecordKind = Type3ChebyshevRecord<'a>;
    const DATASET_NAME: &'static str = "Chebyshev Type 14";

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
        let degree = slice[slice.len() - 2] as usize;
        let rsize = 2 + 6 * (degree + 1);

        // Check total expected size to avoid panics
        let num_directories = num_records / 100;
        let num_constants = slice[0] as usize;
        let expected_min_len =
            1 + num_constants + num_records * rsize + num_records + 2 * num_directories + 2;
        ensure!(
            slice.len() >= expected_min_len,
            TooFewDoublesSnafu {
                dataset: Self::DATASET_NAME,
                need: expected_min_len,
                got: slice.len(),
            }
        );

        let packet_data_start = 1 + num_constants;
        let packet_data_end = packet_data_start + num_records * rsize;
        let packet_data = &slice[packet_data_start..packet_data_end];

        let epoch_data_start = packet_data_end;
        let epoch_data_end = epoch_data_start + num_records;
        let epoch_data = &slice[epoch_data_start..epoch_data_end];

        let packet_directory_start = epoch_data_end;
        let packet_directory_end = packet_directory_start + num_directories;
        let packet_directory = &slice[packet_directory_start..packet_directory_end];

        let epoch_directory_start = packet_directory_end;
        let epoch_directory_end = epoch_directory_start + num_directories;
        let epoch_directory = &slice[epoch_directory_start..epoch_directory_end];

        Ok(Self {
            degree,
            num_records,
            packet_data,
            epoch_data,
            packet_directory,
            epoch_directory,
        })
    }

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, DecodingError> {
        let rsize = 2 + 6 * (self.degree + 1);
        Ok(Self::RecordKind::from_slice_f64(
            self.packet_data.get(n * rsize..(n + 1) * rsize).ok_or(
                DecodingError::InaccessibleBytes {
                    start: n * rsize,
                    end: (n + 1) * rsize,
                    size: self.packet_data.len(),
                },
            )?,
        ))
    }

    fn evaluate<S: NAIFSummaryRecord>(
        &self,
        epoch: Epoch,
        _: &S,
    ) -> Result<Self::StateKind, InterpolationError> {
        if self.epoch_data.is_empty() {
            return Err(InterpolationError::MissingInterpolationData { epoch });
        }

        let et = epoch.to_et_seconds();

        // Search through a reduced data slice if available
        let (search_data_slice, slice_offset) = if self.epoch_directory.is_empty() {
            (self.epoch_data, 0)
        } else {
            let dir_idx = self
                .epoch_directory
                .partition_point(|&reg_epoch| reg_epoch < et);

            let sub_array_start_idx = if dir_idx == 0 { 0 } else { (dir_idx * 100) - 1 };

            let sub_array_end_idx = (sub_array_start_idx + 99).min(self.num_records - 1);

            (
                &self.epoch_data[sub_array_start_idx..=sub_array_end_idx.max(sub_array_start_idx)],
                sub_array_start_idx,
            )
        };

        let idx = match search_data_slice.binary_search_by(|e| e.partial_cmp(&et).unwrap()) {
            Ok(idx) => idx + slice_offset,
            Err(idx) => idx + slice_offset,
        };

        // Ensure we don't go out of bounds
        let idx = idx.min(self.num_records - 1);

        // Check if the requested epoch is actually within the range of the selected record
        // The record covers [T_{idx-1}, T_{idx}].
        // Wait, if idx is 0, it covers [SegmentStart, T_0].
        // We rely on the DAF summary to have checked that the epoch is within the segment.

        let record = self.nth_record(idx).context(InterpDecodingSnafu)?;

        let radius_s = record.radius.to_seconds();
        let normalized_time = (et - record.midpoint_et_s) / radius_s;

        let mut state = Vector3::zeros();
        let mut rate = Vector3::zeros();

        for (cno, coeffs) in [record.x_coeffs, record.y_coeffs, record.z_coeffs]
            .iter()
            .enumerate()
        {
            let val = chebyshev_eval_poly(normalized_time, coeffs, epoch, self.degree)?;
            state[cno] = val;
        }

        for (cno, coeffs) in [record.vx_coeffs, record.vy_coeffs, record.vz_coeffs]
            .iter()
            .enumerate()
        {
            let val = chebyshev_eval_poly(normalized_time, coeffs, epoch, self.degree)?;
            rate[cno] = val;
        }

        Ok((state, rate))
    }

    fn check_integrity(&self) -> Result<(), IntegrityError> {
        for val in self.packet_data {
            if !val.is_finite() {
                return Err(IntegrityError::SubNormal {
                    dataset: Self::DATASET_NAME,
                    variable: "one of the packet data",
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::naif::daf::NAIFDataSet;
    use crate::naif::spk::summary::SPKSummaryRecord;
    use hifitime::Epoch;

    #[test]
    fn test_chebyshev14_linear() {
        let degree = 1;
        let num_records = 1;
        // rsize = 2 + 6 * (1 + 1) = 14
        let mut slice = vec![0.0; 1 + 14 + 1 + 0 + 2];
        slice[0] = 0.0; // M=0

        // Packet 1
        slice[1] = 10.0; // Midpoint
        slice[2] = 10.0; // Radius
        slice[3] = 5.0; // X coeff 0
        slice[4] = 5.0; // X coeff 1
        slice[9] = 0.5; // VX coeff 0 (constant velocity)

        // Epoch data
        slice[15] = 20.0; // T1

        // Meta data
        slice[16] = degree as f64;
        slice[17] = num_records as f64;

        let dataset = ChebyshevSetType14::from_f64_slice(&slice).unwrap();
        assert_eq!(dataset.degree, degree);
        assert_eq!(dataset.num_records, num_records);

        let summary = SPKSummaryRecord::default();

        // Test at t=0
        let epoch_0 = Epoch::from_et_seconds(0.0);
        let (pos, vel) = dataset.evaluate(epoch_0, &summary).unwrap();
        assert!((pos.x - 0.0).abs() < 1e-12);
        assert!((vel.x - 0.5).abs() < 1e-12);

        // Test at t=10
        let epoch_10 = Epoch::from_et_seconds(10.0);
        let (pos, vel) = dataset.evaluate(epoch_10, &summary).unwrap();
        assert!((pos.x - 5.0).abs() < 1e-12);
        assert!((vel.x - 0.5).abs() < 1e-12);

        // Test at t=20
        let epoch_20 = Epoch::from_et_seconds(20.0);
        let (pos, vel) = dataset.evaluate(epoch_20, &summary).unwrap();
        assert!((pos.x - 10.0).abs() < 1e-12);
        assert!((vel.x - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_chebyshev14_multiple_records() {
        let degree = 1;
        let num_records = 2;
        // rsize = 14
        // Total = 1 + 2*14 + 2 + 0 + 2 = 33
        let mut slice = vec![0.0; 33];
        slice[0] = 0.0; // M=0

        // Packet 1 (t in [0, 10], midpoint 5, radius 5)
        slice[1] = 5.0;
        slice[2] = 5.0;
        slice[3] = 2.5; // x = 2.5 + 2.5*tn. At tn=-1 (t=0), x=0. At tn=1 (t=10), x=5.
        slice[4] = 2.5;
        slice[9] = 0.5; // vx = 0.5

        // Packet 2 (t in [10, 30], midpoint 20, radius 10)
        slice[15] = 20.0;
        slice[16] = 10.0;
        slice[17] = 15.0; // x = 15 + 10*tn. At tn=-1 (t=10), x=5. At tn=1 (t=30), x=25.
        slice[18] = 10.0;
        slice[23] = 1.0; // vx = 1.0

        // Epoch data
        slice[29] = 10.0;
        slice[30] = 30.0;

        // Meta data
        slice[31] = degree as f64;
        slice[32] = num_records as f64;

        let dataset = ChebyshevSetType14::from_f64_slice(&slice).unwrap();
        let summary = SPKSummaryRecord::default();

        // Test in first packet
        let epoch_5 = Epoch::from_et_seconds(5.0);
        let (pos, vel) = dataset.evaluate(epoch_5, &summary).unwrap();
        assert!((pos.x - 2.5).abs() < 1e-12);
        assert!((vel.x - 0.5).abs() < 1e-12);

        // Test in second packet
        let epoch_20 = Epoch::from_et_seconds(20.0);
        let (pos, vel) = dataset.evaluate(epoch_20, &summary).unwrap();
        assert!((pos.x - 15.0).abs() < 1e-12);
        assert!((vel.x - 1.0).abs() < 1e-12);

        // Test boundary t=10
        let epoch_10 = Epoch::from_et_seconds(10.0);
        let (pos, vel) = dataset.evaluate(epoch_10, &summary).unwrap();
        // Since T1=10.0, binary search should find idx 0.
        assert!((pos.x - 5.0).abs() < 1e-12);
        assert!((vel.x - 0.5).abs() < 1e-12);

        // Test t=10.1 (second packet)
        let epoch_10_1 = Epoch::from_et_seconds(10.1);
        let (pos, vel) = dataset.evaluate(epoch_10_1, &summary).unwrap();
        // x = 15 + 10 * (10.1 - 20) / 10 = 15 + (10.1 - 20) = 5.1
        assert!((pos.x - 5.1).abs() < 1e-12);
        assert!((vel.x - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_chebyshev14_directory_100() {
        // Test that N=100 gives 1 directory entry
        let degree = 1;
        let num_records = 100;
        let num_dirs = 1; // 100 / 100 = 1
        let rsize = 14;
        // 1 (M_count) + 0 (M) + 100*14 (Packets) + 100 (Epochs) + 2*1 (Dirs) + 2 (Meta) = 1505
        let len = 1 + 100 * rsize + 100 + 2 * num_dirs + 2;
        let mut slice = vec![0.0; len];
        slice[0] = 0.0;
        slice[len - 2] = degree as f64;
        slice[len - 1] = num_records as f64;

        let dataset = ChebyshevSetType14::from_f64_slice(&slice).unwrap();
        assert_eq!(dataset.epoch_directory.len(), 1);
    }
}
