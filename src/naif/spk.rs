/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::daf::DAF;
use crate::naif::{divmod, S_PER_DAY, T0};
use crate::prelude::AniseError;
use core::num;
use hifitime::{Epoch, TimeSystem};
use std::convert::{TryFrom, TryInto};
use std::fmt;

#[derive(Debug)]
pub struct Segment<'a> {
    name: &'a str,
    start_epoch: Epoch,
    end_epoch: Epoch,
    target_id: i32,
    center_id: i32,
    frame_id: i32,
    data_type: DataType,
    pub start_idx: usize,
    end_idx: usize,
}

impl<'a> Segment<'a> {}

impl<'a> Default for Segment<'a> {
    fn default() -> Self {
        Self {
            name: "No name",
            start_epoch: Epoch::from_tdb_seconds(0.0),
            end_epoch: Epoch::from_tdb_seconds(0.0),
            target_id: 0,
            center_id: 0,
            frame_id: 0,
            data_type: DataType::ModifiedDifferenceArrays,
            start_idx: 0,
            end_idx: 0,
        }
    }
}

impl<'a> fmt::Display for Segment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Segment `{}` (tgt: {}, ctr: {}, frame: {}) of type {:?} from {} ({}) to {} ({}) [{}..{}]",
            self.name,
            self.target_id,
            self.center_id,
            self.frame_id,
            self.data_type,
            self.start_epoch.as_gregorian_str(TimeSystem::ET),
            self.start_epoch.as_et_duration().in_seconds(),
            self.end_epoch.as_gregorian_str(TimeSystem::ET),
            self.end_epoch.as_et_duration().in_seconds(),
            self.start_idx,
            self.end_idx
        )
    }
}

#[derive(Debug, Clone)]
pub struct SegmentExportData {
    pub rcrd_mid_point: f64,
    pub rcrd_radius_s: f64,
    pub x_coeffs: Vec<f64>,
    pub y_coeffs: Vec<f64>,
    pub z_coeffs: Vec<f64>,
}

#[derive(Debug)]
pub struct SPK<'a> {
    pub segments: Vec<Segment<'a>>,
    pub daf: &'a DAF<'a>,
}

impl<'a> SPK<'a> {
    /// Returns the segment buffer index and the config data of that segment as (init_s_past_j2k, interval_length, rsize, num_records_in_seg)
    pub fn segment_ptr(
        &self,
        seg_target_id: i32,
    ) -> Result<(&Segment, (f64, usize, usize, usize)), AniseError> {
        for seg in &self.segments {
            if seg.target_id != seg_target_id {
                continue;
            }

            if seg.data_type != DataType::ChebyshevPositionOnly {
                return Err(AniseError::NAIFConversionError(
                    "Only cheby supported".to_string(),
                ));
            }

            // For type 2, the config data is at the very end of the record
            // https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/spk.html#Type%202:%20Chebyshev%20(position%20only)

            let mut byte_idx = seg.end_idx - 4;
            //  1. INIT is the initial epoch of the first record, given in ephemeris seconds past J2000.
            let init_s_past_j2k = self.daf.read_f64(byte_idx);

            byte_idx += 1;

            //  2. INTLEN is the length of the interval covered by each record, in seconds.
            let interval_length = self.daf.read_f64(byte_idx);

            byte_idx += 1;

            //  3. RSIZE is the total size of (number of array elements in) each record.
            let rsize = self.daf.read_f64(byte_idx);

            byte_idx += 1;

            //  4. N is the number of records contained in the segment.
            let num_records_in_seg = self.daf.read_f64(byte_idx);

            return Ok((
                seg,
                (
                    init_s_past_j2k,
                    interval_length as usize,
                    rsize as usize,
                    num_records_in_seg as usize,
                ),
            ));
        }
        Err(AniseError::NAIFConversionError(format!(
            "Could not find segment {}",
            seg_target_id
        )))
    }

    /// Returns all of the coefficients
    pub fn all_coefficients(
        &self,
        seg_target_id: i32,
    ) -> Result<(&Segment, Vec<SegmentExportData>), AniseError> {
        let (seg, (init_s_past_j2k, interval_length, rsize, num_records_in_seg)) =
            self.segment_ptr(seg_target_id)?;

        dbg!((
            seg.start_idx,
            (init_s_past_j2k, interval_length, rsize, num_records_in_seg)
        ));

        let mut full_data = Vec::new();

        dbg!(seg.start_idx, seg.end_idx, num_records_in_seg);

        for index in (0..num_records_in_seg).step_by(rsize) {
            let mut data = Vec::with_capacity(rsize);
            for _ in 0..rsize {
                data.push(0.0);
            }

            self.daf
                .read_f64s_into(seg.start_idx + index - 1, rsize, &mut data);

            let rcrd_mid_point = data[0];
            let rcrd_radius_s = data[1];
            let num_coeffs = (rsize - 2) / 3;
            let mut c_idx = 2;
            let x_coeffs = data[c_idx..c_idx + num_coeffs].to_vec();
            c_idx += num_coeffs;
            let y_coeffs = data[c_idx..c_idx + num_coeffs].to_vec();
            c_idx += num_coeffs;
            let z_coeffs = data[c_idx..c_idx + num_coeffs].to_vec();

            // Prep the data to be exported
            let export = SegmentExportData {
                rcrd_mid_point,
                rcrd_radius_s,
                x_coeffs,
                y_coeffs,
                z_coeffs,
            };

            full_data.push(export);
        }

        Ok((seg, full_data))
    }
}

