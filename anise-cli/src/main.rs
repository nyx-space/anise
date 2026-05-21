extern crate pretty_env_logger;
use std::collections::HashSet;
use std::env::{set_var, var};
use std::io;
use std::path::PathBuf;

use anise::math::interpolation::InterpolationError;
use anise::naif::daf::datatypes::Type2ChebyshevSet;
use anise::naif::daf::{DAF, DafDataType, NAIFDataSet};
use anise::naif::pck::BPCSummaryRecord;
use anise::naif::pretty_print::NAIFPrettyPrint;
use anise::naif::spk::summary::SPKSummaryRecord;
use bytes::BytesMut;
use clap::Parser;
use log::info;
use snafu::prelude::*;
use zerocopy::FromBytes;

use anise::file2heap;
use anise::naif::daf::{DAFError, FileRecord, NAIFRecord, file_record::FileRecordError};
use anise::naif::kpl::parser::{convert_fk, convert_tpc};
use anise::prelude::*;
use anise::structure::dataset::{DataSetError, DataSetType};
use anise::structure::metadata::Metadata;
use anise::structure::{
    EulerParameterDataSet, LocationDataSet, PlanetaryDataSet, SpacecraftDataSet,
};

use anise::analysis::prelude::*;

mod args;
use args::{Actions, AnalysisArgs, CliArgs};

const LOG_VAR: &str = "ANISE_LOG";

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum CliErrors {
    /// File not found or unreadable
    FileNotFound {
        source: io::Error,
    },
    FilePersist {
        source: io::Error,
    },
    CliDAF {
        source: DAFError,
    },
    CliDataType {
        error: Box<dyn std::error::Error>,
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
    SegmentInterpolation {
        source: InterpolationError,
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

    let cli = CliArgs::parse();
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
                    DataSetType::LocationData => {
                        let dataset =
                            LocationDataSet::try_from_bytes(bytes).context(CliDataSetSnafu)?;
                        println!("{dataset}");
                        Ok(())
                    }
                }
            } else {
                // Load the header only
                let file_record_bytes =
                    bytes
                        .get(..FileRecord::SIZE)
                        .ok_or_else(|| CliErrors::ArgumentError {
                            arg: format!(
                                "file too small: expected at least {} bytes",
                                FileRecord::SIZE
                            ),
                        })?;
                let file_record = FileRecord::read_from_bytes(file_record_bytes)
                    .expect("cannot read bytes that exist");
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
            let (bytes, file_record) = read_and_record(file.clone())?;

            match file_record.identification().context(CliFileRecordSnafu)? {
                "PCK" => inspect::<BPCSummaryRecord>(file, bytes),
                "SPK" => inspect::<SPKSummaryRecord>(file, bytes),
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
            let dataset = convert_fk(fkfile, false).context(CliDataSetSnafu)?;

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

            let (bytes, file_record) = read_and_record(action.input.clone())?;

            match file_record.identification().context(CliFileRecordSnafu)? {
                "PCK" => truncate_daf_by_id::<BPCSummaryRecord>(action, bytes),
                "SPK" => truncate_daf_by_id::<SPKSummaryRecord>(action, bytes),
                fileid => Err(CliErrors::ArgumentError {
                    arg: format!("{fileid} is not supported yet"),
                }),
            }
        }
        Actions::RmDAFById(action) => {
            let (bytes, file_record) = read_and_record(action.input.clone())?;

            match file_record.identification().context(CliFileRecordSnafu)? {
                "PCK" => rm_daf_by_id::<BPCSummaryRecord>(action, bytes),
                "SPK" => rm_daf_by_id::<SPKSummaryRecord>(action, bytes),
                fileid => Err(CliErrors::ArgumentError {
                    arg: format!("{fileid} is not supported yet"),
                }),
            }
        }
        Actions::Analysis(action) => analysis(action),
    }
}

fn read_and_record(path_str: PathBuf) -> Result<(bytes::BytesMut, FileRecord), CliErrors> {
    let bytes = file2heap!(path_str).context(AniseSnafu)?;
    // Load the header only
    let file_record_bytes =
        bytes
            .get(..FileRecord::SIZE)
            .ok_or_else(|| CliErrors::ArgumentError {
                arg: format!(
                    "file too small: expected at least {} bytes",
                    FileRecord::SIZE
                ),
            })?;
    let file_record =
        FileRecord::read_from_bytes(file_record_bytes).expect("cannot read bytes that exist");
    Ok((bytes, file_record))
}

