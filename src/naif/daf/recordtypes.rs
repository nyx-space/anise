/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use zerocopy::FromBytes;

use crate::{naif::Endian, prelude::AniseError, DBL_SIZE};
use log::{error, warn};

use super::{NAIFRecord, RCRD_LEN};

#[derive(Debug, FromBytes)]
#[repr(C)]
pub struct DAFFileRecord {
    pub locidw: [u8; 8],
    pub nd: u32,
    pub ni: u32,
    pub locifn: [u8; 60],
    pub forward: u32,
    pub backward: u32,
    pub free_addr: u32,
    pub locfmt: [u8; 8],
    pub prenul: [u8; 603],
    pub ftpstr: [u8; 28],
    pub pstnul: [u8; 297],
}

impl Default for DAFFileRecord {
    fn default() -> Self {
        Self {
            locidw: [0; 8],
            nd: Default::default(),
            ni: Default::default(),
            locifn: [0; 60],
            forward: Default::default(),
            backward: Default::default(),
            free_addr: Default::default(),
            locfmt: [0; 8],
            prenul: [0; 603],
            ftpstr: [0; 28],
            pstnul: [0; 297],
        }
    }
}

impl NAIFRecord for DAFFileRecord {}

impl DAFFileRecord {
    pub fn ni(&self) -> usize {
        self.ni as usize
    }

    pub fn nd(&self) -> usize {
        self.nd as usize
    }

    pub fn fwrd_idx(&self) -> usize {
        self.forward as usize
    }

    pub fn summary_size(&self) -> usize {
        (self.nd + (self.ni + 1) / 2) as usize
    }

    pub fn identification(&self) -> Result<&str, AniseError> {
        let str_locidw = core::str::from_utf8(&self.locidw)
            .map_err(|_| AniseError::DAFParserError("Could not parse endianness".to_owned()))?;

        if &str_locidw[0..3] != "DAF" {
            Err(AniseError::DAFParserError(format!(
                "Cannot parse file whose identifier is not DAF: `{}`",
                str_locidw,
            )))
        } else if str_locidw.chars().nth(3) != Some('/') {
            Err(AniseError::DAFParserError(format!(
                "Cannot parse file whose identifier is not DAF: `{}`",
                str_locidw,
            )))
        } else {
            match str_locidw[4..].trim() {
                "SPK" => Ok("SPK"),
                "PCK" => Ok("PCK"),
                _ => {
                    error!("DAF of type `{}` is not yet supported", &str_locidw[4..]);
                    return Err(AniseError::DAFParserError(format!(
                        "Cannot parse SPICE data of type `{}`",
                        str_locidw
                    )));
                }
            }
        }
    }

    pub fn endianness(&self) -> Result<Endian, AniseError> {
        let str_endianness = core::str::from_utf8(&self.locfmt)
            .map_err(|_| AniseError::DAFParserError("Could not parse endianness".to_owned()))?;

        if str_endianness == "LTL-IEEE" {
            Ok(Endian::Little)
        } else if str_endianness == "BIG-IEEE" {
            Ok(Endian::Big)
        } else {
            Err(AniseError::DAFParserError(format!(
                "Could not understand endianness: `{}`",
                str_endianness
            )))
        }
    }

    pub fn internal_filename(&self) -> Result<&str, AniseError> {
        match core::str::from_utf8(&self.locifn) {
            Ok(filename) => Ok(filename.trim()),
            Err(e) => Err(AniseError::DAFParserError(format!("{e}"))),
        }
    }
}

#[derive(Debug, Default, FromBytes)]
#[repr(C)]
pub struct DAFSummaryRecord {
    next_record: f64,
    prev_record: f64,
    num_summaries: f64,
}

impl NAIFRecord for DAFSummaryRecord {}

impl DAFSummaryRecord {
    pub fn next_record(&self) -> usize {
        self.next_record as usize
    }

    pub fn prev_record(&self) -> usize {
        self.prev_record as usize
    }

    pub fn num_summaries(&self) -> usize {
        self.num_summaries as usize
    }

    pub fn is_final_record(&self) -> bool {
        self.next_record() == 0
    }
}

#[derive(Debug, FromBytes)]
pub struct NameRecord {
    raw_names: [u8; RCRD_LEN],
}

impl Default for NameRecord {
    fn default() -> Self {
        Self {
            raw_names: [0_u8; RCRD_LEN],
        }
    }
}

impl NAIFRecord for NameRecord {}

impl NameRecord {
    /// Returns the number of names in this record
    pub fn num_entries(&self, summary_size: usize) -> usize {
        self.raw_names.len() / summary_size * DBL_SIZE
    }

    pub fn nth_name(&self, n: usize, summary_size: usize) -> &str {
        let this_name =
            &self.raw_names[n * summary_size * DBL_SIZE..(n + 1) * summary_size * DBL_SIZE];
        match core::str::from_utf8(&this_name) {
            Ok(name) => name,
            Err(e) => {
                warn!(
                    "malformed name record: `{e}` from {:?}! Using `UNNAMED OBJECT` instead",
                    this_name
                );
                "UNNAMED OBJECT"
            }
        }
    }
}
