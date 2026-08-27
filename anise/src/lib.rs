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
#[cfg(feature = "analysis")]
pub mod analysis;
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

        id.wrapping_mul(10_000).wrapping_add(wrapped_days)
    }

    #[cfg(test)]
    mod ut_uuid_from_epoch {
        use super::{Epoch, uuid_from_epoch};

        #[test]
        fn large_orientation_id_does_not_overflow() {
            // `orientation_id` is read from a loaded kernel (a PCA body id, a BPC frame id,
            // a location/instrument dataset frame), so it can exceed the ~214_748 point where
            // `id * 10_000` overflows i32. A body-fixed frame for a high-numbered body (e.g.
            // an asteroid at 2_000_001) reaches this through the topocentric / RIC / VNC DCM
            // helpers on an Orbit. The multiply must wrap like the neighbouring add rather than
            // panicking in debug or diverging from the release result.
            let epoch = Epoch::from_tdb_seconds(0.0);
            let big = 2_000_001;
            let offset = uuid_from_epoch(0, epoch); // only the epoch term
            assert_eq!(
                uuid_from_epoch(big, epoch),
                big.wrapping_mul(10_000).wrapping_add(offset)
            );
        }
    }
}

pub mod prelude {
    #[cfg(feature = "metaload")]
    pub use crate::almanac::metaload::MetaAlmanac;

    pub use crate::almanac::Almanac;
    pub use crate::astro::{Aberration, orbit::Orbit};
    pub use crate::errors::InputOutputError;
    pub use crate::frames::*;
    pub use crate::math::units::*;
    pub use crate::naif::daf::NAIFSummaryRecord;
    pub use crate::naif::{BPC, SPK};
    pub use crate::structure::instrument::{FovShape, Instrument};
    pub use crate::time::*;
    pub use std::fs::File;
}

#[cfg(feature = "python")]
mod py_errors;

/// Defines the number of bytes in a double (prevents magic numbers)
pub(crate) const DBL_SIZE: usize = 8;

/// Defines the hash used to identify parents.
pub(crate) type NaifId = i32;

/// Loads a file directly onto the heap, returning a BytesMut
#[macro_export]
macro_rules! file2heap {
    ($filename:tt) => {
        match std::fs::read($filename) {
            Err(e) => Err($crate::errors::InputOutputError::IOError { kind: e.kind() }),
            Ok(bytes) => {
                use bytes::BytesMut;
                Ok(BytesMut::from(&bytes[..]))
            }
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
