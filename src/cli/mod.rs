extern crate clap;
extern crate tabled;

use snafu::prelude::*;
use std::io;

use crate::{
    naif::daf::{file_record::FileRecordError, DAFError},
    structure::dataset::DataSetError,
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
    CliDAF {
        source: DAFError,
    },
    CliFileRecord {
        source: FileRecordError,
    },
    ArgumentError {
        arg: String,
    },
    CliDataSet {
        source: DataSetError,
    },
}
