/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{parse_bytes_as, prelude::AniseError};
use std::convert::TryInto;

const RCRD_LEN: usize = 1024;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Endianness {
    Little,
    Big,
}

#[derive(Debug)]
pub struct DAF<'a> {
    pub locidw: &'a str,
    pub ni: i32,
    pub nd: i32,
    pub locifn: &'a str,
    pub fwrd: i32,
    pub bwrd: i32,
    pub freeaddr: i32,
    pub endianness: Endianness,
    pub bytes: &'a [u8],
}

impl<'a> DAF<'a> {
    /// From https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/daf.html#Structure
    pub fn parse(bytes: &'a [u8]) -> Result<Self, AniseError> {
        let locidw = std::str::from_utf8(&bytes[0..8]).or_else(|_| {
            Err(AniseError::InvalidDAF(
                "Could not parse header (first 8 bytes)".to_owned(),
            ))
        })?;

        let daftype: Vec<&str> = locidw.split("/").collect();
        if daftype.len() != 2 {
            return Err(AniseError::InvalidDAF(format!(
                "Malformed header string: `{}`",
                locidw
            )));
        } else if daftype[1].trim() != "SPK" {
            return Err(AniseError::InvalidDAF(format!(
                "Cannot parse a NAIF DAF of type: `{}`",
                locidw
            )));
        }

        // We need to figure out if this file is big or little endian before we can convert some byte arrays into integer
        let str_endianness = std::str::from_utf8(&bytes[88..96]).or_else(|_| {
            Err(AniseError::InvalidDAF(
                "Could not parse endianness".to_owned(),
            ))
        })?;

        let endianness = if str_endianness == "LTL-IEEE" {
            Endianness::Little
        } else if str_endianness == "BIG-IEEE" {
            Endianness::Big
        } else {
            return Err(AniseError::InvalidDAF(format!(
                "Could not understand endianness: `{}`",
                str_endianness
            )));
        };

        let nd = parse_bytes_as!(i32, &bytes[8..12], endianness);
        let ni = parse_bytes_as!(i32, &bytes[12..16], endianness);
        let fwrd = parse_bytes_as!(i32, &bytes[76..80], endianness);
        let bwrd = parse_bytes_as!(i32, &bytes[80..84], endianness);
        let freeaddr = parse_bytes_as!(i32, &bytes[84..88], endianness);

        let locifn = std::str::from_utf8(&bytes[16..76])
            .or_else(|_| Err(AniseError::InvalidDAF("Could not parse locifn".to_owned())))?;

        // Ignore the FTPSTR (seems null in the DE440 and the padding to complete the record).

        Ok(Self {
            locidw,
            locifn,
            nd,
            ni,
            fwrd,
            bwrd,
            freeaddr,
            endianness,
            bytes,
        })
    }
}
