/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{
    errors::IntegrityError, math::interpolation::InterpolationError, prelude::InputOutputError,
    NaifId,
};
use core::fmt::Display;
use hifitime::Epoch;
use snafu::prelude::*;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub(crate) const RCRD_LEN: usize = 1024;
#[allow(clippy::module_inception)]
pub mod daf;
mod data_types;
pub mod mut_daf;
pub use data_types::DataType as DafDataType;
pub mod file_record;
pub mod name_record;
pub mod summary_record;
// Defines the supported data types
pub mod datatypes;

pub use daf::DAF;

use crate::errors::DecodingError;
use core::fmt::Debug;
pub use file_record::FileRecord;
pub use name_record::NameRecord;
pub use summary_record::SummaryRecord;

use self::file_record::FileRecordError;

pub trait NAIFRecord:
    IntoBytes + FromBytes + Sized + Default + Debug + Immutable + KnownLayout
{
    const SIZE: usize = core::mem::size_of::<Self>();
}

pub trait NAIFSummaryRecord: NAIFRecord + Copy + Immutable + KnownLayout {
    type Error: 'static + std::error::Error;

    fn start_index(&self) -> usize;
    fn data_type(&self) -> Result<DafDataType, Self::Error>;
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
    /// Updates the indexes of this summary (used when modifying a DAF).
    fn update_indexes(&mut self, start: usize, end: usize);
    /// Updates the epochs of this summary (used when modifying a DAF).
    fn update_epochs(&mut self, start_epoch: Epoch, end_epoch: Epoch);
    /// Name of this NAIF type
    const NAME: &'static str;
}

pub trait NAIFDataSet<'a>: Sized + Display + PartialEq {
    /// The underlying record representation
    type RecordKind: NAIFDataRecord<'a>;

    /// The state that is returned from an evaluation of this data set
    type StateKind;

    /// The name of this data set, used in errors
    const DATASET_NAME: &'static str;

    /// Builds this dataset given a slice of f64 data
    fn from_f64_slice(slice: &'a [f64]) -> Result<Self, DecodingError>;

    /// Builds the DAF array representing this data
    fn to_f64_daf_vec(&self) -> Result<Vec<f64>, InterpolationError> {
        Err(InterpolationError::UnsupportedOperation {
            kind: Self::DATASET_NAME,
            op: "building new DAF",
        })
    }

    fn nth_record(&self, n: usize) -> Result<Self::RecordKind, DecodingError>;

    fn evaluate<S: NAIFSummaryRecord>(
        &self,
        epoch: Epoch,
        summary: &S,
    ) -> Result<Self::StateKind, InterpolationError>;

    /// Checks the integrity of this data set, returns an error if the data has issues.
    fn check_integrity(&self) -> Result<(), IntegrityError>;

    /// Returns a copy of Self where the data corresponds to the start and end times provided.
    /// If either is set to None, then that data will not be modified.
    ///
    /// # Rounding of truncation
    /// The exact new start and new end times may not match the exact data. This function only truncates
    /// the available data and does not make any modifications to it (i.e. it does not recompute the interpolation coefficients).
    ///
    /// # Error
    /// This function will return an error if the new start is before the current start or if the new end is after the current end of the data.
    /// That's because it can't make up new data.
    fn truncate<S: NAIFSummaryRecord>(
        self,
        _summary: &S,
        _new_start: Option<Epoch>,
        _new_end: Option<Epoch>,
    ) -> Result<Self, InterpolationError> {
        Err(InterpolationError::UnsupportedOperation {
            kind: Self::DATASET_NAME,
            op: "truncation",
        })
    }
}

pub trait NAIFDataRecord<'a>: Display {
    fn from_slice_f64(slice: &'a [f64]) -> Self;
}

