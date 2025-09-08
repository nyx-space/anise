/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::fmt;

use super::framedef::CustomFrameDef;
use crate::{almanac::Almanac, astro::Aberration, errors::AlmanacError, prelude::Frame};
// TODO: Once https://github.com/Nadrieril/dhall-rust/issues/242 is closed, enable Dhall serialization.
// Will be implemented in https://github.com/nyx-space/anise/issues/466
// use serde_derive::{Deserialize, Serialize};
// use serde_dhall::StaticType;

#[derive(Clone, Debug, PartialEq)]
pub enum FrameSpec {
    Loaded(Frame),
    Manual {
        name: String,
        defn: Box<CustomFrameDef>,
    },
}

impl fmt::Display for FrameSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Loaded(frame) => write!(f, "{frame:x}"),
            Self::Manual { name, defn: _ } => write!(f, "{name}"),
        }
    }
}

/// StateDef allows defining a state from one frame (`from_frame`) to another (`to_frame`)
#[derive(Clone, Debug, PartialEq)]
pub struct StateSpec {
    pub from_frame: FrameSpec,
    pub to_frame: FrameSpec,
    pub ab_corr: Option<Aberration>,
}

impl fmt::Display for StateSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.from_frame, self.to_frame)
    }
}
