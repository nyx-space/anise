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
use crate::math::interpolation::{hermite_eval, lagrange_eval};
use crate::math::Vector6;
use crate::naif::daf::data_types::DataType;
use crate::prelude::{Almanac, Orbit};
use core::fmt;
use covariance::interpolate_covar_log_euclidean;
use hifitime::Epoch;
use snafu::ResultExt;
use std::collections::BTreeMap;

#[cfg(feature = "python")]
use pyo3::prelude::*;

mod almanac;
mod covariance;
mod oem;
#[cfg(feature = "python")]
mod python;
pub use covariance::{Covariance, LocalFrame};

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.ephemeris", get_all))]
pub struct EphemEntry {
    /// Orbit of this ephemeris entry
    pub orbit: Orbit,
    /// Optional covariance associated with this orbit
    pub covar: Option<Covariance>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.ephemeris"))]
pub struct Ephemeris {
    object_id: String,
    interpolation: DataType,
    degree: usize,
    /// Ephemeris entries in chronological order
    state_data: BTreeMap<Epoch, EphemEntry>,
}

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

    /// Returns whether all of the data in this ephemeris includes the covariance.
    ///
    /// :rtype: bool
    pub fn includes_covariance(&self) -> bool {
        self.state_data
            .values()
            .filter(|entry| entry.covar.is_none())
            .count()
            > 0
    }

    /// Inserts a new ephemeris entry to this ephemeris (it is automatically sorted chronologically).
    pub fn insert(&mut self, entry: EphemEntry) {
        self.state_data.insert(entry.orbit.epoch, entry);
    }

    /// Inserts a new orbit (without covariance) to this ephemeris (it is automatically sorted chronologically).
    pub fn insert_orbit(&mut self, orbit: Orbit) {
        self.state_data
            .insert(orbit.epoch, EphemEntry { orbit, covar: None });
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
    ) -> Result<Option<(Epoch, Covariance)>, EphemerisError> {
        let entry = self.nearest_before(epoch, almanac)?;
        Ok(entry.covar.map(|c| (entry.orbit.epoch, c)))
    }

    pub fn nearest_covar_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Option<(Epoch, Covariance)>, EphemerisError> {
        let entry = self.nearest_after(epoch, almanac)?;
        Ok(entry.covar.map(|c| (entry.orbit.epoch, c)))
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
                        .map(|entry| entry.orbit.to_cartesian_pos_vel()[i + 3])
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
        if let Ok(Some((epoch0, covar0))) = self.nearest_covar_before(epoch, almanac) {
            if let Ok(Some((epoch1, mut covar1))) = self.nearest_covar_after(epoch, almanac) {
                if covar0.local_frame != covar1.local_frame {
                    // Rotate the second covariance into the frame of the first.
                    let orbit0 = self.nearest_orbit_before(epoch, almanac)?;
                    let orbit1 = self.nearest_orbit_after(epoch, almanac)?;
                    let dcm_0_to_inertial = orbit0.dcm_to_inertial(covar0.local_frame).context(
                        EphemerisPhysicsSnafu {
                            action: "rotating orbit0 covariance",
                        },
                    )?;

                    let dcm_1_to_inertial = orbit1.dcm_to_inertial(covar1.local_frame).context(
                        EphemerisPhysicsSnafu {
                            action: "rotating orbit1 covariance",
                        },
                    )?;

                    let dcm = (dcm_0_to_inertial * dcm_1_to_inertial.transpose())
                        .expect("internal error");
                    // Rotate covar1 from its frame to the frame of the covar0
                    covar1.matrix = dcm.state_dcm() * covar1.matrix * dcm.state_dcm().transpose();
                }
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

    /// Interpolate the ephemeris at the provided epoch, returning only the orbit.
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
    pub fn covar_at(
        &self,
        epoch: Epoch,
        local_frame: LocalFrame,
        almanac: &Almanac,
    ) -> Result<Option<Covariance>, EphemerisError> {
        // 1. Retrieve the bounding covariance records
        // Note: We ignore the Orbit interpolation here because we only need the
        // Orbits at the ENDPOINTS to compute the rotation DCMs.
        let prev_entry = self.nearest_before(epoch, almanac)?;
        let next_entry = self.nearest_after(epoch, almanac)?;

        // If we have no covariance data at the endpoints, we can't do anything.
        if prev_entry.covar.is_none() || next_entry.covar.is_none() {
            return Ok(None);
        }

        // 2. Define a helper to rotate a specific entry's covariance into the target frame.
        // We capture 'almanac' and 'local_frame' from the environment.
        let rotate_to_target = |entry: EphemEntry| -> Result<Covariance, EphemerisError> {
            let mut covar = entry.covar.unwrap();
            let orbit = entry.orbit;

            // If it's already in the right frame, no-op
            if covar.local_frame == local_frame {
                return Ok(covar);
            }

            // Calculate Target Frame -> Inertial
            let desired_frame_to_inertial =
                orbit
                    .dcm_to_inertial(local_frame)
                    .context(EphemerisPhysicsSnafu {
                        action: "rotating desired covariance to Inertial",
                    })?;

            // Calculate Current Covar Frame -> Inertial
            let cur_frame_to_inertial =
                orbit
                    .dcm_to_inertial(covar.local_frame)
                    .context(EphemerisPhysicsSnafu {
                        action: "rotating source covariance to Inertial",
                    })?;

            // M = R_target_to_inertial * (R_source_to_inertial)^T
            // M maps Source -> Target
            let dcm = (desired_frame_to_inertial * cur_frame_to_inertial.transpose())
                .expect("internal error: DCM multiplication failed");

            // Apply 6x6 Rotation: P_new = M * P_old * M^T
            covar.matrix = dcm.state_dcm() * covar.matrix * dcm.state_dcm().transpose();
            covar.local_frame = local_frame;

            Ok(covar)
        };

        // 3. Rotate endpoints to the STABLE frame
        let prev_covar = rotate_to_target(prev_entry)?;
        let next_covar = rotate_to_target(next_entry)?;

        // 4. Calculate Alpha
        let t0 = prev_entry.orbit.epoch;
        let t1 = next_entry.orbit.epoch;
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
    use super::{Almanac, DataType, Ephemeris, LocalFrame};
    use hifitime::{Epoch, Unit};
    use nalgebra::{Matrix6, SymmetricEigen, Vector6};

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
            Vector6::new(0.961317, 0.854136, 0.922597, 0.007343, 0.006684, 0.007138);
        let ric_err = diag_sqrt - ric_diag_sigmas;
        println!("{diag_sqrt:.6e}\n{ric_diag_sigmas:.6e}\n{ric_err:0.6e}",);
        let ric_pos_km_err = ric_err.fixed_rows::<3>(0);
        let ric_vel_km_s_err = ric_err.fixed_rows::<3>(3);
        assert!(ric_pos_km_err.norm() < 0.06);
        assert!(ric_vel_km_s_err.norm() < 1e-3);
    }
}
