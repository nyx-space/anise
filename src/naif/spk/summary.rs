/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt;
use hifitime::Epoch;
use zerocopy::FromBytes;

use crate::naif::daf::{NAIFRecord, NAIFSummaryRecord};

#[derive(Clone, Copy, Debug, Default, FromBytes)]
#[repr(C)]
pub struct SPKSummaryRecord {
    pub start_epoch_et_s: f64,
    pub end_epoch_et_s: f64,
    pub target_id: i32,
    pub center_id: i32,
    pub frame_id: i32,
    pub data_type_i: i32,
    pub start_idx: i32,
    pub end_idx: i32,
}

impl SPKSummaryRecord {
    pub fn start_epoch(&self) -> Epoch {
        Epoch::from_et_seconds(self.start_epoch_et_s)
    }
    pub fn end_epoch(&self) -> Epoch {
        Epoch::from_et_seconds(self.end_epoch_et_s)
    }
    pub fn start_index(&self) -> usize {
        self.start_idx as usize
    }
    pub fn end_index(&self) -> usize {
        self.end_idx as usize
    }
}

impl NAIFRecord for SPKSummaryRecord {}

impl NAIFSummaryRecord for SPKSummaryRecord {
    fn start_idx(&self) -> usize {
        self.start_idx as usize
    }

    fn end_idx(&self) -> usize {
        self.end_idx as usize
    }
}

impl fmt::Display for SPKSummaryRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SPK Summary for TGT={} CTR={} FRM={} from {:E} to {:E}",
            self.target_id,
            self.center_id,
            self.frame_id,
            self.start_epoch(),
            self.end_epoch()
        )
    }
}
