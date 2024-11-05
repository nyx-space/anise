/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::astro::{AzElRange, Occultation};
use anise::constants::celestial_objects::{EARTH, VENUS};
use anise::constants::frames::{
    EARTH_ITRF93, EARTH_J2000, IAU_EARTH_FRAME, IAU_MOON_FRAME, MOON_J2000, SUN_J2000, VENUS_J2000,
};
use anise::constants::orientations::ITRF93;
use anise::constants::usual_planetary_constants::MEAN_EARTH_ANGULAR_VELOCITY_DEG_S;
use anise::math::Vector3;
use anise::prelude::*;

// Corresponds to an error of 2e-2 meters, or 20 millimeters
const POSITION_EPSILON_KM: f64 = 2e-5;
// Corresponds to an error of 5e-7 meters per second, or 0.5 micrometers per second
const VELOCITY_EPSILON_KM_S: f64 = 5e-10;

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

#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn de440s_transform_verif_venus2emb() {
    let _ = pretty_env_logger::try_init();

    let spk_path = "../data/de440s.bsp";
    let bpc_path = "../data/earth_latest_high_prec.bpc";

    // Load into ANISE
    let spk = SPK::load(spk_path).unwrap();
    let bpc = BPC::load(bpc_path).unwrap();

    // Load into SPICE
    spice::furnsh(spk_path);
    spice::furnsh(bpc_path);

    let almanac = Almanac::default()
        .with_spk(spk)
        .unwrap()
        .with_bpc(bpc)
        .unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2020, 2, 7);

    let state = almanac
        .transform(VENUS_J2000, EARTH_ITRF93, epoch, Aberration::NONE)
        .unwrap();

    let (spice_state, _) = spice::spkezr("VENUS", epoch.to_et_seconds(), "ITRF93", "NONE", "EARTH");

    let pos_expct_km = Vector3::new(spice_state[0], spice_state[1], spice_state[2]);
    let vel_expct_km_s = Vector3::new(spice_state[3], spice_state[4], spice_state[5]);

    dbg!(pos_expct_km - state.radius_km);
    dbg!(vel_expct_km_s - state.velocity_km_s);

    assert!(
        relative_eq!(state.radius_km, pos_expct_km, epsilon = POSITION_EPSILON_KM),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km - state.radius_km
    );

    assert!(
        relative_eq!(
            state.velocity_km_s,
            vel_expct_km_s,
            epsilon = VELOCITY_EPSILON_KM_S
        ),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s - state.velocity_km_s
    );

    let state_rtn = almanac
        .transform(EARTH_J2000, Frame::new(VENUS, ITRF93), epoch, None)
        .unwrap();

    println!("state = {state}");
    println!("state_rtn = {state_rtn}");

    let (spice_state, _) = spice::spkezr("EARTH", epoch.to_et_seconds(), "ITRF93", "NONE", "VENUS");

    let pos_rtn_expct_km = Vector3::new(spice_state[0], spice_state[1], spice_state[2]);
    let vel_rtn_expct_km_s = Vector3::new(spice_state[3], spice_state[4], spice_state[5]);

    dbg!(pos_expct_km + pos_rtn_expct_km);
    dbg!(vel_expct_km_s + vel_rtn_expct_km_s);

    dbg!(pos_expct_km + state_rtn.radius_km);
    dbg!(pos_rtn_expct_km - state_rtn.radius_km);
    dbg!(vel_expct_km_s + state_rtn.velocity_km_s);
    dbg!(vel_rtn_expct_km_s - state_rtn.velocity_km_s);

    assert!(
        relative_eq!(
            state_rtn.radius_km,
            pos_rtn_expct_km,
            epsilon = POSITION_EPSILON_KM
        ),
        "pos = {}\nexp = {pos_rtn_expct_km}\nerr = {:e}",
        state_rtn.radius_km,
        pos_rtn_expct_km - state_rtn.radius_km
    );

    assert!(
        relative_eq!(
            state_rtn.velocity_km_s,
            vel_rtn_expct_km_s,
            epsilon = VELOCITY_EPSILON_KM_S
        ),
        "vel = {}\nexp = {vel_rtn_expct_km_s}\nerr = {:e}",
        state_rtn.velocity_km_s,
        vel_rtn_expct_km_s - state.velocity_km_s
    );

    // Check that the return state is exactly opposite to the forward state
    assert!(
        relative_eq!(
            state_rtn.radius_km,
            -state.radius_km,
            epsilon = core::f64::EPSILON
        ),
        "pos = {}\nexp = {}\nerr = {:e}",
        state_rtn.radius_km,
        -state.radius_km,
        state_rtn.radius_km - state.radius_km
    );

    assert!(
        relative_eq!(
            state_rtn.velocity_km_s,
            -state.velocity_km_s,
            epsilon = core::f64::EPSILON
        ),
        "vel = {}\nexp = {}\nerr = {:e}",
        state.velocity_km_s,
        -state_rtn.velocity_km_s,
        state.velocity_km_s - state_rtn.velocity_km_s,
    );

    // Unload spice
    spice::unload(bpc_path);
    spice::unload(spk_path);

    // Finally, check that ANISE's SPKEZR works as expected.
    let state_ezr = almanac.spk_ezr(EARTH, epoch, ITRF93, VENUS, None).unwrap();
    assert_eq!(state_ezr, state_rtn);
}

