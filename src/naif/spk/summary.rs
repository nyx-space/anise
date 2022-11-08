/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::Epoch;
use zerocopy::{FromBytes, LayoutVerified};

use crate::naif::daf::RCRD_LEN;
use crate::naif::recordtypes::{DAFFileRecord, DAFSummaryRecord, NAIFRecord, NameRecord};
use crate::prelude::AniseError;

#[derive(Debug)]
#[repr(C)]
pub struct DAFBytes<'a, R: NAIFRecord> {
    pub file_record: DAFFileRecord,
    pub daf_summary: DAFSummaryRecord,
    pub name_record: NameRecord,
    pub data_summaries: &'a [R],
}

#[derive(Debug, Default, FromBytes)]
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

#[derive(Debug, Default, FromBytes)]
#[repr(C)]
pub struct PCKSummaryRecord {
    pub start_epoch_et_s: f64,
    pub end_epoch_et_s: f64,
    pub frame_id: i32,
    pub inertial_frame_id: i32,
    pub data_type_i: i32,
    pub start_idx: i32,
    pub end_idx: i32,
}

impl NAIFRecord for PCKSummaryRecord {}

impl<'a, R: NAIFRecord> DAFBytes<'a, R> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, AniseError> {
        let file_rcrd = DAFFileRecord::read_from(&bytes[..DAFFileRecord::SIZE]).unwrap();

        // Move onto the next record.
        // let mut rcrd_idx = (file_rcrd.fwrd_idx() - 1) * RCRD_LEN;
        let rcrd_idx = file_rcrd.fwrd_idx() * RCRD_LEN - RCRD_LEN;
        let rcrd_bytes = bytes
            .get(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or(AniseError::MalformedData(file_rcrd.fwrd_idx() + RCRD_LEN))?;

        let daf_summary =
            DAFSummaryRecord::read_from(&rcrd_bytes[0..DAFSummaryRecord::SIZE]).unwrap();

        // The SPK summaries are defined in this same record, so let's read them now.
        let data_summaries = match LayoutVerified::new_slice(&rcrd_bytes[DAFSummaryRecord::SIZE..])
        {
            Some(data) => data.into_slice(),
            None => &[R::default(); 0],
        };

        // Move onto the next record.
        // rcrd_idx = (file_rcrd.fwrd_idx() - 0) * RCRD_LEN;
        let rcrd_idx = (file_rcrd.fwrd_idx() + 1) * RCRD_LEN - RCRD_LEN;
        let rcrd_bytes = bytes
            .get(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or(AniseError::MalformedData(file_rcrd.fwrd_idx() + RCRD_LEN))?;
        let name_record = NameRecord::read_from(rcrd_bytes).unwrap();

        Ok(Self {
            file_record: file_rcrd,
            daf_summary,
            name_record,
            data_summaries,
        })
    }

    pub fn nth_summary(&self, n: usize) -> Result<(&str, &R), AniseError> {
        let name = self
            .name_record
            .nth_name(n, self.file_record.ni(), self.file_record.nd());

        let summary = &self.data_summaries[n];

        Ok((name, summary))
    }
}
