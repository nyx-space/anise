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
    naif::daf::{NAIFRecord, NAIFSummaryRecord},
    orientations::OrientationError,
};
use hifitime::Epoch;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[cfg(feature = "python")]
use pyo3::prelude::*;

use super::daf::DafDataType;

#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.internals"))]
#[derive(Clone, Copy, Debug, Default, IntoBytes, FromBytes, KnownLayout, Immutable, PartialEq)]
#[repr(C)]
pub struct BPCSummaryRecord {
    pub start_epoch_et_s: f64,
    pub end_epoch_et_s: f64,
    pub frame_id: i32,
    pub inertial_frame_id: i32,
    pub data_type_i: i32,
    pub start_idx: i32,
    pub end_idx: i32,
    pub unused: i32,
}

impl BPCSummaryRecord {}

#[cfg(feature = "python")]
#[pymethods]
impl BPCSummaryRecord {
    /// Returns the start epoch of this BPC Summary
    pub fn start_epoch(&self) -> Epoch {
        <Self as NAIFSummaryRecord>::start_epoch(self)
    }

    /// Returns the start epoch of this BPC Summary

    pub fn end_epoch(&self) -> Epoch {
        <Self as NAIFSummaryRecord>::end_epoch(self)
    }
}

impl NAIFRecord for BPCSummaryRecord {}

impl NAIFSummaryRecord for BPCSummaryRecord {
    const NAME: &'static str = "BPCSummaryRecord";

    type Error = OrientationError;

    fn data_type(&self) -> Result<DafDataType, Self::Error> {
        DafDataType::try_from(self.data_type_i).map_err(|source| OrientationError::BPC {
            action: "converting data type from i32",
            source,
        })
    }

    fn start_index(&self) -> usize {
        self.start_idx as usize
    }

    fn end_index(&self) -> usize {
        self.end_idx as usize
    }

    fn start_epoch(&self) -> Epoch {
        Epoch::from_et_seconds(self.start_epoch_et_s)
    }

    fn end_epoch(&self) -> Epoch {
        Epoch::from_et_seconds(self.end_epoch_et_s)
    }

    fn id(&self) -> i32 {
        self.frame_id
    }

    fn start_epoch_et_s(&self) -> f64 {
        self.start_epoch_et_s
    }

    fn end_epoch_et_s(&self) -> f64 {
        self.end_epoch_et_s
    }

    fn update_indexes(&mut self, start: usize, end: usize) {
        self.start_idx = start as i32;
        self.end_idx = end as i32;
    }

    fn update_epochs(&mut self, start_epoch: Epoch, end_epoch: Epoch) {
        self.start_epoch_et_s = start_epoch.to_et_seconds();
        self.end_epoch_et_s = end_epoch.to_et_seconds();
    }
}
