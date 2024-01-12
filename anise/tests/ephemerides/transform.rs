/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::constants::frames::{EARTH_ITRF93, VENUS_J2000};
use anise::math::Vector3;
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
