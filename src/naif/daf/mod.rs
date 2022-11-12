/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt::Display;
use zerocopy::{FromBytes, LayoutVerified};

pub(crate) const RCRD_LEN: usize = 1024;
pub mod recordtypes;

use crate::{prelude::AniseError, DBL_SIZE};
pub use recordtypes::{DAFFileRecord, DAFSummaryRecord, NameRecord};

pub trait NAIFRecord: FromBytes + Sized + Default {
    const SIZE: usize = core::mem::size_of::<Self>();
}

pub trait NAIFSummaryRecord: NAIFRecord + Copy {
    fn start_idx(&self) -> usize;
    fn end_idx(&self) -> usize;
}

pub trait NAIFDataSet<'a>: Display {
    type RecordKind: Display;

    fn from_slice_f64(slice: &'a [f64]) -> Self;

    fn nth_record(&self, n: usize) -> Self::RecordKind;
}

pub trait NAIFDataRecord<'a> {
    fn from_slice_f64(slice: &'a [f64]) -> Self;
}

#[derive(Default, Debug)]
pub struct DAF<'a, R: NAIFSummaryRecord> {
    pub file_record: DAFFileRecord,
    pub daf_summary: DAFSummaryRecord,
    pub name_record: NameRecord,
    pub data_summaries: &'a [R],
    /// All of the underlying bytes including what has already been parsed (helps for indexing the data)
    pub bytes: &'a [u8],
}

impl<'a, R: NAIFSummaryRecord> DAF<'a, R> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, AniseError> {
        let file_record = DAFFileRecord::read_from(&bytes[..DAFFileRecord::SIZE]).unwrap();

        // Move onto the next record, DAF indexes start at 1 ... =(
        let rcrd_idx = (file_record.fwrd_idx() - 1) * RCRD_LEN;
        let rcrd_bytes = bytes
            .get(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or(AniseError::MalformedData(file_record.fwrd_idx() + RCRD_LEN))?;

        // TODO: Use the endianness flag
        let daf_summary =
            DAFSummaryRecord::read_from(&rcrd_bytes[..DAFSummaryRecord::SIZE]).unwrap();

        // The SPK summaries are defined in this same record, so let's read them now.
        let data_summaries = match LayoutVerified::new_slice(&rcrd_bytes[DAFSummaryRecord::SIZE..])
        {
            Some(data) => data.into_slice(),
            None => &[R::default(); 0],
        };

        // Move onto the next record.
        let rcrd_idx = file_record.fwrd_idx() * RCRD_LEN;
        let rcrd_bytes = bytes
            .get(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or(AniseError::MalformedData(file_record.fwrd_idx() + RCRD_LEN))?;
        let name_record = NameRecord::read_from(rcrd_bytes).unwrap();

        Ok(Self {
            file_record,
            daf_summary,
            name_record,
            data_summaries,
            bytes,
        })
    }

    pub fn nth_summary(&self, n: usize) -> Result<(&str, &R), AniseError> {
        let name = self
            .name_record
            .nth_name(n, self.file_record.summary_size());

        let summary = &self.data_summaries[n];

        Ok((name.trim(), summary))
    }

    /// Provided a name that is in the summary, return its full data, if name is available.
    pub fn data_from_name<S: NAIFDataSet<'a>>(&self, name: &str) -> Result<S, AniseError> {
        // O(N) search through the summaries
        for idx in 0..self
            .name_record
            .num_entries(self.file_record.summary_size())
        {
            let (this_name, _) = self.nth_summary(idx)?;
            if name.trim() == this_name.trim() {
                // Found it!
                return self.nth_data(idx);
            }
        }
        Err(AniseError::DAFParserError(format!(
            "Could not find data for {name}"
        )))
    }

    /// Provided a name that is in the summary, return its full data, if name is available.
    pub fn nth_data<S: NAIFDataSet<'a>>(&self, idx: usize) -> Result<S, AniseError> {
        let (_, this_summary) = self.nth_summary(idx)?;
        // Grab the data in native endianness (TODO: How to support both big and little endian?)
        let data: &'a [f64] = LayoutVerified::new_slice(
            self.bytes
                .get((this_summary.start_idx() - 1) * DBL_SIZE..this_summary.end_idx() * DBL_SIZE)
                .ok_or(AniseError::MalformedData(this_summary.end_idx() + RCRD_LEN))?,
        )
        .unwrap()
        .into_slice();
        // Convert it
        return Ok(S::from_slice_f64(data));
    }
}
