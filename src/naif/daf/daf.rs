/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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
use crate::errors::DecodingError;
use crate::naif::daf::DecodingDataSnafu;
use crate::{errors::IntegrityError, DBL_SIZE};
use bytes::Bytes;
use core::hash::Hash;
use core::ops::Deref;
use hifitime::Epoch;
use log::{error, trace, warn};
use snafu::ResultExt;
use std::fmt::Debug;
use std::marker::PhantomData;

use zerocopy::AsBytes;
use zerocopy::{FromBytes, Ref};

// Thanks ChatGPT for the idea !

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
    pub bytes: Bytes,
    pub crc32_checksum: u32,
    pub _daf_type: PhantomData<R>,
}

impl<R: NAIFSummaryRecord> DAF<R> {
    /// Compute the CRC32 of the underlying bytes
    pub fn crc32(&self) -> u32 {
        crc32fast::hash(&self.bytes)
    }

    /// Scrubs the data by computing the CRC32 of the bytes and making sure that it still matches the previously known hash
    pub fn scrub(&self) -> Result<(), IntegrityError> {
        if self.crc32() == self.crc32_checksum {
            Ok(())
        } else {
            // Compiler will optimize the double computation away
            Err(IntegrityError::ChecksumInvalid {
                expected: self.crc32_checksum,
                computed: self.crc32(),
            })
        }
    }

    /// Parse the DAF only if the CRC32 checksum of the data is valid
    pub fn check_then_parse<B: Deref<Target = [u8]>>(
        bytes: B,
        expected: u32,
    ) -> Result<Self, DAFError> {
        let computed = crc32fast::hash(&bytes);
        if computed != expected {
            return Err(DAFError::DAFIntegrity {
                source: IntegrityError::ChecksumInvalid { expected, computed },
            });
        }

        Self::parse(bytes)
    }

    pub fn load<P: AsRef<Path> + Debug>(path: P) -> Result<Self, DAFError> {
        match File::open(&path) {
            Err(source) => Err(DAFError::IO {
                action: format!("loading {path:?}"),
                source,
            }),
            Ok(file) => unsafe {
                use memmap2::MmapOptions;
                match MmapOptions::new().map(&file) {
                    Err(source) => Err(DAFError::IO {
                        action: format!("mmap of {path:?}"),
                        source,
                    }),
                    Ok(mmap) => {
                        let bytes = Bytes::copy_from_slice(&mmap);
                        Self::parse(bytes)
                    }
                }
            },
        }
    }

    /// Parse the provided static byte array as a SPICE Double Array File
    pub fn from_static<B: Deref<Target = [u8]>>(bytes: &'static B) -> Result<Self, DAFError> {
        Self::parse(Bytes::from_static(bytes))
    }

    /// Parse the provided bytes as a SPICE Double Array File
    pub fn parse<B: Deref<Target = [u8]>>(bytes: B) -> Result<Self, DAFError> {
        let crc32_checksum = crc32fast::hash(&bytes);
        let me = Self {
            bytes: Bytes::copy_from_slice(&bytes),
            crc32_checksum,
            _daf_type: PhantomData,
        };
        // Check that these calls will succeed.
        me.file_record()?;
        me.name_record()?;
        Ok(me)
    }

    pub fn file_record(&self) -> Result<FileRecord, DAFError> {
        let file_record = FileRecord::read_from(&self.bytes[..FileRecord::SIZE]).unwrap();
        // Check that the endian-ness is compatible with this platform.
        file_record
            .endianness()
            .with_context(|_| FileRecordSnafu { kind: R::NAME })?;
        Ok(file_record)
    }

