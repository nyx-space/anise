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
                } else {
                    warn!("Unknown parameter `{param}` -- ignoring");
                }
            }
        }
    }
}

#[test]
fn test_parse_pck() {
    use crate::naif::kpl::parser::parse_file;
    let assignments = parse_file::<_, TPCItem>("data/pck00008.tpc", false).unwrap();

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
}

#[test]
fn test_parse_gm() {
    use crate::naif::kpl::parser::parse_file;
    let assignments = parse_file::<_, TPCItem>("data/gm_de431.tpc", false).unwrap();

    // Basic values testing
    assert_eq!(
        assignments[&1].data[&Parameter::GravitationalParameter],
        KPLValue::Float(2.2031780000000021E+04)
    );

    assert_eq!(
        assignments[&399].data[&Parameter::GravitationalParameter],
        KPLValue::Float(3.9860043543609598E+05)
    );
}

#[test]
fn test_anise_conversion() {
    use crate::naif::kpl::parser::parse_file;
    use crate::structure::planetocentric::{
        ellipsoid::Ellipsoid, phaseangle::PhaseAngle, planetary_constant::PlanetaryConstant,
    };

    let gravity_data = parse_file::<_, TPCItem>("data/gm_de431.tpc", false).unwrap();
    let mut planetary_data = parse_file::<_, TPCItem>("data/pck00008.tpc", false).unwrap();

    for (key, value) in gravity_data {
        match planetary_data.get_mut(&key) {
            Some(planet_data) => {
                for (gk, gv) in value.data {
                    planet_data.data.insert(gk, gv);
                }
            }
            None => {}
        }
    }

    // Now that planetary_data has everything, we'll create a vector of the planetary data in the ANISE ASN1 format.

    let mut anise_data = vec![];
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
                                    let pola_ra = PhaseAngle::maybe_new(&pole_ra_data);
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

                                    PlanetaryConstant {
                                        object_id,
                                        mu_km3_s2: *mu_km3_s2,
                                        shape: ellipsoid,
                                        pole_right_ascension: pola_ra,
                                        pole_declination: pola_dec,
                                        prime_meridian: prime_mer,
                                        nut_prec_angles: Default::default(),
                                    }
                                }
                                _ => unreachable!(),
                            },
                            None => {
                                // Assume not rotation data available
                                PlanetaryConstant {
                                    object_id,
                                    mu_km3_s2: *mu_km3_s2,
                                    shape: ellipsoid,
                                    ..Default::default()
                                }
                            }
                        };

                        anise_data.push(constant);
                    }
                    _ => panic!("{mu_km3_s2_value:?}"),
                }
            }
            None => {
                println!(
                    "{object_id} => No gravity data in {:?}",
                    planetary_data.data
                )
            }
        }
    }

    println!("Added {} items", anise_data.len());
}
