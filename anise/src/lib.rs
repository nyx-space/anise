#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

extern crate const_format;
extern crate hifitime;
extern crate log;

pub mod almanac;
pub mod astro;
pub mod constants;
pub mod ephemerides;
pub mod errors;
pub mod frames;
pub mod math;
pub mod naif;
pub mod orientations;
pub mod structure;

/// Re-export of hifitime
pub mod time {
    pub use core::str::FromStr;
    pub use hifitime::*;

    // Stupid but safe algo to find a new frame ID that only collides on the same microsecond
    pub(crate) fn uuid_from_epoch(id: i32, epoch: Epoch) -> i32 {
        let wrapped_days = epoch
            .to_tdb_duration()
            .to_unit(hifitime::Unit::Microsecond)
            .floor()
            .rem_euclid(f64::from(i32::MAX)) as i32;

        (id * 10_000).wrapping_add(wrapped_days)
    }
}

pub mod prelude {
    #[cfg(feature = "metaload")]
    pub use crate::almanac::metaload::MetaAlmanac;

    pub use crate::almanac::Almanac;
    pub use crate::astro::{orbit::Orbit, Aberration};
    pub use crate::errors::InputOutputError;
    pub use crate::frames::*;
    pub use crate::math::units::*;
    pub use crate::naif::daf::NAIFSummaryRecord;
    pub use crate::naif::{BPC, SPK};
    pub use crate::time::*;
    pub use std::fs::File;
}

#[cfg(feature = "python")]
mod py_errors;

/// Defines the number of bytes in a double (prevents magic numbers)
pub(crate) const DBL_SIZE: usize = 8;

/// Defines the hash used to identify parents.
pub(crate) type NaifId = i32;

/// Memory maps a file and **copies** the data on the heap prior to returning a pointer to this heap data.
#[macro_export]
macro_rules! file2heap {
    ($filename:tt) => {
        match std::fs::File::open($filename) {
            Err(e) => Err($crate::errors::InputOutputError::IOError { kind: e.kind() }),
            Ok(file) => unsafe {
                use bytes::Bytes;
                use memmap2::MmapOptions;
                match MmapOptions::new().map(&file) {
                    Err(_) => Err($crate::errors::InputOutputError::IOUnknownError),
                    Ok(mmap) => {
                        let bytes = Bytes::copy_from_slice(&mmap);
                        Ok(bytes)
                    }
                }
            },
        }
    };
}

/// Memory maps a file and **copies** the data on the heap prior to returning a pointer to this heap data.
#[macro_export]
macro_rules! file_mmap {
    ($filename:tt) => {
        match File::open($filename) {
            Err(e) => Err(InputOutputError::IOError { kind: e.kind() }),
            Ok(file) => unsafe {
                use memmap2::MmapOptions;
                match MmapOptions::new().map(&file) {
                    Err(_) => Err(InputOutputError::IOUnknownError),
                    Ok(mmap) => Ok(mmap),
                }
            },
        }
    };
}
