use crate::{errors::AniseError, generated::anise_generated::anise::root_as_anise, prelude::Anise};

/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate memmap2;

use std::convert::TryFrom;

impl<'a> Anise<'a> {}

impl<'a> TryFrom<&'a [u8]> for Anise<'a> {
    type Error = AniseError;

    fn try_from(buf: &'a [u8]) -> Result<Self, Self::Error> {
        match root_as_anise(&buf) {
            Ok(a) => Ok(a),
            Err(e) => Err(AniseError::from(e)),
        }
    }
}

/// file_mmap allows reading a file without memory allocation
#[macro_export]
macro_rules! file_mmap {
    ($filename:ident) => {
        match File::open($filename) {
            Err(e) => Err(AniseError::IOError(e.kind())),
            Ok(file) => unsafe {
                use memmap2::MmapOptions;
                match MmapOptions::new().map(&file) {
                    Err(e) => Err(AniseError::IOUnknownError),
                    Ok(mmap) => Ok(mmap),
                }
            },
        }
    };
}
