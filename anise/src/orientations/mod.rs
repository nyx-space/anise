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
    prelude::FrameUid, structure::dataset::DataSetError,
};

mod paths;
mod rotate_to_parent;
mod rotations;

#[derive(Debug, Snafu, PartialEq)]
#[snafu(visibility(pub(crate)))]
pub enum OrientationError {
    /// Somehow you've entered code that should not be reachable, please file a bug.
    Unreachable,
    #[snafu(display(
         "could not load BPC because all {max_slots} are used (modify `MAX_LOADED_BPCS` at build time)"
     ))]
    StructureIsFull { max_slots: usize },
    #[snafu(display(
        "Could not rotate from {from} to {to}: no common origin found at epoch {epoch}"
    ))]
    RotationOrigin {
        from: FrameUid,
        to: FrameUid,
        epoch: Epoch,
    },
    #[snafu(display("no orientation data loaded (must call load_bpc or DataSet::from_bytes)"))]
    NoOrientationsLoaded,
    #[snafu(display("when {action} caused {source}"))]
    BPC {
        action: &'static str,
        #[snafu(backtrace)]
        source: DAFError,
    },
    #[snafu(display("during an orientation operation: {source}"))]
    OrientationPhysics {
        #[snafu(backtrace)]
        source: PhysicsError,
    },
    #[snafu(display("during an orientation interpolation {source}"))]
    OrientationInterpolation {
        #[snafu(backtrace)]
        source: InterpolationError,
    },
    #[snafu(display("during an orientation query {source}"))]
    OrientationDataSet {
        #[snafu(backtrace)]
        source: DataSetError,
    },
    #[snafu(display("unknown orientation ID associated with `{name}`"))]
    OrientationNameToId { name: String },
}
