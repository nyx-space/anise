/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::NaifId;
use core::fmt::Display;
use hifitime::Epoch;
use snafu::prelude::*;
use zerocopy::{AsBytes, FromBytes};

pub(crate) const RCRD_LEN: usize = 1024;
#[allow(clippy::module_inception)]
pub mod daf;
pub mod file_record;
pub mod name_record;
pub mod summary_record;

pub use daf::DAF;

use crate::{errors::DecodingError, prelude::AniseError};
use core::fmt::Debug;
pub use file_record::FileRecord;
pub use name_record::NameRecord;
pub use summary_record::SummaryRecord;

pub trait NAIFRecord: AsBytes + FromBytes + Sized + Default + Debug {
    const SIZE: usize = core::mem::size_of::<Self>();
}

pub trait NAIFSummaryRecord: NAIFRecord + Copy {
    fn start_index(&self) -> usize;
    fn end_index(&self) -> usize;
    /// Returns the start epoch in high precision Epoch
    fn start_epoch(&self) -> Epoch;
    /// Returns the end epoch in high precision Epoch
    fn end_epoch(&self) -> Epoch;
    /// Returns the start epoch in TDB seconds
    fn start_epoch_et_s(&self) -> f64;
    /// Returns the end epoch in TDB seconds
    fn end_epoch_et_s(&self) -> f64;
    /// Returns whatever is the ID of this summary record.
    fn id(&self) -> i32;
    fn is_empty(&self) -> bool {
        self.start_index() == self.end_index()
    }
    /// Name of this NAIF type
    fn name<'a>() -> &'a str;
}

pub trait NAIFDataSet<'a>: Sized + Display {
    /// The underlying record representation
    type RecordKind: NAIFDataRecord<'a>;

    /// The summary record supported by this data set
    type SummaryKind: NAIFSummaryRecord;

    /// The state that is returned from an evaluation of this data set
    type StateKind;

    /// Builds this dataset given a slice of f64 data
    fn from_slice_f64(slice: &'a [f64]) -> Result<Self, DecodingError>;

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, AniseError>;

    fn evaluate(
        &self,
        epoch: Epoch,
        summary: &Self::SummaryKind,
    ) -> Result<Self::StateKind, AniseError>;

    /// Checks the integrity of this data set, returns an error if the data has issues.
    fn check_integrity(&self) -> Result<(), AniseError>;
}

pub trait NAIFDataRecord<'a>: Display {
    fn from_slice_f64(slice: &'a [f64]) -> Self;
}

/// Errors associated with handling NAIF DAF files
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum DAFError<'a> {
    /// Somehow you've entered code that should not be reachable, please file a bug.
    Unreachable,
    #[snafu(display("No DAF/{kind} data have been loaded"))]
    NoDAFLoaded { kind: &'a str },
    /// While searching for the root of the loaded ephemeris tree, we're recursed more times than allowed.
    MaxRecursionDepth,
    #[snafu(display("DAF/{kind}: summary {id} not present"))]
    SummaryIdError { kind: &'a str, id: NaifId },
    #[snafu(display(
        "DAF/{kind}: summary {id} not present or does not cover requested epoch of {epoch}"
    ))]
    SummaryIdAtEpochError {
        kind: &'a str,
        id: NaifId,
        epoch: Epoch,
    },
    #[snafu(display("DAF/{kind}: summary `{name}` not present"))]
    SummaryNameError { kind: &'a str, name: &'a str },
    #[snafu(display(
        "DAF/{kind}: summary `{name}` not present or does not cover requested epoch of {epoch}"
    ))]
    SummaryNameAtEpochError {
        kind: &'a str,
        name: &'a str,
        epoch: Epoch,
    },
    #[snafu(display("DAF/{kind}: no interpolation data for `{name}` at {epoch}"))]
    InterpolationDataErrorFromName {
        kind: &'a str,
        name: &'a str,
        epoch: Epoch,
    },
    #[snafu(display("DAF/{kind}: no interpolation data for {id} at {epoch}"))]
    InterpolationDataErrorFromId {
        kind: &'a str,
        id: NaifId,
        epoch: Epoch,
    },
    #[snafu(display(
        "DAF/{kind}: file record is empty (ensure file is valid, e.g. do you need to run git-lfs)"
    ))]
    EmptyFileRecord { kind: &'a str },
    #[snafu(display(
        "DAF/{kind}: summary contains no data (start and end index both set to {idx})"
    ))]
    EmptyData { kind: &'a str, idx: usize },
    #[snafu(display("DAF/{kind}: no data record for `{name}`"))]
    NameError { kind: &'a str, name: &'a str },
    #[snafu(display("DAF/{kind}: summary: {source}"))]
    DecodingSummary {
        kind: &'a str,
        source: DecodingError<'a>,
    },
    #[snafu(display("DAF/{kind}: data index {idx}: {source}"))]
    DecodingData {
        kind: &'a str,
        idx: usize,
        source: DecodingError<'a>,
    },
}
