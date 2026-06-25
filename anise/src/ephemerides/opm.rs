/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{EphemerisError, OPMTimeParsingSnafu};
use crate::ephemerides::ephemeris::{Covariance, LocalFrame};
use crate::math::{Matrix6, Vector3, Vector6};
use crate::prelude::{Frame, Orbit};
use crate::structure::spacecraft::{DragData, Mass, SRPData, SpacecraftData};
use hifitime::{
    Duration, Epoch,
    efmt::{Format, Formatter},
};
use snafu::ResultExt;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;

#[cfg(feature = "python")]
use ndarray::Array1;
#[cfg(feature = "python")]
use numpy::{PyArray1, PyReadonlyArray1};
#[cfg(feature = "python")]
use pyo3::exceptions::PyValueError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyType;

/// A single maneuver as described in a CCSDS OPM.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "python", pyclass(from_py_object, module = "anise.astro"))]
pub struct Maneuver {
    /// Epoch of ignition
    pub epoch_ignition: Epoch,
    /// Duration of the maneuver (zero for an impulsive maneuver)
    pub duration: Duration,
    /// Change in mass (kilograms, should be negative or zero)
    pub delta_mass_kg: f64,
    /// Reference frame of the delta-v vector
    pub ref_frame: LocalFrame,
    /// Delta-v vector in km/s, expressed in `ref_frame`
    pub delta_v_km_s: Vector3,
}

/// A CCSDS Orbit Parameter Message (OPM): a single state vector
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "python", pyclass(from_py_object, module = "anise.astro"))]
pub struct Opm {
    /// Human-readable object name
    pub object_name: String,
    /// Object identifier
    pub object_id: String,
    /// The orbital state at the OPM epoch
    pub orbit: Orbit,
    /// Spacecraft physical constants (mass, SRP, drag); fields are `None` when absent
    pub spacecraft_data: SpacecraftData,
    /// Optional covariance at the state epoch
    pub covariance: Option<Covariance>,
    /// Zero or more maneuvers
    pub maneuvers: Vec<Maneuver>,
}

impl Opm {
    /// Builds a new OPM from its mandatory state vector. Optional data is set via public fields.
    pub fn new(orbit: Orbit) -> Self {
        Self {
            object_name: String::new(),
            object_id: String::new(),
            orbit,
            spacecraft_data: SpacecraftData::default(),
            covariance: None,
            maneuvers: Vec::new(),
        }
    }

    /// Adds a maneuver to this OPM.
    pub fn add_maneuver(&mut self, maneuver: Maneuver) {
        self.maneuvers.push(maneuver);
    }

    /// Writes this OPM to the provided path in CCSDS OPM (KVN) format.
    pub fn write_ccsds_opm<P: AsRef<Path>>(
        &self,
        path: P,
        originator: Option<String>,
        object_name: Option<String>,
    ) -> Result<(), EphemerisError> {
        let file = File::create(&path).map_err(|e| EphemerisError::OPMWritingError {
            details: format!("could not create file: {e}"),
        })?;
        let mut writer = BufWriter::new(file);

        let err_hdlr = |e| EphemerisError::OPMWritingError {
            details: format!("{e}"),
        };

        let iso8601_no_ts =
            Format::from_str("%Y-%m-%dT%H:%M:%S.%f").expect("static format string is valid");

        // Header
        writeln!(writer, "CCSDS_OPM_VERS = 3.0\n").map_err(err_hdlr)?;
        writeln!(
            writer,
            "COMMENT Built by ANISE, a modern rewrite of NASA/NAIF SPICE (https://nyxspace.com/anise)"
        )
        .map_err(err_hdlr)?;
        writeln!(
            writer,
            "CREATION_DATE = {}",
            Formatter::new(
                Epoch::now().map_err(|e| EphemerisError::OPMWritingError {
                    details: format!("could not get current epoch: {e}"),
                })?,
                iso8601_no_ts,
            )
        )
        .map_err(err_hdlr)?;
        writeln!(
            writer,
            "ORIGINATOR = {}\n",
            originator.unwrap_or("Nyx Space ANISE".to_string())
        )
        .map_err(err_hdlr)?;

        // Metadata
        let object_name = object_name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| {
                let n = self.object_name.trim();
                if n.is_empty() { "UNKNOWN" } else { n }
            });
        writeln!(writer, "OBJECT_NAME = {object_name}").map_err(err_hdlr)?;
        writeln!(writer, "OBJECT_ID = {}", self.object_id).map_err(err_hdlr)?;

