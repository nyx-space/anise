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

use crate::{
    errors::{DecodingError, IntegrityError, TooFewDoublesSnafu},
    math::{
        Vector3,
        interpolation::{InterpDecodingSnafu, InterpolationError, chebyshev_eval},
    },
    naif::daf::{NAIFDataRecord, NAIFDataSet, NAIFSummaryRecord},
};

#[derive(PartialEq)]
pub struct Type2ChebyshevSet<'a> {
    pub init_epoch: Epoch,
    pub interval_length: Duration,
    pub rsize: usize,
    pub num_records: usize,
    pub record_data: &'a [f64],
}

impl Type2ChebyshevSet<'_> {
    pub fn degree(&self) -> usize {
        (self.rsize - 2) / 3 - 1
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

        // For trimmed kernels, the summary bounds may no longer align exactly with the
        // dataset footer epoch. Use the dataset init epoch for record selection.
        let ephem_start_delta_s = epoch.to_et_seconds() - self.init_epoch.to_et_seconds();

        // A tiny interval length makes this ratio saturate the usize cast, so add with
        // saturation to avoid an overflow panic before the clamp to num_records.
        Ok(((ephem_start_delta_s / window_duration_s) as usize)
            .saturating_add(1)
            .min(self.num_records))
    }
}

impl fmt::Display for Type2ChebyshevSet<'_> {
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

impl<'a> NAIFDataSet<'a> for Type2ChebyshevSet<'a> {
    type StateKind = (Vector3, Vector3);
    type RecordKind = Type2ChebyshevRecord<'a>;
    const DATASET_NAME: &'static str = "Chebyshev Type 2";

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
        // A valid Type 2 record holds two metadata doubles (midpoint, radius) plus at least
        // one Chebyshev coefficient per axis, so rsize must be at least 5. Anything smaller
        // makes degree() and the record decoder underflow when they compute (rsize - 2) / 3 - 1.
        if rsize < 5 {
            return Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: Self::DATASET_NAME,
                    variable: "record size (rsize)",
                    value: rsize as f64,
                    reason: "must be at least 5 to hold the record metadata and coefficients",
                },
            });
        }
        let num_records = slice[slice.len() - 1] as usize;
        // A segment with no records would underflow num_records - 1 in evaluate() and truncate(),
        // and a record count that overruns the footer-trimmed data would slice out of bounds when
        // a record is decoded. Reject both, using saturating_mul so the product cannot wrap first.
        if num_records == 0 {
            return Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: Self::DATASET_NAME,
                    variable: "number of records (num_records)",
                    value: 0.0,
                    reason: "must be at least 1",
                },
            });
        }
        let total_size = num_records.saturating_mul(rsize);
        if total_size > slice.len().saturating_sub(4) {
            return Err(DecodingError::Integrity {
                source: IntegrityError::InvalidValue {
                    dataset: Self::DATASET_NAME,
                    variable: "total records size",
                    value: total_size as f64,
                    reason: "total size of records exceeds the available data slice",
                },
            });
        }

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

        // Now, build the X, Y, Z data from the record data.
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
            let (val, deriv) =
                chebyshev_eval(normalized_time, coeffs, radius_s, epoch, self.degree())?;
            state[cno] = val;
            rate[cno] = deriv;
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
            self.spline_idx(end, summary)? - 1
        } else {
            self.num_records - 1
        };

        self.record_data = &self.record_data[start_idx * self.rsize..(end_idx + 1) * self.rsize];
        self.num_records = self.record_data.len() / self.rsize;
        self.init_epoch = self
            .nth_record(0)
            .context(InterpDecodingSnafu)?
            .midpoint_epoch()
            - 0.5 * self.interval_length;

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

pub struct Type2ChebyshevRecord<'a> {
    pub midpoint_et_s: f64,
    pub radius: Duration,
    pub x_coeffs: &'a [f64],
    pub y_coeffs: &'a [f64],
    pub z_coeffs: &'a [f64],
}

impl Type2ChebyshevRecord<'_> {
    pub fn midpoint_epoch(&self) -> Epoch {
        Epoch::from_et_seconds(self.midpoint_et_s)
    }
}

