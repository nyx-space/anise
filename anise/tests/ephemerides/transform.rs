/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::constants::frames::{EARTH_ITRF93, IAU_MOON_FRAME, LUNA_J2000, VENUS_J2000};
use anise::math::{Matrix3, Vector3};
use anise::prelude::*;

// Corresponds to an error of 2e-2 meters, or 20 millimeters
const POSITION_EPSILON_KM: f64 = 2e-5;
// Corresponds to an error of 5e-7 meters per second, or 0.5 micrometers per second
const VELOCITY_EPSILON_KM_S: f64 = 5e-10;

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

    // TODO https://github.com/nyx-space/anise/issues/130
    // Test the opposite translation
    // let state_rtn = almanac
    //     .transform_from_to(EARTH_ITRF93, VENUS_J2000, epoch, Aberration::None)
    //     .unwrap();

    // println!("state = {state}");
    // println!("state_rtn = {state_rtn}");

    // let (spice_state, _) = spice::spkezr("EARTH", epoch.to_et_seconds(), "ITRF93", "NONE", "VENUS");

    // let pos_rtn_expct_km = Vector3::new(spice_state[0], spice_state[1], spice_state[2]);
    // let vel_rtn_expct_km_s = Vector3::new(spice_state[3], spice_state[4], spice_state[5]);

    // dbg!(pos_expct_km + pos_rtn_expct_km);
    // dbg!(vel_expct_km_s + vel_rtn_expct_km_s);

    // dbg!(pos_expct_km + state_rtn.radius_km);
    // dbg!(pos_rtn_expct_km - state_rtn.radius_km);
    // dbg!(vel_expct_km_s + state_rtn.velocity_km_s);
    // dbg!(vel_rtn_expct_km_s - state_rtn.velocity_km_s);

    // assert!(
    //     relative_eq!(
    //         state_rtn.radius_km,
    //         -pos_expct_km,
    //         epsilon = POSITION_EPSILON_KM
    //     ),
    //     "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
    //     state_rtn.radius_km,
    //     pos_expct_km + state_rtn.radius_km
    // );

    // assert!(
    //     relative_eq!(
    //         state.velocity_km_s,
    //         -vel_expct_km_s,
    //         epsilon = VELOCITY_EPSILON_KM_S
    //     ),
    //     "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
    //     state.velocity_km_s,
    //     vel_expct_km_s + state.velocity_km_s
    // );

    // // Check that the return state is exactly opposite to the forward state
    // assert!(
    //     relative_eq!(state_rtn.radius_km, -state.radius_km, epsilon = EPSILON),
    //     "pos = {}\nexp = {}\nerr = {:e}",
    //     state_rtn.radius_km,
    //     -state.radius_km,
    //     state_rtn.radius_km - state.radius_km
    // );

    // assert!(
    //     relative_eq!(
    //         state_rtn.velocity_km_s,
    //         -state.velocity_km_s,
    //         epsilon = EPSILON
    //     ),
    //     "vel = {}\nexp = {}\nerr = {:e}",
    //     state.velocity_km_s,
    //     -state_rtn.velocity_km_s,
    //     state.velocity_km_s - state_rtn.velocity_km_s,
    // );

    // Unload spice
    spice::unload(bpc_path);
    spice::unload(spk_path);
}

#[test]
fn specific_test() {
    let _ = pretty_env_logger::try_init();

    let almanac = MetaAlmanac::default().process().unwrap();

    // [Luna J2000] 2024-09-22T08:45:22 UTC	position = [638.053603, -1776.813629, 195.147575] km	velocity = [-0.017910, -0.181449, -1.584180] km/s

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
        LUNA_J2000,
    );

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
        LUNA_J2000,
    );

    // Expected rotation matrix
    let rot_data = [
        [
            -6.12817863e-01,
            -7.32495428e-01,
            -2.96487292e-01,
            0.00000000e+00,
            0.00000000e+00,
            0.00000000e+00,
        ],
        [
            7.90219432e-01,
            -5.69347499e-01,
            -2.26708345e-01,
            0.00000000e+00,
            0.00000000e+00,
            0.00000000e+00,
        ],
        [
            -2.74147191e-03,
            -3.73220943e-01,
            9.27738439e-01,
            0.00000000e+00,
            0.00000000e+00,
            0.00000000e+00,
        ],
        [
            2.10320098e-06,
            -1.51584874e-06,
            -6.02140019e-07,
            -6.12817863e-01,
            -7.32495428e-01,
            -2.96487292e-01,
        ],
        [
            1.63104266e-06,
            1.94960401e-06,
            7.89028888e-07,
            7.90219432e-01,
            -5.69347499e-01,
            -2.26708345e-01,
        ],
        [
            9.01406584e-10,
            9.38047187e-10,
            3.80031722e-10,
            -2.74147191e-03,
            -3.73220943e-01,
            9.27738439e-01,
        ],
    ];
    let rot_mat = Matrix3::new(
        rot_data[0][0],
        rot_data[0][1],
        rot_data[0][2],
        rot_data[1][0],
        rot_data[1][1],
        rot_data[1][2],
        rot_data[2][0],
        rot_data[2][1],
        rot_data[2][2],
    );

    // Check that's the rotation matrix we get in ANISE.
    // let dcm = almanac
    //     .rotate_from_to(LUNA_J2000, IAU_MOON_FRAME, epoch)
    //     .unwrap();
    let dcm = almanac
        .rotate_from_to(LUNA_J2000, IAU_MOON_FRAME, epoch)
        .unwrap();

    println!("{dcm}");
    println!("{rot_mat}");
}