        let frame = self.orbit.frame;
        let center = format!("{frame:e}");
        let ref_frame = format!("{frame:o}");
        writeln!(writer, "CENTER_NAME = {center}").map_err(err_hdlr)?;
        writeln!(
            writer,
            "REF_FRAME = {}",
            match ref_frame.trim() {
                "J2000" => "EME2000",
                other => other,
            }
        )
        .map_err(err_hdlr)?;
        writeln!(writer, "TIME_SYSTEM = {}\n", self.orbit.epoch.time_scale).map_err(err_hdlr)?;

        // State vector
        writeln!(writer, "COMMENT State Vector").map_err(err_hdlr)?;
        writeln!(
            writer,
            "EPOCH = {}",
            Formatter::new(self.orbit.epoch, iso8601_no_ts)
        )
        .map_err(err_hdlr)?;
        writeln!(writer, "X = {:E}", self.orbit.radius_km.x).map_err(err_hdlr)?;
        writeln!(writer, "Y = {:E}", self.orbit.radius_km.y).map_err(err_hdlr)?;
        writeln!(writer, "Z = {:E}", self.orbit.radius_km.z).map_err(err_hdlr)?;
        writeln!(writer, "X_DOT = {:E}", self.orbit.velocity_km_s.x).map_err(err_hdlr)?;
        writeln!(writer, "Y_DOT = {:E}", self.orbit.velocity_km_s.y).map_err(err_hdlr)?;
        writeln!(writer, "Z_DOT = {:E}", self.orbit.velocity_km_s.z).map_err(err_hdlr)?;

        // Spacecraft parameters
        if self.spacecraft_data.mass.is_some()
            || self.spacecraft_data.srp_data.is_some()
            || self.spacecraft_data.drag_data.is_some()
        {
            writeln!(writer, "\nCOMMENT Spacecraft parameters").map_err(err_hdlr)?;
            if let Some(mass) = self.spacecraft_data.mass {
                writeln!(writer, "MASS = {:E}", mass.dry_mass_kg).map_err(err_hdlr)?;
            }
            if let Some(srp) = self.spacecraft_data.srp_data {
                writeln!(writer, "SOLAR_RAD_AREA = {:E}", srp.area_m2).map_err(err_hdlr)?;
                writeln!(writer, "SOLAR_RAD_COEFF = {:E}", srp.coeff_reflectivity)
                    .map_err(err_hdlr)?;
            }
            if let Some(drag) = self.spacecraft_data.drag_data {
                writeln!(writer, "DRAG_AREA = {:E}", drag.area_m2).map_err(err_hdlr)?;
                writeln!(writer, "DRAG_COEFF = {:E}", drag.coeff_drag).map_err(err_hdlr)?;
            }
        }

        // Covariance
        if let Some(cov) = &self.covariance {
            let cov_frame = match cov.local_frame {
                LocalFrame::Inertial => "EME2000",
                LocalFrame::RIC => "RTN",
                LocalFrame::VNC => "TNW",
                LocalFrame::RCN => {
                    return Err(EphemerisError::OPMWritingError {
                        details: "RCN frame is not supported for OPM covariance export".to_string(),
                    });
                }
            };
            writeln!(writer, "\nCOMMENT Covariance").map_err(err_hdlr)?;
            writeln!(writer, "COV_REF_FRAME = {cov_frame}").map_err(err_hdlr)?;
            for (kw, row, col) in COV_KEYS {
                writeln!(writer, "{kw} = {:E}", cov.matrix[(row, col)]).map_err(err_hdlr)?;
            }
        }

