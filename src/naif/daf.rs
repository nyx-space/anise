/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

pub(crate) use super::Endian;
use crate::{parse_bytes_as, prelude::AniseError, DBL_SIZE};
use core::convert::TryInto;
use log::{debug, error, info};

pub(crate) const RCRD_LEN: usize = 1024;
pub(crate) const INT_SIZE: usize = 4;

#[derive(Debug)]
pub struct DAF<'a> {
    pub idword: &'a str,
    pub internal_filename: &'a str,
    /// The number of integer components in each array summary.
    pub ni: usize,
    /// The number of double precision components in each array summary.
    pub nd: usize,
    /// The record number of the initial summary record in the file.
    pub fwrd: usize,
    /// The record number of the final summary record in the file.
    pub bwrd: usize,
    pub freeaddr: usize,
    pub endianness: Endian,
    pub bytes: &'a [u8],
}

impl<'a> DAF<'a> {
    /// From https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/daf.html#Structure
    pub fn parse(bytes: &'a [u8]) -> Result<Self, AniseError> {
        let locidw = core::str::from_utf8(bytes.get(0..8).ok_or(AniseError::MalformedData(8))?)
            .map_err(|_| {
                AniseError::DAFParserError("Could not parse header (first 8 bytes)".to_owned())
            })?;

        if !locidw.contains('/') {
            return Err(AniseError::DAFParserError(format!(
                "Cannot parse file whose identifier is: `{}`",
                locidw
            )));
        }

        let daftype_it = locidw.split('/');
        for (idx, content) in daftype_it.enumerate() {
            if idx == 0 && content != "DAF" {
                return Err(AniseError::DAFParserError(format!(
                    "Cannot parse file whose identifier is not DAF: `{}`",
                    locidw
                )));
            } else if idx == 1 {
                match content.trim() {
                    "SPK" => {
                        debug!("Parsing DAF as SPK");
                    }
                    "PCK" => {
                        info!("Parsing DAF as PCF (good luck)");
                    }
                    _ => {
                        error!("DAF of type {content} is not yet supported");
                        return Err(AniseError::DAFParserError(format!(
                            "Cannot parse SPICE data of type `{}`",
                            locidw
                        )));
                    }
                }
            } else if idx > 1 {
                return Err(AniseError::DAFParserError(format!(
                    "Malformed header string: `{}`",
                    locidw
                )));
            }
        }

        // We need to figure out if this file is big or little endian before we can convert some byte arrays into integer
        let str_endianness =
            core::str::from_utf8(bytes.get(88..96).ok_or(AniseError::MalformedData(96))?)
                .map_err(|_| AniseError::DAFParserError("Could not parse endianness".to_owned()))?;

        let endianness = if str_endianness == "LTL-IEEE" {
            Endian::Little
        } else if str_endianness == "BIG-IEEE" {
            Endian::Big
        } else {
            return Err(AniseError::DAFParserError(format!(
                "Could not understand endianness: `{}`",
                str_endianness
            )));
        };

        // Note that we parse as u32 to make sure that it's a 32-bit integer. The docs don't specify if it's signed or not,
        // but it works in either case (I guess that the sign bit is still present but set to zero?)
        let nd = parse_bytes_as!(
            u32,
            bytes
                .get(8..8 + INT_SIZE)
                .ok_or(AniseError::MalformedData(8 + INT_SIZE))?,
            endianness
        ) as usize;
        let ni = parse_bytes_as!(
            u32,
            bytes
                .get(12..12 + INT_SIZE)
                .ok_or(AniseError::MalformedData(12 + INT_SIZE))?,
            endianness
        ) as usize;
        let fwrd = parse_bytes_as!(
            u32,
            bytes
                .get(76..76 + INT_SIZE)
                .ok_or(AniseError::MalformedData(76 + INT_SIZE))?,
            endianness
        ) as usize;
        let bwrd = parse_bytes_as!(
            u32,
            bytes
                .get(80..80 + INT_SIZE)
                .ok_or(AniseError::MalformedData(80 + INT_SIZE))?,
            endianness
        ) as usize;
        let freeaddr = parse_bytes_as!(
            u32,
            bytes
                .get(84..84 + INT_SIZE)
                .ok_or(AniseError::MalformedData(84 + INT_SIZE))?,
            endianness
        ) as usize;

        let locifn = core::str::from_utf8(bytes.get(16..76).ok_or(AniseError::MalformedData(79))?)
            .map_err(|_| AniseError::DAFParserError("Could not parse locifn".to_owned()))?;

        Ok(Self {
            idword: locidw.trim(),
            internal_filename: locifn.trim(),
            nd,
            ni,
            fwrd,
            bwrd,
            freeaddr,
            endianness,
            bytes,
        })
    }

