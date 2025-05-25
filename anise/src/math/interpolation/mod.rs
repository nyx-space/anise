/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

mod chebyshev;
mod hermite;
mod lagrange;

pub use chebyshev::{chebyshev_eval, chebyshev_eval_poly};
pub use hermite::hermite_eval;
use hifitime::Epoch;
pub use lagrange::lagrange_eval;
use snafu::Snafu;

use crate::errors::{DecodingError, MathError};

/// A trait for accessing data, potentially with a stride or other custom logic.
pub trait StridedDataAccess {
    /// Returns the element at the given index.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    fn get(&self, index: usize) -> f64; // TODO: Consider returning Option<f64> or Result<f64, Error> for robustness if needed

    /// Returns the number of elements accessible.
    fn len(&self) -> usize;

    /// Returns `true` if there are no elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl StridedDataAccess for &[f64] {
    #[inline]
    fn get(&self, index: usize) -> f64 {
        self[index]
    }

    #[inline]
    fn len(&self) -> usize {
        <[f64]>::len(self)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        <[f64]>::is_empty(self)
    }
}

/// Defines the maximum degree for an interpolation.
/// Until https://github.com/rust-lang/rust/issues/60551 , we cannot do operations on const generic, so we need some hack around it.
pub(crate) const MAX_SAMPLES: usize = 32;

/// Errors related to interpolation.
#[derive(Copy, Clone, Debug, Snafu, PartialEq)]
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
    #[snafu(display("requested epoch {req} is outside the valid range [{start}, {end}]"))]
    NoInterpolationData {
        req: Epoch,
        start: Epoch,
        end: Epoch,
    },
    #[snafu(display("no interpolation data to {epoch}, but prior checks succeeded (check integrity of the data?)"))]
    MissingInterpolationData { epoch: Epoch },
    #[snafu(display("interpolation data corrupted: {what}"))]
    CorruptedData { what: &'static str },
    #[snafu(display("not enough samples for interpolation: need at least 2, got {got}"))]
    NotEnoughSamples { got: usize },
    #[snafu(display("too many samples for interpolation: max is {max_samples}, got {got}"))]
    TooManySamples { max_samples: usize, got: usize },
    #[snafu(display("{op} is unsupported for {kind}"))]
    UnsupportedOperation {
        kind: &'static str,
        op: &'static str,
    },
    #[snafu(display(
        "{dataset} is not yet supported -- see https://github.com/nyx-space/anise/issues/{issue}"
    ))]
    UnimplementedType { issue: u32, dataset: &'static str },
}
