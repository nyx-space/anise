/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::EphemerisError;
use crate::math::Vector6;
use crate::naif::daf::data_types::DataType;
use crate::prelude::{Frame, Orbit};
use hifitime::Epoch;
use log::warn;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::{Ephemeris, EphemerisRecord};

impl Ephemeris {
    /// Initialize a new ephemeris from the path to an Ansys STK .e file.
    pub fn from_stk_e_file<P: AsRef<Path>>(path: P) -> Result<Self, EphemerisError> {
        // Open the file
        let file = File::open(path).map_err(|e| EphemerisError::OEMParsingError {
            lno: 0,
            details: format!("could not open file: {e}"),
        })?;

        let reader = BufReader::new(file);

        let mut in_state_data = false;
        let mut ignore_block = false;

        // Define header variables we care about.
        let mut center_name = None;
        let mut orient_name = None;
        let mut interpolation = DataType::Type13HermiteUnequalStep;
        let mut degree = 5;
        // There is no object ID in STK files, so we default to "STK_Object" or derive from filename?
        // Let's use "STK_Object" for now.
        let object_id: String = "STK_Object".to_string();
        let mut scenario_epoch: Option<Epoch> = None;

        // Store the temporary data in a BTreeMap so we have O(1) access when adding the covariance information
        // and we can iterate in order when building the vector.
        let mut state_data = BTreeMap::new();

        let parse_one_val = |lno: usize, line: &str, err: &str| -> Result<String, EphemerisError> {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                // Join the rest of the parts as the value, just in case (e.g. date time with spaces)
                Ok(parts[1..].join(" "))
            } else {
                Err(EphemerisError::OEMParsingError {
                    lno,
                    details: err.to_string(),
                })
            }
        };

