/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{
    DAFError, DecodingNameSnafu, NAIFDataSet, NAIFSummaryRecord, NameRecord, RCRD_LEN, daf::DAF,
};
use crate::{
    DBL_SIZE,
    errors::DecodingError,
    naif::daf::{NAIFRecord, SummaryRecord, file_record::FileRecordError},
};
use bytes::BytesMut;
use hifitime::Epoch;
use snafu::ResultExt;
use zerocopy::IntoBytes;

impl<R: NAIFSummaryRecord> DAF<R> {
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
        // NOTE: This function will be rewritten in full after https://github.com/nyx-space/anise/issues/262
        let summaries = self.data_summaries(None)?;
        let this_summary = summaries
            .get(idx)
            .ok_or(DAFError::InvalidIndex { idx, kind: R::NAME })?;

        if self.file_record()?.is_empty() {
            return Err(DAFError::FileRecord {
                kind: R::NAME,
                source: FileRecordError::EmptyRecord,
            });
        }

        let orig_index_end = this_summary.end_index();
        // The summary's start pointer is a 1-based array index, so a zero is
        // malformed; guard the subtraction so a crafted summary errors out
        // instead of underflowing.
        let orig_index_start =
            this_summary
                .start_index()
                .checked_sub(1)
                .ok_or(DAFError::DecodingData {
                    kind: R::NAME,
                    idx,
                    source: DecodingError::InaccessibleBytes {
                        start: 0,
                        end: orig_index_end.saturating_mul(DBL_SIZE),
                        size: self.bytes.len(),
                    },
                })?;
        let orig_data_start = orig_index_start.saturating_mul(DBL_SIZE);
        let orig_data_end = orig_index_end.saturating_mul(DBL_SIZE);

        // The end pointer is read verbatim from the file as well, so it may sit before the start
        // pointer or past the end of the data: `nth_data` rejects both because it reads through
        // `bytes.get`, but the splice below would panic on either.
        if orig_data_end < orig_data_start || orig_data_end > self.bytes.len() {
            return Err(DAFError::DecodingData {
                kind: R::NAME,
                idx,
                source: DecodingError::InaccessibleBytes {
                    start: orig_data_start,
                    end: orig_data_end,
                    size: self.bytes.len(),
                },
            });
        }

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
        // The splice above may have shrunk the data, so the summary record is no longer
        // necessarily within the new bytes.
        let new_size = new_bytes.len();
        let orig_summary_bytes =
            new_bytes
                .get_mut(rcrd_idx..rcrd_idx + RCRD_LEN)
                .ok_or(DAFError::DecodingData {
                    kind: R::NAME,
                    idx,
                    source: DecodingError::InaccessibleBytes {
                        start: rcrd_idx,
                        end: rcrd_idx + RCRD_LEN,
                        size: new_size,
                    },
                })?;
        orig_summary_bytes[SummaryRecord::SIZE..].copy_from_slice(&summary_bytes);

        self.bytes = BytesMut::from_iter(new_bytes);

