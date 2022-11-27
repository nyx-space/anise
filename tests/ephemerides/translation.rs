/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::f64::EPSILON;

use anise::constants::frames::{EARTH_MOON_BARYCENTER_J2000, LUNA_J2000, VENUS_J2000};
use anise::constants::orientations::J2000;
use anise::file_mmap;
use anise::math::Vector3;
use anise::prelude::*;

// Corresponds to an error of 1e-5 meters, or 1e-2 millimeters, or 10 micrometers
const POSITION_EPSILON_KM: f64 = 1e-8;
// Corresponds to an error of 1e-7 meters per second, or 1e-1 micrometers per second, or 100 nanometers per second
const VELOCITY_EPSILON_KM_S: f64 = 1e-10;

#[test]
fn de438s_translation_verif_venus2emb() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.bsp";
    let buf = file_mmap!(path).unwrap();
    let spk = SPK::parse(&buf).unwrap();
    let ctx = Context::from_spk(&spk).unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de438s.bsp')
    >>> sp.furnsh('../../hifitime/naif0012.txt')
    >>> et = sp.utc2et('2002 FEB 07 00:00:00')
    >>> ['{:.16e}'.format(x) for x in sp.spkez(2, et, "J2000", "NONE", 3)[0]]
    ['2.0504464298094124e+08', '-1.3595802361226091e+08', '-6.5722791535179183e+07', '3.7012086122583923e+01', '4.8685441396743641e+01', '2.0519128283382937e+01']
    */

    dbg!(ctx
        .common_ephemeris_path(VENUS_J2000, EARTH_MOON_BARYCENTER_J2000, epoch)
        .unwrap());

    let state = ctx
        .translate_from_to(
            VENUS_J2000,
            EARTH_MOON_BARYCENTER_J2000,
            epoch,
            Aberration::None,
            LengthUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    let pos_expct_km = Vector3::new(
        2.0504464298094124e+08,
        -1.3595802361226091e+08,
        -6.5722791535179183e+07,
    );

    let vel_expct_km_s = Vector3::new(
        3.7012086122583923e+01,
        4.8685441396743641e+01,
        2.0519128283382937e+01,
    );

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, pos_expct_km, epsilon = EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km - state.radius_km
    );

    assert!(
        relative_eq!(state.velocity_km_s, vel_expct_km_s, epsilon = EPSILON),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s - state.velocity_km_s
    );

    // Test the opposite translation
    let state = ctx
        .translate_from_to_km_s_geometric(EARTH_MOON_BARYCENTER_J2000, VENUS_J2000, epoch)
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, -pos_expct_km, epsilon = EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km + state.radius_km
    );

    assert!(
        relative_eq!(state.velocity_km_s, -vel_expct_km_s, epsilon = EPSILON),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s + state.velocity_km_s
    );
}

#[test]
fn de438s_translation_verif_venus2luna() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.bsp";
    let buf = file_mmap!(path).unwrap();
    let spk = SPK::parse(&buf).unwrap();
    let ctx = Context::from_spk(&spk).unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    // Venus to Earth Moon

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de438s.bsp')
    >>> sp.furnsh('../../hifitime/naif0012.txt')
    >>> et = sp.utc2et('2002 FEB 07 00:00:00')
    >>> ['{:.16e}'.format(x) for x in sp.spkez(2, et, "J2000", "NONE", 3)[0]]
    ['2.0512621957198492e+08', '-1.3561254792311624e+08', '-6.5578399676164642e+07', '3.6051374278187268e+01', '4.8889024622166957e+01', '2.0702933800840963e+01']
    >>> ['{:.16e}'.format(x) for x in sp.spkez(3, et, "J2000", "NONE", 301)[0]]
    ['8.1576591043659311e+04', '3.4547568914467981e+05', '1.4439185901453768e+05', '-9.6071184439665624e-01', '2.0358322542331578e-01', '1.8380551745802590e-01']
    */

    let state = ctx
        .translate_from_to(
            VENUS_J2000,
            LUNA_J2000,
            epoch,
            Aberration::None,
            LengthUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    let pos_expct_km = Vector3::new(
        2.0512621957198492e+08,
        -1.3561254792311624e+08,
        -6.5578399676164642e+07,
    );

    let vel_expct_km_s = Vector3::new(
        3.6051374278187268e+01,
        4.8889024622166957e+01,
        2.0702933800840963e+01,
    );

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, pos_expct_km, epsilon = EPSILON),
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

    // Test the opposite translation
    let state = ctx
        .translate_from_to_km_s_geometric(LUNA_J2000, VENUS_J2000, epoch)
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, -pos_expct_km, epsilon = EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km + state.radius_km
    );

    assert!(
        relative_eq!(
            state.velocity_km_s,
            -vel_expct_km_s,
            epsilon = VELOCITY_EPSILON_KM_S
        ),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s + state.velocity_km_s
    );
}

