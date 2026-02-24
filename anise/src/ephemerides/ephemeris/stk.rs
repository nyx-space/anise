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
use crate::math::{Matrix6, Vector6};
use crate::naif::daf::data_types::DataType;
use crate::prelude::{Frame, Orbit};
use hifitime::{Epoch, Unit};
use log::warn;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::{Covariance, Ephemeris, EphemerisRecord, LocalFrame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CovarianceFormat {
    LowerTriangular,
    UpperTriangular,
}

impl Ephemeris {
    /// Initialize a new ephemeris from the path to an Ansys STK .e file.
    pub fn from_stk_e_file<P: AsRef<Path>>(path: P) -> Result<Self, EphemerisError> {
        // Open the file
        let file = File::open(&path).map_err(|e| EphemerisError::STKEParsingError {
            lno: 0,
            details: format!("could not open file: {e}"),
        })?;

        let reader = BufReader::new(file);

        let mut in_state_data = false;
        let mut in_cov_data = false;
        let mut ignore_block = false;

        // Define header variables we care about.
        let mut center_name = None;
        let mut orient_name = None;
        let mut interpolation = DataType::Type9LagrangeUnequalStep;
        let mut samples_m1 = 5;
        let object_id: String = path
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("STK_Object")
            .to_string();
        let mut scenario_epoch: Option<Epoch> = None;
        let mut cov_format = CovarianceFormat::LowerTriangular; // Default for CovarianceTimePosVel
        let mut is_kilometers = false;

        // Store the temporary data in a BTreeMap so we have O(1) access when adding the covariance information
        // and we can iterate in order when building the vector.
        let mut state_data = BTreeMap::new();
        let mut state_cov = BTreeMap::new();

        // Buffer to accumulate tokens across lines for covariance parsing
        let mut cov_token_buffer: Vec<f64> = Vec::with_capacity(22);

        let parse_one_val = |lno: usize, line: &str, err: &str| -> Result<String, EphemerisError> {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                // Join the rest of the parts as the value, just in case (e.g. date time with spaces)
                Ok(parts[1..].join(" "))
            } else {
                Err(EphemerisError::STKEParsingError {
                    lno,
                    details: err.to_string(),
                })
            }
        };

        for (lno, line_res) in reader.lines().enumerate() {
            let line_orig = line_res.map_err(|e| EphemerisError::STKEParsingError {
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
            } else if line.starts_with("NumberOfEphemerisPoints")
                || line.starts_with("NumberOfCovariancePoints")
            {
                // Ignore for now, we just read lines
            } else if line.starts_with("ScenarioEpoch") {
                let epoch_str = parse_one_val(lno, line, "no value for ScenarioEpoch")?;
                // Parse using hifitime's from_format_str
                // STK format: "16 Feb 2027 14:05:57.645126" -> "%d %b %Y %H:%M:%S.%f"
                // Assuming UTC as per instructions.
                match Epoch::from_format_str(&epoch_str, "%d %b %Y %H:%M:%S.%f") {
                    Ok(e) => scenario_epoch = Some(e),
                    Err(e) => {
                        return Err(EphemerisError::STKEParsingError {
                            lno,
                            details: format!("could not parse ScenarioEpoch `{epoch_str}`: {e}"),
                        })
                    }
                }
            } else if line.starts_with("InterpolationMethod") {
                let method = parse_one_val(lno, line, "no value for InterpolationMethod")?;
                if method.eq_ignore_ascii_case("Lagrange") {
                    interpolation = DataType::Type9LagrangeUnequalStep;
                } else if method.eq_ignore_ascii_case("Hermite") {
                    interpolation = DataType::Type13HermiteUnequalStep;
                } else {
                    warn!("unsupported interpolation `{method}` using Hermite");
                }
            } else if line.starts_with("InterpolationOrder")
                || line.starts_with("InterpolationSamplesM1")
            {
                let val = parse_one_val(lno, line, "no value for InterpolationOrder/SamplesM1")?;
                match val.parse::<usize>() {
                    Ok(d) => samples_m1 = d,
                    Err(_) => {
                        return Err(EphemerisError::STKEParsingError {
                            lno,
                            details: format!("invalid InterpolationOrder/SamplesM1 {val}"),
                        })
                    }
                }
            } else if line.starts_with("InterpolationSamples") {
                let val = parse_one_val(lno, line, "no value for InterpolationSamples")?;
                match val.parse::<usize>() {
                    // Ephemeris.degree is the number of samples. STK provides order (samples - 1).
                    Ok(d) => samples_m1 = d.saturating_sub(1),
                    Err(_) => {
                        return Err(EphemerisError::STKEParsingError {
                            lno,
                            details: format!("invalid InterpolationSamples {val}"),
                        })
                    }
                }
            } else if line.starts_with("CentralBody") {
                center_name = Some(parse_one_val(lno, line, "no value for CentralBody")?);
            } else if line.starts_with("CoordinateSystem") {
                orient_name = Some(parse_one_val(lno, line, "no value for CoordinateSystem")?);
            } else if line.starts_with("DistanceUnit") {
                let unit = parse_one_val(lno, line, "no value for DistanceUnit")?;
                if unit.eq_ignore_ascii_case("Kilometers") {
                    is_kilometers = true;
                } else if unit.eq_ignore_ascii_case("Meters") {
                    is_kilometers = false;
                } else {
                    return Err(EphemerisError::STKEParsingError {
                        lno,
                        details: format!(
                            "DistanceUnit is {unit}, only Kilometers and Meters supported"
                        ),
                    });
                }
            } else if line.starts_with("TimeSystem") {
                // We ignore TimeSystem because we assume UTC based on instructions for ScenarioEpoch
                // If specific handling is needed later, add here.
                let time_system = parse_one_val(lno, line, "no value for TimeSystem")?;
                if !time_system.eq_ignore_ascii_case("UTC") {
                    return Err(EphemerisError::STKEParsingError {
                        lno,
                        details: format!(
                            "unsupported TimeSystem `{time_system}`: only 'UTC' is supported"
                        ),
                    });
                }
            } else if line
                .split_whitespace()
                .next()
                .is_some_and(|w| w.eq_ignore_ascii_case("CovarianceFormat"))
            {
                let fmt = parse_one_val(lno, line, "no value for CovarianceFormat")?;
                cov_format = match fmt.to_ascii_lowercase().as_str() {
                    "lowertriangular" | "lt" => CovarianceFormat::LowerTriangular,
                    "uppertriangular" | "ut" => CovarianceFormat::UpperTriangular,
                    _ => {
                        return Err(EphemerisError::STKEParsingError {
                            lno,
                            details: format!("invalid CovarianceFormat `{fmt}`"),
                        });
                    }
                };
            } else if line.eq_ignore_ascii_case("BEGIN EphemerisTimePosVel")
                || line.eq_ignore_ascii_case("EphemerisTimePosVel")
            {
                in_state_data = true;
                continue;
            } else if line.eq_ignore_ascii_case("END EphemerisTimePosVel") {
                in_state_data = false;
            } else if line.eq_ignore_ascii_case("BEGIN CovarianceTimePosVel")
                || line.eq_ignore_ascii_case("CovarianceTimePosVel")
            {
                in_cov_data = true;
                in_state_data = false;
                cov_token_buffer.clear();
                continue;
            } else if line.eq_ignore_ascii_case("END CovarianceTimePosVel") {
                in_cov_data = false;
                if !cov_token_buffer.is_empty() {
                    warn!("Parsing ended with incomplete covariance data (buffer not empty)");
                }
            } else if in_state_data {
                // Parse data line
                // Format: Time X Y Z Vx Vy Vz
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 7 {
                    return Err(EphemerisError::STKEParsingError {
                        lno,
                        details: format!("expected at least 7 columns, found {}", parts.len()),
                    });
                }

                let time_offset: f64 =
                    parts[0]
                        .parse()
                        .map_err(|_| EphemerisError::STKEParsingError {
                            lno,
                            details: format!("invalid time offset {}", parts[0]),
                        })?;

                let start_epoch = scenario_epoch.ok_or(EphemerisError::STKEParsingError {
                    lno,
                    details: "ScenarioEpoch not found before data".to_string(),
                })?;

                // STK .e time is seconds from ScenarioEpoch
                let epoch = start_epoch + Unit::Second * time_offset;

                let mut state_vec = Vector6::zeros();
                for i in 0..6 {
                    let mut val: f64 =
                        parts[i + 1]
                            .parse()
                            .map_err(|_| EphemerisError::STKEParsingError {
                                lno,
                                details: format!("invalid state value {}", parts[i + 1]),
                            })?;
                    if !is_kilometers {
                        val *= 1e-3;
                    }
                    state_vec[i] = val;
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

                let frame =
                    Frame::from_name(&center_name_str_cap, orient_name_str).map_err(|e| {
                        EphemerisError::STKEParsingError {
                            lno,
                            details: format!(
                                "frame error `{center_name_str_cap:?} {orient_name_str:?}`: {e}"
                            ),
                        }
                    })?;

                let orbit = Orbit::from_cartesian_pos_vel(state_vec, epoch, frame);
                state_data.insert(epoch, EphemerisRecord { orbit, covar: None });
            } else if in_cov_data {
                // Parse covariance tokens
                // Format: Time + 21 values, spread across lines/whitespace
                let tokens = line.split_whitespace();
                for token in tokens {
                    let val =
                        token
                            .parse::<f64>()
                            .map_err(|_| EphemerisError::STKEParsingError {
                                lno,
                                details: format!("invalid covariance value {token}"),
                            })?;
                    cov_token_buffer.push(val);
                }

                // Process buffer as long as we have full records (22 tokens)
                while cov_token_buffer.len() >= 22 {
                    // Extract one record
                    let mut record_vals: Vec<f64> = cov_token_buffer.drain(0..22).collect();

                    if !is_kilometers {
                        for val in record_vals.iter_mut().skip(1) {
                            *val *= 1e-6;
                        }
                    }

                    let time_offset = record_vals[0];
                    let start_epoch = scenario_epoch.ok_or(EphemerisError::STKEParsingError {
                        lno,
                        details: "ScenarioEpoch not found before data".to_string(),
                    })?;
                    let epoch = start_epoch + Unit::Second * time_offset;

                    // Remaining 21 values are covariance
                    let mut matrix = Matrix6::zeros();
                    let mut idx = 1; // Start from index 1 in record_vals

                    match cov_format {
                        CovarianceFormat::LowerTriangular => {
                            // C[0][0]
                            // C[1][0] C[1][1]
                            // ...
                            for r in 0..6 {
                                for c in 0..=r {
                                    matrix[(r, c)] = record_vals[idx];
                                    matrix[(c, r)] = record_vals[idx]; // Symmetric
                                    idx += 1;
                                }
                            }
                        }
                        CovarianceFormat::UpperTriangular => {
                            // C[0][0] C[0][1] ...
                            //         C[1][1] ...
                            for r in 0..6 {
                                for c in r..6 {
                                    matrix[(r, c)] = record_vals[idx];
                                    matrix[(c, r)] = record_vals[idx]; // Symmetric
                                    idx += 1;
                                }
                            }
                        }
                    }

                    state_cov.insert(
                        epoch,
                        Covariance {
                            matrix,
                            local_frame: LocalFrame::Inertial,
                        },
                    );
                }
            }
        }

        if state_data.is_empty() {
            return Err(EphemerisError::STKEParsingError {
                lno: 0,
                details: "ephemeris file contains no state data".to_string(),
            });
        }

        // Merge covariance into state_data
        for (epoch, covar) in state_cov {
            if let Some(record) = state_data.get_mut(&epoch) {
                record.covar = Some(covar);
            } else {
                warn!("covariance at {epoch} has no corresponding state data, ignoring");
            }
        }

        // Finalize degree
        let degree = if interpolation == DataType::Type9LagrangeUnequalStep {
            samples_m1.saturating_add(1)
        } else {
            samples_m1.saturating_mul(2).saturating_add(1)
        };

        Ok(Ephemeris {
            object_id,
            degree,
            interpolation,
            state_data,
        })
    }
}
