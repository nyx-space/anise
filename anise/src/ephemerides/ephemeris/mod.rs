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
use crate::math::rotation::DCM;
use crate::math::Vector6;
use crate::naif::daf::data_types::DataType;
use crate::prelude::{uuid_from_epoch, Almanac, Orbit};
use core::fmt;
use covariance::interpolate_covar_log_euclidean;
use hifitime::Epoch;
use snafu::ResultExt;
use std::collections::BTreeMap;

#[cfg(feature = "python")]
use pyo3::prelude::*;

mod covariance;
mod oem;
#[cfg(feature = "python")]
mod python;
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
        if let Ok(Some((epoch0, covar0))) = self.nearest_covar_before(epoch, almanac) {
            if let Ok(Some((epoch1, mut covar1))) = self.nearest_covar_after(epoch, almanac) {
                // TODO: Rotate the covariances if need be!
                if covar0.local_frame != covar1.local_frame {
                    // Rotate the second covariance into the frame of the first.
                    let orbit0 = self.nearest_orbit_before(epoch, almanac)?;
                    let orbit1 = self.nearest_orbit_after(epoch, almanac)?;
                    let dcm_0_to_inertial =
                        match covar0.local_frame {
                            LocalFrame::Inertial => DCM::identity(
                                uuid_from_epoch(orbit0.frame.orientation_id, orbit0.epoch),
                                orbit0.frame.orientation_id,
                            ),
                            LocalFrame::RIC => orbit0.dcm_from_ric_to_inertial().context(
                                EphemerisPhysicsSnafu {
                                    action: "rotating covariance from RIC to Inertial",
                                },
                            )?,
                            LocalFrame::RCN => orbit0.dcm_from_ric_to_inertial().context(
                                EphemerisPhysicsSnafu {
                                    action: "rotating covariance from RCN to Inertial",
                                },
                            )?,
                            LocalFrame::VNC => orbit0.dcm_from_ric_to_inertial().context(
                                EphemerisPhysicsSnafu {
                                    action: "rotating covariance from VNC to Inertial",
                                },
                            )?,
                        };

                    let dcm_1_to_inertial =
                        match covar1.local_frame {
                            LocalFrame::Inertial => DCM::identity(
                                uuid_from_epoch(orbit1.frame.orientation_id, orbit1.epoch),
                                orbit1.frame.orientation_id,
                            ),
                            LocalFrame::RIC => orbit1.dcm_from_ric_to_inertial().context(
                                EphemerisPhysicsSnafu {
                                    action: "rotating covariance from RIC to Inertial",
                                },
                            )?,
                            LocalFrame::RCN => orbit1.dcm_from_ric_to_inertial().context(
                                EphemerisPhysicsSnafu {
                                    action: "rotating covariance from RCN to Inertial",
                                },
                            )?,
                            LocalFrame::VNC => orbit1.dcm_from_ric_to_inertial().context(
                                EphemerisPhysicsSnafu {
                                    action: "rotating covariance from VNC to Inertial",
                                },
                            )?,
                        };

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

    /// Interpolate the ephemeris at the provided epoch, returning only the covariance.
    pub fn covar_at(
        &self,
        epoch: Epoch,
        local_frame: LocalFrame,
        almanac: &Almanac,
    ) -> Result<Option<Covariance>, EphemerisError> {
        let entry = self.at(epoch, almanac)?;

        if let Some(mut covar) = entry.covar {
            let orbit = entry.orbit;

            let desired_frame_to_inertial = match local_frame {
                LocalFrame::Inertial => DCM::identity(
                    uuid_from_epoch(orbit.frame.orientation_id, orbit.epoch),
                    orbit.frame.orientation_id,
                ),
                LocalFrame::RIC => {
                    orbit
                        .dcm_from_ric_to_inertial()
                        .context(EphemerisPhysicsSnafu {
                            action: "rotating covariance from RIC to Inertial",
                        })?
                }
                LocalFrame::RCN => {
                    orbit
                        .dcm_from_ric_to_inertial()
                        .context(EphemerisPhysicsSnafu {
                            action: "rotating covariance from RCN to Inertial",
                        })?
                }
                LocalFrame::VNC => {
                    orbit
                        .dcm_from_ric_to_inertial()
                        .context(EphemerisPhysicsSnafu {
                            action: "rotating covariance from VNC to Inertial",
                        })?
                }
            };

            let cur_frame_to_inertial = match covar.local_frame {
                LocalFrame::Inertial => DCM::identity(
                    uuid_from_epoch(orbit.frame.orientation_id, orbit.epoch),
                    orbit.frame.orientation_id,
                ),
                LocalFrame::RIC => {
                    orbit
                        .dcm_from_ric_to_inertial()
                        .context(EphemerisPhysicsSnafu {
                            action: "rotating covariance from RIC to Inertial",
                        })?
                }
                LocalFrame::RCN => {
                    orbit
                        .dcm_from_ric_to_inertial()
                        .context(EphemerisPhysicsSnafu {
                            action: "rotating covariance from RCN to Inertial",
                        })?
                }
                LocalFrame::VNC => {
                    orbit
                        .dcm_from_ric_to_inertial()
                        .context(EphemerisPhysicsSnafu {
                            action: "rotating covariance from VNC to Inertial",
                        })?
                }
            };

            let dcm = (desired_frame_to_inertial * cur_frame_to_inertial.transpose())
                .expect("internal error");

            // Rotate covar1 from its frame to the frame of the covar0
            covar.matrix = dcm.state_dcm() * covar.matrix * dcm.state_dcm().transpose();

            Ok(Some(covar))
        } else {
            Ok(None)
        }
        // Ok(self.at(epoch, almanac)?.covar)
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
        // NOTE this will need changing after I implement the rotations
        println!("delta before = {:e}", halfway - before);
        println!("delta after = {:e}", after - halfway)
    }
}
