extern crate pretty_env_logger;
use std::env::{set_var, var};

use anise::structure::{EulerParameterDataSet, PlanetaryDataSet, SpacecraftDataSet};
use snafu::prelude::*;

use anise::cli::args::{Actions, Args};
use anise::cli::inspect::{BpcRow, SpkRow};
use anise::cli::{AniseSnafu, CliDAFSnafu, CliDataSetSnafu, CliErrors, CliFileRecordSnafu};
use anise::file2heap;
use anise::naif::daf::{FileRecord, NAIFRecord, NAIFSummaryRecord};
use anise::naif::kpl::parser::{convert_fk, convert_tpc};
use anise::prelude::*;
use anise::structure::dataset::DataSetType;
use anise::structure::metadata::Metadata;
use clap::Parser;
use log::info;
use tabled::{settings::Style, Table};
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
            let bytes = file2heap!(file).with_context(|_| AniseSnafu)?;
            // Try to load this as a dataset by first trying to load the metadata
            if let Ok(metadata) = Metadata::decode_header(&bytes) {
                // Now, we can load this depending on the kind of data that it is
                match metadata.dataset_type {
                    DataSetType::NotApplicable => unreachable!("no such ANISE data yet"),
                    DataSetType::SpacecraftData => {
                        // Decode as spacecraft data
                        let dataset = SpacecraftDataSet::try_from_bytes(&bytes)
                            .with_context(|_| CliDataSetSnafu)?;
                        println!("{dataset}");
                        Ok(())
                    }
                    DataSetType::PlanetaryData => {
                        // Decode as planetary data
                        let dataset = PlanetaryDataSet::try_from_bytes(&bytes)
                            .with_context(|_| CliDataSetSnafu)?;
                        println!("{dataset}");
                        Ok(())
                    }
                    DataSetType::EulerParameterData => {
                        // Decode as euler paramater data
                        let dataset = EulerParameterDataSet::try_from_bytes(&bytes)
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
                    // Build the rows of the table
                    let mut rows = Vec::new();

                    for (sno, summary) in pck.data_summaries().unwrap().iter().enumerate() {
                        let name_rcrd = pck.name_record().unwrap();
                        let name =
                            name_rcrd.nth_name(sno, pck.file_record().unwrap().summary_size());
                        if summary.is_empty() {
                            continue;
                        }
                        rows.push(BpcRow {
                            name: name.to_string(),
                            start_epoch: format!("{:E}", summary.start_epoch()),
                            end_epoch: format!("{:E}", summary.end_epoch()),
                            duration: summary.end_epoch() - summary.start_epoch(),
                            interpolation_kind: format!("{}", summary.data_type_i),
                            frame: format!("{}", summary.frame_id),
                            inertial_frame: format!("{}", summary.inertial_frame_id),
                        });
                    }

                    let mut tbl = Table::new(rows);
                    tbl.with(Style::modern());
                    println!("{tbl}");

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
                    // Build the rows of the table
                    let mut rows = Vec::new();

                    for (sno, summary) in spk.data_summaries().unwrap().iter().enumerate() {
                        let name_rcrd = spk.name_record().unwrap();
                        let name =
                            name_rcrd.nth_name(sno, spk.file_record().unwrap().summary_size());
                        if summary.is_empty() {
                            continue;
                        }

                        rows.push(SpkRow {
                            name: name.to_string(),
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
