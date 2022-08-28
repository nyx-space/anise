/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

extern crate crc32fast;
extern crate der;
use self::datatype::DataType;
use self::segment::{SegMetaData, Segment, SegmentExportData};

use super::daf::{Endianness, DAF};
use crate::asn1::common::InterpolationKind;
use crate::asn1::context::AniseContext;
use crate::asn1::ephemeris::Ephemeris;
use crate::asn1::metadata::Metadata;
use crate::asn1::spline::Splines;
use crate::asn1::splinecoeffs::Coefficient;
use crate::asn1::splinekind::SplineKind;
use crate::constants::orientations::J2000;
use crate::errors::InternalErrorKind;
use crate::prelude::AniseError;
use crate::{file_mmap, parse_bytes_as, DBL_SIZE};
use crc32fast::hash;
use der::{Decode, Encode};
use hifitime::{Epoch, TimeSystem, TimeUnits};
use log::{info, warn};
use std::convert::TryInto;
use std::f64::EPSILON;
use std::fmt;
use std::fs::{remove_file, File};
use std::io::Write;

pub mod datatype;
pub mod segment;

#[derive(Debug)]
pub struct SPK<'a> {
    pub segments: Vec<Segment<'a>>,
    pub daf: &'a DAF<'a>,
}

impl<'a> SPK<'a> {
    /// Returns the segment buffer index and the config data of that segment as (init_s_past_j2k, interval_length, rsize, num_records_in_seg)
    pub fn segment_ptr(&self, seg_target_id: i32) -> Result<(&Segment, SegMetaData), AniseError> {
        for seg in &self.segments {
            if seg.target_id != seg_target_id {
                continue;
            }

            if seg.data_type != DataType::ChebyshevPositionOnly {
                return Err(AniseError::NAIFParseError(
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
        Err(AniseError::NAIFParseError(format!(
            "Could not find segment {}",
            seg_target_id
        )))
    }

    /// Returns all of the coefficients
    pub fn copy_coeffs(
        &self,
        seg_target_id: i32,
    ) -> Result<(&Segment, SegMetaData, Vec<SegmentExportData>), AniseError> {
        let (seg, meta) = self.segment_ptr(seg_target_id)?;

        let mut full_data = Vec::new();

        let mut dbl_idx = (seg.start_idx - 1) * DBL_SIZE;
        for rnum in (0..meta.num_records_in_seg * meta.rsize).step_by(meta.rsize) {
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

            let raw_x_coeffs = &self.daf.bytes[r_dbl_idx..r_dbl_idx + DBL_SIZE * meta.degree()];

            let x_coeffs: Vec<f64> = (0..meta.degree())
                .map(|item| {
                    parse_bytes_as!(
                        f64,
                        raw_x_coeffs[DBL_SIZE * item..DBL_SIZE * (item + 1)],
                        Endianness::Little
                    )
                })
                .collect::<_>();
            r_dbl_idx += DBL_SIZE * meta.degree();
            let raw_y_coeffs = &self.daf.bytes[r_dbl_idx..r_dbl_idx + DBL_SIZE * meta.degree()];
            let y_coeffs: Vec<f64> = (0..meta.degree())
                .map(|item| {
                    parse_bytes_as!(
                        f64,
                        raw_y_coeffs[DBL_SIZE * item..DBL_SIZE * (item + 1)],
                        Endianness::Little
                    )
                })
                .collect::<_>();
            r_dbl_idx += DBL_SIZE * meta.degree();
            let raw_z_coeffs = &self.daf.bytes[r_dbl_idx..r_dbl_idx + DBL_SIZE * meta.degree()];
            let z_coeffs: Vec<f64> = (0..meta.degree())
                .map(|item| {
                    parse_bytes_as!(
                        f64,
                        raw_z_coeffs[DBL_SIZE * item..DBL_SIZE * (item + 1)],
                        Endianness::Little
                    )
                })
                .collect::<_>();

            // Prep the data to be exported
            let export = SegmentExportData {
                rcrd_mid_point,
                rcrd_radius_s,
                x_coeffs,
                y_coeffs,
                z_coeffs,
                ..Default::default()
            };

            if rnum == 0 {
                info!(
                    "[copy_coeffs] [{}..{}] ({} rps) `{}` (tgt={}, ctr={}, frame={}) {:?} | {} .. {}",
                    seg.start_idx,
                    seg.end_idx,
                    meta.num_records_in_seg,
                    seg.name,
                    seg.target_id,
                    seg.center_id,
                    seg.frame_id,
                    seg.data_type,
                    seg.start_epoch.as_gregorian_str(TimeSystem::ET),
                    seg.end_epoch.as_gregorian_str(TimeSystem::ET),
                );
                // The rcrd_radius_s should be a round integer, so let's check that
                assert!((rcrd_radius_s % rcrd_radius_s.floor()).abs() < EPSILON);
            }

            full_data.push(export);
            r_dbl_idx += DBL_SIZE * meta.degree();
            dbl_idx = r_dbl_idx;
        }

        Ok((seg, meta, full_data))
    }

    /// Converts the provided SPK to an ANISE file
    ///
    /// WARNING: The segment name will be automatically switched to the human name of the celestial body
    /// from its unspecific "DE-0438LE-0438" if that name can be infered correctly.
    pub fn to_anise(
        &'a self,
        orig_file: &str,
        filename: &str,
        skip_empty: bool,
        check: bool,
    ) -> Result<(), AniseError> {
        // Start the trajectory file so we can populate the lookup table (LUT)
        let mut ctx = AniseContext {
            metadata: Metadata {
                originator: orig_file,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut all_intermediate_files = Vec::new();

        for (idx, seg) in self.segments.iter().enumerate() {
            let (seg, meta, seg_coeffs) = self.copy_coeffs(seg.target_id)?;
            if seg_coeffs.len() <= 1 && skip_empty {
                warn!("[to_anise] skipping empty {seg}");
                continue;
            }
            // Some files don't have a useful name in the segments, so we append the target ID in case
            let hashed_name = hash(seg.human_name().as_bytes());

            let degree = (meta.rsize - 2) / 3;
            let kind = SplineKind::FixedWindow {
                window_duration_s: meta.interval_length as f64,
            };

            let config = seg.data_type.to_anise_spline_coeff(degree);
            let mut spline_data = Vec::with_capacity(20_000);

            // Build the splines
            for seg_coeff in &seg_coeffs {
                // Check that the interval length is indeed twice the radius
                assert_eq!(meta.interval_length as f64, 2. * seg_coeff.rcrd_radius_s);

                for coeff in &seg_coeff.x_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }

                for coeff in &seg_coeff.y_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }
                for coeff in &seg_coeff.z_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }

                for coeff in &seg_coeff.vx_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }

                for coeff in &seg_coeff.vy_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }
                for coeff in &seg_coeff.vz_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }
            }

            // Compute the crc32 of this data
            let chksum = hash(&spline_data);
            // Build the spline struct
            let splines = Splines {
                kind,
                config,
                data_checksum: chksum,
                data: &spline_data,
            };

            let parent_ephemeris_hash = hash(Segment::id_to_human_name(seg.center_id)?.as_bytes());

            // Create the ephemeris
            let ephem = Ephemeris {
                name: seg.human_name(),
                ref_epoch: seg.start_epoch.into(),
                backward: false,
                interpolation_kind: InterpolationKind::ChebyshevSeries,
                parent_ephemeris_hash,
                orientation_hash: J2000,
                splines,
            };

            // Serialize this ephemeris and rebuild the full file in a minute

            let mut buf = Vec::new();
            let fname = format!("{filename}-{idx}-{hashed_name}.tmp");
            all_intermediate_files.push(fname.clone());
            match File::create(fname) {
                Ok(mut file) => {
                    if let Err(e) = ephem.encode_to_vec(&mut buf) {
                        return Err((InternalErrorKind::from(e)).into());
                    }
                    if let Err(e) = file.write_all(&buf) {
                        return Err(e.kind().into());
                    }
                }
                Err(e) => {
                    return Err(AniseError::IOError(e.kind()));
                }
            }
        }

        // Now concat all of the files
        let mut all_bufs = Vec::new();
        for fname in &all_intermediate_files {
            let bytes = file_mmap!(fname).unwrap();
            all_bufs.push(bytes);
        }
        // Unwrap all of the possibly failing calls because we just created these files so we assume they're valid
        for buf in &all_bufs {
            let ephem: Ephemeris = Ephemeris::from_der(buf).unwrap();
            ctx.append_ephemeris_mut(ephem).unwrap();
        }

        ctx.save_as(filename, true)?;
        // And delete the temporary files
        for fname in &all_intermediate_files {
            remove_file(fname).unwrap();
        }

        // Now, let's load this newly created file and make sure that everything matches
        if check {
            info!("[to_anise] checking conversion was correct (this will take a while)");
            // Load this ANIS file and make sure that it matches the original data.
            let bytes = file_mmap!(filename).unwrap();
            let ctx = AniseContext::from_bytes(&bytes);
            // If we skiped empty ephems, we can't check thet exact length, so skip that
            if !skip_empty {
                assert_eq!(
                    ctx.ephemeris_lut.hashes.len(),
                    self.segments.len(),
                    "Incorrect number of ephem in map"
                );
                assert_eq!(
                    ctx.ephemeris_lut.indexes.len(),
                    self.segments.len(),
                    "Incorrect number of ephem in map"
                );
            }

            for (eidx, ephem) in ctx.ephemeris_data.iter().enumerate() {
                let seg_target_id = Segment::human_name_to_id(ephem.name)?;
                // Fetch the SPK segment
                let (seg, meta, all_seg_data) = self.copy_coeffs(seg_target_id).unwrap();
                if all_seg_data.is_empty() {
                    continue;
                }

                assert_eq!(
                    seg.start_epoch,
                    ephem.start_epoch().epoch,
                    "start epochs differ for {} (eidx = {}): {} != {}",
                    ephem.name,
                    eidx,
                    seg.start_epoch.as_gregorian_str(TimeSystem::ET),
                    ephem.start_epoch().epoch.as_gregorian_str(TimeSystem::ET),
                );

                let splines = &ephem.splines;
                match splines.kind {
                    SplineKind::FixedWindow { window_duration_s } => {
                        assert_eq!(
                            window_duration_s, meta.interval_length as f64,
                            "incorrect interval duration"
                        );
                    }
                    _ => panic!("wrong spline kind"),
                };

                assert_eq!(splines.config.num_position_coeffs, 3);
                assert_eq!(splines.config.num_position_dt_coeffs, 0);
                assert_eq!(splines.config.num_velocity_coeffs, 0);
                assert_eq!(splines.config.num_velocity_dt_coeffs, 0);
                assert_eq!(splines.config.num_epochs, 0);

                info!(
                    "[to_anise] metadata OK for {}. Now checking each coefficient.",
                    ephem.name
                );

                for (sidx, seg_data) in all_seg_data.iter().enumerate() {
                    for (cidx, x_truth) in seg_data.x_coeffs.iter().enumerate() {
                        assert_eq!(splines.fetch(sidx, cidx, Coefficient::X).unwrap(), *x_truth);
                    }

                    for (cidx, y_truth) in seg_data.y_coeffs.iter().enumerate() {
                        assert_eq!(splines.fetch(sidx, cidx, Coefficient::Y).unwrap(), *y_truth);
                    }

                    for (cidx, z_truth) in seg_data.z_coeffs.iter().enumerate() {
                        assert_eq!(splines.fetch(sidx, cidx, Coefficient::Z).unwrap(), *z_truth);
                    }
                }

                info!("[to_anise] spline data OK for {}.", ephem.name);
            }
        }

        Ok(())
    }
}

impl<'a> TryInto<SPK<'a>> for &'a DAF<'a> {
    type Error = AniseError;

    fn try_into(self) -> Result<SPK<'a>, Self::Error> {
        let mut spk = SPK {
            // Alloc for conversion of SPICE files is _reasonable_ as it won't be used onboard
            segments: Vec::new(),
            daf: self,
        };

        // Convert the summaries into segments
        for seg_data in self.summaries() {
            let (name, f64_data, int_data) = seg_data;
            if f64_data.len() != 2 {
                return Err(AniseError::NAIFParseError(format!(
                    "SPK should have exactly two f64 data, found {}",
                    f64_data.len()
                )));
            }
            // NOTE: SPICE stores data in J2000 but hifitime stores it in J1900, so we shift everything by one century
            let start_epoch = Epoch::from_et_seconds(f64_data[0]) + 1.0.centuries();
            let end_epoch = Epoch::from_et_seconds(f64_data[1]) + 1.0.centuries();

            if int_data.len() != 6 {
                return Err(AniseError::NAIFParseError(format!(
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
        writeln!(f, "{} with segments:", self.daf.idword)?;
        for seg in &self.segments {
            writeln!(f, "\t{}", seg)?;
        }
        fmt::Result::Ok(())
    }
}
