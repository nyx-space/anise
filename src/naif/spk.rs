/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate crc32fast;
extern crate der;
use super::daf::{Endianness, DAF, DBL_SIZE};
use crate::asn1::common::InterpolationKind;
use crate::asn1::ephemeris::Ephemeris;
use crate::asn1::root::{Metadata, TrajectoryFile};
use crate::asn1::spline::{SplineCoeffCount, SplineKind, Splines};
use crate::asn1::time::Epoch as AniseEpoch;
use crate::{file_mmap, parse_bytes_as};
use std::fs::File;
use std::intrinsics::transmute;
use std::io::Write;
// use crate::asn1::SplineAsn1;
// use crate::generated::anise_generated::anise::common::InterpolationKind;
// use crate::generated::anise_generated::anise::ephemeris::{
//     Ephemeris, EphemerisArgs, EqualTimeSteps, EqualTimeStepsArgs, Interpolator, Spline, SplineArgs,
// };
// use crate::generated::anise_generated::anise::time::System;
// use crate::generated::anise_generated::anise::{MapToIndex, MapToIndexArgs};
use crate::prelude::AniseError;
use crc32fast::hash;
use der::{Decode, Encode};
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
    // pub x_coeffs: &'a [u8],
    // pub y_coeffs: &'a [u8],
    // pub z_coeffs: &'a [u8],
    // pub vx_coeffs: &'a [f64],
    // pub vy_coeffs: &'a [f64],
    // pub vz_coeffs: &'a [f64],
}

#[derive(Debug)]
pub struct SPK<'a> {
    pub segments: Vec<Segment<'a>>,
    pub daf: &'a DAF<'a>,
}

pub struct SegMetaData {
    pub init_s_past_j2k: f64,
    pub interval_length: usize,
    pub rsize: usize,
    pub num_records_in_seg: usize,
}

