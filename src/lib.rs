/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

extern crate const_format;
extern crate hifitime;
extern crate log;

pub mod astro;
pub mod cli;
pub mod constants;
pub mod context;
pub mod ephemerides;
pub mod errors;
pub mod frames;
pub mod math;
pub mod naif;
pub mod shapes;

/// Re-export of hifitime
pub mod time {
    pub use hifitime::*;
}

pub mod prelude {
    pub use crate::astro::Aberration;
    pub use crate::context::Context;
    pub use crate::errors::AniseError;
    pub use crate::frames::*;
    pub use crate::math::units::*;
    pub use crate::naif::daf::NAIFSummaryRecord;
    pub use crate::naif::{BPC, SPK};
    pub use crate::time::*;
    pub use std::fs::File;
}

/// Defines the number of bytes in a double (prevents magic numbers)
pub(crate) const DBL_SIZE: usize = 8;

/// Defines the hash used to identify parents.
pub(crate) type NaifId = i32;

/// file_mmap allows reading a file without memory allocation
#[macro_export]
macro_rules! file_mmap {
    ($filename:tt) => {
        match File::open($filename) {
            Err(e) => Err(AniseError::IOError(e.kind())),
            Ok(file) => unsafe {
                use bytes::Bytes;
                use memmap2::MmapOptions;
                match MmapOptions::new().map(&file) {
                    Err(_) => Err(AniseError::IOUnknownError),
                    Ok(mmap) => {
                        // XXX: Find a way to make this work.
                        let bytes: Bytes = (&*mmap).into();
                        Ok(bytes)
                    }
                }
            },
        }
    };
}