    pub fn comments(&self) -> Result<String, AniseError> {
        let mut rslt = String::new();
        // FWRD has the initial record of the summary. So we assume that all records between the second record and that one are comments
        for rid in 1..self.fwrd {
            // match core::str::from_utf8(&self.bytes[rid * RCRD_LEN..(rid + 1) * RCRD_LEN]) {
            match core::str::from_utf8(
                self.bytes
                    .get(rid * RCRD_LEN..(rid + 1) * RCRD_LEN)
                    .ok_or(AniseError::MalformedData(0))?,
            ) {
                Ok(s) => rslt += s.replace("\u{0}\u{0}", " ").replace('\u{0}', "\n").trim(),
                Err(e) => {
                    let valid_s = core::str::from_utf8(
                        &self.bytes[rid * RCRD_LEN..(rid * RCRD_LEN + e.valid_up_to())],
                    )
                    .unwrap();
                    rslt += valid_s
                        .replace("\u{0}\u{0}", " ")
                        .replace('\u{0}', "\n")
                        .trim()
                }
            }
        }

        Ok(rslt)
    }

    /// The summaries are needed to decode the rest of the file
    pub fn summaries(&self) -> Result<Vec<(&'a str, Vec<f64>, Vec<i32>)>, AniseError> {
        // Each summary need to be read in bytes of 8*nd then 4*self.ni
        let mut record_num = self.fwrd;
        let mut rtn = Vec::new();
        loop {
            if record_num == 0 {
                break;
            }
            let record = self.record(record_num)?;
            // Note that the segment control data are stored as f64 but need to be converted to usize
            let next_record = parse_bytes_as!(
                f64,
                record
                    .get(0..DBL_SIZE)
                    .ok_or(AniseError::MalformedData(DBL_SIZE))?,
                self.endianness
            ) as usize;

            let nsummaries = parse_bytes_as!(
                f64,
                record
                    .get(16..16 + DBL_SIZE)
                    .ok_or(AniseError::MalformedData(16 + DBL_SIZE))?,
                self.endianness
            ) as usize;

            // Parse the data of the summary.
            let name_record = self.record(record_num + 1)?;
            let length = DBL_SIZE * self.nd + INT_SIZE * self.ni;
            for i in (0..nsummaries * length).step_by(length) {
                let j = 3 * DBL_SIZE + i;
                let name = core::str::from_utf8(&name_record[i..i + length]).unwrap();
                if name.starts_with(' ') {
                    println!("WARNING: Parsing might be wrong because the first character of the name summary is a space: `{}`", name);
                    println!(
                        "Full name data: `{}`",
                        core::str::from_utf8(&name_record[..1000]).unwrap()
                    );
                }
                let summary_data = &record[j..j + length];
                let mut f64_summary = Vec::with_capacity(self.nd);
                for double_data in summary_data[0..DBL_SIZE * self.nd].chunks(DBL_SIZE) {
                    f64_summary.push(parse_bytes_as!(f64, double_data, self.endianness));
                }
                let mut int_summary = Vec::with_capacity(self.ni);
                for int_data in summary_data
                    [DBL_SIZE * self.nd..(self.nd * DBL_SIZE + self.ni * INT_SIZE)]
                    .chunks(INT_SIZE)
                {
                    int_summary.push(parse_bytes_as!(i32, int_data, self.endianness));
                }
                // Add this data to the return vec
                rtn.push((name, f64_summary, int_summary));
            }
            record_num = next_record;
        }

        Ok(rtn)
    }

    /// Records are indexed from one!!
    fn record(&self, num: usize) -> Result<&'a [u8], AniseError> {
        let start_idx = num * RCRD_LEN - RCRD_LEN;
        self.bytes
            .get(start_idx..start_idx + RCRD_LEN)
            .ok_or(AniseError::MalformedData(start_idx + RCRD_LEN))
    }

    /// Returns the 64-bit float at the provided address
    pub(crate) fn read_f64(&self, byte_idx: usize) -> f64 {
        parse_bytes_as!(
            f64,
            &self.bytes[DBL_SIZE * byte_idx..DBL_SIZE * (byte_idx + 1)],
            self.endianness
        )
    }
}