impl<'a> SPK<'a> {
    /// Returns the segment buffer index and the config data of that segment as (init_s_past_j2k, interval_length, rsize, num_records_in_seg)
    pub fn segment_ptr(&self, seg_target_id: i32) -> Result<(&Segment, SegMetaData), AniseError> {
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
                SegMetaData {
                    init_s_past_j2k,
                    interval_length: interval_length as usize,
                    rsize: rsize as usize,
                    num_records_in_seg: num_records_in_seg as usize,
                },
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
    ) -> Result<(&Segment, SegMetaData, Vec<SegmentExportData>), AniseError> {
        let (seg, meta) = self.segment_ptr(seg_target_id)?;

        let mut full_data = Vec::new();

        let mut dbl_idx = seg.start_idx;
        for _ in (0..(meta.num_records_in_seg - 1) * meta.rsize).step_by(meta.rsize) {
            let mut r_dbl_idx = dbl_idx;
            let rcrd_mid_point = parse_bytes_as!(
                f64,
                &self.daf.bytes[r_dbl_idx..DBL_SIZE + r_dbl_idx],
                Endianness::Little
            );
            r_dbl_idx += DBL_SIZE;
            let rcrd_radius_s = parse_bytes_as!(
                f64,
                &self.daf.bytes[r_dbl_idx..DBL_SIZE + r_dbl_idx],
                Endianness::Little
            );
            r_dbl_idx += DBL_SIZE;

            let degree = (meta.rsize - 2) / 3 - 1;

            let raw_x_coeffs = &self.daf.bytes[r_dbl_idx..r_dbl_idx + DBL_SIZE * degree];

            let x_coeffs: Vec<f64> = (0..degree)
                .map(|item| {
                    parse_bytes_as!(
                        f64,
                        raw_x_coeffs[DBL_SIZE * item..DBL_SIZE * (item + 1)],
                        Endianness::Little
                    )
                })
                .collect::<_>();
            r_dbl_idx += DBL_SIZE * degree;
            let raw_y_coeffs = &self.daf.bytes[r_dbl_idx..r_dbl_idx + DBL_SIZE * degree];
            let y_coeffs: Vec<f64> = (0..degree)
                .map(|item| {
                    parse_bytes_as!(
                        f64,
                        raw_y_coeffs[DBL_SIZE * item..DBL_SIZE * (item + 1)],
                        Endianness::Little
                    )
                })
                .collect::<_>();
            r_dbl_idx += DBL_SIZE * degree;
            let raw_z_coeffs = &self.daf.bytes[r_dbl_idx..r_dbl_idx + DBL_SIZE * degree];
            let z_coeffs: Vec<f64> = (0..degree)
                .map(|item| {
                    parse_bytes_as!(
                        f64,
                        raw_z_coeffs[DBL_SIZE * item..DBL_SIZE * (item + 1)],
                        Endianness::Little
                    )
                })
                .collect::<_>();

            r_dbl_idx += DBL_SIZE * degree;
            dbl_idx = r_dbl_idx;
            // Prep the data to be exported
            let export = SegmentExportData {
                rcrd_mid_point,
                rcrd_radius_s,
                x_coeffs,
                y_coeffs,
                z_coeffs,
            };

            full_data.push(export);
            dbl_idx += meta.rsize * DBL_SIZE;
        }

        Ok((seg, meta, full_data))
    }
    /// Converts the provided SPK to an ANISE file
    pub fn to_anise(&'a self, orig_file: &str, filename: &str) {
        let meta = Metadata {
            file_version: crate::asn1::root::Semver {
                major: 4,
                minor: 3,
                patch: 1,
            },
            originator: orig_file,
            ..Default::default()
        };

        // Start the trajectory file so we can populate the lookup table (LUT)

        let mut traj_file = TrajectoryFile::default();
        traj_file.metadata = meta;

        // let mut all_splines = [Spline::default(); 20_000];
        let mut all_intermediate_files = Vec::new();

        for (idx, seg) in self.segments.iter().enumerate() {
            let (_, meta, seg_coeffs) = self.all_coefficients(seg.target_id).unwrap();
            if seg_coeffs.is_empty() {
                continue;
            }
            // let mut all_splines = Vec::with_capacity(20_000);
            // Some files don't have a useful name in the segments, so we append the target ID in case
            let name = format!("{} #{}", seg.name, seg.target_id);
            let hashed_name = hash(name.as_bytes());
            traj_file.ephemeris_lut.indexes.add(idx as u16).unwrap();
            traj_file.ephemeris_lut.hashes.add(hashed_name).unwrap();

            let degree = (meta.rsize - 2) / 3 - 1;
            let kind = SplineKind::FixedWindow {
                window_duration_s: seg_coeffs[0].rcrd_radius_s,
            };
            let config = SplineCoeffCount {
                degree: degree.try_into().unwrap(),
                num_position_coeffs: 3,
                num_velocity_coeffs: 3,
                ..Default::default()
            };
            let mut spline_data = Vec::with_capacity(20_000);

            // Build the splines
            for seg_coeff in &seg_coeffs {
                // for coeffidx in (0..seg_coeff.x_coeffs.len()).step_by(DBL_SIZE) {
                //     // Rebuild the f64 one at a time
                //     let coeff = parse_bytes_as!(
                //         f64,
                //         &seg_coeff.x_coeffs[coeffidx..DBL_SIZE],
                //         Endianness::Little
                //     );

                //     for coeffbyte in coeff.to_be_bytes() {
                //         spline_data.push(coeffbyte);
                //     }
                // }

                // for coeffidx in (0..seg_coeff.y_coeffs.len()).step_by(DBL_SIZE) {
                //     // Rebuild the f64 one at a time
                //     let coeff = parse_bytes_as!(
                //         f64,
                //         &seg_coeff.y_coeffs[coeffidx..DBL_SIZE],
                //         Endianness::Little
                //     );

                //     for coeffbyte in coeff.to_be_bytes() {
                //         spline_data.push(coeffbyte);
                //     }
                // }

                // for coeffidx in (0..seg_coeff.z_coeffs.len()).step_by(DBL_SIZE) {
                //     // Rebuild the f64 one at a time
                //     let coeff = parse_bytes_as!(
                //         f64,
                //         &seg_coeff.z_coeffs[coeffidx..DBL_SIZE],
                //         Endianness::Little
                //     );

                //     for coeffbyte in coeff.to_be_bytes() {
                //         spline_data.push(coeffbyte);
                //     }
                // }

                for coeff in &seg_coeff.x_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                    // spline_data.push(*coeff);
                }

                for coeff in &seg_coeff.y_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                    // spline_data.push(*coeff);
                }
                for coeff in &seg_coeff.z_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                    // spline_data.push(*coeff);
                }
                // for coeff in seg_coeff.vx_coeffs {
                //     for coeffbyte in coeff.to_be_bytes() {
                //         spline_data.push(coeffbyte);
                //     }
                // }
                // for coeff in seg_coeff.vy_coeffs {
                //     for coeffbyte in coeff.to_be_bytes() {
                //         spline_data.push(coeffbyte);
                //     }
                // }
                // for coeff in seg_coeff.vz_coeffs {
                //     for coeffbyte in coeff.to_be_bytes() {
                //         spline_data.push(coeffbyte);
                //     }
                // }
            }
            // Build the spline struct
            let splines = Splines {
                kind,
                config,
                data: &spline_data,
            };

            // Create the ephemeris
            let ephem = Ephemeris {
                name: name.as_str(),
                ref_epoch: AniseEpoch {
                    epoch: seg.start_epoch,
                    system: TimeSystem::TDB,
                },
                backward: false,
                interpolation_kind: InterpolationKind::ChebyshevSeries,
                parent_ephemeris_hash: 0, // TODO: Fix this
                orientation_hash: 0,      // TODO: Set J2000 orientation
                splines,
            };

            // Serialize this ephemeris and rebuild the full file in a minute

            let mut buf = Vec::new();
            let fname = format!("{idx}-{hashed_name}-{filename}");
            all_intermediate_files.push(fname.clone());
            let mut file = File::create(fname).unwrap();
            ephem.encode_to_vec(&mut buf).unwrap();
            // let ephem_dec: Ephemeris = Ephemeris::from_der(&buf).unwrap();
            file.write_all(&buf).unwrap();
        }

        // Now concat all of the files
        let mut all_bufs = Vec::new();
        for fname in all_intermediate_files {
            let bytes = file_mmap!(fname).unwrap();
            all_bufs.push(bytes);
        }
        for buf in &all_bufs {
            let ephem: Ephemeris = Ephemeris::from_der(&buf).unwrap();
            println!("Add {}", ephem.name);
            traj_file.ephemeris_data.add(ephem).unwrap();
        }

        let mut buf = Vec::new();
        let mut file = File::create(filename).unwrap();
        traj_file.encode_to_vec(&mut buf).unwrap();
        file.write_all(&buf).unwrap();
    }
}

impl<'a> TryInto<SPK<'a>> for &'a DAF<'a> {
    type Error = AniseError;

    fn try_into(self) -> Result<SPK<'a>, Self::Error> {
        let mut spk = SPK {
            // TODO : find a way to avoid alloc ?
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
