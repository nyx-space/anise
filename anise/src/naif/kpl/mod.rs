/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::str::FromStr;
use std::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

use snafu::{whatever, Whatever};

use self::parser::Assignment;

pub mod fk;

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
    pub fn to_vec_f64(&self) -> Result<Vec<f64>, Whatever> {
        match self {
            KPLValue::Matrix(data) => Ok(data.clone()),
            _ => whatever!("can only convert matrices to vec of f64 but this is {self:?}"),
        }
    }

    pub fn to_i32(&self) -> Result<i32, Whatever> {
        match self {
            KPLValue::Integer(data) => Ok(*data),
            _ => whatever!("can only convert Integer to i32 but this is {self:?}"),
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

impl TryFrom<&KPLValue> for f64 {
    type Error = Whatever;

    fn try_from(value: &KPLValue) -> Result<Self, Self::Error> {
        match value {
            KPLValue::Float(data) => Ok(*data),
            _ => whatever!("can only convert float to f64 but this is {value:?}"),
        }
    }
}

/// Known KPL parameters
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Parameter {
    NutPrecRa,
    NutPrecDec,
    NutPrecPm,
    NutPrecAngles,
    MaxPhaseDegree,
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
    type Err = Whatever;

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
            "MAX_PHASE_DEGREE" => Ok(Self::MaxPhaseDegree),
            "GMLIST" | "NAME" | "SPEC" => {
                whatever!("unsupported parameter `{s}`")
            }
            _ => {
                whatever!("unknown parameter `{s}`")
            }
        }
    }
}
