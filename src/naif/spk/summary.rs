/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt;
use hifitime::Epoch;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

use crate::{
    ephemerides::EphemerisError,
    naif::daf::{NAIFRecord, NAIFSummaryRecord},
    prelude::{Frame, FrameUid},
};

#[derive(Clone, Copy, Debug, Default, AsBytes, FromZeroes, FromBytes, PartialEq)]
#[repr(C)]
pub struct SPKSummaryRecord {
    pub start_epoch_et_s: f64,
    pub end_epoch_et_s: f64,
    pub target_id: i32,
    pub center_id: i32,
    pub frame_id: i32,
    pub data_type_i: i32,
    pub start_idx: i32,
    pub end_idx: i32,
}

impl<'a> SPKSummaryRecord {
    /// Returns the target frame UID of this summary
    pub fn target_frame_uid(&self) -> FrameUid {
        FrameUid {
            ephemeris_id: self.target_id,
            orientation_id: self.frame_id,
        }
    }

    /// Returns the center frame UID of this summary
    pub fn center_frame_uid(&self) -> FrameUid {
        FrameUid {
            ephemeris_id: self.center_id,
            orientation_id: self.frame_id,
        }
    }

    /// Returns the target frame UID of this summary
    pub fn target_frame(&self) -> Frame {
        Frame::from(self.target_frame_uid())
    }

    /// Returns the center frame UID of this summary
    pub fn center_frame(&self) -> Frame {
        Frame::from(self.center_frame_uid())
    }

    #[cfg(feature = "spkezr_validation")]
    pub fn spice_name(&self) -> Result<&'a str, EphemerisError> {
        Self::id_to_spice_name(self.target_id)
    }

    /// Converts the provided ID to its human name.
    /// Only works for the common celestial bodies
    #[cfg(feature = "spkezr_validation")]
    pub fn id_to_spice_name(id: i32) -> Result<&'a str, EphemerisError> {
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
                _ => Err(EphemerisError::IdToName { id }),
            }
        } else if id == 301 {
            Ok("Moon")
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
                _ => Err(EphemerisError::IdToName { id }),
            }
        } else {
            Err(EphemerisError::IdToName { id })
        }
    }

    /// Converts the provided ID to its human name.
    /// Only works for the common celestial bodies
    #[cfg(feature = "spkezr_validation")]
    pub fn spice_name_to_id(name: &'a str) -> Result<i32, EphemerisError> {
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
            _ => Err(EphemerisError::NameToId {
                name: name.to_string(),
            }),
        }
    }
}

impl NAIFRecord for SPKSummaryRecord {}

impl NAIFSummaryRecord for SPKSummaryRecord {
    const NAME: &'static str = "SPKSummaryRecord";
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
        self.target_id
    }

    fn start_epoch_et_s(&self) -> f64 {
        self.start_epoch_et_s
    }

    fn end_epoch_et_s(&self) -> f64 {
        self.end_epoch_et_s
    }
}

impl fmt::Display for SPKSummaryRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SPK Summary for TGT={} CTR={} FRM={} from {:E} to {:E}",
            self.target_id,
            self.center_id,
            self.frame_id,
            self.start_epoch(),
            self.end_epoch()
        )
    }
}
