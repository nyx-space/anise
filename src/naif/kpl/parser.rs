/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

// Credit: ChatGPT for 80% of the code to parse the file from the SPICE docs.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use log::{error, info, warn};

use crate::almanac::MAX_PLANETARY_DATA;
use crate::naif::kpl::tpc::TPCItem;
use crate::naif::kpl::Parameter;
use crate::structure::dataset::{DataSet, DataSetBuilder, DataSetError, DataSetType};
use crate::structure::metadata::Metadata;
use crate::structure::planetocentric::ellipsoid::Ellipsoid;
use crate::structure::planetocentric::phaseangle::PhaseAngle;
use crate::structure::planetocentric::PlanetaryData;

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

pub fn parse_file<P: AsRef<Path>, I: KPLItem>(
    file_path: P,
    show_comments: bool,
) -> Result<HashMap<i32, I>, DataSetError> {
    let file = File::open(file_path).expect("Failed to open file");
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

pub fn convert_tpc<'a, P: AsRef<Path>>(
    pck: P,
    gm: P,
) -> Result<DataSet<'a, PlanetaryData, MAX_PLANETARY_DATA>, DataSetError> {
    let mut buf = vec![];
    let mut dataset_builder = DataSetBuilder::default();

    let gravity_data = parse_file::<_, TPCItem>(gm, false)?;
    let mut planetary_data = parse_file::<_, TPCItem>(pck, false)?;

    for (key, value) in gravity_data {
        if let Some(planet_data) = planetary_data.get_mut(&key) {
            for (gk, gv) in value.data {
                planet_data.data.insert(gk, gv);
            }
        }
    }

    // Now that planetary_data has everything, we'll create a vector of the planetary data in the ANISE ASN1 format.

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
                                _ => todo!(),
                            },
                            None => None,
                        };

                        let constant = match planetary_data.data.get(&Parameter::PoleRa) {
                            Some(data) => match data {
                                KPLValue::Matrix(pole_ra_data) => {
                                    let pola_ra = PhaseAngle::maybe_new(pole_ra_data);
                                    let pola_dec_data: Vec<f64> = planetary_data.data
                                        [&Parameter::PoleDec]
                                        .to_vec_f64()
                                        .unwrap();
                                    let pola_dec = PhaseAngle::maybe_new(&pola_dec_data);

                                    let prime_mer_data: Vec<f64> = planetary_data.data
                                        [&Parameter::PoleDec]
                                        .to_vec_f64()
                                        .unwrap();
                                    let prime_mer = PhaseAngle::maybe_new(&prime_mer_data);

                                    PlanetaryData {
                                        object_id,
                                        mu_km3_s2: *mu_km3_s2,
                                        shape: ellipsoid,
                                        pole_right_ascension: pola_ra,
                                        pole_declination: pola_dec,
                                        prime_meridian: prime_mer,
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
                                    ..Default::default()
                                }
                            }
                        };

                        dataset_builder.push_into(&mut buf, constant, Some(object_id), None)?;
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

    println!("Added {} items", dataset_builder.dataset.lut.by_id.len());

    let mut dataset = dataset_builder.finalize(buf)?;
    dataset.metadata = Metadata::default();
    dataset.metadata.dataset_type = DataSetType::PlanetaryData;

    Ok(dataset)
}
