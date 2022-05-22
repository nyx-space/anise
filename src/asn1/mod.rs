extern crate der;
extern crate hifitime;
pub mod common;
pub mod context;
pub mod ephemeris;
pub mod lookuptable;
pub mod metadata;
pub mod semver;
pub mod spline;
pub mod splinecoeffs;
pub mod splinekind;
pub mod time;

use self::semver::Semver;
/// The current version of ANISE
pub const ANISE_VERSION: Semver = Semver {
    major: 0,
    minor: 0,
    patch: 1,
};
