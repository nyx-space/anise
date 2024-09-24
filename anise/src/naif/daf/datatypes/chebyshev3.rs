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
        interpolation::{chebyshev_eval_poly, InterpDecodingSnafu, InterpolationError},
        Vector3,
    },
    naif::daf::{NAIFDataRecord, NAIFDataSet, NAIFSummaryRecord},
};

#[derive(PartialEq)]
pub struct Type3ChebyshevSet<'a> {
    pub init_epoch: Epoch,
    pub interval_length: Duration,
    pub rsize: usize,
    pub num_records: usize,
    pub record_data: &'a [f64],
}

impl<'a> Type3ChebyshevSet<'a> {
    pub fn degree(&self) -> usize {
        (self.rsize - 2) / 6 - 1
    }

    fn spline_idx<S: NAIFSummaryRecord>(
        &self,
        epoch: Epoch,
        summary: &S,
    ) -> Result<usize, InterpolationError> {
        if epoch < summary.start_epoch() - 1_i64.nanoseconds()
            || epoch > summary.end_epoch() + 1_i64.nanoseconds()
        {
            // No need to go any further.
            return Err(InterpolationError::NoInterpolationData {
                req: epoch,
                start: summary.start_epoch(),
                end: summary.end_epoch(),
            });
        }

        let window_duration_s = self.interval_length.to_seconds();

        let ephem_start_delta_s = epoch.to_et_seconds() - summary.start_epoch_et_s();

        Ok(((ephem_start_delta_s / window_duration_s) as usize + 1).min(self.num_records))
    }
}

impl<'a> fmt::Display for Type3ChebyshevSet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "start: {:E}\tlength: {}\trsize: {}\tnum_records: {}\tlen data: {}",
            self.init_epoch,
            self.interval_length,
            self.rsize,
            self.num_records,
            self.record_data.len()
        )
    }
}

impl<'a> NAIFDataSet<'a> for Type3ChebyshevSet<'a> {
    type StateKind = (Vector3, Vector3);
    type RecordKind = Type3ChebyshevRecord<'a>;
    const DATASET_NAME: &'static str = "Chebyshev Type 3";

    fn from_f64_slice(slice: &'a [f64]) -> Result<Self, DecodingError> {
        ensure!(
            slice.len() >= 5,
            TooFewDoublesSnafu {
                dataset: Self::DATASET_NAME,
                need: 5_usize,
                got: slice.len()
            }
        );
        // For this kind of record, the data is stored at the very end of the dataset
        let seconds_since_j2000 = slice[slice.len() - 4];
        if !seconds_since_j2000.is_finite() {
            return Err(DecodingError::Integrity {
                source: IntegrityError::SubNormal {
                    dataset: Self::DATASET_NAME,
                    variable: "seconds since J2000 ET",
                },
            });
        }

        let start_epoch = Epoch::from_et_seconds(seconds_since_j2000);

        let interval_length_s = slice[slice.len() - 3];
        if !interval_length_s.is_finite() {
            return Err(DecodingError::Integrity {
                source: IntegrityError::SubNormal {
                    dataset: Self::DATASET_NAME,
                    variable: "interval length in seconds",
                },
            });
        } else if interval_length_s <= 0.0 {
            return Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: Self::DATASET_NAME,
                    variable: "interval length in seconds",
                    value: interval_length_s,
                    reason: "must be strictly greater than zero",
                },
            });
        }

        let interval_length = interval_length_s.seconds();
        let rsize = slice[slice.len() - 2] as usize;
        let num_records = slice[slice.len() - 1] as usize;

        Ok(Self {
            init_epoch: start_epoch,
            interval_length,
            rsize,
            num_records,
            record_data: &slice[0..slice.len() - 4],
        })
    }

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, DecodingError> {
        Ok(Self::RecordKind::from_slice_f64(
            self.record_data
                .get(n * self.rsize..(n + 1) * self.rsize)
                .ok_or(DecodingError::InaccessibleBytes {
                    start: n * self.rsize,
                    end: (n + 1) * self.rsize,
                    size: self.record_data.len(),
                })?,
        ))
    }

    fn evaluate<S: NAIFSummaryRecord>(
        &self,
        epoch: Epoch,
        summary: &S,
    ) -> Result<(Vector3, Vector3), InterpolationError> {
        let spline_idx = self.spline_idx(epoch, summary)?;

        let window_duration_s = self.interval_length.to_seconds();
        let radius_s = window_duration_s / 2.0;

        let record = self
            .nth_record(spline_idx - 1)
            .context(InterpDecodingSnafu)?;

        let normalized_time = (epoch.to_et_seconds() - record.midpoint_et_s) / radius_s;

        let mut state = Vector3::zeros();
        let mut rate = Vector3::zeros();

        for (cno, coeffs) in [record.x_coeffs, record.y_coeffs, record.z_coeffs]
            .iter()
            .enumerate()
        {
            let val = chebyshev_eval_poly(normalized_time, coeffs, epoch, self.degree())?;
            state[cno] = val;
        }

        for (cno, coeffs) in [record.vx_coeffs, record.vy_coeffs, record.vz_coeffs]
            .iter()
            .enumerate()
        {
            let val = chebyshev_eval_poly(normalized_time, coeffs, epoch, self.degree())?;
            rate[cno] = val;
        }

        Ok((state, rate))
    }

    fn check_integrity(&self) -> Result<(), IntegrityError> {
        // Verify that none of the data is invalid once when we load it.
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

    fn truncate<S: NAIFSummaryRecord>(
        mut self,
        summary: &S,
        new_start: Option<Epoch>,
        new_end: Option<Epoch>,
    ) -> Result<Self, InterpolationError> {
        let start_idx = if let Some(start) = new_start {
            self.spline_idx(start, summary)? - 1
        } else {
            0
        };

        let end_idx = if let Some(end) = new_end {
            self.spline_idx(end, summary)?
        } else {
            self.num_records - 1
        };

        self.record_data = &self.record_data[start_idx * self.rsize..(end_idx + 1) * self.rsize];
        self.num_records = (self.record_data.len() / self.rsize) - 1;
        self.init_epoch = self.nth_record(0).unwrap().midpoint_epoch() - 0.5 * self.interval_length;

        Ok(self)
    }

    /// Builds the DAF array representing a Chebyshev Type 2 interpolation set.
    fn to_f64_daf_vec(&self) -> Result<Vec<f64>, InterpolationError> {
        let mut data = self.record_data.to_vec();
        data.push(self.init_epoch.to_et_seconds());
        data.push(self.interval_length.to_seconds());
        data.push(self.rsize as f64);
        data.push(self.num_records as f64);

        Ok(data)
    }
}

