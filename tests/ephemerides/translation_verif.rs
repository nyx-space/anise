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
use anise::file_mmap;
use anise::math::Vector3;
use anise::prelude::*;

// For the Earth Moon Barycenter to Luna, there velocity error is up to 3e-14 km/s, or 3e-11 m/s, or 13 picometers per second.
const VELOCITY_EPSILON_KM_S: f64 = 1e-13;

#[test]
fn de438s_translation_verif_venus2emb() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.anise";
    let buf = file_mmap!(path).unwrap();
    let ctx: AniseContext = (&buf).try_into().unwrap();

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
        .common_ephemeris_path(VENUS_J2000, EARTH_MOON_BARYCENTER_J2000)
        .unwrap());

    let (pos, vel, _) = ctx
        .translate_from_to(
            VENUS_J2000,
            EARTH_MOON_BARYCENTER_J2000,
            epoch,
            Aberration::None,
            DistanceUnit::Kilometer,
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
        relative_eq!(pos, pos_expct_km, epsilon = EPSILON),
        "pos = {pos}\nexp = {pos_expct_km}\nerr = {:e}",
        pos_expct_km - pos
    );

    assert!(
        relative_eq!(vel, vel_expct_km_s, epsilon = EPSILON),
        "vel = {vel}\nexp = {vel_expct_km_s}\nerr = {:e}",
        vel_expct_km_s - vel
    );

    // Test the opposite translation
    let (pos, vel, _) = ctx
        .translate_from_to_km_s_geometric(EARTH_MOON_BARYCENTER_J2000, VENUS_J2000, epoch)
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(pos, -pos_expct_km, epsilon = EPSILON),
        "pos = {pos}\nexp = {pos_expct_km}\nerr = {:e}",
        pos_expct_km + pos
    );

    assert!(
        relative_eq!(vel, -vel_expct_km_s, epsilon = EPSILON),
        "vel = {vel}\nexp = {vel_expct_km_s}\nerr = {:e}",
        vel_expct_km_s + vel
    );
}

#[test]
fn de438s_translation_verif_venus2luna() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.anise";
    let buf = file_mmap!(path).unwrap();
    let ctx: AniseContext = (&buf).try_into().unwrap();

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

    let (pos, vel, _) = ctx
        .translate_from_to(
            VENUS_J2000,
            LUNA_J2000,
            epoch,
            Aberration::None,
            DistanceUnit::Kilometer,
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
        relative_eq!(pos, pos_expct_km, epsilon = EPSILON),
        "pos = {pos}\nexp = {pos_expct_km}\nerr = {:e}",
        pos_expct_km - pos
    );

    assert!(
        relative_eq!(vel, vel_expct_km_s, epsilon = VELOCITY_EPSILON_KM_S),
        "vel = {vel}\nexp = {vel_expct_km_s}\nerr = {:e}",
        vel_expct_km_s - vel
    );

    // Test the opposite translation
    let (pos, vel, _) = ctx
        .translate_from_to_km_s_geometric(LUNA_J2000, VENUS_J2000, epoch)
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(pos, -pos_expct_km, epsilon = EPSILON),
        "pos = {pos}\nexp = {pos_expct_km}\nerr = {:e}",
        pos_expct_km + pos
    );

    assert!(
        relative_eq!(vel, -vel_expct_km_s, epsilon = VELOCITY_EPSILON_KM_S),
        "vel = {vel}\nexp = {vel_expct_km_s}\nerr = {:e}",
        vel_expct_km_s + vel
    );
}

#[test]
fn de438s_translation_verif_emb2luna() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.anise";
    let buf = file_mmap!(path).unwrap();
    let ctx: AniseContext = (&buf).try_into().unwrap();

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

    let (pos, vel, _) = ctx
        .translate_from_to(
            EARTH_MOON_BARYCENTER_J2000,
            LUNA_J2000,
            epoch,
            Aberration::None,
            DistanceUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

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
        relative_eq!(pos, pos_expct_km, epsilon = EPSILON),
        "pos = {pos}\nexp = {pos_expct_km}\nerr = {:e}",
        pos_expct_km - pos
    );

    assert!(
        relative_eq!(vel, vel_expct_km_s, epsilon = VELOCITY_EPSILON_KM_S),
        "vel = {vel}\nexp = {vel_expct_km_s}\nerr = {:e}",
        vel_expct_km_s - vel
    );

    // Try the opposite
    let (pos, vel, _) = ctx
        .translate_from_to(
            LUNA_J2000,
            EARTH_MOON_BARYCENTER_J2000,
            epoch,
            Aberration::None,
            DistanceUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(pos, -pos_expct_km, epsilon = EPSILON),
        "pos = {pos}\nexp = {pos_expct_km}\nerr = {:e}",
        pos_expct_km + pos
    );

    assert!(
        relative_eq!(vel, -vel_expct_km_s, epsilon = VELOCITY_EPSILON_KM_S),
        "vel = {vel}\nexp = {vel_expct_km_s}\nerr = {:e}",
        vel_expct_km_s + vel
    );
}

