use std::path::PathBuf;

use clap::{Parser, Subcommand};
use hifitime::Epoch;

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
        file: PathBuf,
        /// CRC32 checksum
        crc32_checksum: u32,
    },
    /// Inspects what's in an ANISE file (and also checks the integrity)
    Inspect {
        /// Path to ANISE or NAIF file
        file: PathBuf,
    },
    /// Convert the provided KPL files into ANISE datasets
    ConvertTpc {
        /// Path to the KPL PCK/TPC file (e.g. pck00008.tpc)
        pckfile: PathBuf,
        /// Path to the KPL gravity data TPC file (e.g. gm_de431.tpc)
        gmfile: PathBuf,
        /// Output ANISE binary file
        outfile: PathBuf,
    },
    /// Convert the provided Frame Kernel into an ANISE dataset
    ConvertFk {
        /// Path to the FK (e.g. moon_080317.fk)
        fkfile: PathBuf,
        /// Output ANISE binary file
        outfile: PathBuf,
    },
    /// Truncate the segment of the provided ID of the input NAIF DAF file to the provided start and end epochs
    /// Limitation: this may not work correctly if there are several segments with the same ID.
    /// Only works with Chebyshev Type 2 data types (i.e. planetary ephemerides).
    TruncDAFById {
        /// Input DAF file, SPK or BPC
        input: PathBuf,
        /// Output DAF file path
        output: PathBuf,
        /// ID of the segment to truncate
        id: i32,
        /// New start epoch of the segment
        start: Option<Epoch>,
        /// New end epoch of the segment
        end: Option<Epoch>,
    },
    /// Rename the segment with the provided "old name" to the "new name" in any DAF file (SPK or BPC).
    RenameDAFSegment {
        /// Input DAF file, SPK or BPC
        input: PathBuf,
        /// Output DAF file path
        output: PathBuf,
        /// Old segment name
        old_name: String,
        /// New segment name
        new_name: String,
    },
}
