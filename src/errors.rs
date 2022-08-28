/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::{Epoch, TimeSystem};

use crate::asn1::semver::Semver;
use crate::der::Error as Asn1Error;
use crate::der::Error as DerError;
use crate::frame::Frame;
use core::convert::From;
use core::fmt;
use std::io::ErrorKind as IOErrorKind;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AniseError {
    /// Raised for an error in reading or writing the file(s)
    IOError(IOErrorKind),
    /// Raised if an IO error occured but its representation is not simple (and therefore not an std::io::ErrorKind).
    IOUnknownError,
    /// Raise if a division by zero was to occur
    DivisionByZero,
    /// Raised when requesting the value of a parameter but it does not have any representation (typically the coefficients are an empty array)
    ParameterNotSpecified,
    /// For some reason weird reason (malformed file?), data that was expected to be in an array wasn't.
    IndexingError,
    /// If the NAIF file cannot be read or isn't supported
    NAIFParseError(String),
    InvalidTimeSystem,
    /// Raised if the checksum of the encoded data does not match the stored data.
    IntegrityError(IntegrityErrorKind),
    /// Raised if the file could not be decoded correctly
    DecodingError(Asn1Error),
    /// Raised if the ANISE version of the file is incompatible with the library.
    IncompatibleVersion {
        got: Semver,
        exp: Semver,
    },
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
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InternalErrorKind {
    /// Appending to the lookup table failed
    LUTAppendFailure,
    /// May happen if the interpolation scheme is not yet supported
    InterpolationNotSupported,
    Asn1Error(DerError),
    /// Some generic internal error, check the logs of the program and file a bug report
    Generic,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum IntegrityErrorKind {
    /// Checksum of ephemeris data differs from expected checksum
    ChecksumInvalid,
    /// Data between two ephemerides expected to be identical mismatch (may happen on merger of files)
    DataMismatchOnMerge,
    /// Could not fetch spline data that was expected to be there
    DataMissing,
    /// The lookup table is broken somehow
    LookupTable,
    /// Raised if a transformation is requested but the frames have no common origin
    DisjointRoots { from_frame: Frame, to_frame: Frame },
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

impl From<DerError> for InternalErrorKind {
    fn from(e: DerError) -> Self {
        Self::Asn1Error(e)
    }
}

impl fmt::Display for AniseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::IOError(e) => write!(f, "ANISE error: IOError: {:?}", e),
            Self::IOUnknownError => write!(f, "ANISE error: IOUnknownError"),
            Self::DivisionByZero => write!(f, "ANISE error: DivisionByZero"),
            Self::ParameterNotSpecified => write!(f, "ANISE error: ParameterNotSpecified"),
            Self::IndexingError => write!(f, "ANISE error: IndexingError"),
            Self::NAIFParseError(reason) => {
                write!(f, "ANISE error: invalid NAIF DAF file: {}", reason)
            }
            Self::InvalidTimeSystem => write!(f, "ANISE error: invalid time system"),
            Self::IntegrityError(e) => write!(f, "ANISE error: data integrity error: {:?}", e),
            Self::DecodingError(err) => write!(
                f,
                "ANISE error: bytes could not be decoded into a valid ANISE file - {}",
                err
            ),
            Self::ItemNotFound => write!(f, "ANISE error: requested item not found in context"),
            Self::IncompatibleVersion { got, exp } => write!(
                f,
                "ANISE error: Incompatible version: got {}.{}.{} - expected {}.{}.{}",
                got.major, got.minor, got.patch, exp.major, exp.minor, exp.patch
            ),
            Self::InternalError(e) => {
                write!(f, "ANISE internal error: {:?} -- please report a bug", e)
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
                "ANISE error: No interpolation as epoch {}", e.as_gregorian_str(TimeSystem::ET) 
            ),
        }
    }
}

impl std::error::Error for AniseError {}
