use flatbuffers::InvalidFlatbuffer;
use std::convert::From;
use std::fmt;
use std::io::ErrorKind as IOErrorKind;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AniseError {
    InvalidFile(InvalidFlatbuffer),
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
}

impl From<IOErrorKind> for AniseError {
    fn from(e: IOErrorKind) -> Self {
        Self::IOError(e)
    }
}

impl From<InvalidFlatbuffer> for AniseError {
    fn from(e: InvalidFlatbuffer) -> Self {
        Self::InvalidFile(e)
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
            Self::InvalidFile(e) => write!(f, "Anise Error: InvalidFile: {:?}", e),
            Self::NAIFConversionError(reason) => {
                write!(f, "Anise Error: invalid NAIF DAF file: {}", reason)
            }
            Self::InvalidTimeSystem => write!(f, "Anise Error: invalid time system"),
            Self::IntegrityError => write!(
                f,
                "Anise Error: data array checksum verification failed (file is corrupted)"
            ),
        }
    }
}

impl std::error::Error for AniseError {}
