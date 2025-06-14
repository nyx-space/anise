/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

// Credit: ChatGPT for 80% of the code to parse the file from the SPICE docs.

use core::fmt;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use log::{error, info, warn};

use crate::constants::orientations::J2000;
use crate::math::rotation::{r1, r2, r3, Quaternion, DCM};
use crate::math::Matrix3;
use crate::naif::kpl::fk::FKItem;
use crate::naif::kpl::tpc::TPCItem;
use crate::naif::kpl::Parameter;
use crate::structure::dataset::{DataSetError, DataSetType};
use crate::structure::metadata::Metadata;
use crate::structure::planetocentric::ellipsoid::Ellipsoid;
use crate::structure::planetocentric::phaseangle::PhaseAngle;
use crate::structure::planetocentric::{PlanetaryData, MAX_NUT_PREC_ANGLES};
use crate::structure::{EulerParameterDataSet, PlanetaryDataSet};

use super::{KPLItem, KPLValue};

#[derive(Debug, PartialEq, Eq)]
enum BlockType {
    Comment,
    Data,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub keyword: String,
    pub value: String,
}

impl Assignment {
    pub fn to_value(&self) -> KPLValue {
        let value = &self.value;
        // Sanitize the input
        let value = value.
            // Remove parentheses
            // Convert remove the extra single quotes
            // there usually aren't commas, only sometimes
            replace(['(', ')', ',', '\''], "");

        let vec: Vec<&str> = value.split_whitespace().filter(|s| !s.is_empty()).collect();
        // If there are multiple items, we assume this is a vector
        if vec.len() > 1 {
            KPLValue::Matrix(
                vec.iter()
                    .map(|s| s.parse::<f64>().unwrap_or(0.0))
                    .collect(),
            )
        } else if vec.is_empty() {
            // Return the original value as a string
            KPLValue::String(self.value.clone())
        } else {
            // We have exactly one item, let's try to convert it as an integer first
            if let Ok(as_int) = vec[0].parse::<i32>() {
                KPLValue::Integer(as_int)
            } else if let Ok(as_f64) = vec[0].trim().replace("D", "E").parse::<f64>() {
                KPLValue::Float(as_f64)
            } else {
                // Darn, let's default to string
                KPLValue::String(value.clone())
            }
        }
    }
}

pub fn parse_file<P: AsRef<Path> + fmt::Debug, I: KPLItem>(
    file_path: P,
    show_comments: bool,
) -> Result<HashMap<i32, I>, DataSetError> {
    let file =
        File::open(&file_path).unwrap_or_else(|_| panic!("Failed to open file {file_path:?}"));
    let mut reader = BufReader::new(file);
    parse_bytes(&mut reader, show_comments)
}

pub fn parse_bytes<R: BufRead, I: KPLItem>(
    reader: &mut R,
    show_comments: bool,
) -> Result<HashMap<i32, I>, DataSetError> {
    let mut block_type = BlockType::Comment;
    let mut assignments = vec![];

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue, // skip lines that can't be read (invalid UTF-8)
        };
        let tline = line.trim();

        if tline.starts_with("\\begintext") {
            block_type = BlockType::Comment;
            continue;
        } else if tline.starts_with("\\begindata") {
            block_type = BlockType::Data;
            continue;
        }

        if block_type == BlockType::Comment && show_comments {
            println!("{line}");
        } else if block_type == BlockType::Data {
            let parts: Vec<&str> = line.split('=').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let keyword = parts[0];
                let value = parts[1];
                let assignment = Assignment {
                    keyword: keyword.to_string(),
                    value: value.to_string(),
                };
                assignments.push(assignment);
            } else if let Some(mut assignment) = assignments.pop() {
                // This is a continuation of the previous line, so let's grab the data and append the value we're reading now.
                // We're adding the full line with all of the extra spaces because the parsing needs those delimiters to not bunch together all of the floats.
                assignment.value += &line;
                assignments.push(assignment);
            }
        }
    }
    // Now let's parse all of the assignments and put it into a pretty hash map.
    let mut map = HashMap::new();
    for item in assignments {
        let key = I::extract_key(&item);
        if key == -1 {
            // This is metadata
            continue;
        }
        map.entry(key).or_insert_with(|| I::default());
        let body_map = map.get_mut(&key).unwrap();
        body_map.parse(item);
    }
    Ok(map)
}