impl fmt::Display for Type2ChebyshevRecord<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "start: {:E}\tend: {:E}\nx: {:?}\ny: {:?}\nz: {:?}",
            self.midpoint_epoch() - self.radius,
            self.midpoint_epoch() + self.radius,
            self.x_coeffs,
            self.y_coeffs,
            self.z_coeffs
        )
    }
}

impl<'a> NAIFDataRecord<'a> for Type2ChebyshevRecord<'a> {
    fn from_slice_f64(slice: &'a [f64]) -> Self {
        let num_coeffs = (slice.len() - 2) / 3;
        let end_x_idx = num_coeffs + 2;
        let end_y_idx = 2 * num_coeffs + 2;
        Self {
            midpoint_et_s: slice[0],
            radius: slice[1].seconds(),
            x_coeffs: &slice[2..end_x_idx],
            y_coeffs: &slice[end_x_idx..end_y_idx],
            z_coeffs: &slice[end_y_idx..],
        }
    }
}

#[cfg(test)]
mod chebyshev_ut {
    use crate::{
        errors::{DecodingError, IntegrityError},
        naif::{daf::NAIFDataSet, spk::summary::SPKSummaryRecord},
    };
    use hifitime::Epoch;

    use super::Type2ChebyshevSet;

    #[test]
    fn too_small() {
        if Type2ChebyshevSet::from_f64_slice(&[0.1, 0.2, 0.3, 0.4])
            != Err(DecodingError::TooFewDoubles {
                dataset: "Chebyshev Type 2",
                got: 4,
                need: 5,
            })
        {
            panic!("test failure");
        }
    }

    #[test]
    fn subnormal() {
        match Type2ChebyshevSet::from_f64_slice(&[0.0, f64::INFINITY, 0.0, 0.0, 0.0]) {
            Ok(_) => panic!("test failed on invalid init_epoch"),
            Err(e) => {
                assert_eq!(
                    e,
                    DecodingError::Integrity {
                        source: IntegrityError::SubNormal {
                            dataset: "Chebyshev Type 2",
                            variable: "seconds since J2000 ET",
                        },
                    }
                );
            }
        }

        match Type2ChebyshevSet::from_f64_slice(&[0.0, 0.0, f64::INFINITY, 0.0, 0.0]) {
            Ok(_) => panic!("test failed on invalid interval_length"),
            Err(e) => {
                assert_eq!(
                    e,
                    DecodingError::Integrity {
                        source: IntegrityError::SubNormal {
                            dataset: "Chebyshev Type 2",
                            variable: "interval length in seconds",
                        },
                    }
                );
            }
        }

        match Type2ChebyshevSet::from_f64_slice(&[0.0, 0.0, -1e-16, 0.0, 0.0]) {
            Ok(_) => panic!("test failed on invalid interval_length"),
            Err(e) => {
                assert_eq!(
                    e,
                    DecodingError::Integrity {
                        source: IntegrityError::InvalidValue {
                            dataset: "Chebyshev Type 2",
                            variable: "interval length in seconds",
                            value: -1e-16,
                            reason: "must be strictly greater than zero"
                        },
                    }
                );
            }
        }

        // Load a slice whose metadata is OK but the record data is not. The footer carries a
        // single valid-sized record (rsize 5, num_records 1) so decoding succeeds and the
        // subnormal value in the record data is only caught by check_integrity.
        let dataset = Type2ChebyshevSet::from_f64_slice(&[
            f64::INFINITY,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            2e-16,
            5.0,
            1.0,
        ])
        .unwrap();
        match dataset.check_integrity() {
            Ok(_) => panic!("test failed on invalid interval_length"),
            Err(e) => {
                assert_eq!(
                    e,
                    IntegrityError::SubNormal {
                        dataset: "Chebyshev Type 2",
                        variable: "one of the record data",
                    },
                );
            }
        }
    }