        // Maneuvers
        for man in &self.maneuvers {
            let ref_frame = match man.ref_frame {
                LocalFrame::Inertial => "EME2000",
                LocalFrame::RIC => "RTN",
                LocalFrame::VNC => "TNW",
                LocalFrame::RCN => {
                    return Err(EphemerisError::OPMWritingError {
                        details: "RCN frame is not supported for OPM maneuver export".to_string(),
                    });
                }
            };
            writeln!(writer, "\nCOMMENT Maneuver").map_err(err_hdlr)?;
            writeln!(
                writer,
                "MAN_EPOCH_IGNITION = {}",
                Formatter::new(man.epoch_ignition, iso8601_no_ts)
            )
            .map_err(err_hdlr)?;
            writeln!(writer, "MAN_DURATION = {:E}", man.duration.to_seconds()).map_err(err_hdlr)?;
            writeln!(writer, "MAN_DELTA_MASS = {:E}", man.delta_mass_kg).map_err(err_hdlr)?;
            writeln!(writer, "MAN_REF_FRAME = {ref_frame}").map_err(err_hdlr)?;
            writeln!(writer, "MAN_DV_1 = {:E}", man.delta_v_km_s.x).map_err(err_hdlr)?;
            writeln!(writer, "MAN_DV_2 = {:E}", man.delta_v_km_s.y).map_err(err_hdlr)?;
            writeln!(writer, "MAN_DV_3 = {:E}", man.delta_v_km_s.z).map_err(err_hdlr)?;
        }

