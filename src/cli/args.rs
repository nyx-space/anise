use std::path::PathBuf;

use super::clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name="ANISE", author="Rabotin and ANISE contributors", version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Actions,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Subcommand)]
pub enum Actions {
    /// Convert a supported SPICE file into an ANISE file
    Convert {
        /// Set to true to include empty ephemerides in the ANISE file (default: false)
        #[clap(long)]
        allow_empty: bool,
        /// Path to SPICE file
        #[clap(parse(from_os_str))]
        file: PathBuf,
    },
    /// Checks the integrity of the file
    Check {
        /// Path to ANISE file
        #[clap(parse(from_os_str))]
        file: PathBuf,
    },
    /// Inspects what's in an ANISE file (and also checks the integrity)
    Inspect {
        /// Path to ANISE file
        #[clap(parse(from_os_str))]
        file: PathBuf,
    },
    /// Merge several ANISE files together
    Merge {
        /// Input files are all of the files but the last one
        files: Vec<PathBuf>,
    },
}
