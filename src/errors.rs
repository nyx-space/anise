use std::io::Error as IOError;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AniseError {
    IOError(IOError),
}
