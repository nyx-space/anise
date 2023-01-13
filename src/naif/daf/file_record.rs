/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use zerocopy::{AsBytes, FromBytes};

use crate::{naif::Endian, prelude::AniseError};
use log::error;

use super::NAIFRecord;

#[derive(Debug, Clone, FromBytes, AsBytes, PartialEq)]
#[repr(C)]
pub struct FileRecord {
    pub id_str: [u8; 8],
    pub nd: u32,
    pub ni: u32,
    pub internal_filename: [u8; 60],
    pub forward: u32,
    pub backward: u32,
    pub free_addr: u32,
    pub endian_str: [u8; 8],
    pub pre_null: [u8; 603],
    pub ftp_str: [u8; 28],
    pub pst_null: [u8; 297],
}

impl Default for FileRecord {
    fn default() -> Self {
        Self {
            id_str: [0; 8],
            nd: Default::default(),
            ni: Default::default(),
            internal_filename: [0; 60],
            forward: Default::default(),
            backward: Default::default(),
            free_addr: Default::default(),
            endian_str: [0; 8],
            pre_null: [0; 603],
            ftp_str: [0; 28],
            pst_null: [0; 297],
        }
    }
}

impl NAIFRecord for FileRecord {}

impl FileRecord {
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
        let str_locidw = core::str::from_utf8(&self.id_str).map_err(|_| {
            AniseError::DAFParserError("Could not parse identification string".to_owned())
        })?;

        if &str_locidw[0..3] != "DAF" || str_locidw.chars().nth(3) != Some('/') {
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
                    Err(AniseError::DAFParserError(format!(
                        "Cannot parse SPICE data of type `{}`",
                        str_locidw
                    )))
                }
            }
        }
    }

    pub fn endianness(&self) -> Result<Endian, AniseError> {
        let str_endianness = core::str::from_utf8(&self.endian_str)
            .map_err(|_| AniseError::DAFParserError("Could not parse endianness".to_owned()))?;

        let file_endian = if str_endianness == "LTL-IEEE" {
            Endian::Little
        } else if str_endianness == "BIG-IEEE" {
            Endian::Big
        } else {
            return Err(AniseError::DAFParserError(format!(
                "Could not understand endianness: `{}`",
                str_endianness
            )));
        };
        if file_endian != Endian::f64_native() || file_endian != Endian::u64_native() {
            Err(AniseError::DAFParserError(
                "Input file has different endian-ness than the platform and cannot be decoded"
                    .to_string(),
            ))
        } else {
            Ok(file_endian)
        }
    }

    pub fn internal_filename(&self) -> Result<&str, AniseError> {
        match core::str::from_utf8(&self.internal_filename) {
            Ok(filename) => Ok(filename.trim()),
            Err(e) => Err(AniseError::DAFParserError(format!("{e}"))),
        }
    }

    /// Returns whether this record was just null bytes
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}
