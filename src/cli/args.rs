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
    /// Checks the integrity of the file
    Check {
        /// Path to ANISE file
        #[clap(parse(from_os_str))]
        file: PathBuf,
        /// CRC32 checksum
        crc32_checksum: u32,
    },
    /// Inspects what's in an ANISE file (and also checks the integrity)
    Inspect {
        /// Path to ANISE file
        #[clap(parse(from_os_str))]
        file: PathBuf,
    },
}