#[rstest]
fn spice_verif_iau_moon(almanac: Almanac) {
    let _ = pretty_env_logger::try_init();

    let epoch = Epoch::from_str("2024-09-22T08:45:22 UTC").unwrap();
    // This state is identical in ANISE and SPICE, queried from a BSP.
    let orbit_moon_j2k = Orbit::new(
        638.053603,
        -1776.813629,
        195.147575,
        -0.017910,
        -0.181449,
        -1.584180,
        epoch,
        MOON_J2000,
    );

    let anise_iau_moon = almanac
        .transform_to(orbit_moon_j2k, IAU_MOON_FRAME, None)
        .unwrap();

    // We know from the other tests that the Moon IAU rotation is the same in ANISE and SPICE.
    // However, when queried using the `transform` function in ANISE v0.2.1, there is a difference.
    let spice_iau_moon = Orbit::new(
        8.52638439e+02,
        1.47158517e+03,
        8.42440758e+02,
        6.17492780e-01,
        4.46032072e-01,
        -1.40193607e+00,
        epoch,
        IAU_MOON_FRAME,
    );

    println!("ANISE\n{anise_iau_moon}\nSPICE\n{spice_iau_moon}");
    let rss_pos_km = anise_iau_moon.rss_radius_km(&spice_iau_moon).unwrap();
    let rss_vel_km_s = anise_iau_moon.rss_velocity_km_s(&spice_iau_moon).unwrap();

    dbg!(rss_pos_km, rss_vel_km_s);

    // ANISE uses hifitime which is more precise than SPICE at time computations.
    // The Moon angular acceleration is expressed in centuries sicne J2000, where Hifitime does not suffer from rounding errors.
    assert!(rss_pos_km < 0.004);
    assert!(rss_vel_km_s < 1e-5);
}

