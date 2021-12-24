use crate::prelude::AniseError;

/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub enum Endianness {
    Little,
    Big,
}

pub struct DAF<'a> {
    pub locidw: String,
    pub ni: i32,
    pub nd: i32,
    pub locifn: String,
    pub fwrd: i32,
    pub bwrd: i32,
    pub free: i32,
    pub locfmt: Endianness,
    pub ftpstr: String,
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

        dbg!(locidw);

        todo!()
    }
}
