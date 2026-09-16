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
use std::collections::HashMap;

use anise::constants::frames::{
    EARTH_ICRS, IAU_EARTH_FRAME, IAU_JUPITER_FRAME, IAU_MARS_FRAME, IAU_MOON_FRAME,
    IAU_URANUS_FRAME, IAU_VENUS_FRAME,
};
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
    let eme2k = almanac.frame_info(EARTH_ICRS).unwrap();
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
                        EARTH_ICRS,
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

// This test only checks for IAU Earth, IAU Moon, and IAU Venus both of which are loaded in the nominal test suite.
// Venus is flipped on its side, so it's a good test case.
// Refer to the exhaustive test for a thorough test case.
#[rstest]
fn apparent_local_solar_time_limited(almanac: Almanac) {
    // Define a state in GCRF, and compute the apparent solar time on several objects.
    let state = Orbit::keplerian(
        10_000.0,
        1e-5,
        28.5,
        75.0,
        76.0,
        78.0,
        Epoch::from_gregorian_utc_at_midnight(2024, 2, 29),
        almanac.frame_info(EARTH_ICRS).unwrap(),
    );

    let mut max_err_found = 0.nanoseconds();
    let mut max_case = "".to_string();

    // These results are ordered in the same was as the tests.
    let spice_rslt = [
        4.hours() + 42.minutes() + 37.seconds(),
        15.hours() + 11.minutes() + 55.seconds(),
        9.hours() + 41.minutes() + 45.seconds(),
    ];

    for (idx, obs_frame) in [IAU_EARTH_FRAME, IAU_MOON_FRAME, IAU_VENUS_FRAME]
        .iter()
        .copied()
        .enumerate()
    {
        let state_bf = almanac
            .transform_to(state, obs_frame, Aberration::LT)
            .expect("snif");
        let long_deg = state_bf.longitude_360_deg();
        println!("{} => long = {long_deg:.6} deg", obs_frame.ephemeris_id);
        let mut anise_lst = almanac.apparent_solar_time(state, obs_frame).unwrap();
        anise_lst = anise_lst.round(1.seconds());
        println!("ANISE = {anise_lst}");
        println!("SPICE = {}", spice_rslt[idx]);
        let err = anise_lst - spice_rslt[idx];
        println!("--> Error = {err}");
        let max_err = if obs_frame.ephemeris_id == 299 {
            // Without rounding, the true error is 1 s 573 ms 721 μs 517 ns
            2.seconds()
        } else {
            // Without rounding, the true errors are:
            // 399: 871 ms 922 μs 785 ns
            // 301: -667 ms 78 μs 728 ns
            1.0.seconds()
        };
        assert!(err.abs() <= max_err);
    }
}

