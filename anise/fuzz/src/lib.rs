use anise::naif::kpl::{Parameter, KPLValue};
use anise::naif::kpl::fk::FKItem;
use anise::naif::kpl::parser::Assignment;
use anise::naif::kpl::tpc::TPCItem;
use anise::math::rotation::DCM;
use libfuzzer_sys::arbitrary;
use std::collections::HashMap;

#[derive(arbitrary::Arbitrary, Debug)]
pub struct ArbitraryAssignment {
    pub keyword: String,
    pub value: String,
}

impl From<ArbitraryAssignment> for Assignment {
    fn from(val: ArbitraryAssignment) -> Self {
        Self {
            keyword: val.keyword,
            value: val.value,
        }
    }
}

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
pub struct ArbitraryFKItem {
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

#[derive(arbitrary::Arbitrary, Debug)]
pub struct ArbitraryTPCItem {
    pub body_id: Option<i32>,
    pub data: HashMap<ArbitraryParameter, ArbitraryKPLValue>
}

impl From<ArbitraryTPCItem> for TPCItem {
    fn from(val: ArbitraryTPCItem) -> Self {
        let data = val.data
            .into_iter()
            .map(|(param, val)| (param.into(), val.into()))
            .collect();

        Self {
            body_id: val.body_id,
            data
        }
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
pub struct ArbitraryDCM {
    pub rot_mat: [[f64; 3]; 3],
    pub from: i32,
    pub to: i32,
    pub rot_mat_dt: Option<[[f64; 3]; 3]>,
}

impl From<ArbitraryDCM> for DCM {
    fn from(val: ArbitraryDCM) -> Self {
        Self {
            rot_mat: val.rot_mat.into(),
            from: val.from,
            to: val.to,
            rot_mat_dt: val.rot_mat_dt.map(|m| m.into()),
        }
    }
}