        writer.flush().map_err(err_hdlr)?;
        Ok(())
    }

    /// Initialize a new OPM from the path to a CCSDS OPM file.
    pub fn from_ccsds_opm_file<P: AsRef<Path>>(path: P) -> Result<Self, EphemerisError> {
        let file = File::open(path).map_err(|e| EphemerisError::OPMParsingError {
            lno: 0,
            details: format!("could not open file: {e}"),
        })?;
        let reader = BufReader::new(file);

        // Metadata
        let mut object_name = String::new();
        let mut object_id = String::new();
        let mut center_name: Option<String> = None;
        let mut ref_frame: Option<String> = None;
        let mut time_system = String::new();

        // State vector
        let mut epoch: Option<Epoch> = None;
        let mut state = Vector6::zeros();

        // Spacecraft parameters
        let mut mass: Option<Mass> = None;
        let mut srp_area: Option<f64> = None;
        let mut srp_coeff: Option<f64> = None;
        let mut drag_area: Option<f64> = None;
        let mut drag_coeff: Option<f64> = None;

        // Covariance
        let mut cov_mat: Option<Matrix6> = None;
        let mut cov_frame: Option<LocalFrame> = None;

        // Maneuvers
        let mut maneuvers = Vec::new();
        let mut cur_man: Option<ManeuverBuilder> = None;

        for (lno, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| EphemerisError::OPMParsingError {
                lno,
                details: format!("could not read line: {e}"),
            })?;
            let line = line.trim();
            if line.is_empty() || line.starts_with("COMMENT") {
                continue;
            }

            // Every OPM data line is `KEY = VALUE`; split once on `=` and match the exact key
            // (avoids prefix collisions like X vs X_DOT).
            let mut split = line.splitn(2, '=');
            let key = split.next().unwrap_or("").trim();
            let val = match split.next() {
                Some(v) => v.trim(),
                None => continue,
            };

            // Numeric values may carry a CCSDS unit annotation, e.g. `6655.9942 [km]`; the number
            // is the first whitespace-delimited token.
            let as_f64 = |val: &str| -> Result<f64, EphemerisError> {
                val.split_whitespace()
                    .next()
                    .unwrap_or(val)
                    .parse::<f64>()
                    .map_err(|_| EphemerisError::OPMParsingError {
                        lno,
                        details: format!("could not parse `{val}` as float for `{key}`"),
                    })
            };

            match key {
                "CCSDS_OPM_VERS" => {
                    let version =
                        val.parse::<f32>()
                            .map_err(|_| EphemerisError::OPMParsingError {
                                lno,
                                details: format!("could not parse OPM version `{val}`"),
                            })?;
                    if !(1.0..=3.0).contains(&version) {
                        return Err(EphemerisError::OPMParsingError {
                            lno,
                            details: format!("CCSDS OPM version {version} not supported"),
                        });
                    }
                }
                "OBJECT_NAME" => object_name = val.to_string(),
                "OBJECT_ID" => object_id = val.to_string(),
                "CENTER_NAME" => center_name = Some(val.to_string()),
                "REF_FRAME" => ref_frame = Some(val.to_string()),
                "TIME_SYSTEM" => time_system = val.to_string(),
                "EPOCH" => {
                    let epoch_str = format!("{val} {time_system}");
                    epoch = Some(Epoch::from_str(epoch_str.trim()).context(
                        OPMTimeParsingSnafu {
                            line: lno,
                            details: format!("`{epoch_str}` for state epoch"),
                        },
                    )?);
                }
                "X" => state[0] = as_f64(val)?,
                "Y" => state[1] = as_f64(val)?,
                "Z" => state[2] = as_f64(val)?,
                "X_DOT" => state[3] = as_f64(val)?,
                "Y_DOT" => state[4] = as_f64(val)?,
                "Z_DOT" => state[5] = as_f64(val)?,
                "MASS" => mass = Some(Mass::from_dry_mass(as_f64(val)?)),
                "SOLAR_RAD_AREA" => srp_area = Some(as_f64(val)?),
                "SOLAR_RAD_COEFF" => srp_coeff = Some(as_f64(val)?),
                "DRAG_AREA" => drag_area = Some(as_f64(val)?),
                "DRAG_COEFF" => drag_coeff = Some(as_f64(val)?),
                "MAN_EPOCH_IGNITION" => {
                    // A new maneuver block begins: finalize any in-progress one.
                    if let Some(builder) = cur_man.take() {
                        maneuvers.push(builder.finalize());
                    }
                    let epoch_str = format!("{val} {time_system}");
                    let ignition =
                        Epoch::from_str(epoch_str.trim()).context(OPMTimeParsingSnafu {
                            line: lno,
                            details: format!("`{epoch_str}` for maneuver ignition"),
                        })?;
                    cur_man = Some(ManeuverBuilder::new(ignition));
                }
                "MAN_DURATION" => {
                    man_field(&mut cur_man, lno, key)?.duration =
                        Some(Duration::from_seconds(as_f64(val)?))
                }
                "MAN_DELTA_MASS" => {
                    man_field(&mut cur_man, lno, key)?.delta_mass_kg = Some(as_f64(val)?)
                }
                "MAN_REF_FRAME" => {
                    man_field(&mut cur_man, lno, key)?.ref_frame =
                        Some(local_frame_from_token(val, lno)?)
                }
                "MAN_DV_1" => man_field(&mut cur_man, lno, key)?.dv[0] = as_f64(val)?,
                "MAN_DV_2" => man_field(&mut cur_man, lno, key)?.dv[1] = as_f64(val)?,
                "MAN_DV_3" => man_field(&mut cur_man, lno, key)?.dv[2] = as_f64(val)?,
                "COV_REF_FRAME" => cov_frame = Some(local_frame_from_token(val, lno)?),
                _ => {
                    if let Some((row, col)) = cov_index(key) {
                        // Covariance entry: fill the lower triangle and mirror it.
                        let value = as_f64(val)?;
                        let mat = cov_mat.get_or_insert_with(Matrix6::zeros);
                        mat[(row, col)] = value;
                        mat[(col, row)] = value;
                    }
                    // Other unrecognized keywords (e.g. Keplerian elements) are ignored.
                }
            }
        }

        // Finalize the last maneuver, if any.
        if let Some(builder) = cur_man.take() {
            maneuvers.push(builder.finalize());
        }

        // Build the frame and orbit from the mandatory state vector.
        let epoch = epoch.ok_or_else(|| EphemerisError::OPMParsingError {
            lno: 0,
            details: "OPM contains no state vector EPOCH".to_string(),
        })?;
        let center_name = center_name.ok_or_else(|| EphemerisError::OPMParsingError {
            lno: 0,
            details: "CENTER_NAME not found in metadata".to_string(),
        })?;
        let ref_frame = ref_frame.ok_or_else(|| EphemerisError::OPMParsingError {
            lno: 0,
            details: "REF_FRAME not found in metadata".to_string(),
        })?;

        // Capitalize each word of the center name (mirrors the OEM parser).
        let center_name = center_name
            .split_whitespace()
            .map(|word| {
                let word = word.to_lowercase();
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        let frame = Frame::from_name(center_name.as_str(), ref_frame.as_str()).map_err(|e| {
            EphemerisError::OPMParsingError {
                lno: 0,
                details: format!("frame error `{center_name:?} {ref_frame:?}`: {e}"),
            }
        })?;

        let orbit = Orbit::from_cartesian_pos_vel(state, epoch, frame);

        // SRP/drag require an area; a missing coefficient keeps the type's physical default.
        let spacecraft_data = SpacecraftData {
            mass,
            srp_data: srp_area.map(|area_m2| {
                let mut srp = SRPData::from_area(area_m2);
                if let Some(coeff) = srp_coeff {
                    srp.coeff_reflectivity = coeff;
                }
                srp
            }),
            drag_data: drag_area.map(|area_m2| {
                let mut drag = DragData::from_area(area_m2);
                if let Some(coeff) = drag_coeff {
                    drag.coeff_drag = coeff;
                }
                drag
            }),
            inertia: None,
        };

        let covariance = cov_mat.map(|matrix| Covariance {
            matrix,
            local_frame: cov_frame.unwrap_or(LocalFrame::Inertial),
        });

        Ok(Opm {
            object_name,
            object_id,
            orbit,
            spacecraft_data,
            covariance,
            maneuvers,
        })
    }
}

