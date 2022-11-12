/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use zerocopy::FromBytes;

use crate::naif::daf::{NAIFRecord, NAIFSummaryRecord};

#[derive(Copy, Clone, Debug, Default, FromBytes)]
#[repr(C)]
pub struct BPCSummaryRecord {
    pub start_epoch_et_s: f64,
    pub end_epoch_et_s: f64,
    pub frame_id: i32,
    pub inertial_frame_id: i32,
    pub data_type_i: i32,
    pub start_idx: i32,
    pub end_idx: i32,
}

impl NAIFRecord for BPCSummaryRecord {}

impl NAIFSummaryRecord for BPCSummaryRecord {
    fn start_idx(&self) -> usize {
        self.start_idx as usize
    }

    fn end_idx(&self) -> usize {
        self.end_idx as usize
    }
}
