/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{EphemerisError, OEMTimeParsingSnafu};
use crate::math::{Matrix6, Vector6};
use crate::naif::daf::data_types::DataType;
use crate::prelude::{Frame, Orbit};
use hifitime::{
    Epoch,
    efmt::{Format, Formatter},
};
use log::warn;
use snafu::ResultExt;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;

use super::{Covariance, Ephemeris, EphemerisRecord, EphemerisSegment, LocalFrame};

type MetadataEpoch = Option<String>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OemParserState {
    Header,
    Metadata,
    States,
    Covariance,
}

fn parse_metadata_epoch(
    value: &MetadataEpoch,
    time_system: &str,
    field: &'static str,
    line: usize,
) -> Result<Option<Epoch>, EphemerisError> {
    value
        .as_ref()
        .map(|value| {
            let epoch_str = format!("{value} {time_system}");
            Epoch::from_str(epoch_str.trim()).context(OEMTimeParsingSnafu {
                line,
                details: format!("`{epoch_str}` for {field}"),
            })
        })
        .transpose()
}

#[allow(clippy::too_many_arguments)]
fn finish_segment(
    segments: &mut Vec<EphemerisSegment>,
    state_data: &mut BTreeMap<Epoch, EphemerisRecord>,
    interpolation: DataType,
    degree: usize,
    time_system: &str,
    useable_start: &MetadataEpoch,
    useable_end: &MetadataEpoch,
    lno: usize,
) -> Result<(), EphemerisError> {
    if state_data.is_empty() {
        return Ok(());
    }

    let raw_start = *state_data
        .first_key_value()
        .expect("state_data is not empty")
        .0;
    let raw_end = *state_data
        .last_key_value()
        .expect("state_data is not empty")
        .0;
    let (useable_start, useable_end) = match (useable_start, useable_end) {
        (None, None) => (raw_start, raw_end),
        (Some(_), Some(_)) => (
            parse_metadata_epoch(useable_start, time_system, "USEABLE_START_TIME", lno)?
                .expect("Some metadata epoch parses to Some"),
            parse_metadata_epoch(useable_end, time_system, "USEABLE_STOP_TIME", lno)?
                .expect("Some metadata epoch parses to Some"),
        ),
        _ => {
            return Err(EphemerisError::OEMParsingError {
                lno: 0,
                details: "USEABLE_START_TIME and USEABLE_STOP_TIME must be provided together"
                    .to_string(),
            });
        }
    };

    if useable_start >= useable_end {
        return Err(EphemerisError::OEMParsingError {
            lno: 0,
            details: "USEABLE_START_TIME must be strictly before USEABLE_STOP_TIME".to_string(),
        });
    }
    if let Some(previous) = segments.last()
        && previous
            .useable_end
            .expect("parsed segments have a useable end")
            > useable_start
    {
        return Err(EphemerisError::OEMParsingError {
            lno: 0,
            details: "OEM useable intervals may share one endpoint but must not overlap"
                .to_string(),
        });
    }

    segments.push(EphemerisSegment {
        interpolation,
        degree,
        useable_start: Some(useable_start),
        useable_end: Some(useable_end),
        state_data: std::mem::take(state_data),
    });
    Ok(())
}

