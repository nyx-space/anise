use crate::asn1::semver::Semver;
use crate::asn1::ANISE_VERSION;
use crate::der::Error as Asn1Error;
use std::convert::From;
use std::fmt;
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
        file_version: Semver,
    },
}

impl From<IOErrorKind> for AniseError {
    fn from(e: IOErrorKind) -> Self {
        Self::IOError(e)
    }
}

impl fmt::Display for AniseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::IOError(e) => write!(f, "Anise Error: IOError: {:?}", e),
            Self::IOUnknownError => write!(f, "Anise Error: IOUnknownError"),
            Self::DivisionByZero => write!(f, "Anise Error: DivisionByZero"),
            Self::ParameterNotSpecified => write!(f, "Anise Error: ParameterNotSpecified"),
            Self::IndexingError => write!(f, "Anise Error: IndexingError"),
            Self::NAIFConversionError(reason) => {
                write!(f, "Anise Error: invalid NAIF DAF file: {}", reason)
            }
            Self::InvalidTimeSystem => write!(f, "Anise Error: invalid time system"),
            Self::IntegrityError => write!(
                f,
                "Anise Error: data array checksum verification failed (file is corrupted)"
            ),
            Self::DecodingError(err) => write!(
                f,
                "Anise Error: bytes could not be decoded into a valid ANISE file - {}",
                err
            ),
            Self::IncompatibleVersion{file_version} => write!(
                f,
                "Anise Error: File encoded with ANISE version {}.{}.{} but this library is for version {}.{}.{}",
                file_version.major,
                file_version.minor,
                file_version.patch,
                ANISE_VERSION.major,
                ANISE_VERSION.minor,
                ANISE_VERSION.patch
            )
        }
    }
}

impl std::error::Error for AniseError {}