#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[rstest]
fn validate_gh_283_multi_barycenter_and_los(almanac: Almanac) {
    let spk_path = "../data/lro.bsp";
    let almanac = almanac.load(spk_path).unwrap();

    const LRO_ID: i32 = -85;
    let lro_frame = Frame::from_ephem_j2000(LRO_ID);

    // Load into SPICE
    spice::furnsh(spk_path);
    spice::furnsh("../data/de440s.bsp");
    spice::furnsh("../data/pck00008.tpc");

    // Regression test for GH #346 where the ephemeris time converted to UTC is a handful of
    // nanoseconds past the midnight so the DAF query would normally fail.
    let gh346_epoch = Epoch::from_gregorian_utc_at_midnight(2023, 12, 15);
    assert!(almanac
        .common_ephemeris_path(lro_frame, SUN_J2000, gh346_epoch)
        .is_ok());
    assert!(almanac
        .transform(lro_frame, SUN_J2000, gh346_epoch, None)
        .is_ok());

    let epoch = Epoch::from_gregorian_utc_at_midnight(2024, 1, 1);

    // First, let's test that the common ephemeris path is correct
    let (node_count, path, common_node) = almanac
        .common_ephemeris_path(lro_frame, SUN_J2000, epoch)
        .unwrap();

    assert_eq!(common_node, 0, "common node should be the SSB");
    assert_eq!(node_count, 3, "node count should be Moon, EMB, SSB");
    assert_eq!(
        path,
        [Some(301), Some(3), Some(0), None, None, None, None, None,],
        "node count should be Moon, EMB, SSB"
    );

    let (spice_lro_state_raw, _) = spice::spkezr(
        &LRO_ID.to_string(),
        epoch.to_et_seconds(),
        "J2000",
        "NONE",
        "SUN",
    );

    let spice_lro_state = Orbit::new(
        spice_lro_state_raw[0],
        spice_lro_state_raw[1],
        spice_lro_state_raw[2],
        spice_lro_state_raw[3],
        spice_lro_state_raw[4],
        spice_lro_state_raw[5],
        epoch,
        SUN_J2000,
    );

    let anise_lro_state = almanac
        .transform(lro_frame, SUN_J2000, epoch, None)
        .unwrap();

    println!("== VALIDATION==\nANISE\n{anise_lro_state}\nSPICE\n{spice_lro_state}");
    let rss_pos_km = anise_lro_state.rss_radius_km(&spice_lro_state).unwrap();
    let rss_vel_km_s = anise_lro_state.rss_velocity_km_s(&spice_lro_state).unwrap();

    dbg!(rss_pos_km, rss_vel_km_s);

    assert!(rss_pos_km < 5e-7);
    assert!(rss_vel_km_s < 1e-12);

    // Compute the line of sight via the AER computation throughout a full orbit.

    // Grab the orbital period in the Moon frame
    let lro_state = almanac
        .transform(lro_frame, MOON_J2000, epoch, None)
        .unwrap();

    // Build the Madrid DSN gound station
    let latitude_deg = 40.427_222;
    let longitude_deg = 4.250_556;
    let height_km = 0.834_939;
    let iau_earth = almanac.frame_from_uid(IAU_EARTH_FRAME).unwrap();

    let mut obstructions = 0;
    let mut no_obstructions = 0;
    let mut printed_umbra = false;
    let mut printed_visible = false;

    let period = lro_state.period().unwrap();
    println!("LRO period is {period}");

    // SPICE call definitions
    let front = "MOON";
    let fshape = "ELLIPSOID";
    let fframe = "IAU_MOON";
    let back = "SUN";
    let bshape = "ELLIPSOID";
    let bframe = "IAU_SUN";
    let abcorr = "NONE";
    let obsrvr = "-85";

    let mut prev_occult: Option<Occultation> = None;
    let mut prev_aer: Option<AzElRange> = None;
    let mut access_count = 0;
    let mut access_start: Option<Epoch> = None;

    let access_times = [
        Unit::Minute * 4 + Unit::Second * 1,
        Unit::Hour * 1 + Unit::Minute * 6 + 49 * Unit::Second,
    ];

    for epoch in TimeSeries::inclusive(epoch, epoch + period, 1.seconds()) {
        // Rebuild the ground station at this new epoch
        let tx_madrid = Orbit::try_latlongalt(
            latitude_deg,
            longitude_deg,
            height_km,
            MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
            epoch,
            iau_earth,
        )
        .unwrap();

        let rx_lro = almanac
            .transform(lro_frame, MOON_J2000, epoch, None)
            .unwrap();

        let aer = almanac
            .azimuth_elevation_range_sez(rx_lro, tx_madrid, Some(IAU_MOON_FRAME), None)
            .unwrap();

        if let Some(prev_aer) = prev_aer {
            if prev_aer.is_obstructed() && !aer.is_obstructed() {
                // New access
                access_count += 1;
                access_start = Some(aer.epoch);
            } else if !prev_aer.is_obstructed() && aer.is_obstructed() {
                // End of access
                if let Some(access_start) = access_start {
                    // We've had a full access strand.
                    let access_end = aer.epoch;
                    let access_duration = (access_end - access_start).round(Unit::Second * 1);
                    println!(
                        "#{access_count}\t{access_start} - {access_end}\t{}",
                        access_duration
                    );
                    assert_eq!(access_times[access_count], access_duration);
                }
            }
            // dbg!(prev_aer.is_obstructed(), aer.is_obstructed());
        } else if !aer.is_obstructed() {
            access_start = Some(aer.epoch);
        }
        prev_aer = Some(aer);

        if aer.obstructed_by.is_some() {
            obstructions += 1;
        } else {
            no_obstructions += 1;
        }

        // Make sure to print the LRO in the Moon TOD frame.
        // This also checks that the rotation done in the occultation function is performed correctly.
        let rx_lro = almanac
            .transform(lro_frame, IAU_MOON_FRAME, epoch, None)
            .unwrap();

        // Compute the solar eclipsing
        let occult = almanac
            .solar_eclipsing(IAU_MOON_FRAME, rx_lro, None)
            .unwrap();

        let spice_occult = spice::occult(
            front,
            fshape,
            &fframe,
            &back,
            &bshape,
            &bframe,
            &abcorr,
            &obsrvr,
            epoch.to_et_seconds(),
        );

        match spice_occult {
            0 => assert!(occult.is_visible(), "SPICE={spice_occult} BUT {occult}"),
            1 => assert!(occult.is_partial(), "SPICE={spice_occult} BUT {occult}"),
            3 => assert!(occult.is_obstructed(), "SPICE={spice_occult} BUT {occult}"),
            _ => unreachable!(),
        }

        if occult.is_visible() {
            if !printed_visible {
                println!("{occult} @ {rx_lro:x}");
                printed_visible = true;
            }
            if let Some(prev_occult) = &prev_occult {
                if prev_occult.is_partial() {
                    assert_eq!(
                        epoch,
                        Epoch::from_gregorian_utc_hms(2024, 1, 1, 0, 46, 36),
                        "wrong post-penumbra state"
                    );
                }
            }
        } else if occult.is_obstructed() {
            if !printed_umbra {
                println!("{occult} @ {rx_lro:x}");
                assert_eq!(
                    epoch,
                    Epoch::from_gregorian_utc_hms(2024, 1, 1, 0, 1, 55),
                    "wrong first umbra state"
                );
                printed_umbra = true;
            }
            let sun = almanac
                .transform(SUN_J2000, MOON_J2000, epoch, None)
                .unwrap();
            let obstructed = almanac
                .line_of_sight_obstructed(rx_lro, sun, MOON_J2000, None)
                .unwrap();
            assert!(obstructed, "{occult} but not obstructed!");
        } else {
            if prev_occult.as_ref().unwrap().is_visible() {
                assert_eq!(
                    epoch,
                    Epoch::from_gregorian_utc_hms(2024, 1, 1, 0, 1, 42),
                    "wrong first penumbra state"
                );
            }
            println!("{occult}");
        }
        prev_occult = Some(occult)
    }

    if let Some(access_start) = access_start {
        // We've had a full access strand.
        let access_end = epoch + period;
        let access_duration = (access_end - access_start).round(Unit::Second * 1);
        println!(
            "#{access_count}\t{access_start} - {access_end}\t{}",
            access_duration
        );
        assert_eq!(access_times[access_count], access_duration);
    }

    assert_eq!(obstructions, 2762);
    assert_eq!(no_obstructions, 4250);
}
