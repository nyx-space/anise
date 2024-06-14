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

use crate::ephemerides::EphemerisError;
use crate::orientations::OrientationError;
use crate::prelude::FrameUid;
use crate::structure::dataset::DataSetError;
use crate::structure::semver::Semver;
use crate::NaifId;
use core::convert::From;
use der::Error as DerError;
use std::io::ErrorKind as IOErrorKind;

#[cfg(feature = "metaload")]
use crate::almanac::metaload::MetaAlmanacError;
#[cfg(feature = "metaload")]
use crate::almanac::metaload::MetaFile;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(visibility(pub))]
pub enum AlmanacError {
    #[snafu(display("{action} encountered an error with ephemeris computation {source}"))]
    Ephemeris {
        action: &'static str,
        #[snafu(source(from(EphemerisError, Box::new)))]
        source: Box<EphemerisError>,
    },
    #[snafu(display("{action} encountered an error with orientation computation {source}"))]
    Orientation {
        action: &'static str,
        #[snafu(source(from(OrientationError, Box::new)))]
        source: Box<OrientationError>,
    },
    #[snafu(display("{source} encountered when loading {path}"))]
    Loading {
        path: String,
        source: InputOutputError,
    },
    #[snafu(display("{source} encountered when {action}"))]
    TLDataSet {
        action: &'static str,
        source: DataSetError,
    },
    #[snafu(display("{err}"))]
    GenericError { err: String },
    #[cfg(feature = "metaload")]
    #[snafu(display("processing file #{fno} ({file:?}) caused an error: {source}"))]
    Meta {
        fno: usize,
        file: MetaFile,
        source: MetaAlmanacError,
    },
}

pub type AlmanacResult<T> = Result<T, AlmanacError>;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum InputOutputError {
    /// Raised for an error in reading or writing the file(s)
    IOError { kind: IOErrorKind },
    /// Raised if an IO error occurred but its representation is not simple (and therefore not an std::io::ErrorKind).
    IOUnknownError,
}

#[derive(Copy, Clone, Debug, Snafu, PartialEq)]
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
    #[snafu(display("data for {variable}={value} in {dataset} is invalid {reason}"))]
    InvalidValue {
        dataset: &'static str,
        variable: &'static str,
        value: f64,
        reason: &'static str,
    },
}

#[derive(Copy, Clone, PartialEq, Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum MathError {
    #[snafu(display("prevented a division by zero when {action}"))]
    DivisionByZero { action: &'static str },
    #[snafu(display("{msg}: {value}"))]
    DomainError { value: f64, msg: &'static str },
    #[snafu(display("max iterations reached ({iter}) when {action}"))]
    MaxIterationsReached { iter: usize, action: &'static str },
}

#[derive(Copy, Clone, Debug, Snafu, PartialEq)]
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
    #[snafu(display(
        "cannot {action} because rotations {from1}->{to1} and {from2}->{to2} are incompatible"
    ))]
    InvalidRotation {
        action: &'static str,
        from1: NaifId,
        to1: NaifId,
        from2: NaifId,
        to2: NaifId,
    },
    #[snafu(display("cannot rotate state in frame {state_frame} with rotation {from}->{to}"))]
    InvalidStateRotation {
        from: NaifId,
        to: NaifId,
        state_frame: FrameUid,
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
    #[snafu(display("calculation requires hyperbolic orbit, but its eccentricity is {ecc}"))]
    NotHyperbolic { ecc: f64 },
    #[snafu(display("infinite value encountered when {action}"))]
    InfiniteValue { action: &'static str },
    #[snafu(display("{source}"))]
    AppliedMath { source: MathError },
    #[snafu(display("invalid radius: {action}"))]
    RadiusError { action: &'static str },
    #[snafu(display("invalid velocity: {action}"))]
    VelocityError { action: &'static str },
    #[snafu(display("invalid aberration: {action}"))]
    AberrationError { action: &'static str },
}

impl From<IOErrorKind> for InputOutputError {
    fn from(kind: IOErrorKind) -> Self {
        Self::IOError { kind }
    }
}
