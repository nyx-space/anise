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
    efmt::{Format, Formatter},
    Epoch,
};
use log::warn;
use snafu::ResultExt;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;

use super::{Covariance, Ephemeris, EphemerisRecord, LocalFrame};

impl Ephemeris {
    /// Initialize a new ephemeris from the path to a CCSDS OEM file.
    pub fn from_ccsds_oem_file<P: AsRef<Path>>(path: P) -> Result<Self, EphemerisError> {
        // Open the file
        let file = File::open(path).map_err(|e| EphemerisError::OEMParsingError {
            lno: 0,
            details: format!("could not open file: {e}"),
        })?;

        let reader = BufReader::new(file);

        let mut in_state_data = false;
        let mut in_cov_data = false;

        // Define header variables we care about.
        let mut time_system = String::new();
        let mut center_name = None;
        let mut orient_name = None;
        let mut interpolation = DataType::Type13HermiteUnequalStep;
        let mut degree = 5;
        let mut object_id: Option<String> = None;
        let mut cov_epoch = None;
        let mut cov_mat = None;
        let mut cov_frame = None;
        let mut cov_row = 0;

        // Store the temporary data in a BTreeMap so we have O(1) access when adding the covariance information
        // and we can iterate in order when building the vector.
        let mut state_data = BTreeMap::new();

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

            if line.starts_with("CCSDS_OEM_VERS") {
                let version_str = parse_one_val(lno, line, "no value for CCSDS_OEM_VERS")?;
                match version_str.parse::<f32>() {
                    Ok(version_val) => match version_val as i16 {
                        1 | 2 => {}
                        _ => {
                            return Err(EphemerisError::OEMParsingError {
                                lno,
                                details: "CCSDS OEM version {version_val} not supported"
                                    .to_string(),
                            })
                        }
                    },
                    Err(_) => {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: format!("could not parse OEM version `{version_str}`"),
                        })
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
                time_system = parse_one_val(lno, line, "no value for TIME_SYSTEM")?;
            } else if line.starts_with("INTERPOLATION_DEGREE") {
                let interp_str =
                    parse_one_val(lno, line, "no value for INTERPOLATION_DEGREE")?.to_lowercase();

                match interp_str.parse::<usize>() {
                    Ok(ideg) => degree = ideg,
                    Err(_) => {
                        return Err(EphemerisError::OEMParsingError {
                            lno,
                            details: format!("could not parse `{interp_str}` as float"),
                        })
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
            } else if line.starts_with("META_STOP") {
                // We can start parsing now
                in_state_data = true;
                in_cov_data = false;
            } else if line.starts_with("META_START") {
                in_state_data = false;
                in_cov_data = false;
            } else if line.starts_with("COVARIANCE_START") {
                in_state_data = false;
                in_cov_data = true;
            } else if line.starts_with("COVARIANCE_STOP") {
                in_state_data = false;
                in_cov_data = false;
            } else if line.starts_with("COMMENT") {
                // Ignore
            } else if in_state_data {
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
                        })
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
                                })
                            }
                        },
                        None => {
                            return Err(EphemerisError::OEMParsingError {
                                lno,
                                details: format!("missing float in position {}", i + 1),
                            })
                        }
                    };
                }

                // We only reach this point if the state data is fully parsed.
                let orbit = Orbit::from_cartesian_pos_vel(state_vec, epoch, frame);
                state_data.insert(epoch, EphemerisRecord { orbit, covar: None });
            } else if in_cov_data {
                if line.starts_with("EPOCH") {
                    let state_epoch = parse_one_val(lno, line, "no `=` sign for covariance epoch")?;
                    let epoch_str = format!("{state_epoch} {time_system}");
                    let epoch = Epoch::from_str(epoch_str.trim()).context(OEMTimeParsingSnafu {
                        line: lno,
                        details: format!("`{epoch_str}` for covariance epoch"),
                    })?;

                    // Check that we have associated state data
                    if !state_data.contains_key(&epoch) {
                        return Err(EphemerisError::OEMParsingError { lno, details: format!("cannot have covariance data at {epoch} because no orbit data at that epoch")});
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
                            })
                        }
                    };
                } else {
                    // Matrix data!
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
                                    cov_mat.as_mut().unwrap()[(col, cov_row)] = val_f64;
                                    cov_mat.as_mut().unwrap()[(cov_row, col)] = val_f64;
                                }
                                Err(_) => {
                                    return Err(EphemerisError::OEMParsingError {
                                        lno,
                                        details: format!(
                                            "could not parse `{}` as float",
                                            val_str.trim()
                                        ),
                                    })
                                }
                            },
                            None => {
                                return Err(EphemerisError::OEMParsingError {
                                    lno,
                                    details: format!(
                                        "missing float in covariance data position {col}"
                                    ),
                                })
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
                                state_data
                                    .get_mut(&cov_epoch)
                                    .expect("epoch was valid but now no?")
                                    .covar = covar;
                            }
                            None => {
                                return Err(EphemerisError::OEMParsingError {
                                    lno,
                                    details: "no cov epoch ever found?!".to_string(),
                                })
                            }
                        }
                    }
                }
            }
        }

        if state_data.is_empty() {
            return Err(EphemerisError::OEMParsingError {
                lno: 0,
                details: "ephemeris file contains no state data".to_string(),
            });
        }

        // Build the Ephemeris
        if let Some(object_id) = object_id {
            Ok(Ephemeris {
                object_id,
                degree,
                interpolation,
                state_data,
            })
        } else {
            Err(EphemerisError::OEMParsingError {
                lno: 0,
                details: "no OBJECT_ID found throughout the file".to_string(),
            })
        }
    }

    /// Export this Ephemeris to CCSDS OEM format
    pub fn to_ccsds_oem_file<P: AsRef<Path>>(
        &self,
        path: P,
        originator: Option<String>,
        object_name: Option<String>,
    ) -> Result<(), EphemerisError> {
        if self.state_data.is_empty() {
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
        let iso8601_no_ts = Format::from_str("%Y-%m-%dT%H:%M:%S").unwrap();

        // Write mandatory metadata
        writeln!(writer, "CCSDS_OEM_VERS = 2.0\n").map_err(err_hdlr)?;

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
            Formatter::new(Epoch::now().unwrap(), iso8601_no_ts)
        )
        .map_err(err_hdlr)?;
        writeln!(
            writer,
            "ORIGINATOR = {}\n",
            originator.unwrap_or("Nyx Space ANISE".to_string())
        )
        .map_err(err_hdlr)?;

        writeln!(writer, "META_START").map_err(err_hdlr)?;
        writeln!(writer, "\tOBJECT_ID = {}", self.object_id).map_err(err_hdlr)?;

        if let Some(object_name) = object_name {
            writeln!(writer, "\tOBJECT_NAME = {object_name}").map_err(err_hdlr)?;
        }

        let first_orbit = self.state_data.first_key_value().unwrap().1.orbit;
        let first_frame = first_orbit.frame;
        let last_orbit = self.state_data.last_key_value().unwrap().1.orbit;

        let center = format!("{first_frame:e}");
        let ref_frame = format!("{first_frame:o}");
        writeln!(
            writer,
            "\tREF_FRAME = {}",
            match ref_frame.trim() {
                "J2000" => "EME2000",
                _ => ref_frame.trim(),
            }
        )
        .map_err(err_hdlr)?;

        writeln!(writer, "\tCENTER_NAME = {center}",).map_err(err_hdlr)?;
        writeln!(writer, "\tTIME_SYSTEM = {}", first_orbit.epoch.time_scale).map_err(err_hdlr)?;
        writeln!(
            writer,
            "\tINTERPOLATION = {}",
            match self.interpolation {
                DataType::Type9LagrangeUnequalStep => "LAGRANGE",
                DataType::Type13HermiteUnequalStep => "HERMITE",
                _ => unreachable!(),
            }
        )
        .map_err(err_hdlr)?;

        writeln!(writer, "\tINTERPOLATION_DEGREE = {}", self.degree).map_err(err_hdlr)?;

        writeln!(
            writer,
            "\tSTART_TIME = {}.{:0<3}",
            Formatter::new(first_orbit.epoch, iso8601_no_ts),
            first_orbit.epoch.seconds()
        )
        .map_err(err_hdlr)?;
        writeln!(
            writer,
            "\tUSEABLE_START_TIME = {}.{:0<3}",
            Formatter::new(first_orbit.epoch, iso8601_no_ts),
            first_orbit.epoch.seconds()
        )
        .map_err(err_hdlr)?;
        writeln!(
            writer,
            "\tUSEABLE_STOP_TIME = {}.{:0<3}",
            Formatter::new(last_orbit.epoch, iso8601_no_ts),
            last_orbit.epoch.seconds()
        )
        .map_err(err_hdlr)?;
        writeln!(
            writer,
            "\tSTOP_TIME = {}.{:0<3}",
            Formatter::new(last_orbit.epoch, iso8601_no_ts),
            first_orbit.epoch.seconds()
        )
        .map_err(err_hdlr)?;

        writeln!(writer, "META_STOP\n").map_err(err_hdlr)?;

        for (epoch, entry) in self.state_data.iter() {
            let orbit = entry.orbit;
            writeln!(
                writer,
                "{}.{:0<3} {:E} {:E} {:E} {:E} {:E} {:E}",
                Formatter::new(*epoch, iso8601_no_ts),
                epoch.seconds(),
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

        // Add covariance if available
        let mut cov_started = false;
        for (epoch, entry) in self.state_data.iter() {
            if let Some(covar) = &entry.covar {
                if !cov_started {
                    writeln!(writer, "COVARIANCE_START").map_err(err_hdlr)?;
                    cov_started = true;
                }
                writeln!(
                    writer,
                    "EPOCH = {}.{:0<3}",
                    Formatter::new(*epoch, iso8601_no_ts),
                    epoch.seconds()
                )
                .map_err(err_hdlr)?;

                writeln!(
                    writer,
                    "COV_REF_FRAME = {}",
                    match covar.local_frame {
                        LocalFrame::Inertial => "EME2000",
                        LocalFrame::RIC => "RTN",
                        LocalFrame::VNC => "TNW",
                        LocalFrame::RCN => unreachable!(),
                    }
                )
                .map_err(err_hdlr)?;

                // Write the matrix
                // 1
                // 2 3
                // 4 5 6
                // ...
                for row in 0..6 {
                    let mut line = String::new();
                    for col in 0..row + 1 {
                        line.push_str(&format!("{:E} ", covar.matrix[(col, row)]));
                    }
                    writeln!(writer, "{}", line.trim()).map_err(err_hdlr)?;
                }

                #[allow(clippy::writeln_empty_string)]
                writeln!(writer, "").map_err(err_hdlr)?;
            }
        }

        if cov_started {
            writeln!(writer, "COVARIANCE_STOP").map_err(err_hdlr)?;
        }
        Ok(())
    }
}