/// Converts two KPL/TPC files, one defining the planetary constants as text, and the other defining the gravity parameters, into the PlanetaryDataSet equivalent ANISE file.
/// KPL/TPC files must be converted into "PCA" (Planetary Constant ANISE) files before being loaded into ANISE.
pub fn convert_tpc<P: AsRef<Path> + fmt::Debug>(
    pck: P,
    gm: P,
) -> Result<PlanetaryDataSet, DataSetError> {
    let gravity_data = parse_file::<_, TPCItem>(gm, false)?;
    let planetary_data = parse_file::<_, TPCItem>(pck, false)?;
    convert_tpc_items(planetary_data, gravity_data)
}

pub fn convert_tpc_items(
    mut planetary_data: HashMap<i32, TPCItem>,
    gravity_data: HashMap<i32, TPCItem>,
) -> Result<PlanetaryDataSet, DataSetError> {
    let mut dataset = PlanetaryDataSet::default();

    for (key, value) in gravity_data {
        if let Some(planet_data) = planetary_data.get_mut(&key) {
            for (gk, gv) in value.data {
                planet_data.data.insert(gk, gv);
            }
        }
    }

    // Now that planetary_data has everything, we'll create the planetary dataset in the ANISE ASN1 format.

    for (object_id, planetary_data) in planetary_data {
        match planetary_data.data.get(&Parameter::GravitationalParameter) {
            Some(mu_km3_s2_value) => {
                match mu_km3_s2_value {
                    KPLValue::Float(mu_km3_s2) => {
                        // Build the ellipsoid
                        let ellipsoid = match planetary_data.data.get(&Parameter::Radii) {
                            Some(radii_km) => match radii_km {
                                KPLValue::Float(radius_km) => {
                                    Some(Ellipsoid::from_sphere(*radius_km))
                                }
                                KPLValue::Matrix(radii_km) => match radii_km.len() {
                                    2 => Some(Ellipsoid::from_spheroid(radii_km[0], radii_km[1])),
                                    3 => Some(Ellipsoid {
                                        semi_major_equatorial_radius_km: radii_km[0],
                                        semi_minor_equatorial_radius_km: radii_km[1],
                                        polar_radius_km: radii_km[2],
                                    }),
                                    len => {
                                        return Err(DataSetError::Conversion {
                                            action: format!(
                                                "Radii matrix should be length 2 or 3 but was {len}"
                                            ),
                                        })
                                    }
                                },
                                _ => {
                                    return Err(DataSetError::Conversion {
                                        action: format!(
                                            "Radii should be float or matrix, got {radii_km:?}"
                                        ),
                                    })
                                }
                            },
                            None => None,
                        };

                        let mut constant = match planetary_data.data.get(&Parameter::PoleRa) {
                            Some(data) => {
                                match data {
                                    KPLValue::Matrix(pole_ra_data) => {
                                        let mut pole_ra_data = pole_ra_data.clone();
                                        if let Some(coeffs) =
                                            planetary_data.data.get(&Parameter::NutPrecRa)
                                        {
                                            pole_ra_data.extend(coeffs.to_vec_f64().map_err(|_| {
                                            DataSetError::Conversion { action: format!("NutPrecRa coefficients must be a matrix but was {coeffs:?}") }
                                        })?);
                                        }
                                        let pola_ra = PhaseAngle::maybe_new(&pole_ra_data);

                                        let pole_dec_data = planetary_data
                                            .data
                                            .get(&Parameter::PoleDec)
                                            .ok_or(DataSetError::Conversion {
                                                action: "missing PoleDec parameter".to_owned(),
                                            })?;
                                        let mut pola_dec_data: Vec<f64> = pole_dec_data
                                            .to_vec_f64()
                                            .map_err(|_| DataSetError::Conversion {
                                                action: format!(
                                                "PoleDec must be a matrix but was {pole_dec_data:?}"
                                            ),
                                            })?;
                                        if let Some(coeffs) =
                                            planetary_data.data.get(&Parameter::NutPrecDec)
                                        {
                                            pola_dec_data.extend(coeffs.to_vec_f64().map_err(|_| {
                                            DataSetError::Conversion { action: format!("NutPrecDec coefficients must be a matrix but was {coeffs:?}") }
                                        })?);
                                        }
                                        let pola_dec = PhaseAngle::maybe_new(&pola_dec_data);

                                        let prime_mer_data = planetary_data
                                            .data
                                            .get(&Parameter::PrimeMeridian)
                                            .ok_or(DataSetError::Conversion {
                                                action: "missing PrimeMeridian parameter"
                                                    .to_owned(),
                                            })?;
                                        let mut prime_mer_data: Vec<f64> = prime_mer_data
                                        .to_vec_f64()
                                        .map_err(|_| DataSetError::Conversion { action: format!("PrimeMeridian must be a matrix but was {prime_mer_data:?}") })?;
                                        if let Some(coeffs) =
                                            planetary_data.data.get(&Parameter::NutPrecPm)
                                        {
                                            prime_mer_data.extend(coeffs.to_vec_f64().map_err(|_| DataSetError::Conversion { action: format!("NutPrecPm must be a matrix but was {coeffs:?}") })?);
                                        }
                                        let prime_mer = PhaseAngle::maybe_new(&prime_mer_data);

                                        let long_axis = match planetary_data.data.get(&Parameter::LongAxis) {
                                            Some(val) => match val {
                                                KPLValue::Float(data) => Some(*data),
                                                KPLValue::Matrix(data) => {
                                                    if data.is_empty() {
                                                        return Err(DataSetError::Conversion {
                                                            action: "long axis matrix is empty".to_string(),
                                                        });
                                                    }
                                                    Some(data[0])
                                                }
                                                _ => return Err(DataSetError::Conversion {
                                                    action: format!(
                                                        "long axis must be float or matrix, got {val:?}"
                                                    ),
                                                }),
                                            },
                                            None => None,
                                        };

                                        PlanetaryData {
                                            object_id,
                                            parent_id: if [199, 299].contains(&object_id) {
                                                J2000
                                            } else if object_id > 100 {
                                                object_id / 100
                                            } else {
                                                J2000
                                            },
                                            mu_km3_s2: *mu_km3_s2,
                                            shape: ellipsoid,
                                            pole_right_ascension: pola_ra,
                                            pole_declination: pola_dec,
                                            prime_meridian: prime_mer,
                                            long_axis,
                                            ..Default::default()
                                        }
                                    }
                                    _ => {
                                        return Err(DataSetError::Conversion {
                                            action: format!(
                                            "expected Matrix as PoleRa parameter but got {data:?}"
                                        ),
                                        })
                                    }
                                }
                            }
                            None => {
                                // Assume not rotation data available
                                PlanetaryData {
                                    object_id,
                                    mu_km3_s2: *mu_km3_s2,
                                    shape: ellipsoid,
                                    parent_id: J2000,
                                    ..Default::default()
                                }
                            }
                        };

                        // Add the nutation precession angles, which are defined for the system
                        if let Some(nut_prec_val) =
                            planetary_data.data.get(&Parameter::NutPrecAngles)
                        {
                            let phase_deg = match planetary_data
                                .data
                                .get(&Parameter::MaxPhaseDegree)
                            {
                                Some(val) => {
                                    let deg =
                                        (val.to_i32().map_err(|_| DataSetError::Conversion {
                                            action: format!(
                                                "MaxPhaseDegree must be an Integer but was {val:?}"
                                            ),
                                        })? + 1) as usize;

                                    if deg == 0 {
                                        return Err(DataSetError::Conversion {
                                            action: "PhaseDegree must be non-zero".to_owned(),
                                        });
                                    }

                                    deg
                                }
                                None => 2,
                            };
                            let nut_prec_data = nut_prec_val.to_vec_f64().map_err(|_| {
                                DataSetError::Conversion {
                                    action: format!(
                                        "NutPrecAngles must be a Matrix but was {nut_prec_val:?}"
                                    ),
                                }
                            })?;

                            let mut coeffs = [PhaseAngle::<0>::default(); MAX_NUT_PREC_ANGLES];
                            let mut num = 0;
                            for (i, nut_prec) in nut_prec_data.chunks(phase_deg).enumerate() {
                                if i >= coeffs.len() {
                                    return Err(DataSetError::Conversion {
                                        action: format!(
                                            "Index {} exceeds the maximum number of nutation precession angles ({})",
                                            i, coeffs.len()
                                        ),
                                    });
                                }

                                if nut_prec.len() < 2 {
                                    return Err(DataSetError::Conversion {
                                        action: format!(
                                            "Expected nut prec data to be array of length 2 but was {}",
                                            nut_prec.len()
                                        ),
                                    });
                                }

                                coeffs[i] = PhaseAngle::<0> {
                                    offset_deg: nut_prec[0],
                                    rate_deg: nut_prec[1],
                                    ..Default::default()
                                };
                                num += 1;
                            }

                            constant.num_nut_prec_angles = num;
                            constant.nut_prec_angles = coeffs;
                        };

                        // Skip the DER serialization in full.
                        dataset.push(constant, Some(object_id), None)?;
                        info!("Added {object_id}");
                    }
                    _ => error!(
                        "skipping {object_id}: gravity data is {mu_km3_s2_value:?} (want float)"
                    ),
                }
            }
            None => {
                warn!("skipping {object_id}: no gravity data")
            }
        }
    }

    info!("added {} items", dataset.lut.by_id.len());

    dataset.set_crc32();
    dataset.metadata = Metadata::default();
    dataset.metadata.dataset_type = DataSetType::PlanetaryData;

    Ok(dataset)
}