impl<'a> TryInto<SPK<'a>> for &'a DAF<'a> {
    type Error = AniseError;

    fn try_into(self) -> Result<SPK<'a>, Self::Error> {
        let mut spk = SPK {
            segments: Vec::new(),
            daf: &self,
        };

        // Convert the summaries into segments
        for seg_data in self.summaries() {
            let (name, f64_data, int_data) = seg_data;
            if f64_data.len() != 2 {
                return Err(AniseError::NAIFConversionError(format!(
                    "SPK should have exactly two f64 data, found {}",
                    f64_data.len()
                )));
            }
            let start_epoch = Epoch::from_et_seconds(f64_data[0]);
            let end_epoch = Epoch::from_et_seconds(f64_data[1]);

            if int_data.len() != 6 {
                return Err(AniseError::NAIFConversionError(format!(
                    "SPK should have exactly five int data, found {}",
                    int_data.len()
                )));
            }

            let target_id = int_data[0];
            let center_id = int_data[1];
            let frame_id = int_data[2];
            let data_type_i = int_data[3];
            let start_idx = int_data[4] as usize;
            let end_idx = int_data[5] as usize;
            spk.segments.push(Segment {
                name: name.trim(),
                start_epoch,
                end_epoch,
                target_id,
                center_id,
                frame_id,
                data_type: data_type_i.try_into()?,
                start_idx,
                end_idx,
            });
        }

        Ok(spk)
    }
}

impl<'a> fmt::Display for SPK<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} with segments:\n", self.daf.idword)?;
        for seg in &self.segments {
            write!(f, "\t{}\n", seg)?;
        }
        fmt::Result::Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataType {
    /// Type 1
    ModifiedDifferenceArrays,
    /// Type 2
    ChebyshevPositionOnly,
    /// Type 3
    ChebyshevPositionVelocity,
    /// Type 5  (two body propagation)
    DiscreteStates,
    /// Type 8
    LagrangeInterpolationEqualTimeSteps,
    /// Type 9
    LagrangeInterpolationUnequalTimeSteps,
    /// Type 10
    SpaceCommandTwoLineElements,
    /// Type 12
    HermiteInterpolationEqualTimeSteps,
    /// Type 13
    HermiteInterpolationUnequalTimeSteps,
    /// Type 14
    ChebyshevPolynomialsUnequalTimeSteps,
    /// Type 15
    PrecessingConicPropagation,
    /// Type 17
    EquinoctialElements,
    /// Type 18
    ESOCHermiteLagrangeInterpolation,
    /// Type 19
    ESOCPiecewiseInterpolation,
    /// Type 20
    ChebyshevVelocityOnly,
    /// Type 21
    ExtendedModifiedDifferenceArrays,
}

impl TryFrom<i32> for DataType {
    type Error = AniseError;
    fn try_from(data_type: i32) -> Result<Self, AniseError> {
        match data_type {
            1 => Ok(Self::ModifiedDifferenceArrays),
            2 => Ok(Self::ChebyshevPositionOnly),
            3 => Ok(Self::ChebyshevPositionVelocity),
            5 => Ok(Self::DiscreteStates),
            8 => Ok(Self::LagrangeInterpolationEqualTimeSteps),
            9 => Ok(Self::LagrangeInterpolationUnequalTimeSteps),
            10 => Ok(Self::SpaceCommandTwoLineElements),
            12 => Ok(Self::HermiteInterpolationEqualTimeSteps),
            13 => Ok(Self::HermiteInterpolationUnequalTimeSteps),
            14 => Ok(Self::ChebyshevPolynomialsUnequalTimeSteps),
            15 => Ok(Self::PrecessingConicPropagation),
            17 => Ok(Self::EquinoctialElements),
            18 => Ok(Self::ESOCHermiteLagrangeInterpolation),
            19 => Ok(Self::ESOCPiecewiseInterpolation),
            20 => Ok(Self::ChebyshevVelocityOnly),
            21 => Ok(Self::ExtendedModifiedDifferenceArrays),
            _ => Err(AniseError::NAIFConversionError(format!(
                "unknwon data type {}",
                data_type
            ))),
        }
    }
}