        for (lno, line_res) in reader.lines().enumerate() {
            let line_orig = line_res.map_err(|e| EphemerisError::OEMParsingError {
                lno,
                details: format!("could not read line: {e}"),
            })?;

            let line = line_orig.trim();
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Handle blocks to ignore (like SegmentBoundaryTimes)
            if line.eq_ignore_ascii_case("BEGIN SegmentBoundaryTimes") {
                ignore_block = true;
                continue;
            } else if line.eq_ignore_ascii_case("END SegmentBoundaryTimes") {
                ignore_block = false;
                continue;
            }

            if ignore_block {
                continue;
            }

            // STK lines seem to be parsed correctly by starts_with if they start with that keyword.
            // But they might be indented. line is trimmed.

            if line.starts_with("stk.v") {
               // Version check if needed, for now ignore
            } else if line.eq_ignore_ascii_case("BEGIN Ephemeris") {
                // Start of ephemeris block
            } else if line.eq_ignore_ascii_case("END Ephemeris") {
                // End of ephemeris block
            } else if line.starts_with("NumberOfEphemerisPoints") {
                // Ignore for now, we just read lines
            } else if line.starts_with("ScenarioEpoch") {
                let epoch_str = parse_one_val(lno, line, "no value for ScenarioEpoch")?;
                // Parse using hifitime's from_format_str
                // STK format: "16 Feb 2027 14:05:57.645126" -> "%d %b %Y %H:%M:%S.%f"
                // Assuming UTC as per instructions.
                match Epoch::from_format_str(&epoch_str, "%d %b %Y %H:%M:%S.%f") {
                    Ok(e) => scenario_epoch = Some(e),
                    Err(e) => return Err(EphemerisError::OEMParsingError {
                        lno,
                        details: format!("could not parse ScenarioEpoch `{}`: {}", epoch_str, e),
                    }),
                }

            } else if line.starts_with("InterpolationMethod") {
                let method = parse_one_val(lno, line, "no value for InterpolationMethod")?;
                if method.eq_ignore_ascii_case("Lagrange") {
                    interpolation = DataType::Type9LagrangeUnequalStep;
                } else if method.eq_ignore_ascii_case("Hermite") {
                    interpolation = DataType::Type13HermiteUnequalStep;
                } else {
                    warn!("unsupported interpolation `{}` using Hermite", method);
                }
            } else if line.starts_with("InterpolationOrder") || line.starts_with("InterpolationSamplesM1") {
                 let val = parse_one_val(lno, line, "no value for InterpolationOrder/SamplesM1")?;
                 match val.parse::<usize>() {
                    Ok(d) => degree = d,
                    Err(_) => return Err(EphemerisError::OEMParsingError { lno, details: format!("invalid InterpolationOrder/SamplesM1 {}", val) }),
                 }
            } else if line.starts_with("CentralBody") {
                center_name = Some(parse_one_val(lno, line, "no value for CentralBody")?);
            } else if line.starts_with("CoordinateSystem") {
                orient_name = Some(parse_one_val(lno, line, "no value for CoordinateSystem")?);
            } else if line.starts_with("DistanceUnit") {
                // Assume Kilometers for now, check if it is something else
                let unit = parse_one_val(lno, line, "no value for DistanceUnit")?;
                if !unit.eq_ignore_ascii_case("Kilometers") {
                     warn!("DistanceUnit is {}, expected Kilometers. Assuming Kilometers.", unit);
                }
            } else if line.starts_with("TimeSystem") {
                 // We ignore TimeSystem because we assume UTC based on instructions for ScenarioEpoch
                 // If specific handling is needed later, add here.
            } else if line.eq_ignore_ascii_case("BEGIN EphemerisTimePosVel") || line.eq_ignore_ascii_case("EphemerisTimePosVel") {
                in_state_data = true;
                continue;
            } else if line.eq_ignore_ascii_case("END EphemerisTimePosVel") {
                in_state_data = false;
            } else if in_state_data {
                // Parse data line
                // Format: Time X Y Z Vx Vy Vz
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 7 {
                    return Err(EphemerisError::OEMParsingError {
                        lno,
                        details: format!("expected at least 7 columns, found {}", parts.len()),
                    });
                }

                let time_offset: f64 = parts[0].parse().map_err(|_| EphemerisError::OEMParsingError { lno, details: format!("invalid time offset {}", parts[0]) })?;

                let start_epoch = scenario_epoch.ok_or(EphemerisError::OEMParsingError { lno, details: "ScenarioEpoch not found before data".to_string() })?;

                // STK .e time is seconds from ScenarioEpoch
                let epoch = start_epoch + hifitime::Unit::Second * time_offset;

                let mut state_vec = Vector6::zeros();
                for i in 0..6 {
                    state_vec[i] = parts[i+1].parse().map_err(|_| EphemerisError::OEMParsingError { lno, details: format!("invalid state value {}", parts[i+1]) })?;
                }

                let center_name_str = center_name.as_deref().unwrap_or("Earth");
                // Capitalize center name similarly to OEM?
                let center_name_str_cap = center_name_str
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

                let orient_name_str = orient_name.as_deref().unwrap_or("J2000"); // Default to J2000 if not specified? Or ICRF?
                // STK often uses "ICRF" or "J2000".

                let frame = Frame::from_name(&center_name_str_cap, orient_name_str).map_err(|e| {
                        EphemerisError::OEMParsingError {
                            lno,
                            details: format!("frame error `{center_name_str_cap:?} {orient_name_str:?}`: {e}"),
                        }
                    })?;

                let orbit = Orbit::from_cartesian_pos_vel(state_vec, epoch, frame);
                state_data.insert(epoch, EphemerisRecord { orbit, covar: None });
            }
        }

        if state_data.is_empty() {
             return Err(EphemerisError::OEMParsingError {
                lno: 0,
                details: "ephemeris file contains no state data".to_string(),
            });
        }

        Ok(Ephemeris {
            object_id,
            degree,
            interpolation,
            state_data,
        })
    }
}