fn inspect<R>(path_str: PathBuf, bytes: BytesMut) -> Result<(), CliErrors>
where
    R: NAIFSummaryRecord,
    DAF<R>: NAIFPrettyPrint,
{
    info!("Loading {path_str:?} as DAF/SPK");
    let fmt = DAF::<R>::parse(bytes).context(CliDAFSnafu)?;

    info!("CRC32 checksum: 0x{:X}", fmt.crc32());
    if let Some(comments) = fmt.comments().context(CliDAFSnafu)? {
        println!("== COMMENTS ==\n{comments}== END ==");
    } else {
        println!("(File has no comments)");
    }
    println!("{}", fmt.describe());
    Ok(())
}

fn rm_daf_by_id<R>(
    args::RmById { input, output, id }: args::RmById,
    bytes: BytesMut,
) -> Result<(), CliErrors>
where
    R: NAIFSummaryRecord,
{
    info!("Loading {input:?} as DAF/PCK");
    let fmt = DAF::<R>::parse(bytes).context(CliDAFSnafu)?;

    let mut ids = HashSet::new();
    for summary in fmt.data_summaries(None).context(CliDAFSnafu)? {
        ids.insert(summary.id());
    }

    info!("IDs present in file: {ids:?}");

    let (_, _, idx) = fmt.summary_from_id(id).context(CliDAFSnafu)?;

    let mut my_fmt_mut = fmt.clone();
    my_fmt_mut.delete_nth_data(idx).context(CliDAFSnafu)?;

    info!("Saving file to {output:?}");
    my_fmt_mut.persist(output).context(CliDAFSnafu)?;

    Ok(())
}

fn truncate_daf_by_id<R>(
    args::TruncateById {
        input,
        output,
        id,
        start,
        end,
    }: args::TruncateById,
    bytes: BytesMut,
) -> Result<(), CliErrors>
where
    R: NAIFSummaryRecord,
{
    info!("Loading {input:?} as DAF/PCK");
    let fmt = DAF::<R>::parse(bytes).context(CliDAFSnafu)?;

    let mut ids = HashSet::new();
    for summary in fmt.data_summaries(None).context(CliDAFSnafu)? {
        ids.insert(summary.id());
    }

    info!("IDs present in file: {ids:?}");

    let (summary, daf_idx, idx) = fmt.summary_from_id(id).context(CliDAFSnafu)?;

    let data_type = summary.data_type().map_err(|err| CliErrors::CliDataType {
        error: Box::new(err),
    })?;
    ensure!(
        data_type == DafDataType::Type2ChebyshevTriplet,
        ArgumentSnafu {
            arg: format!(
                "{input:?} is of type {data_type:?}, but operation is only valid for Type2ChebyshevTriplet"
            )
        }
    );

    let segment = fmt
        .nth_data::<Type2ChebyshevSet>(daf_idx, idx)
        .context(CliDAFSnafu)?;

    let updated_segment = segment
        .truncate(summary, start, end)
        .context(SegmentInterpolationSnafu)?;

    let mut my_pck_mut = fmt.clone();
    assert!(
        my_pck_mut
            .set_nth_data(
                idx,
                updated_segment,
                start.unwrap_or_else(|| summary.start_epoch()),
                end.unwrap_or_else(|| summary.end_epoch()),
            )
            .is_ok()
    );

    info!("Saving file to {output:?}");
    my_pck_mut.persist(output).context(CliDAFSnafu)?;

    Ok(())
}

fn analysis(args: AnalysisArgs) -> Result<(), CliErrors> {
    let mut almanac = Almanac::default();
    for kernel in args.kernels {
        info!("Loading kernel {kernel:?}");
        almanac = almanac.load(kernel.to_str().unwrap()).map_err(|_e| {
            CliErrors::AniseError {
                source: InputOutputError::IOUnknownError,
            }
        })?;
    }

    let expression_str = std::fs::read_to_string(&args.expression).map_err(|e| CliErrors::FileNotFound { source: e })?;
    let report = ReportScalars::from_s_expr(&expression_str).map_err(|e| {
        CliErrors::CliDataType {
            error: Box::new(e),
        }
    })?;

    let step = hifitime::Duration::from_str(&args.step).map_err(|e| {
        CliErrors::CliDataType {
            error: e.into(),
        }
    })?;

    let time_series = TimeSeries::inclusive(args.start, args.end, step);

    let table = almanac.report_scalars_flat(&report, time_series).map_err(|e| {
        CliErrors::CliDataType {
            error: Box::new(e),
        }
    })?;

    if args.output.extension().map_or(false, |ext| ext == "parquet") {
        #[cfg(feature = "parquet")]
        {
            table.to_parquet(args.output).map_err(|e| {
                CliErrors::CliDataType {
                    error: e,
                }
            })?;
        }
        #[cfg(not(feature = "parquet"))]
        {
             return Err(CliErrors::ArgumentError {
                arg: "Parquet support is not enabled in this build of ANISE".to_string(),
            });
        }
    } else {
        table.to_csv(args.output).map_err(|e| {
            CliErrors::CliDataType {
                error: e,
            }
        })?;
    }

    Ok(())
}
