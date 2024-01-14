/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

mod chebyshev;
mod hermite;

pub(crate) use chebyshev::chebyshev_eval;
pub(crate) use hermite::hermite_eval;
use hifitime::Epoch;
use snafu::Snafu;

use crate::errors::{DecodingError, MathError};

/// Defines the maximum degree for an interpolation.
/// Until https://github.com/rust-lang/rust/issues/60551 , we cannot do operations on const generic, so we need some hack around it.
pub(crate) const MAX_SAMPLES: usize = 32;

#[derive(Debug, Snafu, PartialEq)]
#[snafu(visibility(pub(crate)))]
pub enum InterpolationError {
    #[snafu(display("decoding error during interpolation: {source}"))]
    InterpDecoding {
        #[snafu(backtrace)]
        source: DecodingError,
    },
    #[snafu(display("math error during interpolation: {source}"))]
    InterpMath {
        #[snafu(backtrace)]
        source: MathError,
    },
    #[snafu(display("spline valid from {start} to {end} but requested {req}"))]
    NoInterpolationData {
        req: Epoch,
        start: Epoch,
        end: Epoch,
    },
    #[snafu(display("no interpolation data to {epoch}, but prior checks succeeded (check integrity of the data?)"))]
    MissingInterpolationData { epoch: Epoch },
    #[snafu(display("interpolation data corrupted: {what}"))]
    CorruptedData { what: &'static str },
}
