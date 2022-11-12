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
use self::segment::{Record, SegMetaData, Segment};

use super::dafold::{Endian, DAF};
use crate::constants::orientations::J2000;
use crate::errors::InternalErrorKind;
use crate::prelude::AniseError;
use crate::structure::common::InterpolationKind;
use crate::structure::context::AniseContext;
use crate::structure::ephemeris::Ephemeris;
use crate::structure::metadata::Metadata;
use crate::structure::spline::{Evenness, Field, SplineMeta, Splines, StateKind};
use crate::structure::units::{LengthUnit, TimeUnit};
use crate::{file_mmap, parse_bytes_as, DBL_SIZE};
use crc32fast::hash;
use der::{Decode, Encode};
use hifitime::{Epoch, TimeUnits};
use log::{info, warn};
use std::convert::TryInto;
use std::f64::EPSILON;
use std::fmt;
use std::fs::{remove_file, File};
use std::io::Write;

pub mod datatype;
pub mod recordtypes;
pub mod segment;
pub mod summary;

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

            if seg.data_type != DataType::ChebyshevPositionOnly
                && seg.data_type != DataType::ChebyshevPositionVelocity
            {
                return Err(AniseError::DAFParserError(format!(
                    "{:?} not yet supported",
                    seg.data_type
                )));
            }

            // For type 2, the config data is at the very end of the record
            // https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/spk.html#Type%202:%20Chebyshev%20(position%20only)

            let mut byte_idx = seg.end_idx - 4;
            //  1. INIT is the initial epoch of the first record, given in ephemeris seconds past J2000.
            let init_s_past_j2k = self.daf.read_f64(byte_idx);

            byte_idx += 1;

            //  2. INTLEN is the length of the interval covered by each record, in seconds.
            let interval_length_s = self.daf.read_f64(byte_idx);

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
                    interval_length_s,
                    rsize: rsize as usize,
                    num_records_in_seg: num_records_in_seg as usize,
                },
            ));
        }
        Err(AniseError::DAFParserError(format!(
            "Could not find segment {}",
            seg_target_id
        )))
    }

    /// Returns all of the coefficients
    pub fn copy_segments(
        &self,
        seg_target_id: i32,
    ) -> Result<(&Segment, SegMetaData, Vec<Record>), AniseError> {
        let (seg, meta) = self.segment_ptr(seg_target_id)?;

        let mut records = Vec::new();

        let mut dbl_idx = (seg.start_idx - 1) * DBL_SIZE;
        for rnum in (0..meta.num_records_in_seg * meta.rsize).step_by(meta.rsize) {
            let mut r_dbl_idx = dbl_idx;
            let rcrd_mid_point = parse_bytes_as!(
                f64,
                &self.daf.bytes[r_dbl_idx..DBL_SIZE + r_dbl_idx],
                self.daf.endianness
            );
            r_dbl_idx += DBL_SIZE;
            let rcrd_radius_s = parse_bytes_as!(
                f64,
                &self.daf.bytes[r_dbl_idx..DBL_SIZE + r_dbl_idx],
                self.daf.endianness
            );

            r_dbl_idx += DBL_SIZE;

            let raw_x_coeffs = &self.daf.bytes[r_dbl_idx..r_dbl_idx + DBL_SIZE * meta.degree()];

            let x_coeffs: Vec<f64> = (0..meta.degree())
                .map(|item| {
                    parse_bytes_as!(
                        f64,
                        raw_x_coeffs[DBL_SIZE * item..DBL_SIZE * (item + 1)],
                        self.daf.endianness
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
                        self.daf.endianness
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
                        self.daf.endianness
                    )
                })
                .collect::<_>();

            // Prep the data to be exported
            let rcrd = Record {
                rcrd_mid_point,
                rcrd_radius_s,
                x_coeffs,
                y_coeffs,
                z_coeffs,
                ..Default::default()
            };

            if rnum == 0 {
                info!("[copy_segments] {seg}");
                // The rcrd_radius_s should be a round integer, so let's check that
                assert!(
                    (rcrd_radius_s % rcrd_radius_s.floor()).abs() < EPSILON,
                    "Record radius is not an integer number of seconds"
                );
            }

            records.push(rcrd);
            r_dbl_idx += DBL_SIZE * meta.degree();
            dbl_idx = r_dbl_idx;
        }

        Ok((seg, meta, records))
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
            let (seg, meta, records) = self.copy_segments(seg.target_id)?;
            if records.len() <= 1 && skip_empty {
                warn!("[to_anise] skipping empty {seg}");
                continue;
            }
            // Some files don't have a useful name in the segments, so we append the target ID in case
            let hashed_name = hash(seg.human_name().as_bytes());

            let degree = (meta.rsize - 2) / 3;
            let state_kind = seg.data_type.to_anise_spline_coeff(degree);

            let metadata = SplineMeta {
                evenness: Evenness::Even {
                    duration_ns: ((meta.interval_length_s as f64).seconds()).to_parts().1,
                },
                state_kind,
                ..Default::default()
            };

            let mut spline_data = Vec::with_capacity(20_000);

            // Build the splines
            for record in &records {
                // Check that the interval length is indeed twice the radius, this is fixed.
                assert_eq!(meta.interval_length_s as f64, 2. * record.rcrd_radius_s);

                for midpoint_byte in record.rcrd_mid_point.to_be_bytes() {
                    spline_data.push(midpoint_byte);
                }

                for coeff in &record.x_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }

                for coeff in &record.y_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }
                for coeff in &record.z_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }

                for coeff in &record.vx_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }

                for coeff in &record.vy_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }
                for coeff in &record.vz_coeffs {
                    for coeffbyte in coeff.to_be_bytes() {
                        spline_data.push(coeffbyte);
                    }
                }
            }

            // Compute the crc32 of this data
            let chksum = hash(&spline_data);
            // Build the spline struct
            let splines = Splines {
                metadata,
                data_checksum: chksum,
                data: &spline_data,
            };

            let parent_ephemeris_hash = hash(Segment::id_to_human_name(seg.center_id)?.as_bytes());

            // Create the ephemeris
            let ephem = Ephemeris {
                name: seg.human_name(),
                ref_epoch: seg.start_epoch,
                backward: false,
                interpolation_kind: InterpolationKind::ChebyshevSeries,
                parent_ephemeris_hash,
                orientation_hash: J2000,
                length_unit: LengthUnit::Kilometer,
                time_unit: TimeUnit::Second,
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

        let mut lut_hashes = Vec::new();
        let mut lut_indexes = Vec::new();

        // Unwrap all of the possibly failing calls because we just created these files so we assume they're valid
        for buf in &all_bufs {
            let ephem: Ephemeris = match Ephemeris::from_der(buf) {
                Ok(it) => it,
                Err(err) => return Err(AniseError::DecodingError(err)),
            };
            ctx.append_ephemeris_mut(&mut lut_hashes, &mut lut_indexes, ephem)?;
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
            // If we skipped empty ephemerides, we can't check the exact length, so skip that
            if !skip_empty {
                assert_eq!(
                    ctx.ephemeris_lut.hashes.data.len(),
                    self.segments.len(),
                    "Incorrect number of ephem in map"
                );
                assert_eq!(
                    ctx.ephemeris_lut.indexes.data.len(),
                    self.segments.len(),
                    "Incorrect number of ephem in map"
                );
            }

            for (eidx, ephem) in ctx.ephemeris_data.iter().enumerate() {
                let seg_target_id = Segment::human_name_to_id(ephem.name)?;
                // Fetch the SPK segment
                let (seg, meta, all_seg_data) = self.copy_segments(seg_target_id)?;
                if all_seg_data.is_empty() {
                    continue;
                }

                assert_eq!(
                    seg.start_epoch,
                    ephem.start_epoch(),
                    "start epochs differ for {} (eidx = {}): {:E} != {:E}",
                    ephem.name,
                    eidx,
                    seg.start_epoch,
                    ephem.start_epoch()
                );

                let splines = &ephem.splines;
                match splines.metadata.evenness {
                    Evenness::Even { duration_ns } => {
                        assert_eq!(
                            duration_ns,
                            meta.interval_length_s.seconds().to_parts().1,
                            "incorrect interval duration"
                        );
                    }
                    _ => panic!("wrong spline kind"),
                };

                assert_eq!(
                    splines.metadata.state_kind,
                    StateKind::Position {
                        degree: ((meta.rsize - 2) / 3) as u8
                    }
                );
                assert!(splines.metadata.cov_kind.is_empty());

                info!(
                    "[to_anise] metadata OK for {}. Now checking each coefficient.",
                    ephem.name
                );

                for (sidx, seg_data) in all_seg_data.iter().enumerate() {
                    for (cidx, x_truth) in seg_data.x_coeffs.iter().enumerate() {
                        assert_eq!(splines.fetch(sidx, cidx, Field::X)?, *x_truth);
                    }

                    for (cidx, y_truth) in seg_data.y_coeffs.iter().enumerate() {
                        assert_eq!(splines.fetch(sidx, cidx, Field::Y)?, *y_truth);
                    }

                    for (cidx, z_truth) in seg_data.z_coeffs.iter().enumerate() {
                        assert_eq!(splines.fetch(sidx, cidx, Field::Z)?, *z_truth);
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
        for seg_data in self.summaries()? {
            let (name, f64_data, int_data) = seg_data;
            if f64_data.len() != 2 {
                return Err(AniseError::DAFParserError(format!(
                    "SPK should have exactly two f64 data, found {}",
                    f64_data.len()
                )));
            }

            let start_epoch = Epoch::from_et_seconds(f64_data[0]);
            let end_epoch = Epoch::from_et_seconds(f64_data[1]);

            if int_data.len() != 6 {
                return Err(AniseError::DAFParserError(format!(
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
