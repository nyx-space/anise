/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::fmt;
use core::fmt::Debug;

use crate::constants::celestial_objects::celestial_name_from_id;
use crate::constants::orientations::{orientation_name_from_id, J2000};
use crate::NaifId;

/// Defines a Frame kind, allows for compile time checking of operations.
pub trait FrameTrait: Copy + Debug + PartialEq + Send + Sync {
    /// Returns the ephemeris hash of this frame.
    fn ephemeris_id(&self) -> NaifId;
    /// Returns the orientation hash of this frame.
    fn orientation_id(&self) -> NaifId;
    /// Returns true if the ephemeris origin is equal to the provided hash
    fn ephem_origin_id_match(&self, other_hash: NaifId) -> bool {
        self.ephemeris_id() == other_hash
    }
    /// Returns true if the orientation origin is equal to the provided hash
    fn orient_origin_id_match(&self, other_hash: NaifId) -> bool {
        self.orientation_id() == other_hash
    }
    /// Returns true if the ephemeris origin is equal to the provided frame
    fn ephem_origin_match(&self, other: Self) -> bool {
        self.ephem_origin_id_match(other.ephemeris_id())
    }
    /// Returns true if the orientation origin is equal to the provided frame
    fn orient_origin_match(&self, other: Self) -> bool {
        self.orient_origin_id_match(other.orientation_id())
    }
}

/// A Frame uniquely defined by its ephemeris center and orientation. Refer to FrameDetail for frames combined with parameters.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub ephemeris_id: NaifId,
    pub orientation_id: NaifId,
}

impl FrameTrait for Frame {
    fn ephemeris_id(&self) -> NaifId {
        self.ephemeris_id
    }

    fn orientation_id(&self) -> NaifId {
        self.orientation_id
    }
}

impl Frame {
    /// Constructs a new frame given its ephemeris and orientations hashes.
    pub const fn from_ephem_orient(ephemeris_hash: NaifId, orientation_hash: NaifId) -> Self {
        Self {
            ephemeris_id: ephemeris_hash,
            orientation_id: orientation_hash,
        }
    }

    pub fn from_ephemeris_orientation_names<'a>(
        _ephemeris_name: &'a str,
        _orientation_name: &'a str,
    ) -> Self {
        todo!()
        // Self {
        //     ephemeris_id: hash(ephemeris_name.as_bytes()),
        //     orientation_id: hash(orientation_name.as_bytes()),
        // }
    }

    pub const fn from_ephem_j2000(ephemeris_hash: NaifId) -> Self {
        Self::from_ephem_orient(ephemeris_hash, J2000)
    }

    /// Returns a copy of this Frame whose ephemeris hash is set to the provided hash
    pub const fn with_ephem(&self, new_ephem_hash: NaifId) -> Self {
        Self {
            ephemeris_id: new_ephem_hash,
            orientation_id: self.orientation_id,
        }
    }

    /// Returns a copy of this Frame whose orientation hash is set to the provided hash
    pub const fn with_orient(&self, new_orient_hash: NaifId) -> Self {
        Self {
            ephemeris_id: self.ephemeris_id,
            orientation_id: new_orient_hash,
        }
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let body_name = match celestial_name_from_id(self.ephemeris_id) {
            Some(name) => name.to_string(),
            None => format!("body {}", self.ephemeris_id),
        };

        let orientation_name = match orientation_name_from_id(self.orientation_id) {
            Some(name) => name.to_string(),
            None => format!("orientation {}", self.orientation_id),
        };

        write!(f, "{body_name} {orientation_name}")
    }
}

impl fmt::LowerExp for Frame {
    /// Only prints the ephemeris name
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let body_name = match celestial_name_from_id(self.ephemeris_id) {
            Some(name) => name.to_string(),
            None => format!("{}", self.ephemeris_id),
        };
        write!(f, "{body_name}")
    }
}

impl fmt::Octal for Frame {
    /// Only prints the orientation name
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let orientation_name = match orientation_name_from_id(self.orientation_id) {
            Some(name) => name.to_string(),
            None => format!("orientation {}", self.orientation_id),
        };

        write!(f, "{orientation_name}")
    }
}
