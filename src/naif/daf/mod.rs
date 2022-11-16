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
use hifitime::Epoch;
use zerocopy::FromBytes;

pub(crate) const RCRD_LEN: usize = 1024;
pub mod daf;
pub mod recordtypes;

pub use daf::DAF;

use crate::prelude::AniseError;
pub use recordtypes::{DAFFileRecord, DAFSummaryRecord, NameRecord};

pub trait NAIFRecord: FromBytes + Sized + Default {
    const SIZE: usize = core::mem::size_of::<Self>();
}

pub trait NAIFSummaryRecord: NAIFRecord + Copy {
    fn start_index(&self) -> usize;
    fn end_index(&self) -> usize;
    fn start_epoch(&self) -> Epoch;
    fn end_epoch(&self) -> Epoch;
    /// Returns whatever is the ID of this summary record.
    fn id(&self) -> i32;
}

pub trait NAIFDataSet<'a>: Display {
    /// The underlying record representation
    type RecordKind: NAIFDataRecord<'a>;

    /// The state that is returned from an evaluation of this data set
    type StateKind;

    // TODO: Return a result here.
    fn from_slice_f64(slice: &'a [f64]) -> Self;

    fn nth_record(&self, n: usize) -> Self::RecordKind;

    fn evaluate(&self, epoch: Epoch) -> Result<Self::StateKind, AniseError>;
}

pub trait NAIFDataRecord<'a>: Display {
    fn from_slice_f64(slice: &'a [f64]) -> Self;
}
