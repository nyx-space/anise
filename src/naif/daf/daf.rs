/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

pub use super::recordtypes::{DAFFileRecord, DAFSummaryRecord, NameRecord};
use super::{NAIFDataSet, NAIFRecord, NAIFSummaryRecord};
use crate::{errors::IntegrityErrorKind, prelude::AniseError, DBL_SIZE};
use core::hash::Hash;
use hifitime::Epoch;
use log::{error, trace, warn};
use zerocopy::{FromBytes, LayoutVerified};

pub(crate) const RCRD_LEN: usize = 1024;
#[derive(Clone, Default, Debug)]
pub struct DAF<'a, R: NAIFSummaryRecord> {
    pub file_record: DAFFileRecord,
    pub daf_summary: DAFSummaryRecord,
    pub name_record: NameRecord,
    pub data_summaries: &'a [R],
    /// All of the underlying bytes including what has already been parsed (helps for indexing the data)
    pub bytes: &'a [u8],
    pub crc32_checksum: u32,
}

impl<'a, R: NAIFSummaryRecord> DAF<'a, R> {
    /// Compute the CRC32 of the underlying bytes
    pub fn crc32(&self) -> u32 {
        crc32fast::hash(self.bytes)
    }

    /// Scrubs the data by computing the CRC32 of the bytes and making sure that it still matches the previously known hash
    pub fn scrub(&self) -> Result<(), AniseError> {
        if self.crc32() == self.crc32_checksum {
            Ok(())
        } else {
            // Compiler will optimize the double computation away
            Err(AniseError::IntegrityError(
                IntegrityErrorKind::ChecksumInvalid {
                    expected: self.crc32_checksum,
                    computed: self.crc32(),
                },
            ))
        }
    }

    /// Parse the DAF onl if the CRC32 checksum of the data is valid
    pub fn check_then_parse(bytes: &'a [u8], expected_crc32: u32) -> Result<Self, AniseError> {
        let computed_crc32 = crc32fast::hash(bytes);
        if computed_crc32 == expected_crc32 {
            Self::parse(bytes)
        } else {
            Err(AniseError::IntegrityError(
                IntegrityErrorKind::ChecksumInvalid {
                    expected: expected_crc32,
                    computed: computed_crc32,
                },
            ))
        }
    }

