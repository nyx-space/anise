/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{
    astro::{Aberration, AzElRange},
    constants::SPEED_OF_LIGHT_KM_S,
    ephemerides::{EphemerisError, EphemerisPhysicsSnafu},
    errors::{AlmanacError, EphemerisSnafu, PhysicsError},
    frames::Frame,
    math::angles::{between_0_360, between_pm_180},
    prelude::Orbit,
};

use super::Almanac;
use crate::errors::AlmanacResult;

use hifitime::TimeUnits;
use log::warn;

use snafu::ResultExt;

impl Almanac {
    /// Computes the azimuth (in degrees), elevation (in degrees), and range (in kilometers) of the
    /// receiver state (`rx`) seen from the transmitter state (`tx`), once converted into the SEZ frame of the transmitter.
    ///
    /// # Warning
    /// The obstructing body _should_ be a tri-axial ellipsoid body, e.g. IAU_MOON_FRAME.
    ///
    /// # Algorithm
    /// 1. If any obstructing_bodies are provided, ensure that none of these are obstructing the line of sight between the receiver and transmitter.
    /// 2. Compute the SEZ (South East Zenith) frame of the transmitter.
    /// 3. Rotate the receiver position vector into the transmitter SEZ frame.
    /// 4. Rotate the transmitter position vector into that same SEZ frame.
    /// 5. Compute the range as the norm of the difference between these two position vectors.
    /// 6. Compute the elevation, and ensure it is between +/- 180 degrees.
    /// 7. Compute the azimuth with a quadrant check, and ensure it is between 0 and 360 degrees.
    pub fn azimuth_elevation_range_sez(
        &self,
        rx: Orbit,
        tx: Orbit,
        obstructing_body: Option<Frame>,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<AzElRange> {
        if tx.epoch != rx.epoch {
            return Err(AlmanacError::Ephemeris {
                action: "",
                source: Box::new(EphemerisError::EphemerisPhysics {
                    action: "computing AER",
                    source: PhysicsError::EpochMismatch {
                        action: "computing AER",
                        epoch1: tx.epoch,
                        epoch2: rx.epoch,
                    },
                }),
            });
        }

        let mut obstructed_by = None;
        if let Some(obstructing_body) = obstructing_body {
            if self.line_of_sight_obstructed(tx, rx, obstructing_body, ab_corr)? {
                obstructed_by = Some(obstructing_body);
            }
        }

        // Compute the SEZ DCM
        // SEZ DCM is topo to fixed
        let sez_dcm = tx
            .dcm_from_topocentric_to_body_fixed(-1)
            .context(EphemerisPhysicsSnafu { action: "" })
            .context(EphemerisSnafu {
                action: "computing SEZ DCM for AER",
            })?;

        let tx_sez = (sez_dcm.transpose() * tx)
            .context(EphemerisPhysicsSnafu { action: "" })
            .context(EphemerisSnafu {
                action: "transforming transmitter to SEZ",
            })?;

        // Convert the receiver into the body fixed transmitter frame.
        let rx_in_tx_frame = self.transform_to(rx, tx.frame, ab_corr)?;
        // Convert into SEZ frame
        let rx_sez = (sez_dcm.transpose() * rx_in_tx_frame)
            .context(EphemerisPhysicsSnafu { action: "" })
            .context(EphemerisSnafu {
                action: "transforming received to SEZ",
            })?;

        // Convert receiver into the transmitter frame
        let rx_in_tx_frame = self.transform_to(rx, tx.frame, ab_corr)?;

        // Compute the range ρ in the SEZ frame for az/el
        let rho_sez = rx_sez.radius_km - tx_sez.radius_km;
        // And in the body-fixed transmitter frame for range and range-rate.
        // While the norms of these vectors are identical, we need the exact vectors themselves for the range rate calculation.
        let rho_tx_frame = rx_in_tx_frame.radius_km - tx.radius_km;

        // Compute the range-rate \dot ρ. Note that rx_in_tx_frame is already the relative velocity of rx wrt tx!
        let range_rate_km_s = rho_tx_frame.dot(&rx_in_tx_frame.velocity_km_s) / rho_tx_frame.norm();

        // Finally, compute the elevation (math is the same as declination)
        // Source: Vallado, section 4.4.3
        // Only the sine is needed as per Vallado, and the formula is the same as the declination
        // because we're in the SEZ frame.
        let elevation_deg = between_pm_180((rho_sez.z / rho_sez.norm()).asin().to_degrees());
        if (elevation_deg - 90.0).abs() < 1e-6 {
            warn!("object nearly overhead (el = {elevation_deg:.6} deg), azimuth may be incorrect");
        }
        // For the elevation, we need to perform a quadrant check because it's measured from 0 to 360 degrees.
        let azimuth_deg = between_0_360((rho_sez.y.atan2(-rho_sez.x)).to_degrees());

        Ok(AzElRange {
            epoch: tx.epoch,
            azimuth_deg,
            elevation_deg,
            range_km: rho_sez.norm(),
            range_rate_km_s,
            obstructed_by,
            light_time: (rho_sez.norm() / SPEED_OF_LIGHT_KM_S).seconds(),
        })
    }
}

#[cfg(test)]
mod ut_aer {
    use std::path::Path;

