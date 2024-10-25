/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use super::NAIFRecord;

#[derive(IntoBytes, Clone, Copy, Debug, Default, FromBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct SummaryRecord {
    next_record: f64,
    prev_record: f64,
    num_summaries: f64,
}

impl NAIFRecord for SummaryRecord {}

impl SummaryRecord {
    pub fn next_record(&self) -> usize {
        self.next_record as usize
    }

    pub fn prev_record(&self) -> usize {
        self.prev_record as usize
    }

    pub fn num_summaries(&self) -> usize {
        self.num_summaries as usize
    }

    pub fn is_final_record(&self) -> bool {
        self.next_record() == 0
    }
}
