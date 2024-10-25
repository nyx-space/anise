/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::str::Utf8Error;

use snafu::prelude::*;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::naif::Endian;
use log::error;

use super::NAIFRecord;

#[derive(Debug, Snafu, PartialEq)]
#[snafu(visibility(pub(crate)))]
pub enum FileRecordError {
    #[snafu(display("issue: endian of file does not match the endian order of the machine"))]
    WrongEndian,
    #[snafu(display("endian flag or internal filename is not a valid UTF8 string: {source:?}"))]
    ParsingError {
        source: Utf8Error,
    },
    #[snafu(display("endian flag is `{read}` but it should be either `BIG-IEEE` or `LTL-IEEE`"))]
    InvalidEndian {
        read: String,
    },
    UnsupportedIdentifier {
        loci: String,
    },
    #[snafu(display("indicates this is not a SPICE DAF file"))]
    NotDAF,
    #[snafu(display("has no identifier"))]
    NoIdentifier,
    #[snafu(display("is empty (ensure file is valid, e.g. do you need to run git-lfs)"))]
    EmptyRecord,
}

#[derive(Debug, Clone, FromBytes, KnownLayout, Immutable, IntoBytes, PartialEq)]
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

    pub fn identification(&self) -> Result<&str, FileRecordError> {
        let str_locidw =
            core::str::from_utf8(&self.id_str).map_err(|_| FileRecordError::NoIdentifier)?;

        if &str_locidw[0..3] != "DAF" || str_locidw.chars().nth(3) != Some('/') {
            Err(FileRecordError::NotDAF)
        } else {
            let loci = str_locidw[4..].trim();
            match loci {
                "SPK" => Ok("SPK"),
                "PCK" => Ok("PCK"),
                _ => {
                    error!("DAF of type `{}` is not yet supported", &str_locidw[4..]);
                    Err(FileRecordError::UnsupportedIdentifier {
                        loci: loci.to_string(),
                    })
                }
            }
        }
    }

    pub fn endianness(&self) -> Result<Endian, FileRecordError> {
        let str_endianness = core::str::from_utf8(&self.endian_str).context(ParsingSnafu)?;

        let file_endian = if str_endianness == "LTL-IEEE" {
            Endian::Little
        } else if str_endianness == "BIG-IEEE" {
            Endian::Big
        } else {
            return Err(FileRecordError::InvalidEndian {
                read: str_endianness.to_string(),
            });
        };
        if file_endian != Endian::f64_native() || file_endian != Endian::u64_native() {
            Err(FileRecordError::WrongEndian)
        } else {
            Ok(file_endian)
        }
    }

    pub fn internal_filename(&self) -> Result<&str, FileRecordError> {
        Ok(core::str::from_utf8(&self.internal_filename)
            .context(ParsingSnafu)?
            .trim())
    }

    /// Returns whether this record was just null bytes
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}