    use hifitime::Unit;

    use crate::astro::orbit::Orbit;
    use crate::astro::AzElRange;
    use crate::constants::frames::{EARTH_ITRF93, EARTH_J2000, IAU_EARTH_FRAME};
    use crate::constants::usual_planetary_constants::MEAN_EARTH_ANGULAR_VELOCITY_DEG_S;
    use crate::math::cartesian::CartesianState;
    use crate::prelude::{Almanac, Epoch, MetaAlmanac};

    #[test]
    fn verif_edge_case() {
        let almanac = Almanac::new("../data/pck08.pca").unwrap();
        let itrf93 = almanac.frame_from_uid(EARTH_ITRF93).unwrap();

        // Data from another test case
        let latitude_deg = -7.906_635_7;
        let longitude_deg = 345.5975;
        let height_km = 56.0e-3;
        let epoch = Epoch::from_gregorian_utc_at_midnight(2024, 1, 14);

        let ground_station = Orbit::try_latlongalt(
            latitude_deg,
            longitude_deg,
            height_km,
            MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
            epoch,
            itrf93,
        )
        .unwrap();

        let aer = almanac
            .azimuth_elevation_range_sez(ground_station, ground_station, None, None)
            .unwrap();

        assert!(!aer.is_valid());
    }

    /// Test comes from Nyx v 2.0.0-beta where we propagate a trajectory in GMAT and in Nyx and check that we match the measurement data.
    /// This test MUST be change to a validation instead of a verification.
    /// At the moment, the test checks that the range values are _similar_ to those generated by Nyx _before_ it was updated to use ANISE.
    #[test]
    fn gmat_verif() {
        use core::str::FromStr;

        // Build the Madrid DSN gound station
        let latitude_deg = 40.427_222;
        let longitude_deg = 4.250_556;
        let height_km = 0.834_939;

        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/aer_regression.dhall");
        let almanac = MetaAlmanac::new(path.to_str().unwrap().to_string())
            .unwrap()
            .process(false)
            .unwrap();

        let iau_earth = almanac.frame_from_uid(IAU_EARTH_FRAME).unwrap();
        let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();

        // Now iterate the trajectory to generate the measurements.
        let gmat_ranges_km = [
            9.145_755_787_575_61e4,
            9.996_505_560_799_869e4,
            1.073_229_118_411_670_2e5,
            1.145_516_751_191_464_7e5,
            1.265_739_190_638_930_7e5,
        ];

        let regression_data = [
            AzElRange {
                epoch: Epoch::from_str("2023-11-16T13:35:30.231999909 UTC").unwrap(),
                azimuth_deg: 133.59998745846255,
                elevation_deg: 7.23756749931629,
                range_km: 91457.2680164461,
                range_rate_km_s: 2.198785823156608,
                obstructed_by: None,
                light_time: 305068608 * Unit::Nanosecond,
            },
            AzElRange {
                epoch: Epoch::from_str("2023-11-16T14:41:30.231999930 UTC").unwrap(),
                azimuth_deg: 145.20134040829316,
                elevation_deg: 15.541883052027405,
                range_km: 99963.52694785153,
                range_rate_km_s: 2.1050771837046436,
                obstructed_by: None,
                light_time: 333442434 * Unit::Nanosecond,
            },
            AzElRange {
                epoch: Epoch::from_str("2023-11-16T15:40:30.231999839 UTC").unwrap(),
                azimuth_deg: 157.35605910179052,
                elevation_deg: 21.262025972059224,
                range_km: 107320.26696466877,
                range_rate_km_s: 2.0559576546712433,
                obstructed_by: None,
                light_time: 357981877 * Unit::Nanosecond,
            },
            AzElRange {
                epoch: Epoch::from_str("2023-11-16T16:39:30.232000062 UTC").unwrap(),
                azimuth_deg: 171.0253271744456,
                elevation_deg: 24.777800273900453,
                range_km: 114548.0748997545,
                range_rate_km_s: 2.0308909733778924,
                obstructed_by: None,
                light_time: 382091249 * Unit::Nanosecond,
            },
            AzElRange {
                epoch: Epoch::from_str("2023-11-16T18:18:30.231999937 UTC").unwrap(),
                azimuth_deg: 195.44253883914308,
                elevation_deg: 24.63526601848747,
                range_km: 126569.46572408297,
                range_rate_km_s: 2.021336308601692,
                obstructed_by: None,
                light_time: 422190293 * Unit::Nanosecond,
            },
        ];

        let states = [
            CartesianState::new(
                58643.769881020,
                -61696.430010747,
                -36178.742480219,
                2.148654262,
                -1.202488371,
                -0.714016096,
                Epoch::from_str("2023-11-16T13:35:30.231999909 UTC").unwrap(),
                eme2k,
            ),
            CartesianState::new(
                66932.786922851,
                -66232.181345574,
                -38873.607459037,
                2.040554622,
                -1.092315772,
                -0.649375769,
                Epoch::from_str("2023-11-16T14:41:30.231999930 UTC").unwrap(),
                eme2k,
            ),
            CartesianState::new(
                74004.678508956,
                -69951.392953800,
                -41085.743778595,
                1.956605843,
                -1.011238479,
                -0.601766262,
                Epoch::from_str("2023-11-16T15:40:30.231999839 UTC").unwrap(),
                eme2k,
            ),
            CartesianState::new(
                80796.571971532,
                -73405.942333285,
                -43142.412981359,
                1.882014733,
                -0.942231959,
                -0.561216138,
                Epoch::from_str("2023-11-16T16:39:30.232000062 UTC").unwrap(),
                eme2k,
            ),
            CartesianState::new(
                91643.443331668,
                -78707.208988294,
                -46302.221669744,
                1.773134524,
                -0.846263432,
                -0.504774983,
                Epoch::from_str("2023-11-16T18:18:30.231999937 UTC").unwrap(),
                eme2k,
            ),
        ];

        for (sno, state) in states.iter().copied().enumerate() {
            // Rebuild the ground station at this new epoch
            let madrid = Orbit::try_latlongalt(
                latitude_deg,
                longitude_deg,
                height_km,
                MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
                state.epoch,
                iau_earth,
            )
            .unwrap();

            let aer = almanac
                .azimuth_elevation_range_sez(state, madrid, None, None)
                .unwrap();

            if sno == 0 {
                assert_eq!(
                    format!("{aer}"),
                    format!(
                        "{}: az.: 133.599987 deg    el.: 7.237567 deg    range: 91457.268016 km    range-rate: 2.198786 km/s    obstruction: none",
                        state.epoch
                    )
                );
            }

            let expect = gmat_ranges_km[sno];

            // The verification test was generated years ago using different data than in this test.
            // However, it's been validated in real-world cislunar operations, the best kind of validation.
            // Let's confirm that the data is not garbage compared to GMAT...
            assert!((aer.range_km - expect).abs() < 5.0);
            // ... and assert a regression check too
            assert_eq!(aer, regression_data[sno], "{sno} differ");
        }

        // Ensure that if the state are in another frame, the results are (nearly) identical.

        let states = states.map(|state| almanac.transform_to(state, EARTH_ITRF93, None).unwrap());

        for (sno, state) in states.iter().copied().enumerate() {
            // Rebuild the ground station at this new epoch
            let madrid = Orbit::try_latlongalt(
                latitude_deg,
                longitude_deg,
                height_km,
                MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
                state.epoch,
                iau_earth,
            )
            .unwrap();

            let aer = almanac
                .azimuth_elevation_range_sez(state, madrid, None, None)
                .unwrap();

            if sno == 0 {
                assert_eq!(
                    format!("{aer}"),
                    format!(
                        "{}: az.: 133.599987 deg    el.: 7.237567 deg    range: 91457.268016 km    range-rate: 2.198786 km/s    obstruction: none",
                        state.epoch
                    )
                );
            }

            let expect = gmat_ranges_km[sno];

            // The verification test was generated years ago using different data than in this test.
            // However, it's been validated in real-world cislunar operations, the best kind of validation.
            // Let's confirm that the data is not garbage compared to GMAT...
            assert!((aer.range_km - expect).abs() < 5.0);
            // ... and assert a regression check too, with some small error for the transformation
            assert!(
                (aer.range_km - regression_data[sno].range_km).abs() < 1e-10,
                "{sno}"
            );
            assert!(
                (aer.range_rate_km_s - regression_data[sno].range_rate_km_s).abs() < 1e-10,
                "{sno}"
            );
            assert!(
                (aer.elevation_deg - regression_data[sno].elevation_deg).abs() < 1e-10,
                "{sno}"
            );
            assert!(
                (aer.azimuth_deg - regression_data[sno].azimuth_deg).abs() < 1e-10,
                "{sno}"
            );
        }
    }
}
