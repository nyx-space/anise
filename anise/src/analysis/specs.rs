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

use hifitime::Epoch;
use snafu::ResultExt;

use super::framedef::CustomFrameDef;
use crate::{
    almanac::Almanac,
    analysis::{AlmanacStateSpecSnafu, AnalysisError},
    astro::Aberration,
    math::cartesian::CartesianState,
    prelude::Frame,
};
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
    pub target_frame: FrameSpec,
    pub observer_frame: FrameSpec,
    pub ab_corr: Option<Aberration>,
}

impl fmt::Display for StateSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.target_frame, self.observer_frame)
    }
}

impl StateSpec {
    /// Evaluates this state specification at the provided epoch.
    pub fn evaluate(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<CartesianState, AnalysisError> {
        if let FrameSpec::Loaded(target_frame) = self.target_frame {
            if let FrameSpec::Loaded(observer_frame) = self.observer_frame {
                almanac
                    .transform(target_frame, observer_frame, epoch, self.ab_corr)
                    .context(AlmanacStateSpecSnafu {
                        spec: Box::new(self.clone()),
                        epoch,
                    })
            } else {
                unimplemented!("custom frames in not yet supported")
            }
        } else {
            unimplemented!("custom frames in not yet supported")
        }
    }
}
