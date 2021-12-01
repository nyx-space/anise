use std::convert::From;
use std::io::ErrorKind as IOErrorKind;

#[derive(Copy, Clone, PartialEq, Debug)]
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
}

impl From<IOErrorKind> for AniseError {
    fn from(e: IOErrorKind) -> Self {
        Self::IOError(e)
    }
}
