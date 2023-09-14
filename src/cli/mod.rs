extern crate clap;
extern crate tabled;

use snafu::prelude::*;
use std::io;

use crate::{
    naif::daf::{file_record::FileRecordError, DAFError},
    prelude::AniseError,
};

pub mod args;

pub mod inspect;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum CliErrors {
    /// File not found or unreadable
    FileNotFound {
        source: io::Error,
    },
    /// ANISE error encountered"
    SomeDAF {
        source: DAFError,
    },
    SomeFileRecord {
        source: FileRecordError,
    },
    ArgumentError {
        arg: String,
    },
    AniseError {
        err: AniseError,
    },
}
