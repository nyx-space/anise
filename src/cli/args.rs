use std::path::PathBuf;

use super::clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name="ANISE", author="Rabotin et ANISE contributors", version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Actions,
}

#[derive(Debug, PartialEq, PartialOrd, Subcommand)]
pub enum Actions {
    /// Convert a supported SPICE file into an ANISE file
    Convert {
        /// Path to SPICE file
        #[clap(parse(from_os_str))]
        file: PathBuf,
    },
    /// Merge several ANISE files together
    Merge {
        /// Input files are all of the files but the last one
        files: Vec<PathBuf>,
    },
}
