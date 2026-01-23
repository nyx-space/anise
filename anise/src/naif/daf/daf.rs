/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::file_record::FileRecordError;
use super::{
    DAFError, DecodingNameSnafu, DecodingSummarySnafu, FileRecordSnafu, NAIFDataSet, NAIFRecord,
    NAIFSummaryRecord,
};
pub use super::{FileRecord, NameRecord, SummaryRecord};
use crate::errors::{DecodingError, InputOutputError};
use crate::naif::daf::DecodingDataSnafu;
use crate::{errors::IntegrityError, DBL_SIZE};
use bytes::{Bytes, BytesMut};
use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;
use core::ops::Deref;
use hifitime::{Epoch, Unit};
use log::{debug, error, trace};
use snafu::ResultExt;

use zerocopy::IntoBytes;
use zerocopy::{FromBytes, Ref};

macro_rules! io_imports {
    () => {
        use std::fs::File;
        use std::io::Result as IoResult;
        use std::io::Write;
        use std::path::Path;
    };
}

io_imports!();

pub(crate) const RCRD_LEN: usize = 1024;
#[derive(Clone, Default, Debug, PartialEq)]
pub struct DAF<R: NAIFSummaryRecord> {
    pub bytes: BytesMut,
    pub crc32: Option<u32>,
    pub _daf_type: PhantomData<R>,
}

impl<R: NAIFSummaryRecord> DAF<R> {
    /// Parse the provided bytes as a SPICE Double Array File.
    ///
    /// # DAF File Structure
    /// A DAF is composed of three main parts:
    /// 1.  **File Record:** The first record (1024 bytes) of the file. It contains metadata about the file, such as the endianness, the number of records, and pointers to the other sections.
    /// 2.  **Comment Area:** An optional area for storing comments.
    /// 3.  **Summary/Name Records and Data:** The remaining records contain the summary records, name records, and the actual data arrays. The file record contains pointers to the start of these sections.
    ///
    /// # Parsing Process
    /// 1.  The entire file is read into a `Bytes` object.
    /// 2.  The CRC32 checksum of the bytes is computed.
    /// 3.  The `file_record` and `name_record` are parsed to ensure the file is a valid DAF.
    pub fn parse<B: Deref<Target = [u8]>>(bytes: B) -> Result<Self, DAFError> {
        let me = Self {
            bytes: BytesMut::from(&bytes[..]),
            crc32: None,
            _daf_type: PhantomData,
        };
        // Check that the file record and name record can be parsed successfully.
        // This validates that the file is a DAF and that the endianness is correct.
        me.file_record()?;
        // Ensure tha twe can parse the first name record.
        me.name_record(None)?;
        Ok(me)
    }

    /// Parse the DAF only if the CRC32 checksum of the data is valid
    pub fn check_then_parse<B: Deref<Target = [u8]>>(
        bytes: B,
        expected: u32,
    ) -> Result<Self, DAFError> {
        let computed = crc32fast::hash(&bytes);
        if computed != expected {
            return Err(DAFError::DAFIntegrity {
                source: IntegrityError::ChecksumInvalid {
                    expected: Some(expected),
                    computed,
                },
            });
        }

        let mut me = Self::parse(bytes)?;
        me.crc32 = Some(computed);
        Ok(me)
    }

    /// Loads the provided path in heap and parse.
    pub fn load(path: &str) -> Result<Self, DAFError> {
        let bytes = match std::fs::read(path) {
            Err(e) => {
                return Err(DAFError::IO {
                    action: format!("loading {path:?}"),
                    source: InputOutputError::IOError { kind: e.kind() },
                })
            }
            Ok(bytes) => BytesMut::from(&bytes[..]),
        };

        Self::parse(bytes)
    }

    /// Parse the provided static byte array as a SPICE Double Array File
    pub fn from_static<B: Deref<Target = [u8]>>(bytes: &'static B) -> Result<Self, DAFError> {
        Self::parse(Bytes::from_static(bytes))
    }

    /// Compute the CRC32 of the underlying bytes
    pub fn crc32(&self) -> u32 {
        crc32fast::hash(&self.bytes)
    }

    /// Sets the CRC32 of this DAF.
    pub fn set_crc32(&mut self) {
        self.crc32 = Some(self.crc32());
    }