/// Ordered CCSDS OPM covariance keywords mapped to their (row, col) in the lower-triangular 6x6,
/// with the state ordered [X, Y, Z, X_DOT, Y_DOT, Z_DOT]. Shared by the parser and writer.
const COV_KEYS: [(&str, usize, usize); 21] = [
    ("CX_X", 0, 0),
    ("CY_X", 1, 0),
    ("CY_Y", 1, 1),
    ("CZ_X", 2, 0),
    ("CZ_Y", 2, 1),
    ("CZ_Z", 2, 2),
    ("CX_DOT_X", 3, 0),
    ("CX_DOT_Y", 3, 1),
    ("CX_DOT_Z", 3, 2),
    ("CX_DOT_X_DOT", 3, 3),
    ("CY_DOT_X", 4, 0),
    ("CY_DOT_Y", 4, 1),
    ("CY_DOT_Z", 4, 2),
    ("CY_DOT_X_DOT", 4, 3),
    ("CY_DOT_Y_DOT", 4, 4),
    ("CZ_DOT_X", 5, 0),
    ("CZ_DOT_Y", 5, 1),
    ("CZ_DOT_Z", 5, 2),
    ("CZ_DOT_X_DOT", 5, 3),
    ("CZ_DOT_Y_DOT", 5, 4),
    ("CZ_DOT_Z_DOT", 5, 5),
];

/// Returns the (row, col) of a CCSDS OPM covariance keyword, if it is one.
fn cov_index(key: &str) -> Option<(usize, usize)> {
    COV_KEYS
        .iter()
        .find(|(kw, _, _)| *kw == key)
        .map(|(_, r, c)| (*r, *c))
}

/// Maps a CCSDS reference-frame token (covariance or maneuver) to an ANISE [LocalFrame], erroring
/// on an unsupported token as the OEM parser does.
fn local_frame_from_token(token: &str, lno: usize) -> Result<LocalFrame, EphemerisError> {
    match token {
        "EME2000" | "ICRF" | "J2000" => Ok(LocalFrame::Inertial),
        "RSW" | "RTN" => Ok(LocalFrame::RIC),
        "TNW" => Ok(LocalFrame::VNC),
        _ => Err(EphemerisError::OPMParsingError {
            lno,
            details: format!("unsupported reference frame `{token}`"),
        }),
    }
}

