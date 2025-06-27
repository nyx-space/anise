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
    errors::PhysicsError, math::interpolation::InterpolationError, naif::daf::DAFError,
    prelude::FrameUid, NaifId,
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
    #[snafu(display(
        "could not load SPK because all {max_slots} are used (modify `MAX_LOADED_SPKS` at build time)"
    ))]
    StructureIsFull { max_slots: usize },
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
    #[snafu(display("Failed to retrieve ephemeris for target {target_id} (NAIF ID) at light-time corrected epoch {corrected_epoch} (original epoch: {original_epoch}, mode: '{aberration_mode}'). This often occurs if the corrected time is outside the available data range for the target. Original error: {source_error}"))]
    LightTimeLookupFailed {
        original_epoch: Epoch,
        corrected_epoch: Epoch,
        target_id: NaifId,
        aberration_mode: String,
        #[snafu(source(from(EphemerisError, Box::new)))] // This ensures the source error is boxed
        source_error: Box<EphemerisError>,
    },
    #[snafu(display("unknown name associated with NAIF ID {id}"))]
    IdToName { id: NaifId },
    #[snafu(display("unknown NAIF ID associated with `{name}`"))]
    NameToId { name: String },
}
