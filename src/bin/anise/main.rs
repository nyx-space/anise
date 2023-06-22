extern crate pretty_env_logger;
use std::env::{set_var, var};

use anise::cli::args::{Actions, Args};
use anise::cli::inspect::{BpcRow, SpkRow};
use anise::cli::CliErrors;
use anise::file_mmap;
use anise::naif::daf::{FileRecord, NAIFRecord, NAIFSummaryRecord};
use anise::naif::kpl::parser::convert_tpc;
use anise::prelude::*;
use anise::structure::dataset::{DataSet, DataSetType};
use anise::structure::metadata::Metadata;
use anise::structure::planetocentric::PlanetaryData;
use anise::structure::spacecraft::SpacecraftData;
use clap::Parser;
use log::{error, info};
use tabled::{Style, Table};
use zerocopy::FromBytes;

const LOG_VAR: &str = "ANISE_LOG";

#[cfg(feature = "std")]
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
                    // Try to load this as a dataset by first trying to load the metadata
                    if let Ok(metadata) = Metadata::decode_header(&bytes) {
                        // Now, we can load this depending on the kind of data that it is
                        match metadata.dataset_type {
                            DataSetType::NotApplicable => unreachable!("no such ANISE data yet"),
                            DataSetType::SpacecraftData => {
                                // Decode as spacecraft data
                                let dataset =
                                    DataSet::<SpacecraftData, 64>::try_from_bytes(&bytes)?;
                                println!("{dataset}");
                                Ok(())
                            }
                            DataSetType::PlanetaryData => {
                                // Decode as planetary data
                                let dataset = DataSet::<PlanetaryData, 64>::try_from_bytes(&bytes)?;
                                println!("{dataset}");
                                Ok(())
                            }
                        }
                    } else {
                        // Load the header only
                        let file_record =
                            FileRecord::read_from(&bytes[..FileRecord::SIZE]).unwrap();
                        match file_record
                            .identification()
                            .map_err(CliErrors::AniseError)?
                        {
                            "PCK" => {
                                info!("Loading {path_str:?} as DAF/PCK");
                                match BPC::check_then_parse(bytes, crc32_checksum) {
                                    Ok(_) => {
                                        info!("[OK] Checksum matches");
                                        Ok(())
                                    }
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
                                match SPK::check_then_parse(bytes, crc32_checksum) {
                                    Ok(_) => {
                                        info!("[OK] Checksum matches");
                                        Ok(())
                                    }
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
                }
                Err(e) => Err(e.into()),
            }
        }
        Actions::Inspect { file } => {
            let path_str = file.clone();
            match file_mmap!(file) {
                Ok(bytes) => {
                    // Load the header only
                    let file_record = FileRecord::read_from(&bytes[..FileRecord::SIZE]).unwrap();

                    match file_record
                        .identification()
                        .map_err(CliErrors::AniseError)?
                    {
                        "PCK" => {
                            info!("Loading {path_str:?} as DAF/PCK");
                            match BPC::parse(bytes) {
                                Ok(pck) => {
                                    info!("CRC32 checksum: 0x{:X}", pck.crc32());
                                    if let Some(comments) = pck.comments()? {
                                        println!("== COMMENTS ==\n{}== END ==", comments);
                                    } else {
                                        println!("(File has no comments)");
                                    }
                                    // Build the rows of the table
                                    let mut rows = Vec::new();

                                    for (sno, summary) in
                                        pck.data_summaries().unwrap().iter().enumerate()
                                    {
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
                                    tbl.with(Style::modern());
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
                            match SPK::parse(bytes) {
                                Ok(spk) => {
                                    info!("CRC32 checksum: 0x{:X}", spk.crc32());
                                    if let Some(comments) = spk.comments()? {
                                        println!("== COMMENTS ==\n{}== END ==", comments);
                                    } else {
                                        println!("(File has no comments)");
                                    }
                                    // Build the rows of the table
                                    let mut rows = Vec::new();

                                    for (sno, summary) in
                                        spk.data_summaries().unwrap().iter().enumerate()
                                    {
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
                                    tbl.with(Style::modern());
                                    println!("{tbl}");

                                    Ok(())
                                }
                                Err(e) => {
                                    error!("Some other error happened when loading {path_str:?}: {e:?}");
                                    Err(CliErrors::AniseError(e))
                                }
                            }
                        }
                        fileid => Err(CliErrors::ArgumentError(format!(
                            "{fileid} is not supported yet"
                        ))),
                    }
                }
                Err(e) => Err(e.into()),
            }
        }
        Actions::ConvertTpc {
            pckfile,
            gmfile,
            outfile,
        } => {
            let dataset = convert_tpc(pckfile, gmfile).map_err(CliErrors::AniseError)?;

            dataset.save_as(outfile, false)?;

            Ok(())
        }
    }
}