        Ok(())
    }

    /// Deletes the data for the n-th segment of this DAF file.
    pub fn delete_nth_data(&mut self, idx: usize) -> Result<(), DAFError> {
        // NOTE: This function will be rewritten in full after https://github.com/nyx-space/anise/issues/262
        let summaries = self.data_summaries(None)?;
        let this_summary = summaries
            .get(idx)
            .ok_or(DAFError::InvalidIndex { idx, kind: R::NAME })?;

        if self.file_record()?.is_empty() {
            return Err(DAFError::FileRecord {
                kind: R::NAME,
                source: FileRecordError::EmptyRecord,
            });
        }

        let orig_index_end = this_summary.end_index();
        // The summary's start pointer is a 1-based array index, so a zero is
        // malformed; guard the subtraction so a crafted summary errors out
        // instead of underflowing.
        let orig_index_start =
            this_summary
                .start_index()
                .checked_sub(1)
                .ok_or(DAFError::DecodingData {
                    kind: R::NAME,
                    idx,
                    source: DecodingError::InaccessibleBytes {
                        start: 0,
                        end: orig_index_end.saturating_mul(DBL_SIZE),
                        size: self.bytes.len(),
                    },
                })?;
        let orig_data_start = orig_index_start.saturating_mul(DBL_SIZE);
        let orig_data_end = orig_index_end.saturating_mul(DBL_SIZE);

        // The end pointer is read verbatim from the file as well, so it may sit before the start
        // pointer or past the end of the data: `nth_data` rejects both because it reads through
        // `bytes.get`, but the drain below would panic on either.
        if orig_data_end < orig_data_start || orig_data_end > self.bytes.len() {
            return Err(DAFError::DecodingData {
                kind: R::NAME,
                idx,
                source: DecodingError::InaccessibleBytes {
                    start: orig_data_start,
                    end: orig_data_end,
                    size: self.bytes.len(),
                },
            });
        }

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
        // The drain above may have shrunk the data, so the summary record is no longer
        // necessarily within the new bytes.
        let new_size = new_bytes.len();
        let orig_summary_bytes =
            new_bytes
                .get_mut(rcrd_idx..rcrd_idx + RCRD_LEN)
                .ok_or(DAFError::DecodingData {
                    kind: R::NAME,
                    idx,
                    source: DecodingError::InaccessibleBytes {
                        start: rcrd_idx,
                        end: rcrd_idx + RCRD_LEN,
                        size: new_size,
                    },
                })?;
        orig_summary_bytes[SummaryRecord::SIZE..].copy_from_slice(&summary_bytes);

        self.bytes = BytesMut::from_iter(new_bytes);

        Ok(())
    }
}

#[cfg(test)]
mod mut_daf_ut {
    use crate::naif::daf::summary_record::SummaryRecord;
    use crate::naif::daf::{DAFError, FileRecord};
    use crate::naif::spk::summary::SPKSummaryRecord;
    use bytes::BytesMut;
    use zerocopy::IntoBytes;

    /// Builds a three record SPK whose single summary has the provided data pointers.
    fn craft(start_idx: i32, end_idx: i32) -> BytesMut {
        let mut file_record = FileRecord::spk("TEST");
        file_record.forward = 2;
        file_record.nd = 2;
        file_record.ni = 6;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(file_record.as_bytes());
        bytes.resize(1024, 0);

        // Summary record (record 2): a single summary.
        let summary_header = SummaryRecord {
            next_record: 0.0,
            prev_record: 0.0,
            num_summaries: 1.0,
        };
        let mut summary = SPKSummaryRecord::default();
        summary.data_type_i = 13;
        summary.start_idx = start_idx;
        summary.end_idx = end_idx;

        let mut summary_record = Vec::new();
        summary_record.extend_from_slice(summary_header.as_bytes());
        summary_record.extend_from_slice(summary.as_bytes());
        summary_record.resize(1024, 0);
        bytes.extend(summary_record);

        // Name record (record 3).
        bytes.extend(vec![0u8; 1024]);

        BytesMut::from_iter(bytes)
    }

    #[test]
    fn malformed_data_range_delete() {
        // Each of these summaries is malformed in a different way: the end pointer is before the
        // start pointer, it is negative and so sign extends to a huge index on the cast to usize,
        // and it is past the end of the file. All three used to panic in the drain.
        for (start_idx, end_idx) in [(5, 2), (1, -3), (1, 100_000)] {
            let mut daf = super::DAF::<SPKSummaryRecord>::parse(craft(start_idx, end_idx)).unwrap();
            match daf.delete_nth_data(0) {
                Err(DAFError::DecodingData { .. }) => {}
                Ok(_) => panic!("unexpected success for summary ({start_idx}, {end_idx})"),
                Err(e) => panic!("unexpected error for summary ({start_idx}, {end_idx}): {e}"),
            }
        }
    }

    #[test]
    fn data_range_over_summary_record_delete() {
        // This summary points at 1600 bytes of data starting at the very beginning of the file, so
        // deleting it leaves fewer bytes than the offset of the summary record that then has to be
        // written back, which used to panic.
        let mut daf = super::DAF::<SPKSummaryRecord>::parse(craft(1, 200)).unwrap();
        match daf.delete_nth_data(0) {
            Err(DAFError::DecodingData { .. }) => {}
            Ok(_) => panic!("unexpected success for a segment overlapping the summary record"),
            Err(e) => panic!("unexpected error: {e}"),
        }
    }
}
