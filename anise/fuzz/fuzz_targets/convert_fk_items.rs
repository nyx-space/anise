#![no_main]
use anise::naif::kpl::{Parameter, KPLValue};
use anise::naif::kpl::fk::FKItem;
use anise::naif::kpl::parser::convert_fk_items;
use std::collections::HashMap;

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary;

#[derive(arbitrary::Arbitrary, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum ArbitraryParameter {
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

impl From<ArbitraryParameter> for Parameter {
    fn from(val: ArbitraryParameter) -> Self {
        match val {
            ArbitraryParameter::NutPrecRa => Parameter::NutPrecRa,
            ArbitraryParameter::NutPrecDec => Parameter::NutPrecDec,
            ArbitraryParameter::NutPrecPm => Parameter::NutPrecPm,
            ArbitraryParameter::NutPrecAngles => Parameter::NutPrecAngles,
            ArbitraryParameter::MaxPhaseDegree => Parameter::MaxPhaseDegree,
            ArbitraryParameter::LongAxis => Parameter::LongAxis,
            ArbitraryParameter::PoleRa => Parameter::PoleRa,
            ArbitraryParameter::PoleDec => Parameter::PoleDec,
            ArbitraryParameter::Radii => Parameter::Radii,
            ArbitraryParameter::PrimeMeridian => Parameter::PrimeMeridian,
            ArbitraryParameter::GeoMagNorthPoleCenterDipoleLatitude => Parameter::GeoMagNorthPoleCenterDipoleLatitude,
            ArbitraryParameter::GeoMagNorthPoleCenterDipoleLongitude => Parameter::GeoMagNorthPoleCenterDipoleLongitude,
            ArbitraryParameter::GravitationalParameter => Parameter::GravitationalParameter,
            ArbitraryParameter::Class => Parameter::Class,
            ArbitraryParameter::Center => Parameter::Center,
            ArbitraryParameter::ClassId => Parameter::ClassId,
            ArbitraryParameter::Angles => Parameter::Angles,
            ArbitraryParameter::Relative => Parameter::Relative,
            ArbitraryParameter::Matrix => Parameter::Matrix,
            ArbitraryParameter::Units => Parameter::Units,
            ArbitraryParameter::Axes => Parameter::Axes,
        }
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
pub enum ArbitraryKPLValue {
    Float(f64),
    Matrix(Vec<f64>),
    String(String),
    Integer(i32),
}

impl From<ArbitraryKPLValue> for KPLValue {
    fn from(val: ArbitraryKPLValue) -> Self {
        match val {
            ArbitraryKPLValue::Float(f) => KPLValue::Float(f),
            ArbitraryKPLValue::Matrix(m) => KPLValue::Matrix(m),
            ArbitraryKPLValue::String(s) => KPLValue::String(s),
            ArbitraryKPLValue::Integer(i) => KPLValue::Integer(i),
        }
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
struct ArbitraryFKItem {
    pub body_id: Option<i32>,
    pub name: Option<String>,
    pub data: HashMap<ArbitraryParameter, ArbitraryKPLValue>
}

impl From<ArbitraryFKItem> for FKItem {
    fn from(val: ArbitraryFKItem) -> Self {
        let data = val.data
            .into_iter()
            .map(|(param, val)| (param.into(), val.into()))
            .collect();

        Self {
            body_id: val.body_id,
            name: val.name,
            data
        }
    }
}

fuzz_target!(|data: HashMap<i32, ArbitraryFKItem>| {
    let assignments = data
        .into_iter()
        .map(|(idx, item)| (idx, item.into()))
        .collect();
    let _ = convert_fk_items(assignments);
});
