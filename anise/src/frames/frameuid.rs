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
    constants::{
        celestial_objects::celestial_name_from_id, orientations::orientation_name_from_id,
    },
    NaifId,
};
use core::fmt;

pub use super::Frame;

/// A unique frame reference that only contains enough information to build the actual Frame object.
/// It cannot be used for any computations, is it be used in any structure apart from error structures.
///
/// # Usage note
/// You should almost always prefer Frame over FrameRef unless you will
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FrameUid {
    pub ephemeris_id: NaifId,
    pub orientation_id: NaifId,
}

impl From<Frame> for FrameUid {
    fn from(frame: Frame) -> Self {
        Self {
            ephemeris_id: frame.ephemeris_id,
            orientation_id: frame.orientation_id,
        }
    }
}

impl From<&Frame> for FrameUid {
    fn from(frame: &Frame) -> Self {
        Self {
            ephemeris_id: frame.ephemeris_id,
            orientation_id: frame.orientation_id,
        }
    }
}

impl From<FrameUid> for Frame {
    fn from(uid: FrameUid) -> Self {
        Self::new(uid.ephemeris_id, uid.orientation_id)
    }
}

impl From<&FrameUid> for Frame {
    fn from(uid: &FrameUid) -> Self {
        Self::new(uid.ephemeris_id, uid.orientation_id)
    }
}

impl fmt::Display for FrameUid {
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
