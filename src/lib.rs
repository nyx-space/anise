/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

extern crate der;
extern crate hifitime;

pub use hifitime::Epoch;

pub mod context;
pub mod errors;

pub mod prelude {
    pub use crate::asn1::context::AniseContext;
    pub use crate::errors::AniseError;
    pub use std::fs::File;
}

pub mod asn1;
pub mod naif;

/// Defines the number of bytes in a double (prevents magic numbers)
pub(crate) const DBL_SIZE: usize = 8;

/// file_mmap allows reading a file without memory allocation
#[macro_export]
macro_rules! file_mmap {
    ($filename:ident) => {
        match File::open($filename) {
            Err(e) => Err(AniseError::IOError(e.kind())),
            Ok(file) => unsafe {
                use memmap2::MmapOptions;
                match MmapOptions::new().map(&file) {
                    Err(_) => Err(AniseError::IOUnknownError),
                    Ok(mmap) => Ok(mmap),
                }
            },
        }
    };
}
