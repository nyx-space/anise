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

use crate::prelude::Frame;
use core::convert::From;
use core::fmt;
use std::io::ErrorKind as IOErrorKind;

#[derive(Clone, PartialEq, Debug)]
pub enum AniseError {
    StructureIsFull,
    /// Raised for an error in reading or writing the file(s)
    IOError(IOErrorKind),
    /// Raised if an IO error occurred but its representation is not simple (and therefore not an std::io::ErrorKind).
    IOUnknownError,
    /// Math error
    MathError(MathErrorKind),
    /// Raised when requesting the value of a parameter but it does not have any representation (typically the coefficients are an empty array)
    ParameterNotSpecified,
    /// The byte stream is missing data that is required to parse.
    MalformedData(usize),
    /// If the NAIF file cannot be read or isn't supported
    DAFParserError(String),
    InvalidTimeSystem,
    /// Raised if the checksum of the encoded data does not match the stored data.
    IntegrityError(IntegrityErrorKind),
    /// Raised if the item sought after is not found in the context
    ItemNotFound,
    /// Raised when requesting the interpolation for data that is not available in this spline.
    NoInterpolationData,
    /// If this is raised, please report a bug
    InternalError(InternalErrorKind),
    /// Raised to prevent overwriting an existing file
    FileExists,
    /// Raised if a transformation is requested but the frames have no common origin
    DisjointFrames {
        from_frame: Frame,
        to_frame: Frame,
    },
    /// Raised if the ephemeris or orientation is deeper to the context origin than this library supports
    MaxTreeDepth,
    /// Raised if there is no interpolation data for the requested epoch, i.e. ephemeris/orientation starts after or ends before the requested epoch
    MissingInterpolationData(Epoch),
    /// Raised if a computation is physically wrong
    PhysicsError(PhysicsErrorKind),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum InternalErrorKind {
    /// Appending to the lookup table failed
    LUTAppendFailure,
    /// May happen if the interpolation scheme is not yet supported
    InterpolationNotSupported,
    /// Some generic internal error, check the logs of the program and file a bug report
    Generic,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum IntegrityErrorKind {
    /// Data checksum differs from expected checksum
    ChecksumInvalid { expected: u32, computed: u32 },
    /// Data between two ephemerides expected to be identical mismatch (may happen on merger of files)
    DataMismatchOnMerge,
    /// Could not fetch spline data that was expected to be there
    DataMissing,
    /// The lookup table is broken somehow
    LookupTable,
    /// Raised if a transformation is requested but the frames have no common origin
    DisjointRoots { from_frame: Frame, to_frame: Frame },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MathErrorKind {
    DivisionByZero,
    StateEpochsDiffer,
    StateFramesDiffer,
    InvalidInterpolationData,
    PolynomialOrderError(usize),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PhysicsErrorKind {
    /// ANISE does not support parabolic orbits because these are not physically real.
    ParabolicOrbit,
    /// True anomaly of the provided hyperbolic orbit is physically impossible
    InvalidHyperbolicTrueAnomaly(f64),
    /// Some computation led to a value being infinite, check the error logs
    InfiniteValue,
}

impl From<IOErrorKind> for AniseError {
    fn from(e: IOErrorKind) -> Self {
        Self::IOError(e)
    }
}

impl From<InternalErrorKind> for AniseError {
    fn from(e: InternalErrorKind) -> Self {
        Self::InternalError(e)
    }
}

impl From<MathErrorKind> for AniseError {
    fn from(e: MathErrorKind) -> Self {
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
            Self::IntegrityError(e) => write!(f, "ANISE error: data integrity error: {e:?}"),
            Self::ItemNotFound => write!(f, "ANISE error: requested item not found in context"),
            Self::InternalError(e) => {
                write!(f, "ANISE internal error: {e:?} -- please report a bug")
            }
            Self::NoInterpolationData => write!(
                f,
                "ANISE error: No interpolation for the requested component"
            ),
            Self::FileExists => write!(
                f,
                "ANISE error: operation aborted to prevent overwriting an existing file"
            ),
            Self::DisjointFrames { from_frame: from, to_frame: to } => write!(
                f,
                "ANISE error: frame {} and {} do not share a common origin",
                to, from
            ),
            Self::MaxTreeDepth => write!(
                f,
                "ANISE error: the ephemeris or orientation is deeper to the context origin than this library supports"
            ),
            Self::MissingInterpolationData(e) => write!(
                f,
                "ANISE error: No interpolation as epoch {e:e}"
            ),
            Self::PhysicsError(e) => write!(f, "ANISE error: Physics error: {e:?}")
        }
    }
}

impl std::error::Error for AniseError {}
