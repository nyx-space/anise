/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::str::FromStr;
use std::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

use crate::prelude::AniseError;

use self::parser::Assignment;

pub mod fk;
#[cfg(feature = "std")]
pub mod parser;
pub mod tpc;

pub trait KPLItem: Debug + Default {
    type Parameter: Eq + Hash;
    /// The key used for fetching
    fn extract_key(data: &Assignment) -> i32;
    fn data(&self) -> &HashMap<Self::Parameter, KPLValue>;
    fn parse(&mut self, data: Assignment);
}

#[derive(Clone, Debug, PartialEq)]
pub enum KPLValue {
    Float(f64),
    Matrix(Vec<f64>),
    String(String),
    Integer(i32),
}

impl KPLValue {
    pub fn to_vec_f64(&self) -> Result<Vec<f64>, AniseError> {
        match self {
            KPLValue::Matrix(data) => Ok(data.clone()),
            _ => Err(AniseError::ParameterNotSpecified),
        }
    }
}

impl From<f64> for KPLValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<i32> for KPLValue {
    fn from(value: i32) -> Self {
        Self::Integer(value)
    }
}

impl From<String> for KPLValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

/// Known KPL parameters
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
    PrimeMeridian,
    GeoMagNorthPoleCenterDipoleLatitude,
    GeoMagNorthPoleCenterDipoleLongitude,
    GravitationalParameter,
    Class,
    Center,
    ClassId,
    Angles,
    Relative,
    Matrix,
    Units,
    Axes,
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
            "PM" => Ok(Self::PrimeMeridian),
            "NUT_PREC_ANGLES" => Ok(Self::NutPrecAngles),
            "N_GEOMAG_CTR_DIPOLE_LAT" => Ok(Self::GeoMagNorthPoleCenterDipoleLatitude),
            "N_GEOMAG_CTR_DIPOLE_LON" => Ok(Self::GeoMagNorthPoleCenterDipoleLongitude),
            "GM" => Ok(Self::GravitationalParameter),
            "CLASS" => Ok(Self::Class),
            "CLASS_ID" => Ok(Self::ClassId),
            "CENTER" => Ok(Self::Center),
            "ANGLES" => Ok(Self::Angles),
            "RELATIVE" => Ok(Self::Relative),
            "MATRIX" => Ok(Self::Matrix),
            "UNITS" => Ok(Self::Units),
            "AXES" => Ok(Self::Axes),
            "GMLIST" | "NAME" | "SPEC" => {
                // This is a known unsupported parameter
                Err(AniseError::ParameterNotSpecified)
            }
            _ => {
                println!("WHAT IS `{s}` ?");
                Err(AniseError::ParameterNotSpecified)
            }
        }
    }
}
