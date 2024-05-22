/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

pub mod daf;

pub mod kpl;
pub mod pck;
pub mod spk;

pub mod pretty_print;

use self::{
    daf::{daf::MutDAF, DAF},
    pck::BPCSummaryRecord,
    spk::summary::SPKSummaryRecord,
};

/// Spacecraft Planetary Kernel
pub type SPK = DAF<SPKSummaryRecord>;
/// Spacecraft Planetary Kernel, mutable, for editing DAF/SPK files
pub type MutSPK = MutDAF<SPKSummaryRecord>;
/// Binary Planetary Constant
pub type BPC = DAF<BPCSummaryRecord>;
/// Binary Planetary Constant, mutable, for editing DAF/PCK files
pub type MutBPC = MutDAF<BPCSummaryRecord>;

#[macro_export]
macro_rules! parse_bytes_as {
    ($type:ident, $input:expr, $order:expr) => {{
        let (int_bytes, _) = $input.split_at(std::mem::size_of::<$type>());

        match $order {
            Endian::Little => $type::from_le_bytes(int_bytes.try_into().unwrap()),
            Endian::Big => $type::from_be_bytes(int_bytes.try_into().unwrap()),
        }
    }};
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Endian {
    Little,
    Big,
}

impl Endian {
    /// Returns the endianness of the platform we're running on for an f64.
    /// This isn't const because f64 comparisons cannot be const yet
    fn f64_native() -> Self {
        let truth: f64 = 0.12345678;
        if (f64::from_ne_bytes(truth.to_be_bytes()) - truth).abs() < f64::EPSILON {
            Self::Big
        } else {
            Self::Little
        }
    }

    /// Returns the endianness of the platform we're running on for an f64.
    const fn u64_native() -> Self {
        let truth: u32 = 0x12345678;
        if u32::from_ne_bytes(truth.to_be_bytes()) == truth {
            Self::Big
        } else {
            Self::Little
        }
    }
}
