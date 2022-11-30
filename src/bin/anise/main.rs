extern crate pretty_env_logger;
use std::env::{set_var, var};

use anise::cli::args::{Actions, Args};
use anise::cli::inspect::{BpcRow, SpkRow};
use anise::cli::CliErrors;
use anise::file_mmap;
use anise::naif::daf::{DAFFileRecord, NAIFRecord, NAIFSummaryRecord};
use anise::prelude::*;
use clap::Parser;
use log::{error, info};
use tabled::{Style, Table};
use zerocopy::FromBytes;

const LOG_VAR: &str = "ANISE_LOG";

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
            match file_mmap!(file) {
                Ok(bytes) => {
                    // Load the header only
                    let file_record =
                        DAFFileRecord::read_from(&bytes[..DAFFileRecord::SIZE]).unwrap();
                    match file_record
                        .identification()
                        .map_err(CliErrors::AniseError)?
                    {
                        "PCK" => {
                            info!("Loading {path_str:?} as DAF/PCK");
                            match BPC::check_then_parse(&bytes, crc32_checksum) {
                                Ok(_) => Ok(()),
                                Err(AniseError::IntegrityError(e)) => {
                                    error!("CRC32 checksums differ for {path_str:?}: {e:?}");
                                    Err(CliErrors::AniseError(AniseError::IntegrityError(e)))
                                }
                                Err(e) => {
                                    error!("Some other error happened when loading {path_str:?}: {e:?}");
                                    Err(CliErrors::AniseError(e))
                                }
                            }
                        }
                        "SPK" => {
                            info!("Loading {path_str:?} as DAF/SPK");
                            match SPK::check_then_parse(&bytes, crc32_checksum) {
                                Ok(_) => Ok(()),
                                Err(AniseError::IntegrityError(e)) => {
                                    error!("CRC32 checksums differ for {path_str:?}: {e:?}");
                                    Err(CliErrors::AniseError(AniseError::IntegrityError(e)))
                                }
                                Err(e) => {
                                    error!("Some other error happened when loading {path_str:?}: {e:?}");
                                    Err(CliErrors::AniseError(e))
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Err(e) => Err(e.into()),
            }
        }
        Actions::Inspect { file } => {
            let path_str = file.clone();
            match file_mmap!(file) {
                Ok(bytes) => {
                    // Load the header only
                    let file_record =
                        DAFFileRecord::read_from(&bytes[..DAFFileRecord::SIZE]).unwrap();

                    match file_record
                        .identification()
                        .map_err(CliErrors::AniseError)?
                    {
                        "PCK" => {
                            info!("Loading {path_str:?} as DAF/PCK");
                            match BPC::parse(&bytes) {
                                Ok(pck) => {
                                    info!("CRC32 checksum: 0x{:X}", pck.crc32());
                                    // Build the rows of the table
                                    let mut rows = Vec::new();

                                    for (sno, summary) in pck.data_summaries.iter().enumerate() {
                                        let name = pck
                                            .name_record
                                            .nth_name(sno, pck.file_record.summary_size());
                                        if summary.is_empty() {
                                            continue;
                                        }
                                        rows.push(BpcRow {
                                            name,
                                            start_epoch: format!("{:E}", summary.start_epoch()),
                                            end_epoch: format!("{:E}", summary.end_epoch()),
                                            duration: summary.end_epoch() - summary.start_epoch(),
                                            interpolation_kind: format!("{}", summary.data_type_i),
                                            frame: format!("{}", summary.frame_id),
                                            inertial_frame: format!(
                                                "{}",
                                                summary.inertial_frame_id
                                            ),
                                        });
                                    }

                                    let mut tbl = Table::new(rows);
                                    tbl.with(Style::rounded());
                                    println!("{tbl}");

                                    Ok(())
                                }
                                Err(e) => {
                                    error!("Some other error happened when loading {path_str:?}: {e:?}");
                                    Err(CliErrors::AniseError(e))
                                }
                            }
                        }
                        "SPK" => {
                            info!("Loading {path_str:?} as DAF/SPK");
                            match SPK::parse(&bytes) {
                                Ok(spk) => {
                                    info!("CRC32 checksum: 0x{:X}", spk.crc32());
                                    // Build the rows of the table
                                    let mut rows = Vec::new();

                                    for (sno, summary) in spk.data_summaries.iter().enumerate() {
                                        let name = spk
                                            .name_record
                                            .nth_name(sno, spk.file_record.summary_size());
                                        if summary.is_empty() {
                                            continue;
                                        }

                                        rows.push(SpkRow {
                                            name,
                                            center: summary.center_id,
                                            start_epoch: format!("{:E}", summary.start_epoch()),
                                            end_epoch: format!("{:E}", summary.end_epoch()),
                                            duration: summary.end_epoch() - summary.start_epoch(),
                                            interpolation_kind: format!("{}", summary.data_type_i),
                                            frame: format!("{}", summary.frame_id),
                                            target: format!("{}", summary.target_id),
                                        });
                                    }

                                    let mut tbl = Table::new(rows);
                                    tbl.with(Style::rounded());
                                    println!("{tbl}");

                                    Ok(())
                                }
                                Err(e) => {
                                    error!("Some other error happened when loading {path_str:?}: {e:?}");
                                    Err(CliErrors::AniseError(e))
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Err(e) => Err(e.into()),
            }
        }
    }
}
