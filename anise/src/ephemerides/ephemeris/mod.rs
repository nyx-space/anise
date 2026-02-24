/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{EphemerisError, EphemerisPhysicsSnafu, OEMTimeParsingSnafu};
use crate::ephemerides::EphemInterpolationSnafu;
use crate::errors::{AlmanacError, AlmanacPhysicsSnafu, OrientationSnafu};
use crate::frames::Frame;
use crate::math::interpolation::{hermite_eval, lagrange_eval};
use crate::math::Vector6;
use crate::naif::daf::data_types::DataType;
use crate::prelude::{Almanac, Orbit};
use core::fmt;
use covariance::interpolate_covar_log_euclidean;
use hifitime::{Epoch, TimeSeries};
use snafu::ResultExt;
use std::collections::{
    btree_map::{IntoValues, Values},
    BTreeMap,
};

#[cfg(feature = "python")]
use pyo3::prelude::*;

mod almanac;
mod covariance;
mod oem;
#[cfg(feature = "python")]
mod python;
mod record;
mod spk;
mod stk;

pub use covariance::{Covariance, LocalFrame};
pub use record::EphemerisRecord;

/// Initializes a new Ephemeris from the list of Orbit instances and a given object ID.
///
/// In Python if you need to build an ephemeris with covariance, initialize with an empty list of
/// orbit instances and then insert each EphemerisRecord with covariance.
///
/// :type orbit_list: list
/// :type object_id: str
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct Ephemeris {
    pub object_id: String,
    pub interpolation: DataType,
    pub degree: usize,
    state_data: BTreeMap<Epoch, EphemerisRecord>,
}

impl Ephemeris {
    pub fn new(object_id: String) -> Self {
        Self {
            object_id,
            interpolation: DataType::Type13HermiteUnequalStep,
            degree: 7,
            state_data: BTreeMap::new(),
        }
    }

