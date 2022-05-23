/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::asn1::semver::Semver;
use crate::der::Error as Asn1Error;
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
    NAIFConversionError(String),
    InvalidTimeSystem,
    /// Raised if the checksum of the encoded data does not match the stored data.
    IntegrityError,
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
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InternalErrorKind {
    /// Appending to the lookup table failed
    LUTAppendFailure,
    /// May happen if the interpolation scheme is not yet supported
    InterpolationNotSupported,
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

impl fmt::Display for AniseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::IOError(e) => write!(f, "ANISE error: IOError: {:?}", e),
            Self::IOUnknownError => write!(f, "ANISE error: IOUnknownError"),
            Self::DivisionByZero => write!(f, "ANISE error: DivisionByZero"),
            Self::ParameterNotSpecified => write!(f, "ANISE error: ParameterNotSpecified"),
            Self::IndexingError => write!(f, "ANISE error: IndexingError"),
            Self::NAIFConversionError(reason) => {
                write!(f, "ANISE error: invalid NAIF DAF file: {}", reason)
            }
            Self::InvalidTimeSystem => write!(f, "ANISE error: invalid time system"),
            Self::IntegrityError => write!(
                f,
                "ANISE error: data array checksum verification failed (file is corrupted)"
            ),
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
        }
    }
}

impl std::error::Error for AniseError {}