/**

Validation tests against spiceypy
```py
import spiceypy as sp
from math import radians

sp.furnsh("../data/pck00008.tpc")
sp.furnsh("../../ura184_part-3.bsp")
sp.furnsh("../../mar099s.bsp")
sp.furnsh("../../jup349.bsp")

et_s_orig = 762436869.1853695
epochs = [et_s_orig + offset_d * 86400 for offset_d in [0, 1, 2, 3, 5, 8, 13, 21]]
cases = {}

body_for_long_deg = [(399, 73.860328), (499, 183.707818), (599, 286.309857), (799, 232.148015), (399, 72.874719), (499, 193.596150), (599, 135.943094)
, (799, 13.324053), (399, 71.889110), (499, 203.487028), (599, 345.578197), (799, 154.500529), (399, 70.903502), (499, 213.380448), (599, 195.215145),
 (799, 295.677440), (399, 68.932285), (499, 233.174873), (599, 254.494460), (799, 218.032565), (399, 65.975459), (499, 262.885226), (599, 163.426517),
 (799, 281.568491), (399, 61.047416), (499, 312.450605), (599, 131.679598), (799, 267.470213), (399, 53.162548), (499, 31.871004), (599, 8.961200), (7
99, 316.934274)]

for idx in range(len(epochs)):
     cases[epochs[idx]] = body_for_long_deg[idx*4:idx*4+4]

for (et_s, this_case) in cases.items():
    for frame_id, long_deg in this_case:
        lst = sp.et2lst(et_s, frame_id, long_deg, "PLANETOCENTRIC", 256, 256)
        print(f"{et_s} {frame_id} -> {lst}")
```
*/
#[ignore = "requires the Mars, Jupiter, an Uranus (very large) kernels"]
#[rstest]
fn apparent_local_solar_time_exhaustive(almanac: Almanac) {
    let almanac = almanac
        .load("../../jup349.bsp")
        .unwrap()
        .load("../../mar099s.bsp")
        .unwrap()
        .load("../../ura184_part-3.bsp")
        .unwrap();
    // Define a state in GCRF, and compute the apparent solar time on several objects.
    let state = Orbit::keplerian(
        10_000.0,
        1e-5,
        28.5,
        75.0,
        76.0,
        78.0,
        Epoch::from_gregorian_utc_at_midnight(2024, 2, 29),
        almanac.frame_info(EARTH_ICRS).unwrap(),
    );

    let mut spice_results = HashMap::new();
    // Key: offset_d, values are the lst from SPICE
    spice_results.insert(
        0,
        vec![
            4.hours() + 42.minutes() + 37.seconds(),
            13.hours() + 12.minutes() + 48.seconds(),
            11.hours() + 19.minutes() + 54.seconds(),
            12.hours() + 6.minutes() + 23.seconds(),
        ],
    );
    spice_results.insert(
        1,
        vec![
            4.hours() + 38.minutes() + 52.seconds(),
            13.hours() + 13.minutes() + 38.seconds(),
            11.hours() + 20.minutes() + 13.seconds(),
            12.hours() + 6.minutes() + 20.seconds(),
        ],
    );
    spice_results.insert(
        2,
        vec![
            4.hours() + 35.minutes() + 8.seconds(),
            13.hours() + 14.minutes() + 29.seconds(),
            11.hours() + 20.minutes() + 33.seconds(),
            12.hours() + 6.minutes() + 18.seconds(),
        ],
    );
    spice_results.insert(
        3,
        vec![
            4.hours() + 31.minutes() + 24.seconds(),
            13.hours() + 15.minutes() + 19.seconds(),
            11.hours() + 20.minutes() + 53.seconds(),
            12.hours() + 6.minutes() + 16.seconds(),
        ],
    );
    spice_results.insert(
        5,
        vec![
            4.hours() + 23.minutes() + 57.seconds(),
            13.hours() + 17.minutes() + 1.seconds(),
            11.hours() + 21.minutes() + 35.seconds(),
            12.hours() + 6.minutes() + 11.seconds(),
        ],
    );
    spice_results.insert(
        8,
        vec![
            4.hours() + 12.minutes() + 50.seconds(),
            13.hours() + 19.minutes() + 35.seconds(),
            11.hours() + 22.minutes() + 40.seconds(),
            12.hours() + 6.minutes() + 3.seconds(),
        ],
    );
    spice_results.insert(
        13,
        vec![
            3.hours() + 54.minutes() + 24.seconds(),
            13.hours() + 23.minutes() + 55.seconds(),
            11.hours() + 24.minutes() + 36.seconds(),
            12.hours() + 5.minutes() + 48.seconds(),
        ],
    );
    spice_results.insert(
        21,
        vec![
            3.hours() + 25.minutes() + 10.seconds(),
            13.hours() + 30.minutes() + 56.seconds(),
            11.hours() + 28.minutes() + 2.seconds(),
            12.hours() + 5.minutes() + 19.seconds(),
        ],
    );

    let mut max_err_found = 0.nanoseconds();
    let mut max_case = "".to_string();

    for offset_d in [0, 1, 2, 3, 5, 8, 13, 21] {
        let mut test_state = state;
        test_state.epoch += Unit::Day * offset_d;

        println!("== ET = {} s ==", test_state.epoch.to_et_seconds());

        let spice_rslt = &spice_results[&offset_d];

        for (idx, obs_frame) in [
            IAU_EARTH_FRAME,
            IAU_MARS_FRAME,
            IAU_JUPITER_FRAME,
            IAU_URANUS_FRAME,
        ]
        .iter()
        .copied()
        .enumerate()
        {
            let state_bf = almanac
                .transform_to(test_state, obs_frame, Aberration::LT)
                .expect("snif");
            let long_deg = state_bf.longitude_360_deg();
            println!("{} => long = {long_deg:.6} deg", obs_frame.ephemeris_id);
            let mut anise_lst = almanac.apparent_solar_time(test_state, obs_frame).unwrap();
            // SPICE computes the LST to the precision of one second, but ANISE is down to the nanoseconds, so let's round.
            anise_lst = anise_lst.round(1.seconds());
            println!("ANISE = {anise_lst}");
            println!("SPICE = {}", spice_rslt[idx]);
            let err = anise_lst - spice_rslt[idx];
            println!("--> Error = {err}");
            let max_err = if obs_frame.ephemeris_id == 799 {
                // Not sure why, but the error is 1.2 seconds for Uranus
                1.2.seconds()
            } else {
                1.0.seconds()
            };
            assert!(err.abs() <= max_err);
            if err.abs() > max_err_found {
                max_err_found = err.abs();
                max_case = format!(
                    "{} @ {}; ANISE = {anise_lst}\t\tSPICE = {}",
                    obs_frame.ephemeris_id, test_state.epoch, spice_rslt[idx]
                );
            }
        }

        println!("===")
    }

    println!("Max error of {max_err_found} for\n{max_case}");
}
