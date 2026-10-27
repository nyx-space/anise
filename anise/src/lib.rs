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
    use std::borrow::Cow;

    /// Truncates the fractional seconds of a time string to the nanosecond resolution of hifitime.
    ///
    /// CCSDS time codes (and the frozen epoch strings stored in frame kernels) may carry any
    /// number of fractional second digits. hifitime's parsers scale the fraction with
    /// `9 - digits`, so ten or more digits underflow: this panics in overflow-checked builds
    /// and silently zeroes the fraction in release. Drop the digits beyond the ninth before
    /// handing the string to hifitime; the fraction is the run of digits after the `.` that
    /// follows the seconds field (`:SS.`), so a Julian date fraction is left untouched.
    pub(crate) fn truncate_subseconds(epoch_str: &str) -> Cow<'_, str> {
        const MAX_SUBSEC_DIGITS: usize = 9;
        let bytes = epoch_str.as_bytes();
        for (dot, _) in bytes.iter().enumerate().filter(|(_, b)| **b == b'.') {
            // Walk back over the seconds digits and require the `:` separator before them.
            let mut sec_start = dot;
            while sec_start > 0 && bytes[sec_start - 1].is_ascii_digit() {
                sec_start -= 1;
            }
            if sec_start == dot || sec_start == 0 || bytes[sec_start - 1] != b':' {
                continue;
            }
            let frac_digits = bytes[dot + 1..]
                .iter()
                .take_while(|b| b.is_ascii_digit())
                .count();
            if frac_digits <= MAX_SUBSEC_DIGITS {
                return Cow::Borrowed(epoch_str);
            }
            // Both cuts fall on ASCII digits, so they are char boundaries.
            let keep_until = dot + 1 + MAX_SUBSEC_DIGITS;
            let resume_from = dot + 1 + frac_digits;
            let mut truncated = String::with_capacity(epoch_str.len() - (resume_from - keep_until));
            truncated.push_str(&epoch_str[..keep_until]);
            truncated.push_str(&epoch_str[resume_from..]);
            return Cow::Owned(truncated);
        }
        Cow::Borrowed(epoch_str)
    }

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

    #[cfg(test)]
    mod ut_truncate_subseconds {
        use super::{Epoch, FromStr, truncate_subseconds};

        #[test]
        fn leaves_nanosecond_or_coarser_fractions_alone() {
            for s in [
                "2020-06-01T12:00:00 UTC",
                "2020-06-01T12:00:00.1 UTC",
                "2020-06-01T12:00:00.123456789 TDB",
                "1 Jun 2020 12:00:00.000000",
                "JD 2451545.1234567890123 TDB",
                "SEC 1234567890.1234567890 TAI",
                "2020-06-01T12:00:00.",
                "",
            ] {
                assert_eq!(truncate_subseconds(s), s);
            }
        }

        #[test]
        fn truncates_fractions_beyond_nanoseconds() {
            assert_eq!(
                truncate_subseconds("2020-06-01T12:00:00.1234567891 UTC"),
                "2020-06-01T12:00:00.123456789 UTC"
            );
            assert_eq!(
                truncate_subseconds("2020-06-01T12:00:00.0000000000000000001"),
                "2020-06-01T12:00:00.000000000"
            );
            assert_eq!(
                truncate_subseconds("1 Jun 2020 12:00:00.0000000001"),
                "1 Jun 2020 12:00:00.000000000"
            );
            assert_eq!(
                truncate_subseconds("2020-06-01T12:00:5.1234567891 UTC"),
                "2020-06-01T12:00:5.123456789 UTC"
            );
        }

        #[test]
        fn hifitime_parses_the_truncated_string() {
            // hifitime computes `9 - digits` for the fraction, which panics on this input under
            // overflow checks and zeroes the fraction in release.
            let epoch = Epoch::from_str(&truncate_subseconds("2020-06-01T12:00:00.1234567891 UTC"))
                .unwrap();
            assert_eq!(
                epoch,
                Epoch::from_str("2020-06-01T12:00:00.123456789 UTC").unwrap()
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
