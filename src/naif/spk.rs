/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate crc32fast;
use super::daf::DAF;
use crate::generated::anise_generated::anise::common::InterpolationKind;
use crate::generated::anise_generated::anise::ephemeris::{
    Ephemeris, EphemerisArgs, EqualTimeSteps, EqualTimeStepsArgs, Interpolator, Spline, SplineArgs,
};
use crate::generated::anise_generated::anise::time::System;
use crate::generated::anise_generated::anise::{MapToIndex, MapToIndexArgs};
use crate::prelude::AniseError;
use crc32fast::hash;
use hifitime::{Epoch, TimeSystem};
use std::convert::{TryFrom, TryInto};
use std::fmt;

#[derive(Debug)]
pub struct Segment<'a> {
    pub name: &'a str,
    pub start_epoch: Epoch,
    pub end_epoch: Epoch,
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
        let (seg, (_, _, rsize, num_records_in_seg)) = self.segment_ptr(seg_target_id)?;

        let mut full_data = Vec::new();

        for index in (0..num_records_in_seg * rsize).step_by(rsize) {
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

    /// Converts the provided SPK to an ANISE file
    pub fn to_anise(&self, orig_file: &str, filename: &str) {
        use crate::prelude::*;
        use std::fs::File;
        use std::io::Write;
        let comment_str = format!("Converted from `{}` read as {}", orig_file, self.daf.idword);
        let publisher_str = "ANISE Toolkit team, v0.1";
        let mut fbb = flatbuffers::FlatBufferBuilder::with_capacity(1024);
        let comments = fbb.create_string(&comment_str);
        let publisher = fbb.create_string(publisher_str);
        let metadata = Metadata::create(
            &mut fbb,
            &MetadataArgs {
                comments: Some(comments),
                publisher: Some(publisher),
                publication_date: Some(&AniseEpoch::new(0.0, 0.0)),
                ..Default::default()
            },
        );

        // Iterate through all the segments and create the ANISE splines
        // Start by building the CRC32 map to index
        // We will store each ephemeris in the same order that they are in the initial file
        let j2000_hash = hash("J2000".as_bytes());
        let mut indexes = Vec::with_capacity(self.segments.len());
        let mut hashes = Vec::with_capacity(self.segments.len());
        let mut ephemerides = Vec::with_capacity(self.segments.len());
        for (idx, seg) in self.segments.iter().enumerate() {
            // Some files don't have a useful name in the segments, so we append the target ID in case
            let name = format!("{} #{}", seg.name, seg.target_id);
            let hashed_name = hash(name.as_bytes());
            indexes.push(idx as u16);
            hashes.push(hashed_name);
            let (_, seg_coeffs) = self.all_coefficients(seg.target_id).unwrap();
            let mut splines = Vec::with_capacity(self.segments.len());
            // Build the splines
            for seg_coeff in &seg_coeffs {
                let s_x = fbb.create_vector_direct(&seg_coeff.x_coeffs);
                let s_y = fbb.create_vector_direct(&seg_coeff.y_coeffs);
                let s_z = fbb.create_vector_direct(&seg_coeff.z_coeffs);
                splines.push(Spline::create(
                    &mut fbb,
                    &SplineArgs {
                        usable_start_epoch: Some(&AniseEpoch::new(0.0, 0.0)),
                        usable_end_epoch: Some(&AniseEpoch::new(0.0, 0.0)),
                        x: Some(s_x),
                        y: Some(s_y),
                        z: Some(s_z),
                        ..Default::default()
                    },
                ));
            }
            let et_splines = fbb.create_vector(&splines);
            // TODO: Support unequal time step splines
            let eqts = EqualTimeSteps::create(
                &mut fbb,
                &EqualTimeStepsArgs {
                    spline_duration_s: seg_coeffs[0].rcrd_radius_s,
                    splines: Some(et_splines),
                },
            );

            // Build the ephemeris for this data
            let e_name = fbb.create_string(&name);
            // BUG: Actually create a hashmap to find the name of the parent
            let name = format!("{} #{}", seg.name, seg.center_id);
            let parent_hash = hash(name.as_bytes());

            let ephem = Ephemeris::create(
                &mut fbb,
                &EphemerisArgs {
                    name: Some(e_name),
                    ref_epoch: Some(&AniseEpoch::new(0.0, 0.0)),
                    ref_system: System::TDB,
                    backward: false,
                    parent_hash,
                    orientation_hash: j2000_hash,
                    constants: None,
                    interpolation_kind: InterpolationKind::ChebyshevSeries,
                    interpolator_type: Interpolator::equal_time_steps,
                    interpolator: Some(eqts.as_union_value()),
                },
            );

            ephemerides.push(ephem);
        }
        // Create the MapToIndex for the ephemeris map
        let mti_hash = fbb.create_vector_direct(&hashes);
        let mti_index = fbb.create_vector_direct(&indexes);
        let ephemeris_map = MapToIndex::create(
            &mut fbb,
            &MapToIndexArgs {
                hash: Some(mti_hash),
                index: Some(mti_index),
            },
        );

        // Create the Ephemeris structure
        let ephem_vec = fbb.create_vector(&ephemerides);

        let root = Anise::create(
            &mut fbb,
            &AniseArgs {
                metadata: Some(metadata),
                ephemeris_map: Some(ephemeris_map),
                ephemerides: Some(ephem_vec),
                ..Default::default()
            },
        );
        fbb.finish(root, Some("ANIS"));

        // Create the file
        let mut file = File::create(filename).unwrap();
        file.write_all(fbb.finished_data()).unwrap();
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
