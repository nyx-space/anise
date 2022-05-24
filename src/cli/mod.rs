extern crate clap;
extern crate thiserror;

use std::io;

use thiserror::Error;

use crate::prelude::AniseError;

pub mod args;

#[derive(Debug, Error)]
pub enum CliErrors {
    #[error("File not found or unreadable")]
    FileNotFound(#[from] io::Error),
    #[error("ANISE error encountered")]
    AniseError(#[from] AniseError),
    #[error("{0}")]
    ArgumentError(String),
}