/// Converts a KPL/FK file, that defines frame constants like fixed rotations, and frame name to ID mappings into the EulerParameterDataSet equivalent ANISE file.
/// KPL/FK files must be converted into "PCA" (Planetary Constant ANISE) files before being loaded into ANISE.
pub fn convert_fk<P: AsRef<Path> + fmt::Debug>(
    fk_file_path: P,
    show_comments: bool,
) -> Result<EulerParameterDataSet, DataSetError> {
    let assignments = parse_file::<_, FKItem>(fk_file_path, show_comments)?;
    convert_fk_items(assignments)
}

pub fn convert_fk_items(
    assignments: HashMap<i32, FKItem>,
) -> Result<EulerParameterDataSet, DataSetError> {
    let mut dataset = EulerParameterDataSet::default();
    let mut ids_to_update = Vec::new();

    // Add all of the data into the data set
    for (id, item) in assignments {
        if !item.data.contains_key(&Parameter::Angles)
            && !item.data.contains_key(&Parameter::Matrix)
        {
            let mut warn = false;
            if let Some(class) = item.data.get(&Parameter::Class) {
                let class_val = class.to_i32().map_err(|_| DataSetError::Conversion {
                    action: format!("Class must be an Integer but was {class:?}"),
                })?;
                if class_val == 2 {
                    // BPC based frame, insert as-is.
                    // Class 2 need a BPC for the full rotation.
                    dataset.push(Quaternion::identity(id, id), Some(id), item.name.as_deref())?;
                } else {
                    warn = true;
                }
            } else {
                warn = true;
            }
            if warn {
                warn!("{id} contains neither angles nor matrix, cannot convert to Euler Parameter");
                continue;
            }
        } else if let Some(angles) = item.data.get(&Parameter::Angles) {
            let unit = item
                .data
                .get(&Parameter::Units)
                .ok_or(DataSetError::Conversion {
                    action: format!("no unit data for FK ID {id}"),
                })?;
            let mut angle_data = angles.to_vec_f64().map_err(|_| DataSetError::Conversion {
                action: format!("Angle data must be a Matrix but was {angles:?}"),
            })?;
            if unit == &KPLValue::String("ARCSECONDS".to_string()) {
                // Convert the angles data into degrees
                for item in &mut angle_data {
                    *item /= 3600.0;
                }
            }
            // Build the quaternion from the Euler matrices
            let from = id;
            let to = item
                .data
                .get(&Parameter::Center)
                .ok_or(DataSetError::Conversion {
                    action: "missing Center parameter".to_owned(),
                })?;
            let to = to.to_i32().map_err(|_| DataSetError::Conversion {
                action: format!("Center parameter must be an Integer but was {to:?}"),
            })?;
            if let Some(class) = item.data.get(&Parameter::Class) {
                let class_val = class.to_i32().map_err(|_| DataSetError::Conversion {
                    action: format!("Class must be an Integer but was {class:?}"),
                })?;
                if class_val == 4 {
                    // This is a relative frame.
                    let relative_to = item.data.get(&Parameter::Relative).ok_or(DataSetError::Conversion {
                        action: format!("frame {id} is class 4 relative to, but the RELATIVE_TO token was not found"),
                    })?;
                    let relative_to =
                        relative_to
                            .to_string()
                            .map_err(|_| DataSetError::Conversion {
                                action: format!(
                                    "Relative must be a String but was {relative_to:?}"
                                ),
                            })?;

                    // Always mark as something to update later.
                    ids_to_update.push((id, relative_to.clone()));
                }
            }

            let mut dcm = Matrix3::identity();

            let axes = item
                .data
                .get(&Parameter::Axes)
                .ok_or(DataSetError::Conversion {
                    action: "Missing Axes parameter".to_owned(),
                })?;
            let axes = axes.to_vec_f64().map_err(|_| DataSetError::Conversion {
                action: format!("Axes must be a Matrix but was {axes:?}"),
            })?;

            if axes.len() != angle_data.len() {
                return Err(DataSetError::Conversion {
                    action: format!(
                        "Mismatch between axes length ({}) and angle_data length ({})",
                        axes.len(),
                        angle_data.len()
                    ),
                });
            }

            for (i, rot) in axes.iter().enumerate() {
                let this_dcm = if rot == &1.0 {
                    r1(angle_data[i].to_radians())
                } else if rot == &2.0 {
                    r2(angle_data[i].to_radians())
                } else {
                    r3(angle_data[i].to_radians())
                };
                dcm *= this_dcm;
            }
            // Convert to quaternion
            let q = DCM {
                rot_mat: dcm,
                to,
                from,
                rot_mat_dt: None,
            }
            .into();

            dataset.push(q, Some(id), item.name.as_deref())?;
        } else if let Some(matrix) = item.data.get(&Parameter::Matrix) {
            let mat_data = matrix.to_vec_f64().map_err(|_| DataSetError::Conversion {
                action: format!("Matrix parameter must be a Matrix but was {matrix:?}"),
            })?;
            if mat_data.len() != 9 {
                return Err(DataSetError::Conversion {
                    action: format!("Matrix data must be length 9 but was {}", mat_data.len()),
                });
            }
            let rot_mat = Matrix3::new(
                mat_data[0],
                mat_data[1],
                mat_data[2],
                mat_data[3],
                mat_data[4],
                mat_data[5],
                mat_data[6],
                mat_data[7],
                mat_data[8],
            );
            let center = item
                .data
                .get(&Parameter::Center)
                .ok_or(DataSetError::Conversion {
                    action: "missing Center parameter".to_owned(),
                })?;
            let to = center.to_i32().map_err(|_| DataSetError::Conversion {
                action: format!("Center parameter must be an Integer but was {center:?}"),
            })?;

            let dcm = DCM {
                from: id,
                to,
                rot_mat,
                rot_mat_dt: None,
            };

            dataset.push(dcm.into(), Some(id), item.name.as_deref())?;
        }
    }

    // Finally, let's update the frames of the IDs defined as relative.
    for (id, relative_to) in ids_to_update {
        let parent_idx = dataset
            .lut
            .by_name
            .get(&(relative_to.as_str().try_into().unwrap()))
            .ok_or(DataSetError::Conversion {
                action: format!(
                    "frame {id} is class 4 relative to `{relative_to}`, but that frame is not found"
                ),
            })?;

        let parent_id = dataset.data[(*parent_idx) as usize].to;

        // Modify this EP.
        let index = dataset.lut.by_id.get(&id).unwrap();
        // Grab the data
        let this_q = dataset.data.get_mut(*index as usize).unwrap();
        this_q.to = parent_id;
    }

    dataset.set_crc32();
    dataset.metadata = Metadata::default();
    dataset.metadata.dataset_type = DataSetType::EulerParameterData;

    Ok(dataset)
}
