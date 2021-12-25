/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::daf::{Endianness, DAF, DBL_SIZE};
use crate::parse_bytes_as;
use crate::prelude::AniseError;
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
    start_idx: usize,
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
            "Segment `{}` (tgt: {}, ctr: {}, frame: {}) of type {:?} from {} ({}) to {} ({})",
            self.name,
            self.target_id,
            self.center_id,
            self.frame_id,
            self.data_type,
            self.start_epoch.as_gregorian_str(TimeSystem::ET),
            self.start_epoch.as_et_duration().in_seconds(),
            self.end_epoch.as_gregorian_str(TimeSystem::ET),
            self.end_epoch.as_et_duration().in_seconds()
        )
    }
}

#[derive(Debug)]
pub struct SPK<'a> {
    pub segments: Vec<Segment<'a>>,
    pub daf: &'a DAF<'a>,
}

impl<'a> SPK<'a> {
    pub fn query(
        &self,
        seg_target_id: i32,
        epoch_et_s: f64,
    ) -> Result<([f64; 3], [f64; 3]), AniseError> {
        for seg in &self.segments {
            if seg.target_id != seg_target_id {
                continue;
            }

            if seg.data_type != DataType::ChebyshevPositionOnly {
                return Err(AniseError::NAIFConversionError(
                    "Only cheby supported".to_string(),
                ));
            }

            let mut pos = [0.0, 0.0, 0.0];
            let mut vel = [0.0, 0.0, 0.0];

            // For type 2, the config data is at the very end of the file
            // https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/spk.html#Type%202:%20Chebyshev%20(position%20only)

            let start_byte = seg.end_idx - 1;
            let end_byte = seg.end_idx;

            dbg!(start_byte, end_byte);

            let init_s = parse_bytes_as!(
                f64,
                &self.daf.bytes[start_byte..start_byte + DBL_SIZE],
                self.daf.endianness
            );

            let initlen = parse_bytes_as!(
                f64,
                &self.daf.bytes[start_byte + DBL_SIZE..start_byte + DBL_SIZE * 2],
                self.daf.endianness
            );

            let rsize = parse_bytes_as!(
                f64,
                &self.daf.bytes[start_byte + DBL_SIZE * 2..start_byte + DBL_SIZE * 3],
                self.daf.endianness
            );

            let n = parse_bytes_as!(
                f64,
                &self.daf.bytes[start_byte + 3 * DBL_SIZE..end_byte],
                self.daf.endianness
            );

            dbg!(init_s, initlen, rsize, n);

            return Ok((pos, vel));
        }
        Err(AniseError::NAIFConversionError(format!(
            "Could not find segment {}",
            seg_target_id
        )))
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
