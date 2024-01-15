extern crate pretty_env_logger;
use std::env::{set_var, var};
use std::io;

use clap::Parser;
use log::info;
use snafu::prelude::*;
use zerocopy::FromBytes;

use anise::file2heap;
use anise::naif::daf::{file_record::FileRecordError, DAFError, FileRecord, NAIFRecord};
use anise::naif::kpl::parser::{convert_fk, convert_tpc};
use anise::prelude::*;
use anise::structure::dataset::{DataSetError, DataSetType};
use anise::structure::metadata::Metadata;
use anise::structure::{EulerParameterDataSet, PlanetaryDataSet, SpacecraftDataSet};

mod args;
use args::{Actions, Args};

const LOG_VAR: &str = "ANISE_LOG";

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
    AniseError {
        source: InputOutputError,
    },
}

fn main() -> Result<(), CliErrors> {
    if var(LOG_VAR).is_err() {
        set_var(LOG_VAR, "INFO");
    }

    if pretty_env_logger::try_init_custom_env(LOG_VAR).is_err() {
        println!("could not init logger");
    }

    let cli = Args::parse();
    match cli.action {
        Actions::Check {
            file,
            crc32_checksum,
        } => {
            let path_str = file.clone();
            let bytes = file2heap!(file).with_context(|_| AniseSnafu)?;
            // Try to load this as a dataset by first trying to load the metadata
            if let Ok(metadata) = Metadata::decode_header(&bytes) {
                // Now, we can load this depending on the kind of data that it is
                match metadata.dataset_type {
                    DataSetType::NotApplicable => unreachable!("no such ANISE data yet"),
                    DataSetType::SpacecraftData => {
                        // Decode as spacecraft data
                        let dataset = SpacecraftDataSet::try_from_bytes(bytes)
                            .with_context(|_| CliDataSetSnafu)?;
                        println!("{dataset}");
                        Ok(())
                    }
                    DataSetType::PlanetaryData => {
                        // Decode as planetary data
                        let dataset = PlanetaryDataSet::try_from_bytes(bytes)
                            .with_context(|_| CliDataSetSnafu)?;
                        println!("{dataset}");
                        Ok(())
                    }
                    DataSetType::EulerParameterData => {
                        // Decode as euler paramater data
                        let dataset = EulerParameterDataSet::try_from_bytes(bytes)
                            .with_context(|_| CliDataSetSnafu)?;
                        println!("{dataset}");
                        Ok(())
                    }
                }
            } else {
                // Load the header only
                let file_record = FileRecord::read_from(&bytes[..FileRecord::SIZE]).unwrap();
                match file_record
                    .identification()
                    .with_context(|_| CliFileRecordSnafu)?
                {
                    "PCK" => {
                        info!("Loading {path_str:?} as DAF/PCK");
                        BPC::check_then_parse(bytes, crc32_checksum)
                            .with_context(|_| CliDAFSnafu)?;
                        info!("[OK] Checksum matches");
                        Ok(())
                    }
                    "SPK" => {
                        info!("Loading {path_str:?} as DAF/SPK");
                        SPK::check_then_parse(bytes, crc32_checksum)
                            .with_context(|_| CliDAFSnafu)?;
                        info!("[OK] Checksum matches");
                        Ok(())
                    }
                    _ => unreachable!(),
                }
            }
        }
        Actions::Inspect { file } => {
            let path_str = file.clone();
            let bytes = file2heap!(file).with_context(|_| AniseSnafu)?;
            // Load the header only
            let file_record = FileRecord::read_from(&bytes[..FileRecord::SIZE]).unwrap();

            match file_record
                .identification()
                .with_context(|_| CliFileRecordSnafu)?
            {
                "PCK" => {
                    info!("Loading {path_str:?} as DAF/PCK");
                    let pck = BPC::parse(bytes).with_context(|_| CliDAFSnafu)?;
                    info!("CRC32 checksum: 0x{:X}", pck.crc32());
                    if let Some(comments) = pck.comments().with_context(|_| CliDAFSnafu)? {
                        println!("== COMMENTS ==\n{}== END ==", comments);
                    } else {
                        println!("(File has no comments)");
                    }
                    println!("{}", pck.describe());
                    Ok(())
                }
                "SPK" => {
                    info!("Loading {path_str:?} as DAF/SPK");
                    let spk = SPK::parse(bytes).with_context(|_| CliDAFSnafu)?;

                    info!("CRC32 checksum: 0x{:X}", spk.crc32());
                    if let Some(comments) = spk.comments().with_context(|_| CliDAFSnafu)? {
                        println!("== COMMENTS ==\n{}== END ==", comments);
                    } else {
                        println!("(File has no comments)");
                    }
                    println!("{}", spk.describe());
                    Ok(())
                }
                fileid => Err(CliErrors::ArgumentError {
                    arg: format!("{fileid} is not supported yet"),
                }),
            }
        }
        Actions::ConvertTpc {
            pckfile,
            gmfile,
            outfile,
        } => {
            let dataset = convert_tpc(pckfile, gmfile).with_context(|_| CliDataSetSnafu)?;

            dataset
                .save_as(&outfile, false)
                .with_context(|_| CliDataSetSnafu)?;

            Ok(())
        }
        Actions::ConvertFk { fkfile, outfile } => {
            let dataset = convert_fk(fkfile, false).unwrap();

            dataset
                .save_as(&outfile, false)
                .with_context(|_| CliDataSetSnafu)?;

            Ok(())
        }
    }
}
