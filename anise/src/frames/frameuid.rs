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
use der::{Decode, Encode, Reader, Writer};
use serde::{Deserialize, Serialize};

pub use super::Frame;

#[cfg(feature = "analysis")]
use serde_dhall::StaticType;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// A unique frame reference that only contains enough information to build the actual Frame object.
/// It cannot be used for any computations, is it be used in any structure apart from error structures.
///
/// :type ephemeris_id: int
/// :type orientation_id: int
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "analysis", derive(StaticType))]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
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

impl Encode for FrameUid {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.ephemeris_id.encoded_len()? + self.orientation_id.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.ephemeris_id.encode(encoder)?;
        self.orientation_id.encode(encoder)
    }
}

impl<'a> Decode<'a> for FrameUid {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            ephemeris_id: decoder.decode()?,
            orientation_id: decoder.decode()?,
        })
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl FrameUid {
    #[new]
    fn py_new(ephemeris_id: NaifId, orientation_id: NaifId) -> Self {
        Self {
            ephemeris_id,
            orientation_id,
        }
    }
}