    /// Scrubs the data by computing the CRC32 of the bytes and making sure that it still matches the previously known hash
    pub fn scrub(&self) -> Result<(), IntegrityError> {
        if let Some(cur_crc32) = self.crc32 {
            if cur_crc32 == self.crc32() {
                return Ok(());
            }
        }
        // Compiler will optimize the double computation away
        Err(IntegrityError::ChecksumInvalid {
            expected: self.crc32,
            computed: self.crc32(),
        })
    }

    /// Reads and parses the file record from the DAF bytes.
    /// The file record is always the first 1024 bytes of the file.
    pub fn file_record(&self) -> Result<FileRecord, DAFError> {
        let file_record = FileRecord::read_from_bytes(
            self.bytes
                .get(..FileRecord::SIZE)
                .ok_or_else(|| DecodingError::InaccessibleBytes {
                    start: 0,
                    end: FileRecord::SIZE,
                    size: self.bytes.len(),
                })
                .context(DecodingDataSnafu {
                    idx: 0_usize,
                    kind: R::NAME,
                })?,
        )
        .unwrap();
        // Check that the endian-ness is compatible with this platform.
        file_record
            .endianness()
            .context(FileRecordSnafu { kind: R::NAME })?;
        Ok(file_record)
    }

    /// Reads and parses the name record from the DAF bytes.
    /// The file record contains a pointer to the start of the name record.
    pub fn name_record(&self, idx: Option<usize>) -> Result<NameRecord, DAFError> {
        let rcrd_idx = idx.unwrap_or(self.file_record()?.fwrd_idx()) * RCRD_LEN;
        let rcrd_bytes = self
            .bytes
            .get(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or_else(|| DecodingError::InaccessibleBytes {
                start: rcrd_idx,
                end: rcrd_idx + RCRD_LEN,
                size: self.bytes.len(),
            })
            .context(DecodingNameSnafu { kind: R::NAME })?;
        Ok(NameRecord::read_from_bytes(rcrd_bytes).unwrap())
    }

    /// Reads and parses the DAF summary record, starting at the provided idx (1-index!) or at the file record's forward index if no index provided.
    pub fn daf_summary(&self, idx: Option<usize>) -> Result<SummaryRecord, DAFError> {
        let rcrd_idx = (idx.unwrap_or(self.file_record()?.fwrd_idx()) - 1) * RCRD_LEN;
        let rcrd_bytes = self
            .bytes
            .get(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or_else(|| DecodingError::InaccessibleBytes {
                start: rcrd_idx,
                end: rcrd_idx + RCRD_LEN,
                size: self.bytes.len(),
            })
            .context(DecodingSummarySnafu { kind: R::NAME })?;

        SummaryRecord::read_from_bytes(&rcrd_bytes[..SummaryRecord::SIZE])
            .or(Err(DecodingError::Casting))
            .context(DecodingSummarySnafu { kind: R::NAME })
    }

    /// Parses and returns a slice of the data summaries, starting at the provided idx (1-index!) or at the file record's forward index if no index provided.
    /// The summaries are located in the same record as the DAF summary.
    pub fn data_summaries(&self, idx: Option<usize>) -> Result<&[R], DAFError> {
        if self.file_record()?.is_empty() {
            return Err(DAFError::FileRecord {
                kind: R::NAME,
                source: FileRecordError::EmptyRecord,
            });
        }

        // The file record's forward pointer points to the first summary record.
        let rcrd_idx = (idx.unwrap_or(self.file_record()?.fwrd_idx()) - 1) * RCRD_LEN;
        let rcrd_bytes = match self
            .bytes
            .get(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or_else(|| DecodingError::InaccessibleBytes {
                start: rcrd_idx,
                end: rcrd_idx + RCRD_LEN,
                size: self.bytes.len(),
            }) {
            Ok(it) => it,
            Err(source) => {
                return Err(DAFError::DecodingSummary {
                    kind: R::NAME,
                    source,
                })
            }
        };

        // The summaries are located after the main DAF summary record within the same record.
        Ok(
            match Ref::<_, [R]>::from_bytes(&rcrd_bytes[SummaryRecord::SIZE..]) {
                Ok(r) => Ref::into_ref(r),
                Err(_) => &{
                    R::default();
                    [] as [R; 0]
                },
            },
        )
    }

    /// Returns the summary given the name of the summary record
    pub fn summary_from_name(&self, name: &str) -> Result<(&R, Option<usize>, usize), DAFError> {
        // Catch the error until we've reached the last summary.
        let mut idx = None;
        loop {
            let summary = self.daf_summary(idx)?;
            match self
                .name_record(idx)?
                .index_from_name::<R>(name, self.file_record()?.summary_size())
            {
                Ok(summary_idx) => {
                    return Ok((&self.data_summaries(idx)?[summary_idx], idx, summary_idx))
                }
                Err(e) => {
                    if summary.is_final_record() {
                        return Err(e);
                    } else {
                        idx = Some(summary.next_record());
                    }
                }
            }
        }
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch
    pub fn summary_from_name_at_epoch(
        &self,
        name: &str,
        epoch: Epoch,
    ) -> Result<(&R, Option<usize>, usize), DAFError> {
        let (summary, daf_idx, idx) = self.summary_from_name(name)?;

        if epoch >= summary.start_epoch() - Unit::Nanosecond * 100
            && epoch <= summary.end_epoch() + Unit::Nanosecond * 100
        {
            Ok((summary, daf_idx, idx))
        } else {
            error!("No summary {name} valid at epoch {epoch}");
            Err(DAFError::SummaryNameAtEpochError {
                kind: R::NAME,
                name: name.to_string(),
                epoch,
            })
        }
    }

    /// Returns the summary given the id of the summary record
    pub fn summary_from_id(&self, id: i32) -> Result<(&R, Option<usize>, usize), DAFError> {
        let mut idx = None;
        loop {
            for (summary_idx, summary) in self.data_summaries(idx)?.iter().enumerate() {
                if summary.id() == id {
                    return Ok((summary, idx, summary_idx));
                }
            }
            let summary = self.daf_summary(idx)?;
            if summary.is_final_record() {
                break;
            } else {
                idx = Some(summary.next_record());
            }
        }

        Err(DAFError::SummaryIdError { kind: R::NAME, id })
    }

    /// Returns the summary given the id of the summary record if that summary has data defined at the requested epoch
    pub fn summary_from_id_at_epoch(
        &self,
        id: i32,
        epoch: Epoch,
    ) -> Result<(&R, Option<usize>, usize), DAFError> {
        // NOTE: We iterate through the whole summary because a specific NAIF ID may be repeated in the summary for different valid epochs
        // so we can't just call `summary_from_id`.
        let mut idx = None;
        loop {
            for (summary_idx, summary) in self.data_summaries(idx)?.iter().enumerate() {
                if summary.id() == id {
                    if epoch >= summary.start_epoch() - Unit::Nanosecond * 100
                        && epoch <= summary.end_epoch() + Unit::Nanosecond * 100
                    {
                        trace!("Found {id} in position {summary_idx}: {summary:?}");
                        return Ok((summary, idx, summary_idx));
                    } else {
                        debug!(
                        "Summary {id} not valid at {epoch:?} (only from {:?} to {:?}, offset of {} - {})",
                        summary.start_epoch(),
                        summary.end_epoch(),
                        epoch - summary.start_epoch(),
                        summary.end_epoch() - epoch
                    );
                    }
                }
            }
            let summary = self.daf_summary(idx)?;
            if summary.is_final_record() {
                break;
            } else {
                idx = Some(summary.next_record());
            }
        }
        Err(DAFError::InterpolationDataErrorFromId {
            kind: R::NAME,
            id,
            epoch,
        })
    }

    /// Provided a name that is in the summary, return its full data, if name is available.
    pub fn data_from_name<'a, S: NAIFDataSet<'a>>(&'a self, name: &str) -> Result<S, DAFError> {
        // O(N) search through the summaries
        let mut daf_idx = None;
        loop {
            let name_rcrd = self.name_record(daf_idx)?;
            for idx in 0..name_rcrd.num_entries(self.file_record()?.summary_size()) {
                let this_name = name_rcrd.nth_name(idx, self.file_record()?.summary_size());

                if name.trim() == this_name.trim() {
                    // Found it!
                    return self.nth_data(daf_idx, idx);
                }
            }
            let summary = self.daf_summary(daf_idx)?;
            if summary.is_final_record() {
                break;
            } else {
                daf_idx = Some(summary.next_record());
            }
        }
        Err(DAFError::NameError {
            kind: R::NAME,
            name: name.to_string(),
        })
    }

    /// Provided a name that is in the summary, return its full data, if name is available.
    /// This function retrieves the data associated with the nth summary record.
    pub fn nth_data<'a, S: NAIFDataSet<'a>>(
        &'a self,
        daf_idx: Option<usize>,
        idx: usize,
    ) -> Result<S, DAFError> {
        let this_summary =
            self.data_summaries(daf_idx)?
                .get(idx)
                .ok_or(DAFError::InvalidIndex {
                    idx,
                    kind: S::DATASET_NAME,
                })?;

        // Grab the data in native endianness
        if self.file_record()?.is_empty() || this_summary.is_empty() {
            return Err(DAFError::FileRecord {
                kind: R::NAME,
                source: FileRecordError::EmptyRecord,
            });
        }

        let start = (this_summary.start_index() - 1) * DBL_SIZE;
        let end = this_summary.end_index() * DBL_SIZE;
        let data: &[f64] = Ref::into_ref(
            Ref::<&[u8], [f64]>::from_bytes(
                match self
                    .bytes
                    .get(start..end)
                    .ok_or_else(|| DecodingError::InaccessibleBytes {
                        start,
                        end,
                        size: self.bytes.len(),
                    }) {
                    Ok(it) => it,
                    Err(source) => {
                        return Err(DAFError::DecodingData {
                            kind: R::NAME,
                            idx,
                            source,
                        })
                    }
                },
            )
            .unwrap(),
        );

        // Convert it
        S::from_f64_slice(data).context(DecodingDataSnafu { kind: R::NAME, idx })
    }

    pub fn comments(&self) -> Result<Option<String>, DAFError> {
        let mut rslt = String::new();
        // FWRD has the initial record of the summary. So we assume that all records between the second record and that one are comments
        // Note: fwrd_idx is 1-based index of the first summary record. So records < fwrd_idx (starting at 2) are comments.
        // In 0-based indexing (where Rec 1 is index 0), comments are at indices 1 .. fwrd_idx-1.
        // We iterate `rid` from 1 up to fwrd_idx-1.
        // Since `fwrd_idx` returns `usize` (the 1-based index), subtracting 1 gives the count of records before summary.
        // Rec 1 is File Record. Rec 2..fwrd_idx-1 are comments.
        // If fwrd_idx is 2 (minimum), range 1..1 is empty. Correct.
        let end_idx = self.file_record()?.fwrd_idx();
        let loop_end = if end_idx > 1 { end_idx - 1 } else { 1 };

        for rid in 1..loop_end {
            let bytes_slice = match self
                .bytes
                .get(rid * RCRD_LEN..(rid + 1) * RCRD_LEN)
                .ok_or_else(|| DecodingError::InaccessibleBytes {
                    start: rid * RCRD_LEN,
                    end: (rid + 1) * RCRD_LEN,
                    size: self.bytes.len(),
                }) {
                Ok(it) => it,
                Err(source) => {
                    return Err(DAFError::DecodingComments {
                        kind: R::NAME,
                        source,
                    })
                }
            };

            let s = match core::str::from_utf8(bytes_slice) {
                Ok(s) => s,
                Err(e) => {
                    // At this point, we know that the bytes are accessible because the embedded `match`
                    // did not fail, so we can perform a direct access.
                    core::str::from_utf8(&bytes_slice[..e.valid_up_to()]).unwrap()
                }
            };

            // Optimization: Avoid allocating intermediate strings.
            // Identify the start and end of meaningful content (skipping whitespace and nulls).
            // Then append the content, replacing nulls with newlines.

            // Find first non-padding char
            if let Some((start, _)) = s
                .char_indices()
                .find(|(_, c)| !c.is_whitespace() && *c != '\0')
            {
                // Find last non-padding char
                // safe to unwrap because we found at least one char above
                let (end_idx, end_char) = s
                    .char_indices()
                    .rev()
                    .find(|(_, c)| !c.is_whitespace() && *c != '\0')
                    .unwrap();
                let end = end_idx + end_char.len_utf8();

                for c in s[start..end].chars() {
                    if c == '\0' {
                        rslt.push('\n');
                    } else {
                        rslt.push(c);
                    }
                }
            }
        }

        if rslt.is_empty() {
            Ok(None)
        } else {
            Ok(Some(rslt))
        }
    }

    /// Writes the contents of this DAF file to a new location.
    /// WARNING: BUGGY! https://github.com/nyx-space/anise/issues/262
    pub fn persist<P: AsRef<Path>>(&self, path: P) -> IoResult<()> {
        let mut fs = File::create(path)?;

        let mut file_rcrd = Vec::from(self.file_record().unwrap().as_bytes());
        file_rcrd.extend(vec![
            0x0;
            (self.file_record().unwrap().fwrd_idx() - 1) * RCRD_LEN
                - file_rcrd.len()
        ]);
        fs.write_all(&file_rcrd)?;

        let mut daf_summary = Vec::from(self.daf_summary(None).unwrap().as_bytes());
        // Extend with the data summaries
        for data_summary in self.data_summaries(None).unwrap() {
            daf_summary.extend(data_summary.as_bytes());
        }
        // And pad with NULL
        daf_summary.extend(vec![0x0; RCRD_LEN - daf_summary.len()]);
        fs.write_all(&daf_summary)?;

        let mut name_rcrd = Vec::from(self.name_record(None).unwrap().as_bytes());
        name_rcrd.extend(vec![0x0; RCRD_LEN - name_rcrd.len()]);
        fs.write_all(&name_rcrd)?;

        fs.write_all(&self.bytes[self.file_record().unwrap().fwrd_idx() * (2 * RCRD_LEN)..])
    }

    /// Returns an iterator over all summary data blocks.
    pub fn iter_summary_blocks<'a>(&'a self) -> DafBlockIterator<'a, R> {
        // Initialize with the first record pointer
        let start = self.file_record().map(|f| f.fwrd_idx()).unwrap_or(0);
        DafBlockIterator {
            daf: self,
            next_idx: if start > 0 { Some(start) } else { None },
        }
    }
}

impl<R: NAIFSummaryRecord> Hash for DAF<R> {
    /// Hash will only hash the bytes, nothing else (since these are derived from the bytes anyway).
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

pub struct DafBlockIterator<'a, R: NAIFSummaryRecord> {
    daf: &'a DAF<R>,
    next_idx: Option<usize>,
}

impl<'a, R: NAIFSummaryRecord> Iterator for DafBlockIterator<'a, R> {
    // Yields the slice of summaries for the current block
    type Item = Result<&'a [R], DAFError>;

    fn next(&mut self) -> Option<Self::Item> {
        // 1. If we have no next index (and it's not the start), we are done.
        //    (We treat "None" as "Start" initially, so we need a flag or
        //     just rely on the 0 check inside daf_summary if we initialize next_idx correctly).

        // Let's assume initialized with None means "Start".
        // But inside iteration, if we hit 0, we set next_idx to a sentinel (e.g. 0) or handle Option.

        // Simpler logic: The iterator state holds the *current* index to read.
        // If it is 0, we stop.
        let curr = self.next_idx?;

        // 2. Read the header to get the *next* pointer
        // We use your existing daf_summary which handles the None -> First logic
        let summary = match self.daf.daf_summary(Some(curr)) {
            Ok(s) => s,
            Err(e) => {
                self.next_idx = None; // Stop iteration on error
                error!("DAF found to be corrupted when iterating through summary blocks: {e}");
                return Some(Err(e));
            }
        };

        // 3. Read the data
        let data = self.daf.data_summaries(Some(curr));

        // 4. Update state for the NEXT iteration
        if summary.is_final_record() {
            self.next_idx = None;
        } else {
            self.next_idx = Some(summary.next_record());
        }

        // 5. Return the data we found
        Some(data)
    }
}

#[cfg(test)]
mod daf_ut {
    use hifitime::Epoch;

    use crate::{
        errors::IntegrityError,
        file2heap,
        naif::{
            daf::{datatypes::HermiteSetType13, file_record::FileRecordError, DAFError},
            BPC,
        },
        prelude::SPK,
    };

    #[test]
    fn crc32_errors() {
        let mut traj = SPK::load("../data/gmat-hermite.bsp").unwrap();
        let nominal_crc = traj.crc32();

        assert_eq!(
            SPK::check_then_parse(
                file2heap!("../data/gmat-hermite.bsp").unwrap(),
                nominal_crc + 1
            ),
            Err(DAFError::DAFIntegrity {
                source: IntegrityError::ChecksumInvalid {
                    expected: Some(nominal_crc + 1),
                    computed: nominal_crc
                },
            })
        );

        // Change the checksum of the traj and check that scrub fails
        traj.set_crc32();
        *traj.crc32.as_mut().unwrap() = nominal_crc + 1;
        assert_eq!(
            traj.scrub(),
            Err(IntegrityError::ChecksumInvalid {
                expected: Some(nominal_crc + 1),
                computed: nominal_crc
            })
        );
    }

    #[test]
    fn summary_from_name() {
        let epoch = Epoch::now().unwrap();
        let traj = SPK::load("../data/gmat-hermite.bsp").unwrap();

        assert_eq!(
            traj.summary_from_name_at_epoch("name", epoch),
            Err(DAFError::NameError {
                kind: "SPKSummaryRecord",
                name: "name".to_string()
            })
        );

        // SPK_SEGMENT

        assert_eq!(
            traj.summary_from_name_at_epoch("SPK_SEGMENT", epoch),
            Err(DAFError::SummaryNameAtEpochError {
                kind: "SPKSummaryRecord",
                name: "SPK_SEGMENT".to_string(),
                epoch
            })
        );

        if traj.nth_data::<HermiteSetType13>(None, 0).unwrap()
            != traj.data_from_name("SPK_SEGMENT").unwrap()
        {
            // We cannot user assert_eq! because the NAIF Data Set do not (and should not) impl Debug
            // These data sets are the full record!
            panic!("nth data test failed");
        }
    }

    #[test]
    fn load_big_endian() {
        // Ensure this fails
        assert_eq!(
            SPK::load("../data/gmat-hermite-big-endian.bsp"),
            Err(DAFError::FileRecord {
                kind: "SPKSummaryRecord",
                source: FileRecordError::WrongEndian
            })
        );

        // Now ensure the error is correctly printed
        if let Err(e) = BPC::load("../data/gmat-hermite-big-endian.bsp") {
            assert_eq!(
                format!("{e}"),
                "DAF/BPCSummaryRecord: file record issue: endian of file does not match the endian order of the machine".to_string()
            );
        }
    }

    #[test]
    fn test_comments_allocation_and_range() {
        use crate::naif::daf::FileRecord;
        use crate::naif::spk::summary::SPKSummaryRecord;
        use zerocopy::IntoBytes;

        // Construct a DAF file in memory
        // Record 1: File Record
        // Record 2: Comment "Hello World"
        // Record 3: Summary Record (should not be read as comment)

        let mut file_record = FileRecord::spk("TEST");
        file_record.forward = 3; // Summary starts at Record 3
        file_record.nd = 2;
        file_record.ni = 6;

        let mut bytes = Vec::new();
        bytes.extend_from_slice(file_record.as_bytes());
        bytes.resize(1024, 0);

        // Record 2: Comment
        let comment = "Hello World";
        let mut comment_record = vec![0u8; 1024];
        comment_record[..comment.len()].copy_from_slice(comment.as_bytes());
        bytes.extend(comment_record);

        // Record 3: Summary (simulate with some data that looks like text to confuse it, or binary)
        // "BADBEEF"
        let mut summary_record = vec![0u8; 1024];
        let fake_summary = "SHOULD_NOT_SEE_THIS";
        summary_record[..fake_summary.len()].copy_from_slice(fake_summary.as_bytes());
        bytes.extend(summary_record);

        // Add Name Record (Rec 4) just in case
        bytes.extend(vec![0u8; 1024]);

        let daf = super::DAF::<SPKSummaryRecord>::parse(&bytes[..]).unwrap();

        let comments = daf.comments().unwrap();

        if let Some(c) = comments {
            assert_eq!(
                c, "Hello World",
                "Comments included summary record content!"
            );
        } else {
            panic!("No comments found!");
        }
    }
}
