/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::f64;

use anise::constants::frames::EARTH_J2000;
use anise::prelude::*;

use rstest::*;

#[fixture]
pub fn almanac() -> Almanac {
    use std::path::PathBuf;

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string()));

    Almanac::new(
        &manifest_dir
            .clone()
            .join("../data/de440s.bsp")
            .to_string_lossy(),
    )
    .unwrap()
    .load(
        &manifest_dir
            .clone()
            .join("../data/pck08.pca")
            .to_string_lossy(),
    )
    .unwrap()
}

/// Computes the beta angle, checks that it remains stable throughout a two body propagation (as it should since it's based on the angular momentum).
/// Importantly, this does not verify the implementation with known values, simply verifies the computation in a few regimes.
/// The beta angle code is identical to that from GMAT <https://github.com/ChristopherRabotin/GMAT/blob/GMAT-R2022a/src/gmatutil/util/CalculationUtilities.cpp#L209-L219>
#[rstest]
fn verif_beta_angle_eclipse_time(almanac: Almanac) {
    let epoch = Epoch::from_gregorian_utc_at_midnight(2024, 1, 1);
    let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();
    let raan_deg = 72.0;
    let aop_deg = 45.0;
    let ta_deg = 270.0;

    for alt_km in [500.0, 1500.0, 22000.0, 36000.0, 50000.0] {
        for inc_deg in [1.0, 18.0, 36.0, 54.0, 72.0, 90.0] {
            // Initialize an orbit at the provided inclination and altitude
            let ecc = inc_deg * 1e-2;
            let orbit = Orbit::try_keplerian_altitude(
                alt_km, ecc, inc_deg, raan_deg, aop_deg, ta_deg, epoch, eme2k,
            )
            .unwrap_or_else(|_| {
                panic!("init failed with alt_km = {alt_km}; inc_deg = {inc_deg}; ecc = {ecc}")
            });

            let mut eclipse_duration = 0.0.seconds();
            let mut sum_beta_angles = 0.0;
            let mut count = 0;
            let step = 1.minutes();
            // Two body propagation of a single orbit, computing whether we're in eclipse or not.
            for new_epoch in TimeSeries::exclusive(epoch, epoch + orbit.period().unwrap(), step) {
                count += 1;
                // Compute the solar eclipsing
                let occult = almanac
                    .solar_eclipsing(
                        EARTH_J2000,
                        orbit.at_epoch(new_epoch).expect("two body prop failed"),
                        None,
                    )
                    .unwrap();
                if occult.is_obstructed() {
                    eclipse_duration += step;
                }
                sum_beta_angles += almanac
                    .beta_angle_deg(orbit, None)
                    .expect("beta angle failed");
            }
            let beta_angle = almanac
                .beta_angle_deg(orbit, None)
                .expect("beta angle failed");
            let avr_beta_angle = sum_beta_angles / (count as f64);

            println!(
                "beta angle = {beta_angle:.6} deg (avr. of {avr_beta_angle:.6} deg)\teclipse duration = {eclipse_duration} (+/- 2 min)"
            );

            assert!(
                (avr_beta_angle - beta_angle).abs() < 1e-12,
                "beta angle should not vary over an orbit: avr = {avr_beta_angle} deg\tinst.: {beta_angle}"
            );
        }
    }
}
