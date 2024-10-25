/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::{marker::PhantomData, ops::Deref};

use super::{
    daf::MutDAF, DAFError, DecodingNameSnafu, IOSnafu, NAIFDataSet, NAIFSummaryRecord, NameRecord,
    RCRD_LEN,
};
use crate::{
    errors::DecodingError,
    file2heap,
    naif::daf::{file_record::FileRecordError, NAIFRecord, SummaryRecord},
    DBL_SIZE,
};
use bytes::BytesMut;
use hifitime::Epoch;
use snafu::ResultExt;
use zerocopy::IntoBytes;

impl<R: NAIFSummaryRecord> MutDAF<R> {
    /// Parse the provided bytes as a SPICE Double Array File
    pub fn parse<B: Deref<Target = [u8]>>(bytes: B) -> Result<Self, DAFError> {
        let crc32_checksum = crc32fast::hash(&bytes);
        let mut buf = BytesMut::with_capacity(0);
        buf.extend(bytes.iter());
        let me = Self {
            bytes: buf,
            crc32_checksum,
            _daf_type: PhantomData,
        };
        // Check that these calls will succeed.
        me.file_record()?;
        me.name_record()?;
        Ok(me)
    }

    pub fn load(path: &str) -> Result<Self, DAFError> {
        let bytes = file2heap!(path).context(IOSnafu {
            action: format!("loading {path:?}"),
        })?;

        Self::parse(bytes)
    }

    /// Sets the name record of this mutable DAF file to the one provided as a parameter.
    pub fn set_name_record(&mut self, new_name_record: NameRecord) -> Result<(), DAFError> {
        let rcrd_idx = self.file_record()?.fwrd_idx() * RCRD_LEN;
        let size = self.bytes.len();
        let rcrd_bytes = self
            .bytes
            .get_mut(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or_else(|| DecodingError::InaccessibleBytes {
                start: rcrd_idx,
                end: rcrd_idx + RCRD_LEN,
                size,
            })
            .context(DecodingNameSnafu { kind: R::NAME })?;
        rcrd_bytes.copy_from_slice(new_name_record.as_bytes());
        Ok(())
    }

    /// Sets the data for the n-th segment of this DAF file.
    pub fn set_nth_data<'a, S: NAIFDataSet<'a>>(
        &mut self,
        idx: usize,
        new_data: S,
        new_start_epoch: Epoch,
        new_end_epoch: Epoch,
    ) -> Result<(), DAFError> {
        let summaries = self.data_summaries()?;
        let this_summary = summaries
            .get(idx)
            .ok_or(DAFError::InvalidIndex { idx, kind: R::NAME })?;

        if self.file_record()?.is_empty() {
            return Err(DAFError::FileRecord {
                kind: R::NAME,
                source: FileRecordError::EmptyRecord,
            });
        }

        let orig_index_start = this_summary.start_index() - 1;
        let orig_index_end = this_summary.end_index();
        let orig_data_start = orig_index_start * DBL_SIZE;
        let orig_data_end = orig_index_end * DBL_SIZE;

        let original_size = ((orig_data_end - orig_data_start) / DBL_SIZE) as isize;

        let new_data_bytes = new_data
            .to_f64_daf_vec()
            .or(Err(DAFError::DataBuildError { kind: R::NAME }))?;

        let new_size = new_data_bytes.len() as isize;

        // Size change will be positive if the new data is _smaller_ than the previous one, and vice versa.
        let size_change = original_size - new_size;

        // Update the bytes
        let mut new_bytes = self.bytes.to_vec();
        new_bytes.splice(
            orig_data_start..orig_data_end,
            new_data_bytes.as_bytes().iter().cloned(),
        );

        let mut new_summaries: Vec<R> = summaries.to_vec();
        for (sno, summary) in new_summaries.iter_mut().enumerate() {
            if sno < idx {
                continue;
            } else if sno == idx {
                // Only update the end index for the data we modified
                summary.update_indexes(
                    orig_index_start + 1,
                    (orig_index_end as isize - size_change) as usize,
                );
                summary.update_epochs(new_start_epoch, new_end_epoch);
            } else if !summary.is_empty() {
                // Shift all of the indexes.
                let prev_start = summary.start_index();
                let prev_end = summary.end_index();
                summary.update_indexes(
                    (prev_start as isize - size_change) as usize,
                    (prev_end as isize - size_change) as usize,
                );
            }
        }

        let summary_bytes: Vec<u8> = new_summaries.as_bytes().to_vec();

        let rcrd_idx = (self.file_record()?.fwrd_idx() - 1) * RCRD_LEN;
        // Note: we use copy_from_slice here because we have the guarantee that the summary bytes are the same length as the original version.
        let orig_summary_bytes =
            &mut new_bytes[rcrd_idx..rcrd_idx + RCRD_LEN][SummaryRecord::SIZE..];
        orig_summary_bytes.copy_from_slice(&summary_bytes);

        self.bytes = BytesMut::from_iter(new_bytes);

        Ok(())
    }

    /// Deletes the data for the n-th segment of this DAF file.
    pub fn delete_nth_data(&mut self, idx: usize) -> Result<(), DAFError> {
        let summaries = self.data_summaries()?;
        let this_summary = summaries
            .get(idx)
            .ok_or(DAFError::InvalidIndex { idx, kind: R::NAME })?;

        if self.file_record()?.is_empty() {
            return Err(DAFError::FileRecord {
                kind: R::NAME,
                source: FileRecordError::EmptyRecord,
            });
        }

        let orig_index_start = this_summary.start_index() - 1;
        let orig_index_end = this_summary.end_index();
        let orig_data_start = orig_index_start * DBL_SIZE;
        let orig_data_end = orig_index_end * DBL_SIZE;

        let original_size = ((orig_data_end - orig_data_start) / DBL_SIZE) as isize;

        // Size change will be positive if the new data is _smaller_ than the previous one, and vice versa.
        let size_change = original_size;

        // Update the bytes
        let mut new_bytes = self.bytes.to_vec();
        new_bytes.drain(orig_data_start..orig_data_end);

        let mut new_summaries: Vec<R> = summaries.to_vec();
        for (sno, summary) in new_summaries.iter_mut().enumerate() {
            if sno < idx {
                continue;
            } else if sno == idx {
                // We've removed this data, so clear the summary.
                *summary = R::default()
            } else if !summary.is_empty() {
                // Shift all of the indexes.
                let prev_start = summary.start_index();
                let prev_end = summary.end_index();
                summary.update_indexes(
                    (prev_start as isize - size_change) as usize,
                    (prev_end as isize - size_change) as usize,
                );
            }
        }

        // Remove empty entries from the summary all together
        let cleaned_summaries: Vec<R> = new_summaries
            .iter()
            .filter(|summary| !summary.is_empty())
            .cloned()
            .collect();

        let mut summary_bytes: Vec<u8> = cleaned_summaries.as_bytes().to_vec();
        // We need to pad with zeros all of the summaries we've removed.
        summary_bytes.extend(vec![0x0; 1000 - summary_bytes.len()]);

        let rcrd_idx = (self.file_record()?.fwrd_idx() - 1) * RCRD_LEN;
        // Note: we use copy_from_slice here because we have the guarantee that the summary bytes are the same length as the original version.
        let orig_summary_bytes =
            &mut new_bytes[rcrd_idx..rcrd_idx + RCRD_LEN][SummaryRecord::SIZE..];
        orig_summary_bytes.copy_from_slice(&summary_bytes);

        self.bytes = BytesMut::from_iter(new_bytes);

        Ok(())
    }
}
