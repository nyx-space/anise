/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::Epoch;
use snafu::prelude::*;

use crate::{
    math::{interpolation::InterpolationError, PhysicsError},
    naif::daf::DAFError,
    prelude::Frame,
};

pub mod paths;
pub mod translate_to_parent;
pub mod translations;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum EphemerisError<'a> {
    /// Somehow you've entered code that should not be reachable, please file a bug.
    Unreachable,
    #[snafu(display(
        "Could not translate from {from} to {to}: no common origin found at epoch {epoch}"
    ))]
    TranslationOrigin {
        from: Frame,
        to: Frame,
        epoch: Epoch,
    },
    #[snafu(display("no ephemeris data loaded (must call load_spk)"))]
    NoEphemerisLoaded,
    #[snafu(display("trying {action} caused {source}"))]
    UnderlyingDAF {
        action: &'a str,
        source: DAFError<'a>,
    },
    #[snafu(display("trying {action} caused {source}"))]
    UnderlyingPhysics {
        action: &'a str,
        source: PhysicsError,
    },
    UnderlyingInterpolation {
        source: InterpolationError<'a>,
    },
}