    /// Parse the provided bytes as a SPICE Double Array File
    pub fn parse(bytes: &'a [u8]) -> Result<Self, AniseError> {
        let crc32_checksum = crc32fast::hash(bytes);
        let file_record = DAFFileRecord::read_from(&bytes[..DAFFileRecord::SIZE]).unwrap();

        // Move onto the next record, DAF indexes start at 1 ... =(
        let rcrd_idx = (file_record.fwrd_idx() - 1) * RCRD_LEN;
        let rcrd_bytes = bytes
            .get(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or_else(|| AniseError::MalformedData(file_record.fwrd_idx() + RCRD_LEN))?;

        // TODO: Use the endianness flag
        let daf_summary =
            DAFSummaryRecord::read_from(&rcrd_bytes[..DAFSummaryRecord::SIZE]).unwrap();

        // The summaries are defined in this same record, so let's read them now.
        let data_summaries = match LayoutVerified::new_slice(&rcrd_bytes[DAFSummaryRecord::SIZE..])
        {
            Some(data) => data.into_slice(),
            None => &[R::default(); 0],
        };

        // Move onto the next record.
        let rcrd_idx = file_record.fwrd_idx() * RCRD_LEN;
        let rcrd_bytes = bytes
            .get(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or_else(|| AniseError::MalformedData(file_record.fwrd_idx() + RCRD_LEN))?;
        let name_record = NameRecord::read_from(rcrd_bytes).unwrap();

        Ok(Self {
            file_record,
            daf_summary,
            name_record,
            data_summaries,
            bytes,
            crc32_checksum,
        })
    }

    pub fn nth_summary(&self, n: usize) -> Result<(&str, &R), AniseError> {
        let name = self
            .name_record
            .nth_name(n, self.file_record.summary_size());

        let summary = &self.data_summaries[n];

        Ok((name.trim(), summary))
    }

    /// Returns the summary given the name of the summary record
    pub fn summary_from_name(&self, name: &str) -> Result<(&R, usize), AniseError> {
        let idx = self
            .name_record
            .index_from_name(name, self.file_record.summary_size())?;

        Ok((self.nth_summary(idx)?.1, idx))
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch
    pub fn summary_from_name_at_epoch(
        &self,
        name: &str,
        epoch: Epoch,
    ) -> Result<(&R, usize), AniseError> {
        let (summary, idx) = self.summary_from_name(name)?;

        if epoch >= summary.start_epoch() && epoch <= summary.end_epoch() {
            Ok((summary, idx))
        } else {
            error!("No summary {name} valid at epoch {epoch}");
            Err(AniseError::MissingInterpolationData(epoch))
        }
    }

    /// Returns the summary given the name of the summary record
    pub fn summary_from_id(&self, id: i32) -> Result<(&R, usize), AniseError> {
        for (idx, summary) in self.data_summaries.iter().enumerate() {
            if summary.id() == id {
                return Ok((summary, idx));
            }
        }

        Err(AniseError::ItemNotFound)
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch
    pub fn summary_from_id_at_epoch(
        &self,
        id: i32,
        epoch: Epoch,
    ) -> Result<(&R, usize), AniseError> {
        // NOTE: We iterate through the whole summary because a specific NAIF ID may be repeated in the summary for different valid epochs
        // so we can't just call `summary_from_id`.
        for (idx, summary) in self.data_summaries.iter().enumerate() {
            if summary.id() == id {
                if epoch >= summary.start_epoch() && epoch <= summary.end_epoch() {
                    trace!("Found {id} in position {idx}: {summary:?}");
                    return Ok((summary, idx));
                } else {
                    warn!(
                        "Summary {id} found but only valid from {} to {} (requested {epoch})",
                        summary.start_epoch(),
                        summary.end_epoch()
                    );
                }
            }
        }
        Err(AniseError::MissingInterpolationData(epoch))
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
        trace!("{idx} -> {this_summary:?}");
        if this_summary.is_empty() {
            return Err(AniseError::InternalError(
                crate::errors::InternalErrorKind::Generic,
            ));
        }
        let data: &'a [f64] = LayoutVerified::new_slice(
            self.bytes
                .get(
                    (this_summary.start_index() - 1) * DBL_SIZE
                        ..this_summary.end_index() * DBL_SIZE,
                )
                .ok_or_else(|| AniseError::MalformedData(this_summary.end_index() + RCRD_LEN))?,
        )
        .unwrap()
        .into_slice();
        // Verify that none of the data is invalid once when we load it.
        for val in data {
            if !val.is_finite() {
                return Err(AniseError::IntegrityError(IntegrityErrorKind::SubNormal));
            }
        }
        // Convert it
        S::from_slice_f64(data)
    }

    pub fn comments(&self) -> Result<String, AniseError> {
        // TODO: This can be cleaned up to avoid allocating a string. In my initial tests there were a bunch of additional spaces, so I canceled those changes.
        let mut rslt = String::new();
        // FWRD has the initial record of the summary. So we assume that all records between the second record and that one are comments
        for rid in 1..self.file_record.fwrd_idx() {
            match core::str::from_utf8(
                self.bytes
                    .get(rid * RCRD_LEN..(rid + 1) * RCRD_LEN)
                    .ok_or(AniseError::MalformedData((rid + 1) * RCRD_LEN))?,
            ) {
                Ok(s) => rslt += s.replace('\u{0}', "\n").trim(),
                Err(e) => {
                    let valid_s = core::str::from_utf8(
                        &self.bytes[rid * RCRD_LEN..(rid * RCRD_LEN + e.valid_up_to())],
                    )
                    .unwrap();
                    rslt += valid_s.replace('\u{0}', "\n").trim()
                }
            }
        }

        Ok(rslt)
    }
}

impl<'a, R: NAIFSummaryRecord> Hash for DAF<'a, R> {
    /// Hash will only hash the bytes, nothing else (since these are derived from the bytes anyway).
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}
