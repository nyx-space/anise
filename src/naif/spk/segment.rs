/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::datatype::DataType;
use hifitime::{Epoch, TimeSystem};
use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct SegMetaData {
    pub init_s_past_j2k: f64,
    pub interval_length: usize,
    pub rsize: usize,
    pub num_records_in_seg: usize,
}

impl SegMetaData {
    /// Returns the degree of this segment.
    /// The docs say that the degree has a minus one compared to this formula, but that prevent proper reading of the file.
    pub(crate) fn degree(&self) -> usize {
        (self.rsize - 2) / 3
    }
}

#[derive(Debug)]
pub struct Segment<'a> {
    pub name: &'a str,
    pub start_epoch: Epoch,
    pub end_epoch: Epoch,
    pub(crate) target_id: i32,
    pub(crate) center_id: i32,
    pub(crate) frame_id: i32,
    pub(crate) data_type: DataType,
    pub start_idx: usize,
    pub end_idx: usize,
}

impl<'a> Segment<'a> {}

impl<'a> Default for Segment<'a> {
    fn default() -> Self {
        Self {
            name: "No name",
            start_epoch: Epoch::from_tdb_seconds(0.0),
            end_epoch: Epoch::from_tdb_seconds(0.0),
            target_id: 0,
            center_id: 0,
            frame_id: 0,
            data_type: DataType::ModifiedDifferenceArrays,
            start_idx: 0,
            end_idx: 0,
        }
    }
}

impl<'a> fmt::Display for Segment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Segment `{}` (tgt: {}, ctr: {}, frame: {}) of type {:?} from {} ({}) to {} ({}) [{}..{}]",
            self.name,
            self.target_id,
            self.center_id,
            self.frame_id,
            self.data_type,
            self.start_epoch.as_gregorian_str(TimeSystem::ET),
            self.start_epoch.as_et_duration().in_seconds(),
            self.end_epoch.as_gregorian_str(TimeSystem::ET),
            self.end_epoch.as_et_duration().in_seconds(),
            self.start_idx,
            self.end_idx
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct SegmentExportData {
    pub rcrd_mid_point: f64,
    pub rcrd_radius_s: f64,
    pub x_coeffs: Vec<f64>,
    pub y_coeffs: Vec<f64>,
    pub z_coeffs: Vec<f64>,
    pub vx_coeffs: Vec<f64>,
    pub vy_coeffs: Vec<f64>,
    pub vz_coeffs: Vec<f64>,
}