    pub fn name_record(&self) -> Result<NameRecord, DAFError> {
        let rcrd_idx = self.file_record()?.fwrd_idx() * RCRD_LEN;
        let rcrd_bytes = self
            .bytes
            .get(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or_else(|| DecodingError::InaccessibleBytes {
                start: rcrd_idx,
                end: rcrd_idx + RCRD_LEN,
                size: self.bytes.len(),
            })
            .with_context(|_| DecodingNameSnafu { kind: R::NAME })?;
        Ok(NameRecord::read_from(rcrd_bytes).unwrap())
    }

    pub fn daf_summary(&self) -> Result<SummaryRecord, DAFError> {
        let rcrd_idx = (self.file_record()?.fwrd_idx() - 1) * RCRD_LEN;
        let rcrd_bytes = self
            .bytes
            .get(rcrd_idx..rcrd_idx + RCRD_LEN)
            .ok_or_else(|| DecodingError::InaccessibleBytes {
                start: rcrd_idx,
                end: rcrd_idx + RCRD_LEN,
                size: self.bytes.len(),
            })
            .with_context(|_| DecodingSummarySnafu { kind: R::NAME })?;

        SummaryRecord::read_from(&rcrd_bytes[..SummaryRecord::SIZE])
            .ok_or(DecodingError::Casting)
            .with_context(|_| DecodingSummarySnafu { kind: R::NAME })
    }

    /// Parses the data summaries on the fly.
    pub fn data_summaries(&self) -> Result<&[R], DAFError> {
        if self.file_record()?.is_empty() {
            return Err(DAFError::FileRecord {
                kind: R::NAME,
                source: FileRecordError::EmptyRecord,
            });
        }

        // Move onto the next record, DAF indexes start at 1 ... =(
        let rcrd_idx = (self.file_record()?.fwrd_idx() - 1) * RCRD_LEN;
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

        // The summaries are defined in the same record as the DAF summary
        Ok(match Ref::new_slice(&rcrd_bytes[SummaryRecord::SIZE..]) {
            Some(data) => data.into_slice(),
            None => &[R::default(); 0],
        })
    }

    /// Returns the summary given the name of the summary record
    pub fn summary_from_name(&self, name: &str) -> Result<(&R, usize), DAFError> {
        let idx = self
            .name_record()?
            .index_from_name::<R>(name, self.file_record()?.summary_size())?;

        Ok((&self.data_summaries()?[idx], idx))
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch
    pub fn summary_from_name_at_epoch(
        &self,
        name: &str,
        epoch: Epoch,
    ) -> Result<(&R, usize), DAFError> {
        let (summary, idx) = self.summary_from_name(name)?;

        if epoch >= summary.start_epoch() && epoch <= summary.end_epoch() {
            Ok((summary, idx))
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
    pub fn summary_from_id(&self, id: i32) -> Result<(&R, usize), DAFError> {
        for (idx, summary) in self.data_summaries()?.iter().enumerate() {
            if summary.id() == id {
                return Ok((summary, idx));
            }
        }

        Err(DAFError::SummaryIdError { kind: R::NAME, id })
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch
    pub fn summary_from_id_at_epoch(&self, id: i32, epoch: Epoch) -> Result<(&R, usize), DAFError> {
        // NOTE: We iterate through the whole summary because a specific NAIF ID may be repeated in the summary for different valid epochs
        // so we can't just call `summary_from_id`.
        for (idx, summary) in self.data_summaries()?.iter().enumerate() {
            if summary.id() == id {
                if epoch >= summary.start_epoch() && epoch <= summary.end_epoch() {
                    trace!("Found {id} in position {idx}: {summary:?}");
                    return Ok((summary, idx));
                } else {
                    warn!(
                        "Summary {id} not valid at {epoch:?} (only from {:?} to {:?}, offset of {} - {})",
                        summary.start_epoch(),
                        summary.end_epoch(),
                        epoch - summary.start_epoch(),
                        summary.end_epoch() - epoch
                    );
                }
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
        let name_rcrd = self.name_record()?;
        for idx in 0..name_rcrd.num_entries(self.file_record()?.summary_size()) {
            let this_name = name_rcrd.nth_name(idx, self.file_record()?.summary_size());

            if name.trim() == this_name.trim() {
                // Found it!
                return self.nth_data(idx);
            }
        }
        Err(DAFError::NameError {
            kind: R::NAME,
            name: name.to_string(),
        })
    }

    /// Provided a name that is in the summary, return its full data, if name is available.
    pub fn nth_data<'a, S: NAIFDataSet<'a>>(&'a self, idx: usize) -> Result<S, DAFError> {
        let this_summary = &self.data_summaries()?[idx];
        // Grab the data in native endianness (TODO: How to support both big and little endian?)
        trace!("{idx} -> {this_summary:?}");
        if self.file_record()?.is_empty() {
            return Err(DAFError::FileRecord {
                kind: R::NAME,
                source: FileRecordError::EmptyRecord,
            });
        }

        let start = (this_summary.start_index() - 1) * DBL_SIZE;
        let end = this_summary.end_index() * DBL_SIZE;
        let data: &[f64] = Ref::new_slice(
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
        .unwrap()
        .into_slice();

        // Convert it
        S::from_slice_f64(data).with_context(|_| DecodingDataSnafu { kind: R::NAME, idx })
        // S::from_slice_f64(data)
    }

    pub fn comments(&self) -> Result<Option<String>, DAFError> {
        // TODO: This can be cleaned up to avoid allocating a string. In my initial tests there were a bunch of additional spaces, so I canceled those changes.
        let mut rslt = String::new();
        // FWRD has the initial record of the summary. So we assume that all records between the second record and that one are comments
        for rid in 1..self.file_record()?.fwrd_idx() {
            match core::str::from_utf8(
                match self
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
                },
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

        if rslt.is_empty() {
            Ok(None)
        } else {
            Ok(Some(rslt))
        }
    }

    /// Writes the contents of this DAF file to a new location.

    pub fn persist<P: AsRef<Path>>(&self, path: P) -> IoResult<()> {
        let mut fs = File::create(path)?;

        let mut file_rcrd = Vec::from(self.file_record().unwrap().as_bytes());
        file_rcrd.extend(vec![
            0x0;
            (self.file_record().unwrap().fwrd_idx() - 1) * RCRD_LEN
                - file_rcrd.len()
        ]);
        fs.write_all(&file_rcrd)?;

        let mut daf_summary = Vec::from(self.daf_summary().unwrap().as_bytes());
        // Extend with the data summaries
        for data_summary in self.data_summaries().unwrap() {
            daf_summary.extend(data_summary.as_bytes());
        }
        // And pad with NULL
        daf_summary.extend(vec![0x0; RCRD_LEN - daf_summary.len()]);
        fs.write_all(&daf_summary)?;

        let mut name_rcrd = Vec::from(self.name_record().unwrap().as_bytes());
        name_rcrd.extend(vec![0x0; RCRD_LEN - name_rcrd.len()]);
        fs.write_all(&name_rcrd)?;

        fs.write_all(&self.bytes[self.file_record().unwrap().fwrd_idx() * (2 * RCRD_LEN)..])
    }
}

impl<R: NAIFSummaryRecord> Hash for DAF<R> {
    /// Hash will only hash the bytes, nothing else (since these are derived from the bytes anyway).
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

#[cfg(test)]
mod daf_ut {
    use hifitime::Epoch;

    use crate::{
        errors::{InputOutputError, IntegrityError},
        file2heap,
        naif::{daf::DAFError, spk::datatypes::HermiteSetType13, SPK},
    };

    use std::fs::File;

    #[test]
    fn crc32_errors() {
        let mut traj = SPK::load("./data/gmat-hermite.bsp").unwrap();
        let nominal_crc = traj.crc32();

        assert_eq!(
            SPK::check_then_parse(
                file2heap!("./data/gmat-hermite.bsp").unwrap(),
                nominal_crc + 1
            ),
            Err(DAFError::DAFIntegrity {
                source: IntegrityError::ChecksumInvalid {
                    expected: nominal_crc + 1,
                    computed: nominal_crc
                },
            })
        );

        // Change the checksum of the traj and check that scrub fails
        traj.crc32_checksum += 1;
        assert_eq!(
            traj.scrub(),
            Err(IntegrityError::ChecksumInvalid {
                expected: nominal_crc + 1,
                computed: nominal_crc
            })
        );
    }

    #[test]
    fn summary_from_name() {
        let epoch = Epoch::now().unwrap();
        let traj = SPK::load("./data/gmat-hermite.bsp").unwrap();

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

        if traj.nth_data::<HermiteSetType13>(0).unwrap()
            != traj.data_from_name("SPK_SEGMENT").unwrap()
        {
            // We cannot user assert_eq! because the NAIF Data Set do not (and should not) impl Debug
            // These data sets are the full record!
            panic!("nth data test failed");
        }
    }
}
