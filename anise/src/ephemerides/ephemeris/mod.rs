/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{EphemerisError, OEMTimeParsingSnafu};
use crate::ephemerides::EphemInterpolationSnafu;
use crate::math::interpolation::{hermite_eval, lagrange_eval};
use crate::math::{Matrix6, Vector6};
use crate::naif::daf::data_types::DataType;
use crate::prelude::{Almanac, Frame, Orbit};
use core::fmt;
use covariance::interpolate_covar_log_euclidean;
use hifitime::Epoch;
use log::warn;
use snafu::ResultExt;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

#[cfg(feature = "python")]
use pyo3::prelude::*;

mod covariance;
pub use covariance::{Covariance, LocalFrame};

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
pub struct EphemEntry {
    /// Orbit of this ephemeris entry
    pub orbit: Orbit,
    /// Optional covariance associated with this orbit
    pub covar: Option<Covariance>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
pub struct Ephemeris {
    object_id: String,
    interpolation: DataType,
    degree: usize,
    /// Ephemeris entries in chronological order
    state_data: BTreeMap<Epoch, EphemEntry>,
}

impl Ephemeris {
    /// Initialize a new ephemeris from the path to a CCSDS OEM file.
    ///
    /// # Limitations
    /// - Support covariance only in EME2000 frame
    pub fn from_ccsds_oem_file<P: AsRef<Path>>(path: P) -> Result<Self, EphemerisError> {
        // Open the file
        let file = File::open(path).map_err(|e| EphemerisError::OEMError {
            lno: 0,
            details: format!("could not open file: {e}"),
        })?;

        let reader = BufReader::new(file);

        let mut in_state_data = false;
        let mut in_cov_data = false;

        // Define header variables we care about.
        let mut time_system = String::new();
        let mut center_name = None;
        let mut orient_name = None;
        let mut interpolation = DataType::Type13HermiteUnequalStep;
        let mut degree = 5;
        let mut object_id: Option<String> = None;
        let mut cov_epoch = None;
        let mut cov_mat = None;
        let mut cov_frame = None;
        let mut cov_row = 0;

        // Store the temporary data in a BTreeMap so we have O(1) access when adding the covariance information
        // and we can iterate in order when building the vector.
        let mut state_data = BTreeMap::new();

        let parse_one_val = |lno: usize, line: &str, err: &str| -> Result<String, EphemerisError> {
            let parts: Vec<&str> = line.split('=').collect();

            match parts.get(1) {
                Some(val_str) => Ok(val_str.trim().to_string()),
                None => Err(EphemerisError::OEMError {
                    lno,
                    details: err.to_string(),
                }),
            }
        };

        for (lno, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| EphemerisError::OEMError {
                lno,
                details: format!("could not read line: {e}"),
            })?;

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("CCSDS_OEM_VERS") {
                let version_str = parse_one_val(lno, line, "no value for CCSDS_OEM_VERS")?;
                match version_str.parse::<f32>() {
                    Ok(version_val) => match version_val as i16 {
                        1 | 2 => {}
                        _ => {
                            return Err(EphemerisError::OEMError {
                                lno,
                                details: "CCSDS OEM version {version_val} not supported"
                                    .to_string(),
                            })
                        }
                    },
                    Err(_) => {
                        return Err(EphemerisError::OEMError {
                            lno,
                            details: format!("could not parse OEM version `{version_str}`"),
                        })
                    }
                }
            }
            if line.starts_with("OBJECT_ID") {
                // Extract the object ID from the line
                let oem_obj_id = parse_one_val(lno, line, "no value for OBJECT_ID")?;
                if let Some(prev_obj_id) = object_id {
                    if oem_obj_id != prev_obj_id {
                        return Err(EphemerisError::OEMError {
                            lno,
                            details: format!(
                                "OEM must have only one object: `{prev_obj_id}` != `{oem_obj_id}`"
                            ),
                        });
                    }
                }
                object_id = Some(oem_obj_id);
            } else if line.starts_with("CENTER_NAME") {
                center_name = Some(parse_one_val(lno, line, "no value for CENTER")?);
            } else if line.starts_with("REF_FRAME") {
                orient_name = Some(parse_one_val(lno, line, "no value for REF_FRAME")?);
            } else if line.starts_with("TIME_SYSTEM") {
                time_system = parse_one_val(lno, line, "no value for TIME_SYSTEM")?;
            } else if line.starts_with("INTERPOLATION_DEGREE") {
                let interp_str =
                    parse_one_val(lno, line, "no value for INTERPOLATION_DEGREE")?.to_lowercase();

                match interp_str.parse::<usize>() {
                    Ok(ideg) => degree = ideg,
                    Err(_) => {
                        return Err(EphemerisError::OEMError {
                            lno,
                            details: format!("could not parse `{interp_str}` as float"),
                        })
                    }
                }
            } else if line.starts_with("INTERPOLATION") {
                let interp_str =
                    parse_one_val(lno, line, "no value for INTERPOLATION")?.to_lowercase();

                match interp_str.as_str() {
                    "lagrange" => interpolation = DataType::Type9LagrangeUnequalStep,
                    "hermite" => interpolation = DataType::Type13HermiteUnequalStep,
                    _ => {
                        warn!("unsupported interpolation `{interp_str}` using Hermite")
                    }
                };
            } else if line.starts_with("META_STOP") {
                // We can start parsing now
                in_state_data = true;
                in_cov_data = false;
            } else if line.starts_with("META_START") {
                in_state_data = false;
                in_cov_data = false;
            } else if line.starts_with("COVARIANCE_START") {
                in_state_data = false;
                in_cov_data = true;
            } else if line.starts_with("COVARIANCE_STOP") {
                in_state_data = false;
                in_cov_data = false;
            } else if line.starts_with("COMMENT") {
                // Ignore
            } else if in_state_data {
                // Capitalize the center name
                let center_name = center_name
                    .as_ref()
                    .unwrap()
                    .split_whitespace()
                    .map(|word| {
                        let word = word.to_lowercase();
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ");

                let frame =
                    Frame::from_name(center_name.as_str(), orient_name.clone().unwrap().as_str())
                        .map_err(|e| EphemerisError::OEMError {
                        lno,
                        details: format!("frame error `{center_name:?} {orient_name:?}`: {e}"),
                    })?;

                // Split the line into components
                let parts: Vec<&str> = line.split_whitespace().collect();
                let mut state_vec = Vector6::zeros();

                // Build the epoch
                let epoch = match parts.first() {
                    Some(state_epoch) => {
                        let epoch_str = format!("{state_epoch} {time_system}");
                        Epoch::from_str(epoch_str.trim()).context(OEMTimeParsingSnafu {
                            line: lno,
                            details: format!("`{epoch_str}` for state epoch"),
                        })?
                    }
                    None => {
                        return Err(EphemerisError::OEMError {
                            lno,
                            details: "no `=` sign for covariance epoch".to_string(),
                        })
                    }
                };

                // Convert the state data
                for i in 0..6 {
                    match parts.get(i + 1) {
                        Some(val_str) => match val_str.trim().parse::<f64>() {
                            Ok(val_f64) => {
                                state_vec[i] = val_f64;
                            }
                            Err(_) => {
                                return Err(EphemerisError::OEMError {
                                    lno,
                                    details: format!(
                                        "could not parse `{}` as float",
                                        val_str.trim()
                                    ),
                                })
                            }
                        },
                        None => {
                            return Err(EphemerisError::OEMError {
                                lno,
                                details: format!("missing float in position {}", i + 1),
                            })
                        }
                    };
                }

                // We only reach this point if the state data is fully parsed.
                let orbit = Orbit::from_cartesian_pos_vel(state_vec, epoch, frame);
                state_data.insert(epoch, EphemEntry { orbit, covar: None });
            } else if in_cov_data {
                if line.starts_with("EPOCH") {
                    let state_epoch = parse_one_val(lno, line, "no `=` sign for covariance epoch")?;
                    let epoch_str = format!("{state_epoch} {time_system}");
                    let epoch = Epoch::from_str(epoch_str.trim()).context(OEMTimeParsingSnafu {
                        line: lno,
                        details: format!("`{epoch_str}` for covariance epoch"),
                    })?;

                    // Check that we have associated state data
                    if !state_data.contains_key(&epoch) {
                        return Err(EphemerisError::OEMError { lno, details: format!("cannot have covariance data at {epoch} because no orbit data at that epoch")});
                    }

                    cov_epoch = Some(epoch);
                    cov_mat = Some(Matrix6::zeros());
                    cov_row = 0;
                } else if line.starts_with("COV_REF_FRAME") {
                    // Only do a check here, nothing to set.
                    let cov_frame_str = parse_one_val(lno, line, "invalid COV_REF_FRAME token")?;
                    match cov_frame_str.as_str() {
                        "EME2000" | "ICRF" => cov_frame = Some(LocalFrame::Inertial),
                        "RSW" | "RTN" => cov_frame = Some(LocalFrame::RIC),
                        "TNW" => cov_frame = Some(LocalFrame::VNC),
                        _ => {
                            return Err(EphemerisError::OEMError {
                                lno,
                                details: format!("invalid COV_REF_FRAME `{cov_frame_str}`"),
                            })
                        }
                    };
                } else {
                    // Matrix data!
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() != cov_row + 1 {
                        return Err(EphemerisError::OEMError {
                            lno,
                            details: format!(
                                "expected {} values for covariance row {cov_row} but got {}",
                                cov_row + 1,
                                parts.len()
                            ),
                        });
                    }

                    for col in 0..cov_row + 1 {
                        match parts.get(col) {
                            Some(val_str) => match val_str.trim().parse::<f64>() {
                                Ok(val_f64) => {
                                    cov_mat.as_mut().unwrap()[(col, cov_row)] = val_f64;
                                    cov_mat.as_mut().unwrap()[(cov_row, col)] = val_f64;
                                }
                                Err(_) => {
                                    return Err(EphemerisError::OEMError {
                                        lno,
                                        details: format!(
                                            "could not parse `{}` as float",
                                            val_str.trim()
                                        ),
                                    })
                                }
                            },
                            None => {
                                return Err(EphemerisError::OEMError {
                                    lno,
                                    details: format!(
                                        "missing float in covariance data position {col}"
                                    ),
                                })
                            }
                        };
                    }
                    cov_row += 1;
                    if cov_row == 6 {
                        // We've parsed everything, set the covariance
                        match cov_epoch {
                            Some(cov_epoch) => {
                                let covar = cov_mat.map(|mat| Covariance {
                                    matrix: mat,
                                    local_frame: cov_frame.unwrap_or(LocalFrame::Inertial),
                                });
                                state_data
                                    .get_mut(&cov_epoch)
                                    .expect("epoch was valid but now no?")
                                    .covar = covar;
                            }
                            None => {
                                return Err(EphemerisError::OEMError {
                                    lno,
                                    details: "no cov epoch ever found?!".to_string(),
                                })
                            }
                        }
                    }
                }
            }
        }

        if state_data.is_empty() {
            return Err(EphemerisError::OEMError {
                lno: 0,
                details: "ephemeris file contains no state data".to_string(),
            });
        }

        // Build the Ephemeris
        if let Some(object_id) = object_id {
            Ok(Ephemeris {
                object_id,
                degree,
                interpolation,
                state_data,
            })
        } else {
            Err(EphemerisError::OEMError {
                lno: 0,
                details: "no OBJECT_ID found throughout the file".to_string(),
            })
        }
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Ephemeris {
    pub fn domain(&self) -> Result<(Epoch, Epoch), EphemerisError> {
        if self.state_data.is_empty() {
            Err(EphemerisError::EphemInterpolation {
                source: crate::math::interpolation::InterpolationError::EmptyInterpolationData {},
            })
        } else {
            Ok((
                *self.state_data.first_key_value().unwrap().0,
                *self.state_data.last_key_value().unwrap().0,
            ))
        }
    }

    pub fn nearest_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<EphemEntry, EphemerisError> {
        self.state_data
            .range(..=epoch)
            .next_back()
            .map(|e| {
                let mut entry = *e.1;
                if let Ok(frame) = almanac.frame_info(entry.orbit.frame) {
                    entry.orbit.frame = frame;
                }
                entry
            })
            .ok_or(EphemerisError::EphemInterpolation {
                source: crate::math::interpolation::InterpolationError::EmptyInterpolationData {},
            })
    }

    pub fn nearest_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<EphemEntry, EphemerisError> {
        self.state_data
            .range(epoch..)
            .next()
            .map(|e| {
                let mut entry = *e.1;
                if let Ok(frame) = almanac.frame_info(entry.orbit.frame) {
                    entry.orbit.frame = frame;
                }
                entry
            })
            .ok_or(EphemerisError::EphemInterpolation {
                source: crate::math::interpolation::InterpolationError::EmptyInterpolationData {},
            })
    }

    pub fn nearest_orbit_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Orbit, EphemerisError> {
        Ok(self.nearest_before(epoch, almanac)?.orbit)
    }

    pub fn nearest_orbit_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Orbit, EphemerisError> {
        Ok(self.nearest_after(epoch, almanac)?.orbit)
    }

    pub fn nearest_covar_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Option<(Covariance, Epoch)>, EphemerisError> {
        let entry = self.nearest_before(epoch, almanac)?;
        Ok(entry.covar.map(|c| (c, entry.orbit.epoch)))
    }

    pub fn nearest_covar_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Option<(Covariance, Epoch)>, EphemerisError> {
        let entry = self.nearest_after(epoch, almanac)?;
        Ok(entry.covar.map(|c| (c, entry.orbit.epoch)))
    }

    /// Interpolate the ephemeris at the provided epoch.
    pub fn at(&self, epoch: Epoch, almanac: &Almanac) -> Result<EphemEntry, EphemerisError> {
        // Grab the N/2 previous states
        let n = self.degree / 2;
        let prev_states = self
            .state_data
            .range(..=epoch)
            .take(n)
            .map(|e| *e.1)
            .collect::<Vec<EphemEntry>>();
        let next_states = self
            .state_data
            .range(epoch..)
            .take(n)
            .map(|e| *e.1)
            .collect::<Vec<EphemEntry>>();

        let states = prev_states.iter().chain(next_states.iter());

        let xs = states
            .clone()
            .map(|entry| entry.orbit.epoch.to_tdb_seconds())
            .collect::<Vec<f64>>();
        let mut orbit_data = Vector6::zeros();

        match self.interpolation {
            DataType::Type9LagrangeUnequalStep => {
                for i in 0..6 {
                    let ys = states
                        .clone()
                        .map(|entry| entry.orbit.to_cartesian_pos_vel()[i])
                        .collect::<Vec<f64>>();

                    let (val, _) = lagrange_eval(&xs, &ys, epoch.to_tdb_seconds())
                        .context(EphemInterpolationSnafu)?;
                    orbit_data[i] = val;
                }
            }
            DataType::Type13HermiteUnequalStep => {
                for i in 0..3 {
                    let ys = states
                        .clone()
                        .map(|entry| entry.orbit.to_cartesian_pos_vel()[i])
                        .collect::<Vec<f64>>();
                    let ydots = states
                        .clone()
                        .map(|entry| entry.orbit.to_cartesian_pos_vel()[i])
                        .collect::<Vec<f64>>();

                    let (val, valdot) = hermite_eval(&xs, &ys, &ydots, epoch.to_tdb_seconds())
                        .context(EphemInterpolationSnafu)?;

                    orbit_data[i] = val;
                    orbit_data[i + 3] = valdot;
                }
            }
            _ => unreachable!(),
        };

        let mut orbit = next_states[0].orbit.with_cartesian_pos_vel(orbit_data);
        orbit.epoch = epoch;
        if let Ok(frame) = almanac.frame_info(orbit.frame) {
            orbit.frame = frame;
        }

        // Interpolate the covariances if they're set
        let mut covar = None;
        if let Ok(Some((covar0, epoch0))) = self.nearest_covar_before(epoch, almanac) {
            if let Ok(Some((covar1, epoch1))) = self.nearest_covar_after(epoch, almanac) {
                // TODO: Rotate the covariances if need be!
                if epoch1 != epoch0 {
                    let alpha = (epoch - epoch0).to_seconds() / (epoch1 - epoch0).to_seconds();

                    if let Some(covar_mat) =
                        interpolate_covar_log_euclidean(covar0.matrix, covar1.matrix, alpha)
                    {
                        covar = Some(Covariance {
                            matrix: covar_mat,
                            local_frame: covar0.local_frame,
                        });
                    }
                }
            }
        }

        let entry = EphemEntry { orbit, covar };

        Ok(entry)
    }

    pub fn orbit_at(&self, epoch: Epoch, almanac: &Almanac) -> Result<Orbit, EphemerisError> {
        Ok(self.at(epoch, almanac)?.orbit)
    }

    pub fn covar_at(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Option<Covariance>, EphemerisError> {
        Ok(self.at(epoch, almanac)?.covar)
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl Ephemeris {
    #[getter]
    fn get_object_id(&self) -> String {
        self.object_id.clone()
    }
}

impl fmt::Display for Ephemeris {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.state_data.is_empty() {
            write!(f, "empty ephem for {}", self.object_id)
        } else {
            let (start, stop) = self.domain().unwrap();
            let span = stop - start;
            write!(
                f,
                "{} ephem from {start} to {stop} ({} states, spans {span})",
                self.object_id,
                self.state_data.len()
            )
        }
    }
}

#[cfg(test)]
mod ut_oem {
    use super::{Almanac, DataType, Ephemeris};
    use hifitime::{Epoch, Unit};

    use rstest::*;

    #[fixture]
    fn almanac() -> Almanac {
        Almanac::default().load("../data/pck11.pca").unwrap()
    }

    #[rstest]
    fn test_parse_oem_leo(almanac: Almanac) {
        let ephem = Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/LEO_10s.oem")
            .expect("could not parse");

        let start = Epoch::from_gregorian_utc_at_noon(2020, 6, 1);

        assert_eq!(ephem.state_data.len(), 361);
        assert_eq!(
            ephem.domain().unwrap(),
            (start, Epoch::from_gregorian_utc_hms(2020, 6, 1, 13, 0, 0))
        );
        assert_eq!(ephem.interpolation, DataType::Type9LagrangeUnequalStep);
        assert_eq!(ephem.degree, 7);

        println!("{ephem}");

        // Check that we can interpolate
        let epoch = start + Unit::Second * 5;
        let halfway_orbit = ephem.orbit_at(epoch, &almanac).unwrap();
        let before = ephem.nearest_orbit_before(epoch, &almanac).unwrap();
        let after = ephem.nearest_orbit_after(epoch, &almanac).unwrap();
        println!("before = {before}\nduring = {halfway_orbit}\nafter = {after}",);
        // Check that the Keplerian data is reasonably constant.
        // Note that the true Hermite test is in the NAIF SPK tests.
        assert!((before.sma_km().unwrap() - halfway_orbit.sma_km().unwrap()).abs() < 1e-1);
        assert!((after.sma_km().unwrap() - halfway_orbit.sma_km().unwrap()).abs() < 1e-1);
    }

    #[test]
    fn test_parse_oem_meo() {
        let ephem = Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/MEO_60s.oem")
            .expect("could not parse");

        assert_eq!(ephem.state_data.len(), 61);
        assert_eq!(
            ephem.domain().unwrap(),
            (
                Epoch::from_gregorian_utc_at_noon(2020, 6, 1),
                Epoch::from_gregorian_utc_hms(2020, 6, 1, 13, 0, 0)
            )
        );
        assert_eq!(ephem.interpolation, DataType::Type9LagrangeUnequalStep);
        assert_eq!(ephem.degree, 5);

        println!("{ephem}");
    }

    #[test]
    fn test_parse_oem_meo_bad() {
        assert!(Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/MEO_60s_bad.oem").is_err());
    }

    #[rstest]
    fn test_parse_oem_covar(almanac: Almanac) {
        let ephem = Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/JPL_MGS_cov.oem")
            .expect("could not parse");

        let (start, end) = (
            Epoch::from_gregorian(
                1996,
                12,
                28,
                21,
                29,
                7,
                267_000_000,
                hifitime::TimeScale::TDB,
            ),
            Epoch::from_gregorian(
                1996,
                12,
                30,
                1,
                28,
                2,
                267_000_000,
                hifitime::TimeScale::TDB,
            ),
        );
        assert_eq!(ephem.state_data.len(), 4);
        assert_eq!(ephem.domain().unwrap(), (start, end));
        assert_eq!(ephem.interpolation, DataType::Type13HermiteUnequalStep);
        assert_eq!(ephem.degree, 7);

        println!("{ephem}");

        // Check that we can interpolate the covariance and that it correctly rotates.
        let epoch = start + Unit::Second * 5;
        let halfway = ephem.covar_at(epoch, &almanac).unwrap().unwrap().matrix;
        let before = ephem
            .nearest_covar_before(epoch, &almanac)
            .unwrap()
            .unwrap()
            .0
            .matrix;
        let after = ephem
            .nearest_covar_after(epoch, &almanac)
            .unwrap()
            .unwrap()
            .0
            .matrix;
        println!("before = {before}\nduring = {halfway}\nafter = {after}");
        // NOTE this will need changing after I implement the rotations
        println!("delta before = {:e}", halfway - before);
        println!("delta after = {:e}", after - halfway)
    }
}
