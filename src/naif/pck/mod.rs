/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{
    naif::daf::{NAIFRecord, NAIFSummaryRecord},
    orientations::OrientationError,
};
use hifitime::Epoch;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

use super::daf::DafDataType;

#[derive(Clone, Copy, Debug, Default, AsBytes, FromZeroes, FromBytes)]
#[repr(C)]
pub struct BPCSummaryRecord {
    pub start_epoch_et_s: f64,
    pub end_epoch_et_s: f64,
    pub frame_id: i32,
    pub inertial_frame_id: i32,
    pub data_type_i: i32,
    pub start_idx: i32,
    pub end_idx: i32,
    pub unused: i32,
}

impl BPCSummaryRecord {
    pub fn data_type(&self) -> Result<DafDataType, OrientationError> {
        DafDataType::try_from(self.data_type_i).map_err(|source| OrientationError::BPC {
            action: "converting data type from i32",
            source,
        })
    }
}

impl NAIFRecord for BPCSummaryRecord {}

impl NAIFSummaryRecord for BPCSummaryRecord {
    const NAME: &'static str = "BPCSummaryRecord";

    fn start_index(&self) -> usize {
        self.start_idx as usize
    }

    fn end_index(&self) -> usize {
        self.end_idx as usize
    }

    fn start_epoch(&self) -> Epoch {
        Epoch::from_et_seconds(self.start_epoch_et_s)
    }

    fn end_epoch(&self) -> Epoch {
        Epoch::from_et_seconds(self.end_epoch_et_s)
    }

    fn id(&self) -> i32 {
        self.frame_id
    }

    fn start_epoch_et_s(&self) -> f64 {
        self.start_epoch_et_s
    }

    fn end_epoch_et_s(&self) -> f64 {
        self.end_epoch_et_s
    }
}
