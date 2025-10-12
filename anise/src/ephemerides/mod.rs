/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::Epoch;
use snafu::prelude::*;

use crate::{
    astro::Aberration, errors::PhysicsError, math::interpolation::InterpolationError,
    naif::daf::DAFError, prelude::FrameUid, NaifId,
};

pub mod paths;
pub mod translate_to_parent;
pub mod translations;

#[derive(Debug, Snafu, PartialEq)]
#[snafu(visibility(pub))]
#[non_exhaustive]
pub enum EphemerisError {
    /// Somehow you've entered code that should not be reachable, please file a bug.
    Unreachable,
    #[snafu(display("could not {action} because {alias} is not loaded"))]
    AliasNotFound { alias: String, action: &'static str },
    #[snafu(display(
        "Could not translate from {from} to {to}: no common origin found at epoch {epoch}"
    ))]
    TranslationOrigin {
        from: FrameUid,
        to: FrameUid,
        epoch: Epoch,
    },
    #[snafu(display("no ephemeris data loaded (must call load_spk)"))]
    NoEphemerisLoaded,
    #[snafu(display("when {action} caused {source}"))]
    SPK {
        action: &'static str,
        #[snafu(backtrace)]
        source: DAFError,
    },
    #[snafu(display("when {action} for ephemeris {source}"))]
    EphemerisPhysics {
        action: &'static str,
        #[snafu(backtrace)]
        source: PhysicsError,
    },
    #[snafu(display("during an ephemeris interpolation {source}"))]
    EphemInterpolation {
        #[snafu(backtrace)]
        source: InterpolationError,
    },
    #[snafu(display("{ab_corr} corrects epoch from {epoch} to {epoch_lt}, but {source}"))]
    LightTimeCorrection {
        epoch: Epoch,
        epoch_lt: Epoch,
        ab_corr: Aberration,
        #[snafu(source(from(EphemerisError, Box::new)))] // This ensures the source error is boxed
        source: Box<EphemerisError>,
    },
    #[snafu(display("unknown name associated with NAIF ID {id}"))]
    IdToName { id: NaifId },
    #[snafu(display("unknown NAIF ID associated with `{name}`"))]
    NameToId { name: String },
}
