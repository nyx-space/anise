extern crate pretty_env_logger;
use std::collections::HashSet;
use std::env::{set_var, var};
use std::io;

use anise::naif::daf::datatypes::Type2ChebyshevSet;
use anise::naif::daf::{DafDataType, NAIFDataSet, DAF};
use anise::naif::pck::BPCSummaryRecord;
use anise::naif::spk::summary::SPKSummaryRecord;
use bytes::Bytes;
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
        unsafe {
            set_var(LOG_VAR, "INFO");
        }
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
            let bytes = file2heap!(file).context(AniseSnafu)?;
            // Try to load this as a dataset by first trying to load the metadata
            if let Ok(metadata) = Metadata::decode_header(&bytes) {
                // Now, we can load this depending on the kind of data that it is
                match metadata.dataset_type {
                    DataSetType::NotApplicable => unreachable!("no such ANISE data yet"),
                    DataSetType::SpacecraftData => {
                        // Decode as spacecraft data
                        let dataset =
                            SpacecraftDataSet::try_from_bytes(bytes).context(CliDataSetSnafu)?;
                        println!("{dataset}");
                        Ok(())
                    }
                    DataSetType::PlanetaryData => {
                        // Decode as planetary data
                        let dataset =
                            PlanetaryDataSet::try_from_bytes(bytes).context(CliDataSetSnafu)?;
                        println!("{dataset}");
                        Ok(())
                    }
                    DataSetType::EulerParameterData => {
                        // Decode as euler parameter data
                        let dataset = EulerParameterDataSet::try_from_bytes(bytes)
                            .context(CliDataSetSnafu)?;
                        println!("{dataset}");
                        Ok(())
                    }
                }
            } else {
                // Load the header only
                let file_record = FileRecord::read_from(&bytes[..FileRecord::SIZE]).unwrap();
                match file_record.identification().context(CliFileRecordSnafu)? {
                    "PCK" => {
                        info!("Loading {path_str:?} as DAF/PCK");
                        BPC::check_then_parse(bytes, crc32_checksum).context(CliDAFSnafu)?;
                        info!("[OK] Checksum matches");
                        Ok(())
                    }
                    "SPK" => {
                        info!("Loading {path_str:?} as DAF/SPK");
                        SPK::check_then_parse(bytes, crc32_checksum).context(CliDAFSnafu)?;
                        info!("[OK] Checksum matches");
                        Ok(())
                    }
                    _ => unreachable!(),
                }
            }
        }
        Actions::Inspect { file } => {
            let path_str = file.clone();
            let bytes = file2heap!(file).context(AniseSnafu)?;
            // Load the header only
            let file_record = FileRecord::read_from(&bytes[..FileRecord::SIZE]).unwrap();

            match file_record.identification().context(CliFileRecordSnafu)? {
                "PCK" => {
                    info!("Loading {path_str:?} as DAF/PCK");
                    let pck = BPC::parse(bytes).context(CliDAFSnafu)?;
                    info!("CRC32 checksum: 0x{:X}", pck.crc32());
                    if let Some(comments) = pck.comments().context(CliDAFSnafu)? {
                        println!("== COMMENTS ==\n{}== END ==", comments);
                    } else {
                        println!("(File has no comments)");
                    }
                    println!("{}", pck.describe());
                    Ok(())
                }
                "SPK" => {
                    info!("Loading {path_str:?} as DAF/SPK");
                    let spk = SPK::parse(bytes).context(CliDAFSnafu)?;

                    info!("CRC32 checksum: 0x{:X}", spk.crc32());
                    if let Some(comments) = spk.comments().context(CliDAFSnafu)? {
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
            let dataset = convert_tpc(pckfile, gmfile).context(CliDataSetSnafu)?;

            dataset.save_as(&outfile, false).context(CliDataSetSnafu)?;

            Ok(())
        }
        Actions::ConvertFk { fkfile, outfile } => {
            let dataset = convert_fk(fkfile, false).unwrap();

            dataset.save_as(&outfile, false).context(CliDataSetSnafu)?;

            Ok(())
        }
        Actions::TruncDAFById(action) => {
            ensure!(
                action.start.is_some() || action.end.is_some(),
                ArgumentSnafu {
                    arg: "you must provide either START or END, or both"
                }
            );

            let input = action.input.clone();
            let bytes = file2heap!(input).context(AniseSnafu)?;
            // Load the header only
            let file_record = FileRecord::read_from(&bytes[..FileRecord::SIZE]).unwrap();

            match file_record.identification().context(CliFileRecordSnafu)? {
                "PCK" => truncate_daf_by_id::<BPCSummaryRecord>(action, bytes),
                "SPK" => truncate_daf_by_id::<SPKSummaryRecord>(action, bytes),
                fileid => Err(CliErrors::ArgumentError {
                    arg: format!("{fileid} is not supported yet"),
                }),
            }
        }
        Actions::RmDAFById { input, output, id } => {
            let path_str = input.clone();
            let bytes = file2heap!(input).context(AniseSnafu)?;
            // Load the header only
            let file_record = FileRecord::read_from(&bytes[..FileRecord::SIZE]).unwrap();

            match file_record.identification().context(CliFileRecordSnafu)? {
                "PCK" => {
                    info!("Loading {path_str:?} as DAF/PCK");
                    let pck = BPC::parse(bytes).context(CliDAFSnafu)?;

                    let mut ids = HashSet::new();
                    for summary in pck.data_summaries().context(CliDAFSnafu)? {
                        ids.insert(summary.id());
                    }

                    info!("IDs present in file: {ids:?}");

                    let (_, idx) = pck.summary_from_id(id).context(CliDAFSnafu)?;

                    let mut my_pck_mut = pck.to_mutable();
                    my_pck_mut.delete_nth_data(idx).context(CliDAFSnafu)?;

                    info!("Saving file to {output:?}");
                    my_pck_mut.persist(output).unwrap();

                    Ok(())
                }
                "SPK" => {
                    info!("Loading {path_str:?} as DAF/PCK");
                    let spk = SPK::parse(bytes).context(CliDAFSnafu)?;

                    let mut ids = HashSet::new();
                    for summary in spk.data_summaries().context(CliDAFSnafu)? {
                        ids.insert(summary.id());
                    }

                    info!("IDs present in file: {ids:?}");

                    let (_, idx) = spk.summary_from_id(id).context(CliDAFSnafu)?;

                    let mut my_spk_mut = spk.to_mutable();
                    my_spk_mut.delete_nth_data(idx).context(CliDAFSnafu)?;

                    info!("Saving file to {output:?}");
                    my_spk_mut.persist(output).unwrap();

                    Ok(())
                }
                fileid => Err(CliErrors::ArgumentError {
                    arg: format!("{fileid} is not supported yet"),
                }),
            }
        }
    }
}

fn truncate_daf_by_id<R>(
    args::TruncateById {
        input,
        output,
        id,
        start,
        end,
    }: args::TruncateById,
    bytes: Bytes,
) -> Result<(), CliErrors>
where
    R: NAIFSummaryRecord,
{
    info!("Loading {input:?} as DAF/PCK");
    let fmt = DAF::<R>::parse(bytes).context(CliDAFSnafu)?;

    let mut ids = HashSet::new();
    for summary in fmt.data_summaries().context(CliDAFSnafu)? {
        ids.insert(summary.id());
    }

    info!("IDs present in file: {ids:?}");

    let (summary, idx) = fmt.summary_from_id(id).context(CliDAFSnafu)?;

    let data_type = DafDataType::try_from(summary.data_type_i()).context(CliDAFSnafu)?;
    ensure!(
                        data_type == DafDataType::Type2ChebyshevTriplet,
                        ArgumentSnafu {
                            arg: format!("{input:?} is of type {data_type:?}, but operation is only valid for Type2ChebyshevTriplet")
                        }
                    );

    let segment = fmt.nth_data::<Type2ChebyshevSet>(idx).unwrap();

    let updated_segment = segment.truncate(summary, start, end).unwrap();

    let mut my_pck_mut = fmt.to_mutable();
    assert!(my_pck_mut
        .set_nth_data(
            idx,
            updated_segment,
            start.or_else(|| Some(summary.start_epoch())).unwrap(),
            end.or_else(|| Some(summary.end_epoch())).unwrap(),
        )
        .is_ok());

    info!("Saving file to {output:?}");
    my_pck_mut.persist(output).unwrap();

    Ok(())
}
