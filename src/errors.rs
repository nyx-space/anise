/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::Epoch;
use snafu::prelude::*;

use crate::prelude::FrameUid;
use crate::structure::semver::Semver;
use crate::NaifId;
use core::convert::From;
use core::fmt;
use der::Error as DerError;
use std::io::ErrorKind as IOErrorKind;

#[derive(Clone, PartialEq, Debug)]
pub enum AniseError {
    StructureIsFull,
    /// Raised for an error in reading or writing the file(s)
    IOError(IOErrorKind),
    /// Raised if an IO error occurred but its representation is not simple (and therefore not an std::io::ErrorKind).
    IOUnknownError,
    /// Math error
    MathError(MathError),
    /// Raised when requesting the value of a parameter but it does not have any representation (typically the coefficients are an empty array)
    ParameterNotSpecified,
    /// The byte stream is missing data that is required to parse.
    MalformedData(usize),
    /// If the NAIF file cannot be read or isn't supported
    DAFParserError(String),
    InvalidTimeSystem,
    /// Raised if the item sought after is not found in the context
    ItemNotFound,
    /// Raised when requesting the interpolation for data that is not available in this spline.
    NoInterpolationData,
    /// Raised to prevent overwriting an existing file
    FileExists,
    /// Raised if the ephemeris or orientation is deeper to the context origin than this library supports
    MaxTreeDepth,
    /// Raised if there is no interpolation data for the requested epoch, i.e. ephemeris/orientation starts after or ends before the requested epoch
    MissingInterpolationData(Epoch),
    /// Raised if a computation is physically wrong
    PhysicsError(PhysicsErrorKind),
    IncompatibleVersion {
        got: Semver,
        exp: Semver,
    },
    DecodingError(der::Error),
    IncompatibleRotation {
        from: i32,
        to: i32,
    },
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum DecodingError {
    #[snafu(display(
        "could not decode {dataset} data -- need at least {need} doubles but found {got}"
    ))]
    TooFewDoubles {
        dataset: &'static str,
        got: usize,
        need: usize,
    },
    #[snafu(display("bytes between indexes {start} and {end} could not be read, array contains {size} bytes (data malformed?)"))]
    InaccessibleBytes {
        start: usize,
        end: usize,
        size: usize,
    },
    #[snafu(display("integrity error during decoding: {source}"))]
    Integrity {
        #[snafu(backtrace)]
        source: IntegrityError,
    },
    #[snafu(display("decoding DER failed: {err}"))]
    DecodingDer { err: DerError },
    #[snafu(display("somehow casting the data failed"))]
    Casting,
    #[snafu(display("could not load ANISE data version {got}, expected {exp}"))]
    AniseVersion { got: Semver, exp: Semver },
    #[snafu(display("data could not be parsed as {kind} despite ANISE version matching (should be loaded as another type?)"))]
    Obscure { kind: &'static str },
}

#[derive(Copy, Clone, PartialEq, Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum IntegrityError {
    /// Data checksum differs from expected checksum
    ChecksumInvalid { expected: u32, computed: u32 },
    /// Data between two ephemerides expected to be identical mismatch (may happen on merger of files)
    DataMismatchOnMerge,
    /// Could not fetch spline data that was expected to be there
    DataMissing,
    /// The lookup table is broken somehow
    LookupTable,
    /// Raised if a transformation is requested but the frames have no common origin
    DisjointRoots {
        from_frame: FrameUid,
        to_frame: FrameUid,
    },
    #[snafu(display(
        "data for {variable} in {dataset} decoded as subnormal double (data malformed?)"
    ))]
    SubNormal {
        dataset: &'static str,
        variable: &'static str,
    },
}

#[derive(Clone, PartialEq, Eq, Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum MathError {
    /// Prevented a division by zero, check data integrity
    DivisionByZero,
    ///
    EpochsDiffer,
    FramesDiffer,
    InvalidInterpolationData,
    PolynomialOrderError {
        order: usize,
    },
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum PhysicsError {
    /// Somehow you've entered code that should not be reachable, please file a bug.
    Unreachable,
    #[snafu(display("epochs {epoch1} and {epoch2} differ while {action}"))]
    EpochMismatch {
        action: &'static str,
        epoch1: Epoch,
        epoch2: Epoch,
    },
    #[snafu(display("frames {frame1} and {frame2} differ while {action}"))]
    FrameMismatch {
        action: &'static str,
        frame1: FrameUid,
        frame2: FrameUid,
    },
    #[snafu(display("origins {from1} and {from2} differ while {action}"))]
    OriginMismatch {
        action: &'static str,
        from1: NaifId,
        from2: NaifId,
    },
    #[snafu(display("{action} requires the time derivative of the DCM but it is not set"))]
    DCMMissingDerivative { action: &'static str },
    #[snafu(display("{action} requires the frame {frame} to have {data} defined"))]
    MissingFrameData {
        action: &'static str,
        data: &'static str,
        frame: FrameUid,
    },
    #[snafu(display("parabolic orbits are physically impossible and the eccentricity calculated to be within {limit:e} of 1.0"))]
    ParabolicEccentricity { limit: f64 },
    #[snafu(display("parabolic orbits are physically impossible and the semilatus rectum (semi-parameter) calculated to be {p}"))]
    ParabolicSemiParam { p: f64 },
    #[snafu(display("hyperbolic true anomaly is physically impossible: {ta_deg} deg"))]
    HyperbolicTrueAnomaly { ta_deg: f64 },
    #[snafu(display("infinite value encountered when {action}"))]
    InfiniteValue { action: &'static str },
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PhysicsErrorKind {}

impl From<IOErrorKind> for AniseError {
    fn from(e: IOErrorKind) -> Self {
        Self::IOError(e)
    }
}

impl From<MathError> for AniseError {
    fn from(e: MathError) -> Self {
        Self::MathError(e)
    }
}

impl fmt::Display for AniseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::StructureIsFull => write!(f, "ANISE error: attempted to load more data but no more room was available"),
            Self::IOError(e) => write!(f, "ANISE error: IOError: {e:?}"),
            Self::IOUnknownError => write!(f, "ANISE error: IOUnknownError"),
            Self::MathError(e) => write!(f, "ANISE error: MathError: {e:?}"),
            Self::ParameterNotSpecified => write!(f, "ANISE error: ParameterNotSpecified"),
            Self::MalformedData(byte) => write!(f, "ANISE error: Malformed data: could not read up to byte {byte}."),
            Self::DAFParserError(reason) => {
                write!(f, "ANISE error: invalid NAIF DAF file: {}", reason)
            }
            Self::InvalidTimeSystem => write!(f, "ANISE error: invalid time system"),
            Self::ItemNotFound => write!(f, "ANISE error: requested item not found in context"),
            Self::NoInterpolationData => write!(
                f,
                "ANISE error: No interpolation for the requested component"
            ),
            Self::FileExists => write!(
                f,
                "ANISE error: operation aborted to prevent overwriting an existing file"
            ),
            Self::MaxTreeDepth => write!(
                f,
                "ANISE error: the ephemeris or orientation is deeper to the context origin than this library supports"
            ),
            Self::MissingInterpolationData(e) => write!(
                f,
                "ANISE error: No interpolation as epoch {e:e}"
            ),
            Self::PhysicsError(e) => write!(f, "ANISE error: Physics error: {e:?}"),
            _ => write!(f, "ANISE error: {self:?}")
        }
    }
}

impl std::error::Error for AniseError {}
