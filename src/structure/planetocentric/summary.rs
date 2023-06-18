/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use zerocopy::{AsBytes, FromBytes};

/// Planetary Constant summary record is an ANISE structure used as a header in the planet constant locator.
#[derive(Clone, Copy, Debug, Default, AsBytes, FromBytes)]
#[repr(C)]
pub struct PCSummaryRecord {
    /// NAIF ID of this object.
    pub object_id: u32,
    /// Representation of the location as a u8
    pub location_repr: u16,
    /// Start index of this constant information. We don't need the end index because the DER structure includes the length.
    pub start_idx: u16,
}

// TODO: import the NameRecord from DAF and use that below.

#[derive(Clone, Copy, Debug)]
pub enum Location {
    /// Data is stored in ANISE planetary constant structure
    APC,
    /// Data is stored in a PCK that is loaded
    PCK,
}

impl From<u16> for Location {
    fn from(value: u16) -> Self {
        if value == 0 {
            Location::APC
        } else {
            Location::PCK
        }
    }
}