    #[test]
    fn rejects_invalid_rsize_and_num_records() {
        // rsize = 1 in the footer is far too small to hold a record. Before this guard the
        // dataset decoded fine but degree() and the record decoder panicked with a subtract
        // overflow while evaluating a crafted segment.
        match Type2ChebyshevSet::from_f64_slice(&[0.0, 0.0, 10.0, 1.0, 1.0]) {
            Ok(_) => panic!("test failed: undersized rsize was accepted"),
            Err(e) => assert_eq!(
                e,
                DecodingError::Integrity {
                    source: IntegrityError::InvalidValue {
                        dataset: "Chebyshev Type 2",
                        variable: "record size (rsize)",
                        value: 1.0,
                        reason: "must be at least 5 to hold the record metadata and coefficients",
                    },
                }
            ),
        }

        // num_records = 0 would underflow num_records - 1 during evaluation and truncation.
        match Type2ChebyshevSet::from_f64_slice(&[0.0, 0.0, 10.0, 5.0, 0.0]) {
            Ok(_) => panic!("test failed: zero num_records was accepted"),
            Err(e) => assert_eq!(
                e,
                DecodingError::Integrity {
                    source: IntegrityError::InvalidValue {
                        dataset: "Chebyshev Type 2",
                        variable: "number of records (num_records)",
                        value: 0.0,
                        reason: "must be at least 1",
                    },
                }
            ),
        }

        // num_records * rsize overruns the footer-trimmed data, so a record decode would slice
        // out of bounds.
        match Type2ChebyshevSet::from_f64_slice(&[0.0, 0.0, 10.0, 5.0, 10.0]) {
            Ok(_) => panic!("test failed: oversized record count was accepted"),
            Err(e) => assert_eq!(
                e,
                DecodingError::Integrity {
                    source: IntegrityError::InvalidValue {
                        dataset: "Chebyshev Type 2",
                        variable: "total records size",
                        value: 50.0,
                        reason: "total size of records exceeds the available data slice",
                    },
                }
            ),
        }
    }

    #[test]
    fn evaluate_uses_init_epoch_for_record_selection() {
        // Three degree-0 records with unique constants per record.
        let dataset = Type2ChebyshevSet::from_f64_slice(&[
            5.0, 5.0, 1.0, 10.0, 100.0, //
            15.0, 5.0, 2.0, 20.0, 200.0, //
            25.0, 5.0, 3.0, 30.0, 300.0, //
            0.0, 10.0, 5.0, 3.0,
        ])
        .unwrap();

        let shifted_summary = SPKSummaryRecord {
            start_epoch_et_s: 4.0,
            end_epoch_et_s: 30.0,
            target_id: 301,
            center_id: 3,
            frame_id: 1,
            data_type_i: 2,
            start_idx: 1,
            end_idx: 19,
        };

        let epoch = Epoch::from_et_seconds(12.0);
        assert_eq!(dataset.spline_idx(epoch, &shifted_summary).unwrap(), 2);

        let (state, _) = dataset.evaluate(epoch, &shifted_summary).unwrap();
        assert_eq!(state[0], 2.0);
        assert_eq!(state[1], 20.0);
        assert_eq!(state[2], 200.0);
    }

    #[test]
    fn tiny_interval_length_does_not_overflow_spline_idx() {
        // A crafted segment with a very small interval length makes the record-selection ratio
        // in spline_idx saturate the usize cast, so the following `+ 1` used to overflow and
        // panic (in debug) before the clamp to num_records. Selecting the last record is fine.
        let dataset = Type2ChebyshevSet::from_f64_slice(&[
            0.0, 1.0, 0.0, 0.0, 0.0, // one degree-0 record
            0.0, 1e-300, 5.0, 1.0, // init_epoch, tiny interval, rsize, num_records
        ])
        .unwrap();
        let summary = SPKSummaryRecord {
            start_epoch_et_s: 0.0,
            end_epoch_et_s: 1e6,
            target_id: 301,
            center_id: 3,
            frame_id: 1,
            data_type_i: 2,
            start_idx: 1,
            end_idx: 9,
        };
        let epoch = Epoch::from_et_seconds(1000.0);
        assert_eq!(dataset.spline_idx(epoch, &summary).unwrap(), 1);
        // The full evaluate path must not panic on this input.
        let _ = dataset.evaluate(epoch, &summary);
    }
}