    /// Returns the interpolation used
    pub fn interpolation(&self) -> DataType {
        self.interpolation
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Ephemeris {
    /// Returns the time domain of this ephemeris.
    ///
    /// :rtype: tuple
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

    /// :rtype: Epoch
    pub fn start_epoch(&self) -> Result<Epoch, EphemerisError> {
        Ok(self.domain()?.0)
    }

    /// :rtype: Epoch
    pub fn end_epoch(&self) -> Result<Epoch, EphemerisError> {
        Ok(self.domain()?.1)
    }

    /// :rtype: str
    pub fn object_id(&self) -> &str {
        &self.object_id
    }

    /// Returns true if all of the data in this ephemeris includes covariance.
    ///
    /// This is a helper function which isn't used in other functions.
    ///
    /// :rtype: bool
    pub fn includes_covariance(&self) -> bool {
        !self.state_data.is_empty() && self.state_data.values().all(|entry| entry.covar.is_some())
    }

    /// Inserts a new ephemeris entry to this ephemeris (it is automatically sorted chronologically).
    /// :type record: EphemerisRecord
    /// :rtype: None
    pub fn insert(&mut self, record: EphemerisRecord) {
        self.state_data.insert(record.orbit.epoch, record);
    }

    /// Inserts a new orbit (without covariance) to this ephemeris (it is automatically sorted chronologically).
    /// :type orbit: Orbit
    /// :rtype: None
    pub fn insert_orbit(&mut self, orbit: Orbit) {
        self.state_data
            .insert(orbit.epoch, EphemerisRecord { orbit, covar: None });
    }

    /// Returns the nearest entry before the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: EphemerisRecord
    pub fn nearest_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<EphemerisRecord, EphemerisError> {
        self.state_data
            .range(..=epoch)
            .next_back()
            .map(|e| {
                let mut record = *e.1;
                if let Ok(frame) = almanac.frame_info(record.orbit.frame) {
                    record.orbit.frame = frame;
                }
                record
            })
            .ok_or(EphemerisError::EphemInterpolation {
                source: crate::math::interpolation::InterpolationError::EmptyInterpolationData {},
            })
    }

    /// Returns the nearest entry after the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: EphemerisRecord
    pub fn nearest_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<EphemerisRecord, EphemerisError> {
        self.state_data
            .range(epoch..)
            .next()
            .map(|e| {
                let mut record = *e.1;
                if let Ok(frame) = almanac.frame_info(record.orbit.frame) {
                    record.orbit.frame = frame;
                }
                record
            })
            .ok_or(EphemerisError::EphemInterpolation {
                source: crate::math::interpolation::InterpolationError::EmptyInterpolationData {},
            })
    }

    /// Returns the nearest orbit before the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: Orbit
    pub fn nearest_orbit_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Orbit, EphemerisError> {
        Ok(self.nearest_before(epoch, almanac)?.orbit)
    }

    /// Returns the nearest orbit after the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: Orbit
    pub fn nearest_orbit_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Orbit, EphemerisError> {
        Ok(self.nearest_after(epoch, almanac)?.orbit)
    }

    /// Returns the nearest covariance before the provided epoch as a tuple (Epoch, Covariance)
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: tuple
    pub fn nearest_covar_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Option<(Epoch, Covariance)>, EphemerisError> {
        let record = self.nearest_before(epoch, almanac)?;
        Ok(record.covar.map(|c| (record.orbit.epoch, c)))
    }

    /// Returns the nearest covariance after the provided epoch as a tuple (Epoch, Covariance)
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: tuple
    pub fn nearest_covar_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Option<(Epoch, Covariance)>, EphemerisError> {
        let record = self.nearest_after(epoch, almanac)?;
        Ok(record.covar.map(|c| (record.orbit.epoch, c)))
    }

    /// Interpolates the ephemeris state and covariance at the provided epoch.
    ///
    /// # Orbit Interpolation
    /// The orbital state is interpolated using high-fidelity numeric methods consistent
    /// with SPICE standards:
    /// * **Type 9 (Lagrange):** Uses an Nth-order Lagrange polynomial interpolation on
    ///   unequal time steps. It interpolates each of the 6 state components (position
    ///   and velocity) independently.
    /// * **Type 13 (Hermite):** Uses an Nth-order Hermite interpolation. This method
    ///   explicitly uses the velocity data (derivatives) to constrain the interpolation
    ///   of the position, ensuring that the resulting position curve is smooth and
    ///   dynamically consistent with the velocity.
    ///
    /// # Covariance Interpolation (Log-Euclidean)
    /// If covariance data is available, this method performs **Log-Euclidean Riemannian
    /// Interpolation**. Unlike standard linear element-wise interpolation, this approach
    /// respects the geometric manifold of Symmetric Positive Definite (SPD) matrices.
    ///
    /// This guarantees that:
    /// 1. **Positive Definiteness:** The interpolated covariance matrix is always mathematically
    ///    valid (all eigenvalues are strictly positive), preventing numerical crashes in downstream filters.
    /// 2. **Volume Preservation:** It prevents the artificial "swelling" (determinant increase)
    ///    of uncertainty that occurs when linearly interpolating between two valid matrices.
    ///    The interpolation follows the "geodesic" (shortest path) on the curved surface of
    ///    covariance matrices.
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: EphemerisRecord
    pub fn at(&self, epoch: Epoch, almanac: &Almanac) -> Result<EphemerisRecord, EphemerisError> {
        let (start, end) = self.domain()?;
        if !(start..=end).contains(&epoch) {
            return Err(EphemerisError::EphemInterpolation {
                source: crate::math::interpolation::InterpolationError::NoInterpolationData {
                    req: epoch,
                    start,
                    end,
                },
            });
        }
        // Grab the N/2 previous states
        let n = self.degree;
        let prev_states: Vec<EphemerisRecord> = {
            let mut states: Vec<EphemerisRecord> = self
                .state_data
                .range(..epoch)
                .rev()
                .take(n)
                .map(|e| *e.1)
                .collect();
            states.reverse();
            states
        };
        let next_states = self
            .state_data
            .range(epoch..)
            .take(n)
            .map(|e| *e.1)
            .collect::<Vec<EphemerisRecord>>();

        let states = prev_states.iter().chain(next_states.iter());

        let xs = states
            .clone()
            .map(|record| record.orbit.epoch.to_tdb_seconds())
            .collect::<Vec<f64>>();
        let mut orbit_data = Vector6::zeros();

        match self.interpolation {
            DataType::Type9LagrangeUnequalStep => {
                for i in 0..6 {
                    let ys = states
                        .clone()
                        .map(|record| record.orbit.to_cartesian_pos_vel()[i])
                        .collect::<Vec<f64>>();

                    let (val, _) = lagrange_eval(&xs, &ys, epoch.to_tdb_seconds())
                        .context(EphemInterpolationSnafu)?;
                    orbit_data[i] = val;
                }
            }
            DataType::Type13HermiteUnequalStep | DataType::Type12HermiteEqualStep => {
                for i in 0..3 {
                    let ys = states
                        .clone()
                        .map(|record| record.orbit.to_cartesian_pos_vel()[i])
                        .collect::<Vec<f64>>();
                    let ydots = states
                        .clone()
                        .map(|record| record.orbit.to_cartesian_pos_vel()[i + 3])
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

        let record = EphemerisRecord {
            orbit,
            covar: self.covar_at(epoch, LocalFrame::Inertial, almanac)?,
        };

        Ok(record)
    }

    /// Interpolate the ephemeris at the provided epoch, returning only the orbit.
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: Orbit
    pub fn orbit_at(&self, epoch: Epoch, almanac: &Almanac) -> Result<Orbit, EphemerisError> {
        Ok(self.at(epoch, almanac)?.orbit)
    }

    /// Interpolate the ephemeris covariance at the provided epoch.
    ///
    /// This method implements a "Rotate-Then-Interpolate" strategy to avoid physical
    /// artifacts when interpolating rotating covariances.
    ///
    /// 1. Finds the nearest covariance before and after the requested epoch.
    /// 2. Rotates BOTH endpoints into the requested `local_frame`.
    /// 3. Interpolates between the two stable matrices using Log-Euclidean Riemannian interpolation.
    ///
    /// :type epoch: Epoch
    /// :type local_frame: LocalFrame
    /// :type almanac: Almanac
    /// :rtype: Covariance
    pub fn covar_at(
        &self,
        epoch: Epoch,
        local_frame: LocalFrame,
        almanac: &Almanac,
    ) -> Result<Option<Covariance>, EphemerisError> {
        // 1. Retrieve the bounding covariance records
        // Note: We ignore the Orbit interpolation here because we only need the
        // Orbits at the ENDPOINTS to compute the rotation DCMs.
        let prev_record = self.nearest_before(epoch, almanac)?;
        let next_record = self.nearest_after(epoch, almanac)?;

        // If we have no covariance data at the endpoints, we can't do anything.
        if prev_record.covar.is_none() || next_record.covar.is_none() {
            return Ok(None);
        }

        // Rotate endpoints to the STABLE frame.
        // We safely unwrap the option since it's only None if the covariance was none, but we checked that before.
        let prev_covar = prev_record
            .covar_in_frame(local_frame)
            .context(EphemerisPhysicsSnafu {
                action: "rotating covariance",
            })?
            .unwrap();
        let next_covar = next_record
            .covar_in_frame(local_frame)
            .context(EphemerisPhysicsSnafu {
                action: "rotating covariance",
            })?
            .unwrap();

        // Calculate Alpha
        let t0 = prev_record.orbit.epoch;
        let t1 = next_record.orbit.epoch;
        let total_dt = (t1 - t0).to_seconds();

        // Handle exact match or zero-duration step
        if total_dt.abs() < 1e-9 {
            return Ok(Some(prev_covar));
        }

        let alpha = (epoch - t0).to_seconds() / total_dt;

        // 5. Interpolate (Log-Euclidean)
        // Now valid because both matrices are in the same, likely stable, frame.
        if let Some(mat) =
            interpolate_covar_log_euclidean(prev_covar.matrix, next_covar.matrix, alpha)
        {
            Ok(Some(Covariance {
                matrix: mat,
                local_frame, // We interpolated in this frame, so the result is in this frame
            }))
        } else {
            // Fallback or Error if PSD check fails (unlikely with valid inputs)
            Ok(None)
        }
    }

    /// Resample this ephemeris, with covariance, at the provided time series
    ///
    /// :type ts: TimeSeries
    /// :type almanac: Almanac
    /// :rtype: Ephemeris
    pub fn resample(&self, ts: TimeSeries, almanac: &Almanac) -> Result<Self, EphemerisError> {
        // NOTE: We clone ourselves because we still need our state data.
        let mut me = self.clone();
        me.state_data.clear();

        for epoch in ts {
            me.insert(self.at(epoch, almanac)?);
        }

        Ok(me)
    }

    /// Transforms this ephemeris into another frame, and rotates the covariance to that frame if the orientations are different.
    /// NOTE: The Nyquist-Shannon theorem is NOT applied here, so the new ephemeris may not be as precise as the original one.
    /// NOTE: If the orientations are different, the covariance will always be in the Inertial frame of the new frame.
    ///
    /// :type new_frame: Frame
    /// :type almanac: Almanac
    /// :rtype: Ephemeris
    pub fn transform(&self, new_frame: Frame, almanac: &Almanac) -> Result<Self, AlmanacError> {
        // NOTE: We clone ourselves because we still need our state data.
        let mut me = self.clone();
        me.state_data.clear();

        for (epoch, orig_record) in self.state_data.iter() {
            let orig_frame = orig_record.orbit.frame;
            let mut new_record = EphemerisRecord {
                orbit: almanac.transform_to(orig_record.orbit, new_frame, None)?,
                covar: orig_record.covar,
            };

            if let Some(covar) = &mut new_record.covar {
                if orig_frame.orientation_id != new_frame.orientation_id {
                    // Query the rotation matrix
                    let dcm = almanac.rotate(orig_frame, new_frame, *epoch).context(
                        OrientationSnafu {
                            action: "rotating covariance",
                        },
                    )?;

                    // Unwrap because we know it is set
                    covar.matrix = dcm.state_dcm()
                        * orig_record
                            .covar_in_frame(LocalFrame::Inertial)
                            .context(AlmanacPhysicsSnafu {
                                action: "computing covar inertial",
                            })?
                            .unwrap()
                            .matrix
                        * dcm.state_dcm().transpose();
                }
            }

            me.insert(new_record);
        }

        Ok(me)
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

impl<'a> IntoIterator for &'a Ephemeris {
    type Item = &'a EphemerisRecord;
    type IntoIter = Values<'a, Epoch, EphemerisRecord>;

    fn into_iter(self) -> Self::IntoIter {
        self.state_data.values()
    }
}

impl IntoIterator for Ephemeris {
    type Item = EphemerisRecord;
    type IntoIter = IntoValues<Epoch, EphemerisRecord>;

    fn into_iter(self) -> Self::IntoIter {
        self.state_data.into_values()
    }
}

#[cfg(test)]
mod ut_oem {
    use super::{Almanac, DataType, Ephemeris, LocalFrame};
    use crate::analysis::prelude::OrbitalElement;
    use crate::prelude::NAIFSummaryRecord;
    use hifitime::{Epoch, TimeSeries, Unit};
    use nalgebra::{Matrix6, SymmetricEigen, Vector6};
    use std::{fs::File, io::Write};

    use rstest::*;

    fn riemannian_distance(p1: &Matrix6<f64>, p2: &Matrix6<f64>) -> f64 {
        // 1. Compute M = P1^-1 * P2.
        // Optimization: Cholesky solve is better than explicit inverse.
        let m = p1.cholesky().unwrap().solve(p2);

        // 2. Eigenvalues of M (Generalized Eigenvalues)
        // Since P1, P2 are SPD, eigenvalues of P1^-1 P2 are real and positive.
        let complex_eigenvalues = m.complex_eigenvalues();

        // 3. Sum of log-squares
        complex_eigenvalues
            .iter()
            .map(|c| c.re.ln().powi(2))
            .sum::<f64>()
            .sqrt()
    }

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

        assert_eq!((&ephem).into_iter().count(), ephem.state_data.len());

        // Check that we can interpolate
        let epoch = start + Unit::Second * 5;
        let halfway_orbit = ephem.orbit_at(epoch, &almanac).unwrap();
        let before = ephem.nearest_orbit_before(epoch, &almanac).unwrap();
        let after = ephem.nearest_orbit_after(epoch, &almanac).unwrap();
        println!("before = {before}\nduring = {halfway_orbit}\nafter = {after}",);
        // Check that the Keplerian data is reasonably constant.
        // Note that the true Hermite test is in the NAIF SPK tests.
        assert!(dbg!(before.sma_km().unwrap() - halfway_orbit.sma_km().unwrap()).abs() < 1e-1);
        assert!(dbg!(after.sma_km().unwrap() - halfway_orbit.sma_km().unwrap()).abs() < 1e-1);
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

        // Ensure that we can build an OEM, re-parse it, and it should match
        let outpath = "../data/tests/ccsds/oem/MEO_60s_rebuilt.oem";
        ephem
            .write_ccsds_oem(outpath, Some("My Originator".to_string()), None)
            .unwrap();

        let ephem2 = Ephemeris::from_ccsds_oem_file(outpath).unwrap();
        assert_eq!(ephem2, ephem);

        // Build the SPK/BSP file as Type13 first
        let my_spk = ephem
            .to_spice_bsp(-159, Some(DataType::Type13HermiteUnequalStep))
            .unwrap();

        let mut file = File::create("../data/tests/naif/spk/meo.bsp").unwrap();
        file.write_all(&my_spk.bytes).unwrap();

        let frcrd = my_spk.file_record().unwrap();
        let name_rcrd = my_spk.name_record(None).unwrap();
        let summary_name = name_rcrd.nth_name(0, frcrd.summary_size());
        assert_eq!(summary_name, "0000-000A (converted by Nyx Space ANISE)");
        let summary = my_spk.summary_from_id(-159).unwrap().0;
        assert_eq!(
            summary.data_type().unwrap(),
            DataType::Type13HermiteUnequalStep
        );
        assert!(
            (summary.start_epoch() - ephem.start_epoch().unwrap()).abs() < Unit::Microsecond * 0.05
        );
        assert!(
            (summary.end_epoch() - ephem.end_epoch().unwrap()).abs() < Unit::Microsecond * 0.05
        );

        // Build without specifying the data type, which causes the builder to default to using a Lagrange interpolation.
        ephem
            .write_spice_bsp(-159, "../data/tests/naif/spk/meo_lagrange.bsp", None)
            .unwrap();
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

        // Check that we can interpolate the covariance
        let epoch = start + Unit::Minute * 15;
        let halfway = ephem
            .covar_at(
                epoch,
                crate::ephemerides::ephemeris::LocalFrame::Inertial,
                &almanac,
            )
            .unwrap()
            .unwrap()
            .matrix;
        let before = ephem
            .nearest_covar_before(epoch, &almanac)
            .unwrap()
            .unwrap()
            .1
            .matrix;
        let after = ephem
            .nearest_covar_after(epoch, &almanac)
            .unwrap()
            .unwrap()
            .1
            .matrix;
        println!("before = {before}\nduring = {halfway}\nafter = {after}");
        assert!((halfway - before).norm() < 1e-14);
        assert!((halfway - after).norm() < 1e-14);

        // Check that we can interpolate throughout the ephemeris
        for epoch in TimeSeries::inclusive(
            ephem.start_epoch().unwrap(),
            ephem.end_epoch().unwrap(),
            Unit::Minute * 1.337,
        ) {
            assert!(ephem.at(epoch, &almanac).is_ok());
        }

        // Re-export with covariance
        let rebuilt_path = "../data/tests/ccsds/oem/JPL_MGS_cov_rebuilt.oem";
        ephem.write_ccsds_oem(rebuilt_path, None, None).unwrap();
        let ephem2 =
            Ephemeris::from_ccsds_oem_file(rebuilt_path).expect("could not parse rebuilt OEM");

        assert!(ephem2.nearest_covar_after(epoch, &almanac).is_ok());
    }

    #[rstest]
    fn test_oem_interp_covar_truth(almanac: Almanac) {
        let ephem = Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/LRO_Nyx.oem")
            .expect("could not parse");

        let start = Epoch::from_gregorian_utc_at_midnight(2024, 1, 1);
        let end = start + Unit::Minute * 3;

        assert_eq!(ephem.state_data.len(), 4);
        assert_eq!(ephem.domain().unwrap(), (start, end));
        assert_eq!(ephem.interpolation, DataType::Type13HermiteUnequalStep);
        assert_eq!(ephem.degree, 7);

        // We have data from Nyx showing the proper covariance in between the data in the OEM.
        // So we'll check that the interpolator somewhat matches that data.
        let offset = Unit::Minute * 1 + Unit::Second * 24.696597;

        let epoch = start + offset;

        // Check that we can interpolate the covariance and that it correctly rotates.
        let bw_1_2 = ephem
            .covar_at(epoch, LocalFrame::Inertial, &almanac)
            .unwrap()
            .unwrap();
        assert_eq!(bw_1_2.local_frame, LocalFrame::Inertial);
        let bw_1_2_truth = Matrix6::new(
            0.209575, 0.4048630, 0.2455520, 0.001016, 0.0019710, 0.0011840, // X column
            0.404863, 1.1089200, -0.066758, 0.001961, 0.0055080, -0.000494, // Y column
            0.245552, -0.066758, 1.1863670, 0.001197, -0.000509, 0.0060070, // Z column
            0.001016, 0.0019610, 0.0011970, 0.000012, 0.0000240, 0.0000140, // Vx column
            0.001971, 0.0055080, -0.000509, 0.000024, 0.0000660, 0.0000060, // Vy column
            0.001184, -0.000494, 0.0060070, 0.000014, 0.0000060, 0.0000720, // Vz column
        );

        // Compute the Riemann distance since we interpolate in Reimann space
        let rdist = riemannian_distance(&bw_1_2.matrix, &bw_1_2_truth);
        assert!(rdist < 0.4, "arbitrary max distance failed");

        let covar_prev = ephem
            .nearest_covar_before(epoch, &almanac)
            .unwrap()
            .unwrap()
            .1
            .matrix;
        let covar_next = ephem
            .nearest_covar_after(epoch, &almanac)
            .unwrap()
            .unwrap()
            .1
            .matrix;

        let det_prev = covar_prev.determinant();
        let det_next = covar_next.determinant();
        let det_interp = bw_1_2.matrix.determinant();
        let det_truth = bw_1_2_truth.determinant();

        // Log-Euclidean guarantees the log-determinant is linearly interpolated!
        // This is a MUCH stricter mathematical check than comparing to external truth.
        let log_det_prev = det_prev.ln();
        let log_det_next = det_next.ln();
        let log_det_interp = det_interp.ln();
        let log_det_truth = det_truth.ln();

        // VALIDATION: Check Log-Euclidean Property (Volume Monotonicity)
        // The log-determinant must be linearly interpolated.
        // This validates the math of the implementation.
        let alpha = (offset - Unit::Minute).to_seconds() / 60.0;
        let expected_log_det = log_det_prev * (1.0 - alpha) + log_det_next * alpha;

        // Tolerance can be very tight because this is purely algebraic
        assert!(
            (log_det_interp - expected_log_det).abs() < 1e-12,
            "Log-Euclidean implementation failed linearity check"
        );

        // OBSERVATION: Compare with Truth
        // We expect a deviation here because interpolation != propagation.
        // This print confirms the "swelling/shrinking" discrepancy.
        let vol_ratio = (log_det_interp - log_det_truth).exp();
        println!("Volume Ratio (Interp/Truth): {vol_ratio:.4}");
        // Truth covar has a larger volume (less negative) because dynamics are not the shortest path in Log-Euclidian space
        assert!((vol_ratio - 0.8).abs() < 0.05);
        // Expect ~0.80 (Interp is smaller)

        dbg!(log_det_prev, log_det_next, log_det_interp, log_det_truth);

        // Ensure that this is a symmetric matrix
        assert!((bw_1_2.matrix.transpose() - bw_1_2.matrix).norm() < 1e-15);
        // Ensure that it's PSD
        let decomp = SymmetricEigen::new(bw_1_2.matrix);
        assert!(decomp.eigenvalues.iter().all(|&e| e >= 0.0));

        // Ensure that we're close to the RIC frame uncertainties

        let bw_1_2_ric = ephem
            .covar_at(epoch, LocalFrame::RIC, &almanac)
            .unwrap()
            .unwrap();

        assert_eq!(bw_1_2_ric.local_frame, LocalFrame::RIC);

        let diag = bw_1_2_ric.matrix.diagonal();
        let diag_sqrt = Vector6::from_iterator(diag.iter().map(|x| x.sqrt()));

        // Nyx reports these as Sigmas, so we apply the square root of the covariance here.
        let ric_diag_sigmas =
            Vector6::new(1.104494, 0.335512, 1.082771, 0.008646, 0.002558, 0.008262);
        let ric_err = diag_sqrt - ric_diag_sigmas;
        println!("{diag_sqrt:.6e}\n{ric_diag_sigmas:.6e}\n{ric_err:0.6e}",);
        let ric_pos_km_err = ric_err.fixed_rows::<3>(0);
        let ric_vel_km_s_err = ric_err.fixed_rows::<3>(3);
        assert!(dbg!(ric_pos_km_err.norm()) < 0.2);
        assert!(dbg!(ric_vel_km_s_err.norm()) < 2.3e-3);
    }

    #[rstest]
    fn test_oem_covar_orbital_element_uncertainty(almanac: Almanac) {
        let ephem = Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/LRO_Nyx.oem")
            .expect("could not parse");

        let start = Epoch::from_gregorian_utc_at_midnight(2024, 1, 1);
        let end = start + Unit::Minute * 3;

        assert_eq!(ephem.state_data.len(), 4);
        assert_eq!(ephem.domain().unwrap(), (start, end));
        assert_eq!(ephem.interpolation, DataType::Type13HermiteUnequalStep);
        assert_eq!(ephem.degree, 7);

        // We have data from Nyx showing the proper covariance in between the data in the OEM.
        // So we'll check that the interpolator somewhat matches that data.
        let offset = Unit::Minute * 1 + Unit::Second * 24.696597;

        let epoch = start + offset;

        let rcrd1 = ephem.at(start, &almanac).unwrap();
        let rcrd2 = ephem.at(epoch, &almanac).unwrap();
        let rcrd3 = ephem.at(end, &almanac).unwrap();

        for oe in [
            OrbitalElement::Rmag,
            OrbitalElement::SemiMajorAxis,
            OrbitalElement::Hmag,
        ] {
            // Test that the covariance interpolation follows the manifold and does not swell.
            let sigma1 = rcrd1.sigma_for(oe).unwrap();
            let sigma2 = rcrd2.sigma_for(oe).unwrap();
            let sigma3 = rcrd3.sigma_for(oe).unwrap();

            dbg!(oe, sigma1, sigma2, sigma3);

            // The range is empty if start > end, so we check both options.
            assert!(
                (sigma1..=sigma3).contains(&sigma2) || (sigma3..=sigma1).contains(&sigma2),
                "failed on {oe:?}"
            );
        }
    }

    #[rstest]
    fn test_parse_stk_e_v12(almanac: Almanac) {
        let path = "../data/tests/ansys-stk/test_v12.e";
        let ephem = Ephemeris::from_stk_e_file(path).expect("Could not parse STK file");

        // Check metadata
        assert_eq!(
            format!("{:?}", ephem.interpolation()),
            "Type9LagrangeUnequalStep"
        );
        assert_eq!(ephem.object_id(), "test_v12");

        // Check domain
        let (start, end) = ephem.domain().expect("Could not get domain");

        // ScenarioEpoch: 1 Jun 2020 12:00:00.000000
        let scenario_epoch = Epoch::from_gregorian_utc_at_noon(2020, 6, 1);

        // First point at 0.0s offset
        let expected_start = scenario_epoch;
        assert!(
            (start - expected_start).to_seconds().abs() < 1e-6,
            "Start epoch mismatch: {start} vs {expected_start}"
        );

        // Last point at 1980.0s offset
        let expected_end = scenario_epoch + Unit::Second * 1980.0;
        assert!(
            (end - expected_end).to_seconds().abs() < 1e-6,
            "End epoch mismatch: {end} vs {expected_end}",
        );

        assert_eq!(ephem.state_data.len(), 34);

        let record = ephem.at(expected_end, &almanac).unwrap();
        assert!(record.covar.is_none());
        assert_eq!(record.orbit.epoch, expected_end);
    }

    #[rstest]
    fn test_parse_stk_e_with_covariance(almanac: Almanac) {
        let path = "../data/tests/ansys-stk/stk_cov.e";
        let ephem = Ephemeris::from_stk_e_file(path).expect("Could not parse STK file");

        // Check metadata
        assert_eq!(
            format!("{:?}", ephem.interpolation()),
            "Type9LagrangeUnequalStep"
        );

        assert!(
            ephem.includes_covariance(),
            "Ephemeris should have covariance"
        );

        let (start, _) = ephem.domain().expect("Could not get domain");

        // Check first point (Time 0.0) -> Sequence 1.0 to 21.0
        // LowerTriangular Order:
        // C[0][0] = 1
        // C[1][0] = 2, C[1][1] = 3
        // C[2][0] = 4, C[2][1] = 5, C[2][2] = 6
        // ...
        // C[5][5] = 21
        let rec0 = ephem.at(start, &almanac).unwrap();
        assert!(rec0.covar.is_some());
        let mat0 = rec0.covar.unwrap().matrix;

        assert!((mat0[(0, 0)] - 1.0).abs() < 1e-9);
        assert!((mat0[(1, 0)] - 2.0).abs() < 1e-9);
        assert!((mat0[(1, 1)] - 3.0).abs() < 1e-9);
        assert!((mat0[(2, 0)] - 4.0).abs() < 1e-9);
        assert!((mat0[(2, 2)] - 6.0).abs() < 1e-9);

        // Check last element C[5][5]
        // Row 3: 7, 8, 9, 10
        // Row 4: 11, 12, 13, 14, 15
        // Row 5: 16, 17, 18, 19, 20, 21
        assert!((mat0[(5, 0)] - 16.0).abs() < 1e-9);
        assert!((mat0[(5, 5)] - 21.0).abs() < 1e-9);

        // Verify Symmetry
        assert!((mat0[(0, 1)] - 2.0).abs() < 1e-9);
        assert!((mat0[(5, 2)] - 18.0).abs() < 1e-9); // C[2][5] symmetric to C[5][2] (value 18)
    }
}
