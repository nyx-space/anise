/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::{collections::HashMap, str::FromStr};

use log::warn;

use super::{parser::Assignment, KPLItem, KPLValue, Parameter};

#[derive(Debug, Default)]
pub struct TPCItem {
    pub body_id: Option<i32>,
    pub data: HashMap<Parameter, KPLValue>,
}

impl KPLItem for TPCItem {
    type Parameter = Parameter;

    fn extract_key(data: &Assignment) -> i32 {
        if data.keyword.starts_with("BODY") {
            let parts: Vec<&str> = data.keyword.split('_').collect();
            parts[0][4..].parse::<i32>().unwrap()
        } else {
            -1
        }
    }

    fn data(&self) -> &HashMap<Self::Parameter, KPLValue> {
        &self.data
    }

    fn parse(&mut self, data: Assignment) {
        if data.keyword.starts_with("BODY") {
            if let Some((body_info, param)) = data.keyword.split_once('_') {
                let body_id = body_info[4..].parse::<i32>().ok();
                if self.body_id.is_some() && self.body_id != body_id {
                    warn!("Got body {body_id:?} but expected {:?}", self.body_id);
                } else {
                    self.body_id = body_id;
                }
                if let Ok(param) = Parameter::from_str(param) {
                    self.data.insert(param, data.to_value());
                } else if param != "GMLIST" {
                    warn!("Unknown parameter `{param}` -- ignoring");
                }
            }
        }
    }
}

#[test]
fn test_parse_pck() {
    use crate::naif::kpl::parser::parse_file;
    let assignments = parse_file::<_, TPCItem>("../data/pck00008.tpc", false).unwrap();

    // Tests that we can parse single and multi-line data for Earth related info
    let expt_nutprec = [
        125.045,
        -1935.5364525,
        250.089,
        -3871.072905,
        260.008,
        475263.3328725,
        176.625,
        487269.629985,
        357.529,
        35999.0509575,
        311.589,
        964468.49931,
        134.963,
        477198.869325,
        276.617,
        12006.300765,
        34.226,
        63863.5132425,
        15.134,
        -5806.6093575,
        119.743,
        131.84064,
        239.961,
        6003.1503825,
        25.053,
        473327.79642,
    ];

    assert_eq!(
        assignments[&3].data[&Parameter::NutPrecAngles],
        KPLValue::Matrix(expt_nutprec.into())
    );
    let expt_pole_ra = [0.0, -0.641, 0.0];
    assert_eq!(
        assignments[&399].data[&Parameter::PoleRa],
        KPLValue::Matrix(expt_pole_ra.into())
    );

    // Check the same for Jupiter, especially since it has a plus sign in front of the f64
    let expt_pole_pm = [284.95, 870.5366420, 0.0];
    assert_eq!(
        assignments[&599].data[&Parameter::PrimeMeridian],
        KPLValue::Matrix(expt_pole_pm.into())
    );

    let expt_nutprec = [
        73.32, 91472.9, 24.62, 45137.2, 283.90, 4850.7, 355.80, 1191.3, 119.90, 262.1, 229.80,
        64.3, 352.35, 2382.6, 113.35, 6070.0, 146.64, 182945.8, 49.24, 90274.4,
    ];
    assert_eq!(
        assignments[&5].data[&Parameter::NutPrecAngles],
        KPLValue::Matrix(expt_nutprec.into())
    );

    // Check for Neptune which has the NUT_PREC_PM
    let expt_nut_prec_pm = [-0.48, 0., 0., 0., 0., 0., 0., 0.];
    assert_eq!(
        assignments[&899].data[&Parameter::NutPrecPm],
        KPLValue::Matrix(expt_nut_prec_pm.into())
    );
}

#[test]
fn test_parse_gm() {
    use crate::naif::kpl::parser::parse_file;
    let assignments = parse_file::<_, TPCItem>("../data/gm_de431.tpc", false).unwrap();

    // Basic values testing
    assert_eq!(
        assignments[&1].data[&Parameter::GravitationalParameter],
        KPLValue::Float(2.203_178_000_000_002E4)
    );

    assert_eq!(
        assignments[&399].data[&Parameter::GravitationalParameter],
        KPLValue::Float(3.986_004_354_360_96E5)
    );
}

#[test]
fn test_anise_conversion() {
    use crate::errors::InputOutputError;
    use crate::naif::kpl::parser::convert_tpc;
    use crate::{file2heap, file_mmap, structure::dataset::DataSet};
    use std::fs::File;
    use std::path::PathBuf;

    let dataset = convert_tpc("../data/pck00008.tpc", "../data/gm_de431.tpc").unwrap();

    assert!(!dataset.is_empty(), "should not be empty");
    assert_eq!(dataset.lut.by_id.len(), 49);

    let path = "../target/gm_pck_08.anise";

    // Test saving
    dataset.save_as(&PathBuf::from(path), true).unwrap();

    // Test reloading
    let bytes = file2heap!(path).unwrap();
    let reloaded = DataSet::from_bytes(bytes);

    assert_eq!(reloaded, dataset);

    // Test loading from file loaded on heap
    use std::fs;
    let data = fs::read(path).unwrap();
    let reloaded = DataSet::from_bytes(data);
    assert_eq!(reloaded, dataset);

    // Test reloading with real mmap
    let mmap = file_mmap!(path).unwrap();
    let reloaded = DataSet::from_bytes(mmap);
    assert_eq!(reloaded, dataset);

    // If all of these work, update the "official" PCA files.
    let pck08 = convert_tpc("../data/pck00008.tpc", "../data/gm_de431.tpc").unwrap();
    println!("PCK08 checksum = {}", pck08.crc32());
    pck08
        .save_as(&PathBuf::from("../data/pck08.pca"), true)
        .unwrap();
    let pck11 = convert_tpc("../data/pck00011.tpc", "../data/gm_de431.tpc").unwrap();
    println!("PCK11 checksum = {}", pck11.crc32());
    pck11
        .save_as(&PathBuf::from("../data/pck11.pca"), true)
        .unwrap();
}