/// Accumulates the fields of a single maneuver block, started by its mandatory ignition epoch.
struct ManeuverBuilder {
    epoch_ignition: Epoch,
    duration: Option<Duration>,
    delta_mass_kg: Option<f64>,
    ref_frame: Option<LocalFrame>,
    dv: Vector3,
}

impl ManeuverBuilder {
    fn new(epoch_ignition: Epoch) -> Self {
        Self {
            epoch_ignition,
            duration: None,
            delta_mass_kg: None,
            ref_frame: None,
            dv: Vector3::zeros(),
        }
    }

    fn finalize(self) -> Maneuver {
        Maneuver {
            epoch_ignition: self.epoch_ignition,
            duration: self.duration.unwrap_or(Duration::ZERO),
            delta_mass_kg: self.delta_mass_kg.unwrap_or(0.0),
            ref_frame: self.ref_frame.unwrap_or(LocalFrame::Inertial),
            delta_v_km_s: self.dv,
        }
    }
}

/// Returns the in-progress maneuver builder, erroring if a `MAN_*` field appears before its
/// `MAN_EPOCH_IGNITION`.
fn man_field<'a>(
    cur_man: &'a mut Option<ManeuverBuilder>,
    lno: usize,
    key: &str,
) -> Result<&'a mut ManeuverBuilder, EphemerisError> {
    cur_man.as_mut().ok_or(EphemerisError::OPMParsingError {
        lno,
        details: format!("`{key}` appeared before MAN_EPOCH_IGNITION"),
    })
}

#[cfg(feature = "python")]
#[pymethods]
impl Maneuver {
    /// :type epoch_ignition: Epoch
    /// :type duration: Duration
    /// :type delta_mass_kg: float
    /// :type ref_frame: LocalFrame
    /// :type delta_v_km_s: numpy.ndarray
    /// :rtype: Maneuver
    #[new]
    fn py_new(
        epoch_ignition: Epoch,
        duration: Duration,
        delta_mass_kg: f64,
        ref_frame: LocalFrame,
        delta_v_km_s: PyReadonlyArray1<f64>,
    ) -> PyResult<Self> {
        let dv = delta_v_km_s.as_slice()?;
        if dv.len() != 3 {
            return Err(PyValueError::new_err("delta_v_km_s must have 3 elements"));
        }
        Ok(Self {
            epoch_ignition,
            duration,
            delta_mass_kg,
            ref_frame,
            delta_v_km_s: Vector3::new(dv[0], dv[1], dv[2]),
        })
    }

    /// :rtype: Epoch
    #[getter]
    fn get_epoch_ignition(&self) -> Epoch {
        self.epoch_ignition
    }

    /// :rtype: Duration
    #[getter]
    fn get_duration(&self) -> Duration {
        self.duration
    }

    /// :rtype: float
    #[getter]
    fn get_delta_mass_kg(&self) -> f64 {
        self.delta_mass_kg
    }

    /// :rtype: LocalFrame
    #[getter]
    fn get_ref_frame(&self) -> LocalFrame {
        self.ref_frame
    }