impl Ephemeris {
    /// Initialize a new ephemeris from the path to a CCSDS OEM file.
    pub fn from_ccsds_oem_file<P: AsRef<Path>>(path: P) -> Result<Self, EphemerisError> {
        // Open the file
        let file = File::open(path).map_err(|e| EphemerisError::OEMParsingError {
            lno: 0,
            details: format!("could not open file: {e}"),
        })?;

        let reader = BufReader::new(file);

        let mut parser_state = OemParserState::Header;

        // Define header variables we care about.
        let mut time_system = String::new();
        let mut message_time_system: Option<String> = None;
        let mut center_name = None;
        let mut orient_name = None;
        let mut interpolation = DataType::Type9LagrangeUnequalStep;
        let mut degree = 5;
        let mut useable_start: MetadataEpoch = None;
        let mut useable_end: MetadataEpoch = None;
        let mut object_id: Option<String> = None;
        let mut cov_epoch = None;
        let mut cov_mat = None;
        let mut cov_frame = None;
        let mut cov_row = 0;

        // Store the temporary data in a BTreeMap so we have O(1) access when adding the covariance information
        // and we can iterate in order when building the vector.
        let mut segment_state_data = BTreeMap::new();
        let mut segments = Vec::new();

        let parse_one_val = |lno: usize, line: &str, err: &str| -> Result<String, EphemerisError> {
            let parts: Vec<&str> = line.split('=').collect();

            match parts.get(1) {
                Some(val_str) => Ok(val_str.trim().to_string()),
                None => Err(EphemerisError::OEMParsingError {
                    lno,
                    details: err.to_string(),
                }),
            }
        };

        for (lno, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| EphemerisError::OEMParsingError {
                lno,
                details: format!("could not read line: {e}"),
            })?;

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Track metadata blocks explicitly. Without this state, a dangling
            // META_START at EOF is silently dropped, while metadata for a later
            // block that omits META_START can mutate and merge into the active one.
            if line.starts_with("META_START") {
                match parser_state {
                    OemParserState::Metadata => {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: "nested META_START is not allowed".to_string(),
                        });
                    }
                    OemParserState::Covariance => {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: "META_START cannot abandon an open covariance section"
                                .to_string(),
                        });
                    }
                    OemParserState::Header | OemParserState::States => {}
                }
                finish_segment(
                    &mut segments,
                    &mut segment_state_data,
                    interpolation,
                    degree,
                    &time_system,
                    &useable_start,
                    &useable_end,
                    lno,
                )?;
                parser_state = OemParserState::Metadata;
                center_name = None;
                orient_name = None;
                time_system.clear();
                interpolation = DataType::Type9LagrangeUnequalStep;
                degree = 5;
                useable_start = None;
                useable_end = None;
                continue;
            }
            if line.starts_with("META_STOP") {
                if parser_state != OemParserState::Metadata {
                    return Err(EphemerisError::OEMParsingError {
                        lno,
                        details: "META_STOP without META_START".to_string(),
                    });
                }
                parser_state = OemParserState::States;
                continue;
            }

            let metadata_key = line.split_once('=').map(|(key, _)| key.trim());
            if matches!(
                metadata_key,
                Some(
                    "OBJECT_NAME"
                        | "OBJECT_ID"
                        | "CENTER_NAME"
                        | "REF_FRAME"
                        | "TIME_SYSTEM"
                        | "START_TIME"
                        | "USEABLE_START_TIME"
                        | "USEABLE_STOP_TIME"
                        | "STOP_TIME"
                        | "INTERPOLATION"
                        | "INTERPOLATION_DEGREE"
                )
            ) && parser_state != OemParserState::Metadata
            {
                return Err(EphemerisError::OEMParsingError {
                    lno,
                    details: format!(
                        "metadata field {} appears outside META_START/META_STOP",
                        metadata_key.expect("matched metadata key")
                    ),
                });
            }

            if line.starts_with("CCSDS_OEM_VERS") {
                let version_str = parse_one_val(lno, line, "no value for CCSDS_OEM_VERS")?;
                match version_str.parse::<f32>() {
                    Ok(version_val) => match version_val as i16 {
                        1..=3 => {}
                        _ => {
                            return Err(EphemerisError::OEMParsingError {
                                lno,
                                details: "CCSDS OEM version {version_val} not supported"
                                    .to_string(),
                            });
                        }
                    },
                    Err(_) => {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: format!("could not parse OEM version `{version_str}`"),
                        });
                    }
                }
            }
            if line.starts_with("OBJECT_ID") {
                // Extract the object ID from the line
                let oem_obj_id = parse_one_val(lno, line, "no value for OBJECT_ID")?;
                if let Some(prev_obj_id) = &object_id {
                    if oem_obj_id != *prev_obj_id {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: format!(
                                "OEM must have only one object: `{prev_obj_id}` != `{oem_obj_id}`"
                            ),
                        });
                    }
                } else {
                    object_id = Some(oem_obj_id);
                }
            } else if line.starts_with("CENTER_NAME") {
                center_name = Some(parse_one_val(lno, line, "no value for CENTER")?);
            } else if line.starts_with("REF_FRAME") {
                orient_name = Some(parse_one_val(lno, line, "no value for REF_FRAME")?);
            } else if line.starts_with("TIME_SYSTEM") {
                let parsed = parse_one_val(lno, line, "no value for TIME_SYSTEM")?;
                if let Some(expected) = &message_time_system {
                    if parsed != *expected {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: format!(
                                "TIME_SYSTEM must remain {expected} throughout the OEM, found {parsed}"
                            ),
                        });
                    }
                } else {
                    message_time_system = Some(parsed.clone());
                }
                time_system = parsed;
            } else if line.starts_with("USEABLE_START_TIME") {
                useable_start = Some(parse_one_val(lno, line, "no value for USEABLE_START_TIME")?);
            } else if line.starts_with("USEABLE_STOP_TIME") {
                useable_end = Some(parse_one_val(lno, line, "no value for USEABLE_STOP_TIME")?);
            } else if line.starts_with("INTERPOLATION_DEGREE") {
                let interp_str =
                    parse_one_val(lno, line, "no value for INTERPOLATION_DEGREE")?.to_lowercase();

                match interp_str.parse::<usize>() {
                    // A degree of zero leaves no samples to interpolate with, and the SPK
                    // writer subtracts one from it, so reject it here like the Python setter does.
                    Ok(0) => {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: "INTERPOLATION_DEGREE must be strictly positive".to_string(),
                        });
                    }
                    Ok(ideg) => degree = ideg,
                    Err(_) => {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: format!("could not parse `{interp_str}` as float"),
                        });
                    }
                }
            } else if line.starts_with("INTERPOLATION") {
                let interp_str =
                    parse_one_val(lno, line, "no value for INTERPOLATION")?.to_lowercase();

                match interp_str.as_str() {
                    "lagrange" => interpolation = DataType::Type9LagrangeUnequalStep,
                    "hermite" => interpolation = DataType::Type13HermiteUnequalStep,
                    _ => {
                        warn!("unsupported interpolation `{interp_str}` using Hermite")
                    }
                };
            } else if line.starts_with("COVARIANCE_START") {
                match parser_state {
                    OemParserState::Metadata => {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: "COVARIANCE_START cannot appear inside metadata".to_string(),
                        });
                    }
                    OemParserState::Covariance => {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: "nested COVARIANCE_START is not allowed".to_string(),
                        });
                    }
                    OemParserState::Header => {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: "COVARIANCE_START must follow an OEM state block".to_string(),
                        });
                    }
                    OemParserState::States => {}
                }
                parser_state = OemParserState::Covariance;
                // Start each block from a clean slate so a stray data row before this
                // block's own EPOCH line cannot latch onto a previous block's matrix.
                cov_epoch = None;
                cov_mat = None;
                cov_frame = None;
                cov_row = 0;
            } else if line.starts_with("COVARIANCE_STOP") {
                if parser_state != OemParserState::Covariance {
                    return Err(EphemerisError::OEMParsingError {
                        lno,
                        details: "COVARIANCE_STOP without COVARIANCE_START".to_string(),
                    });
                }
                if cov_epoch.is_some() {
                    return Err(EphemerisError::OEMParsingError {
                        lno,
                        details: format!(
                            "incomplete covariance matrix: expected six rows but got {cov_row}"
                        ),
                    });
                }
                parser_state = OemParserState::Header;
            } else if line.starts_with("COMMENT") {
                // Ignore
            } else if parser_state == OemParserState::States {
                let center_name_str =
                    center_name
                        .as_ref()
                        .ok_or_else(|| EphemerisError::OEMParsingError {
                            lno,
                            details: "CENTER_NAME not found in metadata".to_string(),
                        })?;
                // Capitalize the center name
                let center_name = center_name_str
                    .split_whitespace()
                    .map(|word| {
                        let word = word.to_lowercase();
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ");

                let orient_name_str =
                    orient_name
                        .as_ref()
                        .ok_or_else(|| EphemerisError::OEMParsingError {
                            lno,
                            details: "REF_FRAME not found in metadata".to_string(),
                        })?;

                let frame =
                    Frame::from_name(center_name.as_str(), orient_name_str).map_err(|e| {
                        EphemerisError::OEMParsingError {
                            lno,
                            details: format!("frame error `{center_name:?} {orient_name:?}`: {e}"),
                        }
                    })?;

                // Split the line into components
                let parts: Vec<&str> = line.split_whitespace().collect();
                let mut state_vec = Vector6::zeros();

                // Build the epoch
                let epoch = match parts.first() {
                    Some(state_epoch) => {
                        let epoch_str = format!("{state_epoch} {time_system}");
                        Epoch::from_str(epoch_str.trim()).context(OEMTimeParsingSnafu {
                            line: lno,
                            details: format!("`{epoch_str}` for state epoch"),
                        })?
                    }
                    None => {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: "no `=` sign for covariance epoch".to_string(),
                        });
                    }
                };

                // Convert the state data
                for i in 0..6 {
                    match parts.get(i + 1) {
                        Some(val_str) => match val_str.trim().parse::<f64>() {
                            Ok(val_f64) => {
                                state_vec[i] = val_f64;
                            }
                            Err(_) => {
                                return Err(EphemerisError::OEMParsingError {
                                    lno,
                                    details: format!(
                                        "could not parse `{}` as float",
                                        val_str.trim()
                                    ),
                                });
                            }
                        },
                        None => {
                            return Err(EphemerisError::OEMParsingError {
                                lno,
                                details: format!("missing float in position {}", i + 1),
                            });
                        }
                    };
                }

                // We only reach this point if the state data is fully parsed.
                let orbit = Orbit::from_cartesian_pos_vel(state_vec, epoch, frame);
                let record = EphemerisRecord { orbit, covar: None };
                segment_state_data.insert(epoch, record);
            } else if parser_state == OemParserState::Covariance {
                if line.starts_with("EPOCH") {
                    if cov_epoch.is_some() {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: format!(
                                "incomplete covariance matrix before next EPOCH: expected six rows but got {cov_row}"
                            ),
                        });
                    }
                    let state_epoch = parse_one_val(lno, line, "no `=` sign for covariance epoch")?;
                    let epoch_str = format!("{state_epoch} {time_system}");
                    let epoch = Epoch::from_str(epoch_str.trim()).context(OEMTimeParsingSnafu {
                        line: lno,
                        details: format!("`{epoch_str}` for covariance epoch"),
                    })?;

                    // Check that we have associated state data
                    if !segment_state_data.contains_key(&epoch) {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: format!(
                                "cannot have covariance data at {epoch} because no orbit data at that epoch"
                            ),
                        });
                    }

                    cov_epoch = Some(epoch);
                    cov_mat = Some(Matrix6::zeros());
                    cov_row = 0;
                } else if line.starts_with("COV_REF_FRAME") {
                    // Only do a check here, nothing to set.
                    let cov_frame_str = parse_one_val(lno, line, "invalid COV_REF_FRAME token")?;
                    match cov_frame_str.as_str() {
                        "EME2000" | "ICRF" => cov_frame = Some(LocalFrame::Inertial),
                        "RSW" | "RTN" => cov_frame = Some(LocalFrame::RIC),
                        "TNW" => cov_frame = Some(LocalFrame::VNC),
                        _ => {
                            return Err(EphemerisError::OEMParsingError {
                                lno,
                                details: format!("invalid COV_REF_FRAME `{cov_frame_str}`"),
                            });
                        }
                    };
                } else {
                    // Matrix data!
                    // A covariance row is only meaningful once an EPOCH line has set up
                    // the target epoch and its destination matrix. A stray data row before
                    // any EPOCH would otherwise unwrap the still-empty matrix and panic.
                    if cov_epoch.is_none() {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: "covariance data appears before its EPOCH line".to_string(),
                        });
                    }
                    // A covariance block only holds the lower triangle of a 6x6 matrix,
                    // so a seventh data row would index past it. Reject it here rather
                    // than letting the matrix write panic.
                    if cov_row >= 6 {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: "too many covariance rows, expected six for a 6x6 matrix"
                                .to_string(),
                        });
                    }
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() != cov_row + 1 {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: format!(
                                "expected {} values for covariance row {cov_row} but got {}",
                                cov_row + 1,
                                parts.len()
                            ),
                        });
                    }

                    for col in 0..cov_row + 1 {
                        match parts.get(col) {
                            Some(val_str) => match val_str.trim().parse::<f64>() {
                                Ok(val_f64) => {
                                    let mat = cov_mat
                                        .as_mut()
                                        .expect("cov_mat is initialized when cov_epoch is set");
                                    mat[(col, cov_row)] = val_f64;
                                    mat[(cov_row, col)] = val_f64;
                                }
                                Err(_) => {
                                    return Err(EphemerisError::OEMParsingError {
                                        lno,
                                        details: format!(
                                            "could not parse `{}` as float",
                                            val_str.trim()
                                        ),
                                    });
                                }
                            },
                            None => {
                                return Err(EphemerisError::OEMParsingError {
                                    lno,
                                    details: format!(
                                        "missing float in covariance data position {col}"
                                    ),
                                });
                            }
                        };
                    }
                    cov_row += 1;
                    if cov_row == 6 {
                        // We've parsed everything, set the covariance
                        match cov_epoch {
                            Some(cov_epoch) => {
                                let covar = cov_mat.map(|mat| Covariance {
                                    matrix: mat,
                                    local_frame: cov_frame.unwrap_or(LocalFrame::Inertial),
                                });
                                segment_state_data
                                    .get_mut(&cov_epoch)
                                    .expect("epoch was valid but now no?")
                                    .covar = covar;
                            }
                            None => {
                                return Err(EphemerisError::OEMParsingError {
                                    lno,
                                    details: "no cov epoch ever found?!".to_string(),
                                });
                            }
                        }
                        // Clear the slot now that the matrix is stored, so any further
                        // data rows before the next EPOCH are rejected rather than
                        // folded into this completed matrix.
                        cov_epoch = None;
                        cov_mat = None;
                        cov_row = 0;
                    }
                }
            }
        }

        if cov_epoch.is_some() {
            return Err(EphemerisError::OEMParsingError {
                lno: 0,
                details: format!(
                    "incomplete covariance matrix at end of file: expected six rows but got {cov_row}"
                ),
            });
        }
        if parser_state == OemParserState::Covariance {
            return Err(EphemerisError::OEMParsingError {
                lno: 0,
                details: "unterminated COVARIANCE_START section at end of file".to_string(),
            });
        }
        if parser_state == OemParserState::Metadata {
            return Err(EphemerisError::OEMParsingError {
                lno: 0,
                details: "unterminated META_START section at end of file".to_string(),
            });
        }

        finish_segment(
            &mut segments,
            &mut segment_state_data,
            interpolation,
            degree,
            &time_system,
            &useable_start,
            &useable_end,
            0,
        )?;

        if segments.is_empty() {
            return Err(EphemerisError::OEMParsingError {
                lno: 0,
                details: "ephemeris file contains no state data".to_string(),
            });
        }

        // Build the Ephemeris
        if let Some(object_id) = object_id {
            Ok(Ephemeris {
                object_id,
                segments,
            })
        } else {
            Err(EphemerisError::OEMParsingError {
                lno: 0,
                details: "no OBJECT_ID found throughout the file".to_string(),
            })
        }
    }

    /// Export this Ephemeris to CCSDS OEM format
    pub fn write_ccsds_oem<P: AsRef<Path>>(
        &self,
        path: P,
        originator: Option<String>,
        object_name: Option<String>,
    ) -> Result<(), EphemerisError> {
        if self.is_empty() {
            return Err(EphemerisError::OEMParsingError {
                lno: 0,
                details: "ephemeris file contains no state data".to_string(),
            });
        }

        let file = File::create(&path).map_err(|e| EphemerisError::OEMWritingError {
            details: format!("could not create file: {e}"),
        })?;
        let mut writer = BufWriter::new(file);

        let err_hdlr = |e| EphemerisError::OEMWritingError {
            details: format!("{e}"),
        };

        // Epoch formmatter.
        let iso8601_no_ts =
            Format::from_str("%Y-%m-%dT%H:%M:%S.%f").expect("static format string is valid");

        // Write mandatory metadata
        writeln!(writer, "CCSDS_OEM_VERS = 3.0\n").map_err(err_hdlr)?;

        writeln!(
            writer,
            "COMMENT Built by ANISE, a modern rewrite of NASA/NAIF SPICE (https://nyxspace.com/anise)",
        )
        .map_err(err_hdlr)?;
        writeln!(
            writer,
            "COMMENT ANISE is open-source software provided under the Mozilla Public License 2.0 (https://github.com/nyx-space/anise)\n"
        )
        .map_err(err_hdlr)?;

        writeln!(
            writer,
            "CREATION_DATE = {}",
            Formatter::new(
                Epoch::now().map_err(|e| EphemerisError::OEMWritingError {
                    details: format!("could not get current epoch: {e}"),
                })?,
                iso8601_no_ts,
            )
        )
        .map_err(err_hdlr)?;
        writeln!(
            writer,
            "ORIGINATOR = {}\n",
            originator.unwrap_or("Nyx Space ANISE".to_string())
        )
        .map_err(err_hdlr)?;

        let object_name = object_name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or("UNKNOWN");
        for segment in self.segment_views() {
            let first_orbit = segment
                .state_data
                .first_key_value()
                .expect("empty segment is never constructed")
                .1
                .orbit;
            let first_frame = first_orbit.frame;
            let center = format!("{first_frame:e}");
            let ref_frame = format!("{first_frame:o}");

            writeln!(writer, "META_START").map_err(err_hdlr)?;
            writeln!(writer, "OBJECT_NAME = {object_name}").map_err(err_hdlr)?;
            writeln!(writer, "OBJECT_ID = {}", self.object_id).map_err(err_hdlr)?;
            writeln!(writer, "CENTER_NAME = {center}").map_err(err_hdlr)?;
            writeln!(
                writer,
                "REF_FRAME = {}",
                match ref_frame.trim() {
                    "J2000" => "EME2000",
                    _ => ref_frame.trim(),
                }
            )
            .map_err(err_hdlr)?;
            writeln!(writer, "TIME_SYSTEM = {}", first_orbit.epoch.time_scale).map_err(err_hdlr)?;
            writeln!(
                writer,
                "START_TIME = {}",
                Formatter::new(segment.total_start, iso8601_no_ts),
            )
            .map_err(err_hdlr)?;
            writeln!(
                writer,
                "USEABLE_START_TIME = {}",
                Formatter::new(segment.useable_start, iso8601_no_ts),
            )
            .map_err(err_hdlr)?;
            writeln!(
                writer,
                "USEABLE_STOP_TIME = {}",
                Formatter::new(segment.useable_end, iso8601_no_ts),
            )
            .map_err(err_hdlr)?;
            writeln!(
                writer,
                "STOP_TIME = {}",
                Formatter::new(segment.total_end, iso8601_no_ts),
            )
            .map_err(err_hdlr)?;
            writeln!(
                writer,
                "INTERPOLATION = {}",
                match segment.interpolation {
                    DataType::Type9LagrangeUnequalStep => "LAGRANGE",
                    DataType::Type13HermiteUnequalStep | DataType::Type12HermiteEqualStep => {
                        "HERMITE"
                    }
                    _ => unreachable!(),
                }
            )
            .map_err(err_hdlr)?;
            writeln!(writer, "INTERPOLATION_DEGREE = {}", segment.degree).map_err(err_hdlr)?;
            writeln!(writer, "META_STOP\n").map_err(err_hdlr)?;

            for (epoch, entry) in segment.state_data {
                let orbit = entry.orbit;
                writeln!(
                    writer,
                    "{} {:E} {:E} {:E} {:E} {:E} {:E}",
                    Formatter::new(*epoch, iso8601_no_ts),
                    orbit.radius_km.x,
                    orbit.radius_km.y,
                    orbit.radius_km.z,
                    orbit.velocity_km_s.x,
                    orbit.velocity_km_s.y,
                    orbit.velocity_km_s.z
                )
                .map_err(err_hdlr)?;
            }

            #[allow(clippy::writeln_empty_string)]
            writeln!(writer, "").map_err(err_hdlr)?;

            let mut cov_started = false;
            for (epoch, entry) in segment.state_data {
                if let Some(covar) = &entry.covar {
                    if !cov_started {
                        writeln!(writer, "COVARIANCE_START").map_err(err_hdlr)?;
                        cov_started = true;
                    }
                    writeln!(writer, "EPOCH = {}", Formatter::new(*epoch, iso8601_no_ts))
                        .map_err(err_hdlr)?;
                    writeln!(
                        writer,
                        "COV_REF_FRAME = {}",
                        match covar.local_frame {
                            LocalFrame::Inertial => "EME2000",
                            LocalFrame::RIC => "RTN",
                            LocalFrame::VNC => "TNW",
                            LocalFrame::RCN => {
                                return Err(EphemerisError::OEMWritingError {
                                    details: "RCN frame is not supported for OEM covariance export"
                                        .to_string(),
                                });
                            }
                        }
                    )
                    .map_err(err_hdlr)?;

                    for row in 0..6 {
                        let mut line = String::new();
                        for col in 0..=row {
                            line.push_str(&format!("{:E} ", covar.matrix[(col, row)]));
                        }
                        writeln!(writer, "{}", line.trim()).map_err(err_hdlr)?;
                    }

                    #[allow(clippy::writeln_empty_string)]
                    writeln!(writer, "").map_err(err_hdlr)?;
                }
            }

            if cov_started {
                writeln!(writer, "COVARIANCE_STOP\n").map_err(err_hdlr)?;
            }
        }
        Ok(())
    }
}