#[test]
#[ignore]
#[cfg(feature = "std")]
fn exhaustive_de438s_translation() {
    use anise::frame::Frame;
    use hifitime::{TimeSeries, TimeUnits};
    use log::info;
    use rstats::{Median, Stats};

    const FAIL_POS_KM: f64 = 1e4;
    const FAIL_VEL_KM_S: f64 = 1e1;

    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // This test will load the BSP both in rust-spice and ANISE and make sure that we get the same data all the time.
    use spice;
    spice::furnsh("data/de438s.bsp");

    let path = "./data/de438s.anise";
    let buf = file_mmap!(path).unwrap();
    let ctx = AniseContext::from_bytes(&buf);

    for (idx1, ephem1) in ctx.ephemeris_data.iter().enumerate() {
        let j2000_ephem1 = Frame::from_ephem_j2000(*ctx.ephemeris_lut.hashes.get(idx1).unwrap());

        for (idx2, ephem2) in ctx.ephemeris_data.iter().enumerate() {
            if ephem1 == ephem2 {
                continue;
            }

            let j2000_ephem2 =
                Frame::from_ephem_j2000(*ctx.ephemeris_lut.hashes.get(idx2).unwrap());

            // Query the ephemeris data for a bunch of different times.
            let start_epoch = if ephem1.start_epoch() < ephem2.start_epoch() {
                ephem2.start_epoch()
            } else {
                ephem1.start_epoch()
            };

            let end_epoch = if ephem1.end_epoch() < ephem2.end_epoch() {
                ephem1.end_epoch()
            } else {
                ephem2.end_epoch()
            };

            // Query at ten thousand items
            let time_step = ((end_epoch - start_epoch).to_seconds() / 1_000.0).seconds();

            let time_it = TimeSeries::exclusive(start_epoch, end_epoch - time_step, time_step);
            info!("Query {} -> {} with {time_it}", j2000_ephem1, j2000_ephem2);

            let mut pos_err = [
                Vec::<f64>::with_capacity(1_000),
                Vec::<f64>::with_capacity(1_000),
                Vec::<f64>::with_capacity(1_000),
            ];
            let mut vel_err = [
                Vec::<f64>::with_capacity(1_000),
                Vec::<f64>::with_capacity(1_000),
                Vec::<f64>::with_capacity(1_000),
            ];

            for epoch in time_it {
                match ctx.translate_from_to_km_s_geometric(j2000_ephem1, j2000_ephem2, epoch) {
                    Ok((pos, vel, _)) => {
                        // Perform the same query in SPICE
                        let (state, _) = spice::spkezr(
                            match ephem1.name {
                                "Luna" => "Moon",
                                _ => ephem1.name,
                            },
                            epoch.to_et_seconds(),
                            "J2000",
                            "NONE",
                            match ephem2.name {
                                "Luna" => "Moon",
                                _ => ephem2.name,
                            },
                        );

                        // Check component by component instead of rebuilding a Vector3 from the SPICE data
                        for i in 0..6 {
                            if i < 3 {
                                let err = (pos[i] - state[i]).abs();
                                pos_err[i].push(err);

                                assert!(
                                    relative_eq!(pos[i], state[i], epsilon = FAIL_POS_KM),
                                    "{epoch:E}\npos[{i}] = {}\nexp = {}\nerr = {:e}",
                                    pos[i],
                                    state[i],
                                    err
                                );
                            } else {
                                let err = (vel[i - 3] - state[i]).abs();
                                vel_err[i - 3].push(err);

                                assert!(
                                    relative_eq!(vel[i - 3], state[i], epsilon = FAIL_VEL_KM_S),
                                    "{epoch:E}vel[{i}] = {}\nexp = {}\nerr = {:e}",
                                    vel[i - 3],
                                    state[i],
                                    err
                                );
                            }
                        }
                    }
                    Err(e) => {
                        panic!("At epoch {epoch:E}: {e}");
                    }
                };
            }

            for i in 0..6 {
                let meanstd = if i < 3 {
                    pos_err[i].ameanstd().unwrap()
                } else {
                    vel_err[i - 3].ameanstd().unwrap()
                };

                let med = if i < 3 {
                    pos_err[i].medinfo()
                } else {
                    vel_err[i - 3].medinfo()
                };

                info!(
                    "Error on {}: mean = {:e}\tdev = {:e}\tlow q = {:e}\tmed = {:e}\tup q = {:e}",
                    match i {
                        0 => "X",
                        1 => "Y",
                        2 => "Z",
                        3 => "VX",
                        4 => "VY",
                        5 => "VZ",
                        _ => unreachable!(),
                    },
                    meanstd.centre,
                    meanstd.dispersion,
                    med.lq,
                    med.median,
                    med.uq
                );
            }
        }
    }
}
