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
    NaifId,
    constants::{
        celestial_objects::celestial_name_from_id, orientations::orientation_name_from_id,
    },
    time::Epoch,
};
use core::fmt;
use core::str::FromStr;
use der::{Decode, Encode, Reader, Writer};
use log::error;
use serde::{Deserialize, Serialize};

pub use super::Frame;

#[cfg(feature = "analysis")]
use serde_dhall::{SimpleType, StaticType};

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// A unique frame reference that only contains enough information to build the actual Frame object.
/// It cannot be used for any computations, is it be used in any structure apart from error structures.
///
/// :type ephemeris_id: int
/// :type orientation_id: int
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct FrameUid {
    pub ephemeris_id: NaifId,
    pub orientation_id: NaifId,
    pub force_inertial: bool,
    pub frozen_epoch: Option<Epoch>,
}

impl From<Frame> for FrameUid {
    fn from(frame: Frame) -> Self {
        Self {
            ephemeris_id: frame.ephemeris_id,
            orientation_id: frame.orientation_id,
            force_inertial: frame.force_inertial,
            frozen_epoch: frame.frozen_epoch,
        }
    }
}

impl From<&Frame> for FrameUid {
    fn from(frame: &Frame) -> Self {
        Self {
            ephemeris_id: frame.ephemeris_id,
            orientation_id: frame.orientation_id,
            force_inertial: frame.force_inertial,
            frozen_epoch: frame.frozen_epoch,
        }
    }
}

impl From<FrameUid> for Frame {
    fn from(uid: FrameUid) -> Self {
        Self {
            ephemeris_id: uid.ephemeris_id,
            orientation_id: uid.orientation_id,
            force_inertial: uid.force_inertial,
            frozen_epoch: uid.frozen_epoch,
            mu_km3_s2: None,
            shape: None,
        }
    }
}

impl From<&FrameUid> for Frame {
    fn from(uid: &FrameUid) -> Self {
        Self {
            ephemeris_id: uid.ephemeris_id,
            orientation_id: uid.orientation_id,
            force_inertial: uid.force_inertial,
            frozen_epoch: uid.frozen_epoch,
            mu_km3_s2: None,
            shape: None,
        }
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

#[cfg(feature = "analysis")]
impl StaticType for FrameUid {
    fn static_type() -> serde_dhall::SimpleType {
        use std::collections::HashMap;
        let mut repr = HashMap::new();

        repr.insert("ephemeris_id".to_string(), SimpleType::Integer);
        repr.insert("orientation_id".to_string(), SimpleType::Integer);
        repr.insert("force_inertial".to_string(), SimpleType::Bool);
        repr.insert(
            "frozen_epoch".to_string(),
            SimpleType::Optional(Box::new(SimpleType::Text)),
        );
        SimpleType::Record(repr)
    }
}

impl Encode for FrameUid {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let mut flags: u8 = 0;
        if self.frozen_epoch.is_some() {
            flags |= 1 << 0;
        }
        self.ephemeris_id.encoded_len()?
            + self.orientation_id.encoded_len()?
            + self.force_inertial.encoded_len()?
            + flags.encoded_len()?
            + self.frozen_epoch.map(|e| e.to_string()).encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        let mut flags: u8 = 0;
        if self.frozen_epoch.is_some() {
            flags |= 1 << 0;
        }
        self.ephemeris_id.encode(encoder)?;
        self.orientation_id.encode(encoder)?;
        self.force_inertial.encode(encoder)?;
        flags.encode(encoder)?;
        self.frozen_epoch.map(|e| e.to_string()).encode(encoder)
    }
}

impl<'a> Decode<'a> for FrameUid {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let ephemeris_id: NaifId = decoder.decode()?;
        let orientation_id: NaifId = decoder.decode()?;
        let force_inertial: bool = decoder.decode()?;

        let flags: u8 = decoder.decode()?;
        let frozen_epoch = if flags & (1 << 0) != 0 {
            let epoch_str: String = decoder.decode()?;
            match Epoch::from_str(&epoch_str) {
                Ok(epoch) => Some(epoch),
                Err(e) => {
                    error!("frozen epoch in frame kernel invalid: {e}");
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            ephemeris_id,
            orientation_id,
            force_inertial,
            frozen_epoch,
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
            force_inertial: false,
            frozen_epoch: None,
        }
    }
    #[getter]
    fn get_force_inertial(&self) -> bool {
        self.force_inertial
    }
    /// :type ephemeris_id: int
    #[setter]
    fn set_force_inertial(&mut self, force_inertial: bool) {
        self.force_inertial = force_inertial;
    }
    /// :rtype: Epoch
    #[getter]
    fn get_frozen_epoch(&self) -> Option<Epoch> {
        self.frozen_epoch
    }
    /// :type frozen_epoch: Epoch, optional
    #[setter]
    fn set_frozen_epoch(&mut self, frozen_epoch: Option<Epoch>) {
        self.frozen_epoch = frozen_epoch;
    }
}