    /// delta-v vector in km/s
    ///
    /// :rtype: numpy.ndarray
    #[getter]
    fn get_delta_v_km_s<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<f64>> {
        let arr = Array1::from_iter(self.delta_v_km_s.iter().copied());
        PyArray1::<f64>::from_owned_array(py, arr)
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Opm {
    /// :type orbit: Orbit
    /// :rtype: Opm
    #[new]
    fn py_new(orbit: Orbit) -> Self {
        Self::new(orbit)
    }

    /// Initializes a new OPM from a file path to a CCSDS OPM file.
    ///
    /// :type path: str
    /// :rtype: Opm
    #[classmethod]
    #[pyo3(name = "from_ccsds_opm_file", signature = (path))]
    fn py_from_ccsds_opm_file(_cls: Bound<'_, PyType>, path: &str) -> Result<Self, EphemerisError> {
        Self::from_ccsds_opm_file(path)
    }

    /// Writes this OPM to CCSDS OPM at the provided path, optionally specifying an originator and/or an object name.
    ///
    /// :type path: str
    /// :type originator: str, optional
    /// :type object_name: str, optional
    /// :rtype: None
    #[pyo3(name = "write_ccsds_opm", signature = (path, originator=None, object_name=None))]
    fn py_write_ccsds_opm(
        &self,
        path: &str,
        originator: Option<String>,
        object_name: Option<String>,
    ) -> Result<(), EphemerisError> {
        self.write_ccsds_opm(path, originator, object_name)
    }

    /// Adds a maneuver to this OPM.
    ///
    /// :type maneuver: Maneuver
    /// :rtype: None
    #[pyo3(name = "add_maneuver")]
    fn py_add_maneuver(&mut self, maneuver: Maneuver) {
        self.add_maneuver(maneuver);
    }

    /// :rtype: str
    #[getter]
    fn get_object_name(&self) -> String {
        self.object_name.clone()
    }
    /// :type object_name: str
    #[setter]
    fn set_object_name(&mut self, object_name: String) {
        self.object_name = object_name;
    }

    /// :rtype: str
    #[getter]
    fn get_object_id(&self) -> String {
        self.object_id.clone()
    }
    /// :type object_id: str
    #[setter]
    fn set_object_id(&mut self, object_id: String) {
        self.object_id = object_id;
    }

    /// :rtype: Orbit
    #[getter]
    fn get_orbit(&self) -> Orbit {
        self.orbit
    }
    /// :type orbit: Orbit
    #[setter]
    fn set_orbit(&mut self, orbit: Orbit) {
        self.orbit = orbit;
    }

    /// :rtype: Covariance
    #[getter]
    fn get_covariance(&self) -> Option<Covariance> {
        self.covariance
    }
    /// :type covariance: Covariance
    #[setter]
    fn set_covariance(&mut self, covariance: Option<Covariance>) {
        self.covariance = covariance;
    }

    /// :rtype: Mass
    #[getter]
    fn get_mass(&self) -> Option<Mass> {
        self.spacecraft_data.mass
    }
    /// :type mass: Mass
    #[setter]
    fn set_mass(&mut self, mass: Option<Mass>) {
        self.spacecraft_data.mass = mass;
    }

    /// :rtype: SRPData
    #[getter]
    fn get_srp_data(&self) -> Option<SRPData> {
        self.spacecraft_data.srp_data
    }
    /// :type srp_data: SRPData
    #[setter]
    fn set_srp_data(&mut self, srp_data: Option<SRPData>) {
        self.spacecraft_data.srp_data = srp_data;
    }

    /// :rtype: DragData
    #[getter]
    fn get_drag_data(&self) -> Option<DragData> {
        self.spacecraft_data.drag_data
    }
    /// :type drag_data: DragData
    #[setter]
    fn set_drag_data(&mut self, drag_data: Option<DragData>) {
        self.spacecraft_data.drag_data = drag_data;
    }

    /// :rtype: typing.List[Maneuver]
    #[getter]
    fn get_maneuvers(&self) -> Vec<Maneuver> {
        self.maneuvers.clone()
    }

    fn __str__(&self) -> String {
        format!(
            "OPM for {} ({}) at {}",
            self.object_name, self.object_id, self.orbit.epoch
        )
    }

    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

#[cfg(test)]
mod ut_opm {
    use super::{LocalFrame, Opm};
    use crate::math::Vector3;

    #[test]
    fn test_parse_opm_sample() {
        let opm = Opm::from_ccsds_opm_file("../data/tests/ccsds/opm/sample.opm")
            .expect("could not parse OPM");

        // Metadata
        assert_eq!(opm.object_name, "ANISE TESTSAT");
        assert_eq!(opm.object_id, "1998-999A");

        // State vector
        assert_eq!(
            opm.orbit.radius_km,
            Vector3::new(6503.514, 1239.647, -717.49)
        );
        assert_eq!(
            opm.orbit.velocity_km_s,
            Vector3::new(-0.87316, 8.74042, -4.191076)
        );

        // Spacecraft parameters
        let mass = opm.spacecraft_data.mass.expect("no mass");
        assert_eq!(mass.dry_mass_kg, 3000.0);
        let srp = opm.spacecraft_data.srp_data.expect("no SRP");
        assert_eq!(srp.area_m2, 18.77);
        assert_eq!(srp.coeff_reflectivity, 1.3);
        let drag = opm.spacecraft_data.drag_data.expect("no drag");
        assert_eq!(drag.area_m2, 18.77);
        assert_eq!(drag.coeff_drag, 2.5);

        // Covariance (named keywords, lower triangular, symmetric)
        let cov = opm.covariance.expect("no covariance");
        assert_eq!(cov.local_frame, LocalFrame::RIC); // RTN -> RIC
        assert_eq!(cov.matrix[(0, 0)], 3.3313494e-04);
        assert_eq!(cov.matrix[(5, 5)], 6.2244443e-10);
        // symmetry: CY_X populates both (1,0) and (0,1)
        assert_eq!(cov.matrix[(1, 0)], 4.6189273e-04);
        assert_eq!(cov.matrix[(0, 1)], 4.6189273e-04);

        // Maneuvers
        assert_eq!(opm.maneuvers.len(), 1);
        let man = &opm.maneuvers[0];
        assert_eq!(man.delta_mass_kg, -1.0);
        assert_eq!(man.ref_frame, LocalFrame::Inertial);
        assert_eq!(man.delta_v_km_s, Vector3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn test_parse_opm_real() {
        // A real operational OPM (EUTELSAT W4, GSOC/DLR) with unit annotations like `[km]`,
        // GPS time scale, Keplerian elements, and multiple maneuvers.
        let opm = Opm::from_ccsds_opm_file("../data/tests/ccsds/opm/eutelsat_w4.opm")
            .expect("could not parse real OPM");

        assert_eq!(opm.object_name, "EUTELSAT W4");
        assert_eq!(opm.object_id, "2000-028A");

        // State vector values must be parsed despite the trailing `[km]` / `[km/s]` units.
        assert_eq!(
            opm.orbit.radius_km,
            Vector3::new(6655.9942, -40218.5751, -82.9177)
        );
        assert_eq!(opm.orbit.velocity_km_s.x, 3.11548208);

        // Spacecraft parameters (units stripped).
        assert_eq!(
            opm.spacecraft_data.mass.expect("no mass").dry_mass_kg,
            1913.0
        );
        assert_eq!(opm.spacecraft_data.srp_data.expect("no SRP").area_m2, 10.0);
        assert_eq!(
            opm.spacecraft_data.drag_data.expect("no drag").coeff_drag,
            2.3
        );

        // Three maneuvers, with both EME2000 (inertial) and RTN (RIC) frames.
        assert_eq!(opm.maneuvers.len(), 3);
        assert_eq!(opm.maneuvers[0].ref_frame, LocalFrame::Inertial);
        assert_eq!(opm.maneuvers[0].delta_mass_kg, -18.418);
        assert_eq!(opm.maneuvers[1].ref_frame, LocalFrame::RIC);
    }

    #[test]
    fn test_opm_roundtrip() {
        let opm = Opm::from_ccsds_opm_file("../data/tests/ccsds/opm/sample.opm")
            .expect("could not parse OPM");

        // Write it back out, re-parse, and confirm it matches.
        let outpath = "../data/tests/ccsds/opm/sample_rebuilt.opm";
        opm.write_ccsds_opm(outpath, Some("My Originator".to_string()), None)
            .expect("could not write OPM");

        let opm2 = Opm::from_ccsds_opm_file(outpath).expect("could not re-parse OPM");
        assert_eq!(opm2, opm);
    }
}
