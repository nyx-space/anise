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
use crate::math::rotation::{r1, r2, r3, DCM};
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
            } else if let Ok(as_f64) = vec[0].parse::<f64>() {
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
    let reader = BufReader::new(file);

    let mut block_type = BlockType::Comment;
    let mut assignments = vec![];

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
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
                // This is a continuation of the previous line, so let's grab the data and append the value we're reding now.
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
    let mut dataset = PlanetaryDataSet::default();

    let gravity_data = parse_file::<_, TPCItem>(gm, false)?;
    let mut planetary_data = parse_file::<_, TPCItem>(pck, false)?;

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
                                    _ => unreachable!(),
                                },
                                _ => panic!("radii_km should be float or matrix, got {radii_km:?}"),
                            },
                            None => None,
                        };

                        let mut constant = match planetary_data.data.get(&Parameter::PoleRa) {
                            Some(data) => match data {
                                KPLValue::Matrix(pole_ra_data) => {
                                    let mut pole_ra_data = pole_ra_data.clone();
                                    if let Some(coeffs) =
                                        planetary_data.data.get(&Parameter::NutPrecRa)
                                    {
                                        pole_ra_data.extend(coeffs.to_vec_f64().unwrap());
                                    }
                                    let pola_ra = PhaseAngle::maybe_new(&pole_ra_data);

                                    let mut pola_dec_data: Vec<f64> = planetary_data.data
                                        [&Parameter::PoleDec]
                                        .to_vec_f64()
                                        .unwrap();
                                    if let Some(coeffs) =
                                        planetary_data.data.get(&Parameter::NutPrecDec)
                                    {
                                        pola_dec_data.extend(coeffs.to_vec_f64().unwrap());
                                    }
                                    let pola_dec = PhaseAngle::maybe_new(&pola_dec_data);

                                    let mut prime_mer_data: Vec<f64> = planetary_data.data
                                        [&Parameter::PrimeMeridian]
                                        .to_vec_f64()
                                        .unwrap();
                                    if let Some(coeffs) =
                                        planetary_data.data.get(&Parameter::NutPrecPm)
                                    {
                                        prime_mer_data.extend(coeffs.to_vec_f64().unwrap());
                                    }
                                    let prime_mer = PhaseAngle::maybe_new(&prime_mer_data);

                                    let long_axis =
                                        match planetary_data.data.get(&Parameter::LongAxis) {
                                            Some(val) => match val {
                                                KPLValue::Float(data) => Some(*data),
                                                KPLValue::Matrix(data) => Some(data[0]),
                                                _ => panic!(
                                                "long axis must be float or matrix, got {val:?}"
                                            ),
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
                                _ => unreachable!(),
                            },
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
                            let phase_deg =
                                match planetary_data.data.get(&Parameter::MaxPhaseDegree) {
                                    Some(val) => (val.to_i32().unwrap() + 1) as usize,
                                    None => 2,
                                };
                            let nut_prec_data = nut_prec_val.to_vec_f64().unwrap();
                            let mut coeffs = [PhaseAngle::<0>::default(); MAX_NUT_PREC_ANGLES];
                            let mut num = 0;
                            for (i, nut_prec) in nut_prec_data.chunks(phase_deg).enumerate() {
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
                        "expected gravity parameter to be a float but got {mu_km3_s2_value:?}"
                    ),
                }
            }
            None => {
                warn!("Skipping {object_id}: no gravity data")
            }
        }
    }

    println!("Added {} items", dataset.lut.by_id.len());

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
    let mut dataset = EulerParameterDataSet::default();

    let assignments = parse_file::<_, FKItem>(fk_file_path, show_comments)?;

    // Add all of the data into the data set
    for (id, item) in assignments {
        if !item.data.contains_key(&Parameter::Angles)
            && !item.data.contains_key(&Parameter::Matrix)
        {
            warn!("{id} contains neither angles nor matrix, cannot convert to Euler Parameter");
            continue;
        } else if let Some(angles) = item.data.get(&Parameter::Angles) {
            let unit = item
                .data
                .get(&Parameter::Units)
                .ok_or(DataSetError::Conversion {
                    action: format!("no unit data for FK ID {id}"),
                })?;
            let mut angle_data = angles.to_vec_f64().unwrap();
            if unit == &KPLValue::String("ARCSECONDS".to_string()) {
                // Convert the angles data into degrees
                for item in &mut angle_data {
                    *item /= 3600.0;
                }
            }
            // Build the quaternion from the Euler matrices
            let from = id;
            let to = item.data[&Parameter::Center].to_i32().unwrap();

            let mut dcm = Matrix3::identity();

            for (i, rot) in item.data[&Parameter::Axes]
                .to_vec_f64()
                .unwrap()
                .iter()
                .enumerate()
            {
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
            let mat_data = matrix.to_vec_f64().unwrap();
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
            let dcm = DCM {
                from: id,
                to: item.data[&Parameter::Center].to_i32().unwrap(),
                rot_mat,
                rot_mat_dt: None,
            };
            dataset.push(dcm.into(), Some(id), item.name.as_deref())?;
        }
    }

    dataset.set_crc32();
    dataset.metadata = Metadata::default();
    dataset.metadata.dataset_type = DataSetType::EulerParameterData;

    Ok(dataset)
}
