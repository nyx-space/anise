/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::prelude::AniseError;

use super::datatype::DataType;
use hifitime::Epoch;
use log::error;
use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct SegMetaData {
    pub init_s_past_j2k: f64,
    pub interval_length_s: f64,
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

impl<'a> Segment<'a> {
    /// Converts the provided ID to its human name.
    /// Only works for the common celestial bodies
    pub(crate) fn id_to_human_name(id: i32) -> Result<&'a str, AniseError> {
        if id % 100 == 99 {
            // This is the planet itself
            match id / 100 {
                1 => Ok("Mercury"),
                2 => Ok("Venus"),
                3 => Ok("Earth"),
                4 => Ok("Mars"),
                5 => Ok("Jupiter"),
                6 => Ok("Saturn"),
                7 => Ok("Uranus"),
                8 => Ok("Neptune"),
                9 => Ok("Pluto"),
                _ => Err(AniseError::DAFParserError(format!(
                    "Human name unknown for {id}"
                ))),
            }
        } else if id == 301 {
            Ok("Luna")
        } else if id <= 10 {
            // This is the barycenter
            match id {
                0 => Ok("Solar System Barycenter"),
                1 => Ok("Mercury"),
                2 => Ok("Venus"),
                3 => Ok("Earth-Moon Barycenter"),
                4 => Ok("Mars Barycenter"),
                5 => Ok("Jupiter Barycenter"),
                6 => Ok("Saturn Barycenter"),
                7 => Ok("Uranus Barycenter"),
                8 => Ok("Neptune Barycenter"),
                9 => Ok("Pluto Barycenter"),
                10 => Ok("Sun"),
                _ => Err(AniseError::DAFParserError(format!(
                    "Human name unknown for barycenter {id}"
                ))),
            }
        } else {
            panic!("Human name unknown for {id}");
        }
    }

    /// Converts the provided ID to its human name.
    /// Only works for the common celestial bodies
    pub(crate) fn human_name_to_id(name: &'a str) -> Result<i32, AniseError> {
        match name {
            "Mercury" => Ok(1),
            "Venus" => Ok(2),
            "Earth" => Ok(399),
            "Mars" => Ok(499),
            "Jupiter" => Ok(599),
            "Saturn" => Ok(699),
            "Uranus" => Ok(799),
            "Neptune" => Ok(899),
            "Pluto" => Ok(999),
            "Luna" => Ok(301),
            "Sun" => Ok(10),
            "Earth-Moon Barycenter" => Ok(3),
            "Mars Barycenter" => Ok(4),
            "Jupiter Barycenter" => Ok(5),
            "Saturn Barycenter" => Ok(6),
            "Uranus Barycenter" => Ok(7),
            "Neptune Barycenter" => Ok(8),
            "Pluto Barycenter" => Ok(9),
            _ => {
                error!("[human_name_to_id] unknown NAIF ID for `{name}`");
                todo!()
            }
        }
    }

    /// Returns the human name of this segment if it can be guessed, else the standard name.
    ///
    /// # Returned value
    /// 1. Typically, this will return the name of the celestial body
    /// 2. The name is appended with "Barycenter" if the celestial object is know to have moons
    ///
    /// # Limitations
    /// 0. In BSP files, the name is stored as a comment and is unstructured. So it's hard to copy those. (Help needed)
    /// 1. One limitation of this approach is that given file may only contain one "Earth"
    /// 2. Another limitation is that this code does not know all of the possible moons in the whole solar system.
    pub(crate) fn human_name(&self) -> &'a str {
        if self.name.starts_with("DE-") {
            match Self::id_to_human_name(self.target_id) {
                Ok(name) => name,
                Err(e) => {
                    error!("{}", e);
                    panic!("Human name unknown for {self}")
                }
            }
        } else {
            self.name
        }
    }
}

impl<'a> Default for Segment<'a> {
    fn default() -> Self {
        Self {
            name: "No name",
            start_epoch: Epoch::from_et_seconds(0.0),
            end_epoch: Epoch::from_et_seconds(0.0),
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
            "Segment `{}` (tgt: {}, ctr: {}, frame: {}) of type {:?} from {:E} ({}) to {:E} ({}) [{}..{}]",
            self.name,
            self.target_id,
            self.center_id,
            self.frame_id,
            self.data_type,
            self.start_epoch,
            self.start_epoch.to_et_duration().to_seconds(),
            self.end_epoch,
            self.end_epoch.to_et_duration().to_seconds(),
            self.start_idx,
            self.end_idx
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct Record {
    pub rcrd_mid_point: f64,
    pub rcrd_radius_s: f64,
    pub x_coeffs: Vec<f64>,
    pub y_coeffs: Vec<f64>,
    pub z_coeffs: Vec<f64>,
    pub vx_coeffs: Vec<f64>,
    pub vy_coeffs: Vec<f64>,
    pub vz_coeffs: Vec<f64>,
}
