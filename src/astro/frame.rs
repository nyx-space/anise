/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::fmt::{Display, Formatter};

use crate::constants::celestial_objects::hash_celestial_name;
use crate::constants::orientations::{hash_orientation_name, J2000};
use crate::HashType;

/// A Frame uniquely defined by its ephemeris center and orientation. Refer to FrameDetail for frames combined with parameters.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RefFrame {
    pub ephemeris_hash: HashType,
    pub orientation_hash: HashType,
}

impl RefFrame {
    /// Constructs a new frame given its ephemeris and orientations hashes.
    pub const fn from_ephem_orient(ephemeris_hash: HashType, orientation_hash: HashType) -> Self {
        Self {
            ephemeris_hash,
            orientation_hash,
        }
    }

    pub const fn from_ephem_j2000(ephemeris_hash: HashType) -> Self {
        Self::from_ephem_orient(ephemeris_hash, J2000)
    }

    /// Returns true if the ephemeris origin is equal to the provided hash
    pub const fn ephem_origin_hash_match(&self, other_hash: HashType) -> bool {
        self.ephemeris_hash == other_hash
    }

    /// Returns true if the orientation origin is equal to the provided hash
    pub const fn orient_origin_hash_match(&self, other_hash: HashType) -> bool {
        self.orientation_hash == other_hash
    }

    /// Returns true if the ephemeris origin is equal to the provided frame
    pub const fn ephem_origin_match(&self, other: Self) -> bool {
        self.ephemeris_hash == other.ephemeris_hash
    }

    /// Returns true if the orientation origin is equal to the provided frame
    pub const fn orient_origin_match(&self, other: Self) -> bool {
        self.orientation_hash == other.orientation_hash
    }

    /// Returns a copy of this Frame whose ephemeris hash is set to the provided hash
    pub const fn with_ephem(&self, new_ephem_hash: HashType) -> Self {
        Self {
            ephemeris_hash: new_ephem_hash,
            orientation_hash: self.orientation_hash,
        }
    }

    /// Returns a copy of this Frame whose orientation hash is set to the provided hash
    pub const fn with_orient(&self, new_orient_hash: HashType) -> Self {
        Self {
            ephemeris_hash: self.ephemeris_hash,
            orientation_hash: new_orient_hash,
        }
    }
}

impl Display for RefFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let body_name = match hash_celestial_name(self.ephemeris_hash) {
            Some(name) => name.to_string(),
            None => format!("body {}", self.ephemeris_hash),
        };

        let orientation_name = match hash_orientation_name(self.orientation_hash) {
            Some(name) => name.to_string(),
            None => format!("orientation {}", self.orientation_hash),
        };

        write!(f, "{body_name} {orientation_name}")
    }
}
