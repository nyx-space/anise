use anise::naif::kpl::{Parameter, KPLValue};
use anise::naif::kpl::fk::FKItem;
use anise::naif::kpl::parser::Assignment;
use anise::naif::kpl::tpc::TPCItem;
use anise::math::rotation::DCM;
use anise::math::rotation::MRP;
use anise::math::rotation::Quaternion;
use anise::math::Vector3;
use anise::frames::Frame;
use libfuzzer_sys::arbitrary;
use hifitime::Epoch;
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

#[derive(arbitrary::Arbitrary, Debug)]
pub struct ArbitraryEpoch {
    pub seconds_since_1900_utc: f64,
}

impl From<ArbitraryEpoch> for Epoch {
    fn from(val: ArbitraryEpoch) -> Self {
        Epoch::from_utc_seconds(val.seconds_since_1900_utc)
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
pub struct ArbitraryFrame {
    pub ephemeris_id: i32,
    pub orientation_id: i32,
}

impl From<ArbitraryFrame> for Frame {
    fn from(val: ArbitraryFrame) -> Self {
        Self {
            ephemeris_id: val.ephemeris_id,
            orientation_id: val.orientation_id,
            mu_km3_s2: None,
            shape: None,
        }
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
pub struct ArbitraryQuaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub from: i32,
    pub to: i32,
}

impl From<ArbitraryQuaternion> for Quaternion {
    fn from(val: ArbitraryQuaternion) -> Self {
        Quaternion::new(val.w, val.x, val.y, val.z, val.from, val.to)
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
pub struct ArbitraryVector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<ArbitraryVector3> for Vector3 {
    fn from(val: ArbitraryVector3) -> Self {
        Vector3::new(val.x, val.y, val.z)
    }
}

#[derive(arbitrary::Arbitrary, Debug)]
pub struct ArbitraryMRP {
    pub s0: f64,
    pub s1: f64,
    pub s2: f64,
    pub from: i32,
    pub to: i32,
}

impl From<ArbitraryMRP> for MRP {
    fn from(val: ArbitraryMRP) -> Self {
        MRP {
            s0: val.s0,
            s1: val.s1,
            s2: val.s2,
            from: val.from,
            to: val.to,
        }
    }
}