#[test]
fn de438s_translation_verif_emb2luna() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.bsp";
    let buf = file_mmap!(path).unwrap();
    let spk = SPK::parse(&buf).unwrap();
    let ctx = Context::from_spk(&spk).unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    // Earth Moon Barycenter to Earth Moon

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de438s.bsp')
    >>> sp.furnsh('../../hifitime/naif0012.txt')
    >>> et = sp.utc2et('2002 FEB 07 00:00:00')
    >>> ['{:.16e}'.format(x) for x in sp.spkez(3, et, "J2000", "NONE", 301)[0]] # Target = 3; Obs = 301
    ['8.1576591043659311e+04', '3.4547568914467981e+05', '1.4439185901453768e+05', '-9.6071184439665624e-01', '2.0358322542331578e-01', '1.8380551745802590e-01']
    */

    let state = ctx
        .translate_from_to(
            EARTH_MOON_BARYCENTER_J2000,
            LUNA_J2000,
            epoch,
            Aberration::None,
            LengthUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    // Check that we correctly set the output frame
    assert_eq!(state.frame, LUNA_J2000);

    let pos_expct_km = Vector3::new(
        8.1576591043659311e+04,
        3.4547568914467981e+05,
        1.4439185901453768e+05,
    );

    let vel_expct_km_s = Vector3::new(
        -9.6071184439665624e-01,
        2.0358322542331578e-01,
        1.8380551745802590e-01,
    );

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, pos_expct_km, epsilon = EPSILON),
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

    // Try the opposite
    let state = ctx
        .translate_from_to(
            LUNA_J2000,
            EARTH_MOON_BARYCENTER_J2000,
            epoch,
            Aberration::None,
            LengthUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, -pos_expct_km, epsilon = EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km + state.radius_km
    );

    assert!(
        relative_eq!(
            state.velocity_km_s,
            -vel_expct_km_s,
            epsilon = VELOCITY_EPSILON_KM_S
        ),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s + state.velocity_km_s
    );
}

#[test]
fn spk_hermite_type31_verif() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.bsp";
    let buf = file_mmap!(path).unwrap();
    let spk = SPK::parse(&buf).unwrap();

    let buf = file_mmap!("./target/Hermite.bsp").unwrap();
    let spacecraft = SPK::parse(&buf).unwrap();

    let ctx = Context::from_spk(&spk)
        .unwrap()
        .load_spk(&spacecraft)
        .unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2024, 7, 1);

    let my_sc_j2k = Frame::from_ephem_orient(-111229882, J2000);

    let state = ctx
        .translate_from_to_km_s_geometric(my_sc_j2k, LUNA_J2000, epoch)
        .unwrap();
    println!("{state:?}");

    // Check that we correctly set the output frame
    assert_eq!(state.frame, LUNA_J2000);

    let pos_expct_km = Vector3::new(
        8.9871033515359500e+02,
        -5.4154398990756522e+02,
        1.5101519596160608e+03,
    );

    let vel_expct_km_s = Vector3::new(
        -1.2836208430532707e+00,
        -9.0227334520037439e-01,
        4.4829903632467799e-01,
    );

    // We expect exactly the same output as SPICE to machine precision.
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
}
