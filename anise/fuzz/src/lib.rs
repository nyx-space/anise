use anise::naif::kpl::{Parameter, KPLValue};
use anise::naif::kpl::fk::FKItem;
use anise::naif::kpl::parser::Assignment;
use anise::naif::kpl::tpc::TPCItem;
use anise::naif::daf::daf::DAF;
use anise::naif::pck::BPCSummaryRecord;
use anise::naif::spk::SPKSummaryRecord;
use anise::math::rotation::DCM;
use anise::math::rotation::MRP;
use anise::math::rotation::Quaternion;
use anise::math::Vector3;
use anise::frames::Frame;
use libfuzzer_sys::arbitrary;
use hifitime::Epoch;
use bytes::Bytes;
use std::collections::HashMap;
use std::marker::PhantomData;

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
/* Stephan's arbitraryBPC
#[derive(arbitrary::Arbitrary, Debug)]
pub struct ArbitraryBPC {
    pub bytes: Vec<u8>,
    pub crc32_checksum: u32,
}

impl From<ArbitraryBPC> for DAF<BPCSummaryRecord> {
    fn from(val: ArbitraryBPC) -> Self {
        DAF {
            bytes: Bytes::from(val.bytes),
            crc32_checksum: val.crc32_checksum,
            _daf_type: PhantomData,
        }
    }
}
*/

#[derive(Arbitrary, Debug)]
pub struct ArbitraryBPCSummaryRecord {
    pub start_epoch_et_s: f64,
    pub end_epoch_et_s: f64,
    pub frame_id: i32,
    pub inertial_frame_id: i32,
    pub data_type_i: i32,
    pub start_idx: i32,
    pub end_idx: i32,
    pub unused: i32,
}

impl From<ArbitraryBPCSummaryRecord> for BPCSummaryRecord {
    fn from(rec: ArbitraryBPCSummaryRecord) -> Self {
        BPCSummaryRecord {
            start_epoch_et_s: rec.start_epoch_et_s,
            end_epoch_et_s: rec.end_epoch_et_s,
            frame_id: rec.frame_id,
            inertial_frame_id: rec.inertial_frame_id,
            data_type_i: rec.data_type_i,
            start_idx: rec.start_idx,
            end_idx: rec.end_idx,
            unused: rec.unused,
        }
    }
}

#[derive(Arbitrary, Debug)]
pub struct ArbitraryBPC {
    #[arbitrary(with = arbitrary_bpc)]
    pub inner: DAF<BPCSummaryRecord>,
}

fn arbitrary_bpc(u: &mut Unstructured) -> arbitrary::Result<DAF<BPCSummaryRecord>> {
    // Generate random number of summary records (0-MAX_LOADED_BPCS for safety)
    let num_records = u.int_in_range(0..=7)?;
    let mut summaries = Vec::with_capacity(num_records);
    
    // Generate arbitrary BPC summary records
    for _ in 0..num_records {
        summaries.push(ArbitraryBPCSummaryRecord::arbitrary(u)?.into());
    }

    // Generate random data buffer, max 4kb?
    let data_size = u.int_in_range(0..=4096)?;
    let mut data = vec![0u8; data_size];
    u.fill_buffer(&mut data)?;

    // Create BPC with potentially invalid data to test error paths
    Ok(DAF::<BPCSummaryRecord>::new(summaries, data.into()))
}

#[derive(Arbitrary, Debug)]
pub struct ArbitrarySPKSummaryRecord {
    pub start_epoch_et_s: f64,
    pub end_epoch_et_s: f64,
    pub target_id: i32,
    pub center_id: i32,
    pub frame_id: i32,
    pub data_type_i: i32,
    pub start_idx: i32,
    pub end_idx: i32, 
}

impl From<ArbitrarySPKSummaryRecord> for SPKSummaryRecord {
    fn from(rec: ArbitrarySPKSummaryRecord) -> Self {
        SPKSummaryRecord {
            start_epoch_et_s: rec.start_epoch_et_s,
            end_epoch_et_s: rec.end_epoch_et_s,
            target_id: rec.target_id,
            center_id: rec.center_id,
            frame_id: rec.frame_id,
            data_type_i: rec.data_type_i,
            start_idx: rec.start_idx,
            end_idx: rec.end_idx,
        }
    }
}

#[derive(Arbitrary, Debug)]
pub struct ArbitrarySPK {
    #[arbitrary(with = arbitrary_spk)]
    pub inner: DAF<SPKSummaryRecord>,
}

fn arbitrary_spk(u: &mut Unstructured) -> arbitrary::Result<DAF<SPKSummaryRecord>> {
    // Generate random number of summary records (0-MAX_LOADED_SPKS for safety)
    let num_records = u.int_in_range(0..=31)?;
    let mut summaries = Vec::with_capacity(num_records);
    
    // Generate arbitrary summary records
    for _ in 0..num_records {
        summaries.push(ArbitrarySPKSummaryRecord::arbitrary(u)?.into());
    }

    // Generate random data buffer, max 4kb?
    let data_size = u.int_in_range(0..=4096)?;
    let mut data = vec![0u8; data_size];
    u.fill_buffer(&mut data)?;

    // Create SPK with potentially invalid data to test error paths
    Ok(DAF::<SPKSummaryRecord>::new(summaries, data.into()))
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

#[derive(arbitrary::Arbitrary, Debug)]
pub struct ArbitrarySPK {
    #[arbitrary(with = arbitrary_spk_bytes)]
    bytes: Vec<u8>,
}

fn arbitrary_spk_bytes(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    
    // Generate valid SPK header
    let file_record = FileRecord {
        identification: "SPK".to_string(),
        format_version: *u.choose(&[1, 2])?,
        start_epoch_et_s: u.arbitrary()?,
        end_epoch_et_s: u.arbitrary()?,
    };
    bytes.extend(file_record.as_bytes());
    
    // Generate 1-5 segments
    for _ in 0..u.int_in_range(1..=5)? {
        let summary = SPKSummaryRecord {
            target_id: u.arbitrary()?,
            center_id: u.arbitrary()?,
            start_epoch_et_s: u.arbitrary()?,
            end_epoch_et_s: u.arbitrary()?,
            data_type: *u.choose(&[1, 2, 3])?, // Valid SPK data types
        };
        bytes.extend(summary.as_bytes());
        
        // Generate random segment data (128-1024 bytes)
        bytes.extend(u.bytes(u.int_in_range(128..=1024)?)?);
    }
    
    // Compute and append CRC32 checksum
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&bytes);
    bytes.extend(&hasher.finalize().to_le_bytes());
    
    Ok(bytes)
}























