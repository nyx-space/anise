/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::{collections::HashMap, str::FromStr};

use crate::prelude::AniseError;

use super::{parser::Assignment, KPLItem};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Parameter {
    NutPrecRa,
    NutPrecDec,
    NutPrecPm,
    NutPrecAngles,
    LongAxis,
    PoleRa,
    PoleDec,
    Radii,
    PolarMotion,
    GeoMagNorthPoleCenterDipoleLatitude,
    GeoMagNorthPoleCenterDipoleLongitude,
    GravitationalParameter,
}

impl FromStr for Parameter {
    type Err = AniseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NUT_PREC_RA" => Ok(Self::NutPrecRa),
            "NUT_PREC_DEC" => Ok(Self::NutPrecDec),
            "NUT_PREC_PM" => Ok(Self::NutPrecPm),
            "LONG_AXIS" => Ok(Self::LongAxis),
            "POLE_DEC" => Ok(Self::PoleDec),
            "POLE_RA" => Ok(Self::PoleRa),
            "RADII" => Ok(Self::Radii),
            "PM" => Ok(Self::PolarMotion),
            "NUT_PREC_ANGLES" => Ok(Self::NutPrecAngles),
            "N_GEOMAG_CTR_DIPOLE_LAT" => Ok(Self::GeoMagNorthPoleCenterDipoleLatitude),
            "N_GEOMAG_CTR_DIPOLE_LON" => Ok(Self::GeoMagNorthPoleCenterDipoleLongitude),
            "GM" => Ok(Self::GravitationalParameter),
            _ => {
                println!("WHAT? `{s}`");
                Err(AniseError::ParameterNotSpecified)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct TPCItem {
    pub body_id: Option<i32>,
    pub data: HashMap<Parameter, Vec<f64>>,
}

impl KPLItem for TPCItem {
    type Parameter = Parameter;

    fn extract_key(keyword: &str) -> i32 {
        if keyword.starts_with("BODY") {
            let parts: Vec<&str> = keyword.split('_').collect();
            parts[0][4..].parse::<i32>().unwrap()
        } else {
            -1
        }
    }

    fn data(&self) -> &HashMap<Self::Parameter, Vec<f64>> {
        &self.data
    }

    fn parse(&mut self, data: Assignment) {
        if data.keyword.starts_with("BODY") {
            if let Some((body_info, param)) = data.keyword.split_once('_') {
                let body_id = body_info[4..].parse::<i32>().ok();
                if self.body_id.is_some() && self.body_id != body_id {
                    println!("Got body {body_id:?} but expected {:?}", self.body_id);
                } else {
                    self.body_id = body_id;
                }
                let param = Parameter::from_str(param).unwrap();
                self.data.insert(param, data.value_to_vec_f64());
            }
        }
    }
}

#[test]
fn test_parse_pck() {
    use crate::naif::kpl::parser::parse_file;
    let assignments = parse_file::<_, TPCItem>("data/pck00008.tpc", false).unwrap();
    dbg!(assignments);
}

#[test]
fn test_parse_gm() {
    use crate::naif::kpl::parser::parse_file;
    let assignments = parse_file::<_, TPCItem>("data/gm_de431.tpc", false).unwrap();
    dbg!(assignments);
}