/// Errors associated with handling NAIF DAF files
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum DAFError {
    #[snafu(display("No DAF/{kind} data have been loaded"))]
    NoDAFLoaded { kind: &'static str },
    /// While searching for the root of the loaded ephemeris tree, we're recursed more times than allowed.
    MaxRecursionDepth,
    #[snafu(display("DAF/{kind}: summary {id} not present"))]
    SummaryIdError { kind: &'static str, id: NaifId },
    #[snafu(display(
        "DAF/{kind}: summary {id} not present or does not cover requested epoch of {epoch}"
    ))]
    SummaryIdAtEpochError {
        kind: &'static str,
        id: NaifId,
        epoch: Epoch,
    },
    #[snafu(display("DAF/{kind}: summary `{name}` not present"))]
    SummaryNameError { kind: &'static str, name: String },
    #[snafu(display(
        "DAF/{kind}: summary `{name}` not present or does not cover requested epoch of {epoch}"
    ))]
    SummaryNameAtEpochError {
        kind: &'static str,
        name: String,
        epoch: Epoch,
    },
    #[snafu(display("DAF/{kind}: no interpolation data for `{name}` at {epoch}"))]
    InterpolationDataErrorFromName {
        kind: &'static str,
        name: String,
        epoch: Epoch,
    },
    #[snafu(display("DAF/{kind}: no interpolation data for {id} at {epoch}"))]
    InterpolationDataErrorFromId {
        kind: &'static str,
        id: NaifId,
        epoch: Epoch,
    },
    #[snafu(display("DAF/{kind}: file record {source}"))]
    FileRecord {
        kind: &'static str,
        #[snafu(backtrace)]
        source: FileRecordError,
    },
    #[snafu(display(
        "DAF/{kind}: summary contains no data (start and end index both set to {idx})"
    ))]
    EmptySummary { kind: &'static str, idx: usize },
    #[snafu(display("DAF/{kind}: no data record for `{name}`"))]
    NameError { kind: &'static str, name: String },
    #[snafu(display("DAF/{kind}: summary: {source}"))]
    DecodingSummary {
        kind: &'static str,
        #[snafu(backtrace)]
        source: DecodingError,
    },
    #[snafu(display("DAF/{kind}: comments: {source}"))]
    DecodingComments {
        kind: &'static str,
        #[snafu(backtrace)]
        source: DecodingError,
    },
    #[snafu(display("DAF/{kind}: name: {source}"))]
    DecodingName {
        kind: &'static str,
        #[snafu(backtrace)]
        source: DecodingError,
    },
    #[snafu(display("DAF/{kind}: data index {idx}: {source}"))]
    DecodingData {
        kind: &'static str,
        idx: usize,
        #[snafu(backtrace)]
        source: DecodingError,
    },
    DAFIntegrity {
        #[snafu(backtrace)]
        source: IntegrityError,
    },
    #[snafu(display("while {action} encountered input/output error {source}"))]
    IO {
        action: String,
        source: InputOutputError,
    },
    #[snafu(display("data type {id}: {kind} (corrupted data?)"))]
    Datatype { id: i32, kind: &'static str },
    #[snafu(display("{dtype:?} not supported for {kind}"))]
    UnsupportedDatatype {
        dtype: DafDataType,
        kind: &'static str,
    },
    #[snafu(display("DAF/{kind}: data index {idx} is invalid"))]
    InvalidIndex { kind: &'static str, idx: usize },
    #[snafu(display("could not build data vector of type DAF/{kind}"))]
    DataBuildError { kind: &'static str },
}

// Manual implementation of PartialEq because IOError does not derive it, sadly.
impl PartialEq for DAFError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NoDAFLoaded { kind: l_kind }, Self::NoDAFLoaded { kind: r_kind }) => {
                l_kind == r_kind
            }
            (
                Self::SummaryIdError {
                    kind: l_kind,
                    id: l_id,
                },
                Self::SummaryIdError {
                    kind: r_kind,
                    id: r_id,
                },
            ) => l_kind == r_kind && l_id == r_id,
            (
                Self::SummaryIdAtEpochError {
                    kind: l_kind,
                    id: l_id,
                    epoch: l_epoch,
                },
                Self::SummaryIdAtEpochError {
                    kind: r_kind,
                    id: r_id,
                    epoch: r_epoch,
                },
            ) => l_kind == r_kind && l_id == r_id && l_epoch == r_epoch,
            (
                Self::SummaryNameError {
                    kind: l_kind,
                    name: l_name,
                },
                Self::SummaryNameError {
                    kind: r_kind,
                    name: r_name,
                },
            ) => l_kind == r_kind && l_name == r_name,
            (
                Self::SummaryNameAtEpochError {
                    kind: l_kind,
                    name: l_name,
                    epoch: l_epoch,
                },
                Self::SummaryNameAtEpochError {
                    kind: r_kind,
                    name: r_name,
                    epoch: r_epoch,
                },
            ) => l_kind == r_kind && l_name == r_name && l_epoch == r_epoch,
            (
                Self::InterpolationDataErrorFromName {
                    kind: l_kind,
                    name: l_name,
                    epoch: l_epoch,
                },
                Self::InterpolationDataErrorFromName {
                    kind: r_kind,
                    name: r_name,
                    epoch: r_epoch,
                },
            ) => l_kind == r_kind && l_name == r_name && l_epoch == r_epoch,
            (
                Self::InterpolationDataErrorFromId {
                    kind: l_kind,
                    id: l_id,
                    epoch: l_epoch,
                },
                Self::InterpolationDataErrorFromId {
                    kind: r_kind,
                    id: r_id,
                    epoch: r_epoch,
                },
            ) => l_kind == r_kind && l_id == r_id && l_epoch == r_epoch,
            (
                Self::FileRecord {
                    kind: l_kind,
                    source: l_source,
                },
                Self::FileRecord {
                    kind: r_kind,
                    source: r_source,
                },
            ) => l_kind == r_kind && l_source == r_source,
            (
                Self::EmptySummary {
                    kind: l_kind,
                    idx: l_idx,
                },
                Self::EmptySummary {
                    kind: r_kind,
                    idx: r_idx,
                },
            ) => l_kind == r_kind && l_idx == r_idx,
            (
                Self::NameError {
                    kind: l_kind,
                    name: l_name,
                },
                Self::NameError {
                    kind: r_kind,
                    name: r_name,
                },
            ) => l_kind == r_kind && l_name == r_name,
            (
                Self::DecodingSummary {
                    kind: l_kind,
                    source: l_source,
                },
                Self::DecodingSummary {
                    kind: r_kind,
                    source: r_source,
                },
            ) => l_kind == r_kind && l_source == r_source,
            (
                Self::DecodingComments {
                    kind: l_kind,
                    source: l_source,
                },
                Self::DecodingComments {
                    kind: r_kind,
                    source: r_source,
                },
            ) => l_kind == r_kind && l_source == r_source,
            (
                Self::DecodingName {
                    kind: l_kind,
                    source: l_source,
                },
                Self::DecodingName {
                    kind: r_kind,
                    source: r_source,
                },
            ) => l_kind == r_kind && l_source == r_source,
            (
                Self::DecodingData {
                    kind: l_kind,
                    idx: l_idx,
                    source: l_source,
                },
                Self::DecodingData {
                    kind: r_kind,
                    idx: r_idx,
                    source: r_source,
                },
            ) => l_kind == r_kind && l_idx == r_idx && l_source == r_source,
            (Self::DAFIntegrity { source: l_source }, Self::DAFIntegrity { source: r_source }) => {
                l_source == r_source
            }
            (
                Self::IO {
                    action: l_action,
                    source: _l_source,
                },
                Self::IO {
                    action: r_action,
                    source: _r_source,
                },
            ) => l_action == r_action,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