#[derive(Debug, PartialEq)]
pub struct Type3ChebyshevRecord<'a> {
    pub midpoint_et_s: f64,
    pub radius: Duration,
    pub x_coeffs: &'a [f64],
    pub y_coeffs: &'a [f64],
    pub z_coeffs: &'a [f64],
    pub vx_coeffs: &'a [f64],
    pub vy_coeffs: &'a [f64],
    pub vz_coeffs: &'a [f64],
}

impl<'a> Type3ChebyshevRecord<'a> {
    pub fn midpoint_epoch(&self) -> Epoch {
        Epoch::from_et_seconds(self.midpoint_et_s)
    }
}

impl<'a> fmt::Display for Type3ChebyshevRecord<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "start: {}\tend: {}\nx:  {:?}\ny:  {:?}\nz:  {:?}\nvx: {:?}\nvy: {:?}\nvz: {:?}",
            self.midpoint_epoch() - self.radius,
            self.midpoint_epoch() + self.radius,
            self.x_coeffs,
            self.y_coeffs,
            self.z_coeffs,
            self.vx_coeffs,
            self.vy_coeffs,
            self.vz_coeffs
        )
    }
}

impl<'a> NAIFDataRecord<'a> for Type3ChebyshevRecord<'a> {
    fn from_slice_f64(slice: &'a [f64]) -> Self {
        let num_coeffs = (slice.len() - 2) / 6;

        let end_x_idx = num_coeffs + 2;
        let end_y_idx = 2 * num_coeffs + 2;
        let end_z_idx = 3 * num_coeffs + 2;
        let end_vx_idx = 4 * num_coeffs + 2;
        let end_vy_idx = 5 * num_coeffs + 2;
        Self {
            midpoint_et_s: slice[0],
            radius: slice[1].seconds(),
            x_coeffs: &slice[2..end_x_idx],
            y_coeffs: &slice[end_x_idx..end_y_idx],
            z_coeffs: &slice[end_y_idx..end_z_idx],
            vx_coeffs: &slice[end_z_idx..end_vx_idx],
            vy_coeffs: &slice[end_vx_idx..end_vy_idx],
            vz_coeffs: &slice[end_vy_idx..],
        }
    }
}

#[cfg(test)]
mod chebyshev_ut {
    use crate::{
        errors::{DecodingError, IntegrityError},
        naif::daf::NAIFDataSet,
    };

    use super::Type3ChebyshevSet;

    #[test]
    fn too_small() {
        if Type3ChebyshevSet::from_f64_slice(&[0.1, 0.2, 0.3, 0.4])
            != Err(DecodingError::TooFewDoubles {
                dataset: "Chebyshev Type 3",
                got: 4,
                need: 5,
            })
        {
            panic!("test failure");
        }
    }

    #[test]
    fn subnormal() {
        match Type3ChebyshevSet::from_f64_slice(&[0.0, f64::INFINITY, 0.0, 0.0, 0.0]) {
            Ok(_) => panic!("test failed on invalid init_epoch"),
            Err(e) => {
                assert_eq!(
                    e,
                    DecodingError::Integrity {
                        source: IntegrityError::SubNormal {
                            dataset: "Chebyshev Type 3",
                            variable: "seconds since J2000 ET",
                        },
                    }
                );
            }
        }

        match Type3ChebyshevSet::from_f64_slice(&[0.0, 0.0, f64::INFINITY, 0.0, 0.0]) {
            Ok(_) => panic!("test failed on invalid interval_length"),
            Err(e) => {
                assert_eq!(
                    e,
                    DecodingError::Integrity {
                        source: IntegrityError::SubNormal {
                            dataset: "Chebyshev Type 3",
                            variable: "interval length in seconds",
                        },
                    }
                );
            }
        }

        match Type3ChebyshevSet::from_f64_slice(&[0.0, 0.0, -1e-16, 0.0, 0.0]) {
            Ok(_) => panic!("test failed on invalid interval_length"),
            Err(e) => {
                assert_eq!(
                    e,
                    DecodingError::Integrity {
                        source: IntegrityError::InvalidValue {
                            dataset: "Chebyshev Type 3",
                            variable: "interval length in seconds",
                            value: -1e-16,
                            reason: "must be strictly greater than zero"
                        },
                    }
                );
            }
        }

        // Load a slice whose metadata is OK but the record data is not
        let dataset =
            Type3ChebyshevSet::from_f64_slice(&[f64::INFINITY, 0.0, 2e-16, 0.0, 0.0]).unwrap();
        match dataset.check_integrity() {
            Ok(_) => panic!("test failed on invalid interval_length"),
            Err(e) => {
                assert_eq!(
                    e,
                    IntegrityError::SubNormal {
                        dataset: "Chebyshev Type 3",
                        variable: "one of the record data",
                    },
                );
            }
        }
    }
}
