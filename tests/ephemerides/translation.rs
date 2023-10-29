/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::f64::EPSILON;

use anise::constants::frames::{EARTH_J2000, EARTH_MOON_BARYCENTER_J2000, LUNA_J2000, VENUS_J2000};
use anise::file2heap;
use anise::math::Vector3;
use anise::prelude::*;

// Corresponds to an error of 2e-5 meters, or 2e-2 millimeters, or 20 micrometers
const POSITION_EPSILON_KM: f64 = 2e-8;
// Corresponds to an error of 5e-6 meters per second, or 5.0 micrometers per second
const VELOCITY_EPSILON_KM_S: f64 = 5e-9;

#[test]
fn de440s_translation_verif_venus2emb() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de440s.bsp";
    let buf = file2heap!(path).unwrap();
    let spk = SPK::parse(buf).unwrap();
    let ctx = Almanac::from_spk(spk).unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de440s.bsp')
    >>> sp.furnsh('../../hifitime/naif0012.txt')
    >>> et = sp.utc2et('2002 FEB 07 00:00:00')
    >>> ['{:.16e}'.format(x) for x in sp.spkez(2, et, "J2000", "NONE", 3)[0]]
    ['2.0504464297378346e+08',
    '-1.3595802364930704e+08',
    '-6.5722791478621781e+07',
    '3.7012086125533884e+01',
    '4.8685441394651654e+01',
    '2.0519128282958704e+01']
    */

    let state = ctx
        .translate_from_to(
            VENUS_J2000,
            EARTH_MOON_BARYCENTER_J2000,
            epoch,
            Aberration::None,
        )
        .unwrap();

    let pos_expct_km = Vector3::new(
        2.050_446_429_737_834_6e8,
        -1.359_580_236_493_070_4e8,
        -6.572_279_147_862_178e7,
    );

    let vel_expct_km_s = Vector3::new(
        3.701_208_612_553_388_4e1,
        4.868_544_139_465_165_4e1,
        2.051_912_828_295_870_4e1,
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
        .translate_from_to_geometric(EARTH_MOON_BARYCENTER_J2000, VENUS_J2000, epoch)
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
    let path = "./data/de440s.bsp";
    let buf = file2heap!(path).unwrap();
    let spk = SPK::parse(buf).unwrap();
    let ctx = Almanac::from_spk(spk).unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    // Venus to Earth Moon

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de440s.bsp')
    >>> sp.furnsh('../../hifitime/naif0012.txt')
    >>> et = sp.utc2et('2002 FEB 07 00:00:00')
    >>> ['{:.16e}'.format(x) for x in sp.spkez(2, et, "J2000", "NONE", 301)[0]]
    ['2.0512621956428146e+08',
    '-1.3561254796010864e+08',
    '-6.5578399619259715e+07',
    '3.6051374280511325e+01',
    '4.8889024619544145e+01',
    '2.0702933797799531e+01']

    */

    let state = ctx
        .translate_from_to(VENUS_J2000, LUNA_J2000, epoch, Aberration::None)
        .unwrap();

    let pos_expct_km = Vector3::new(
        2.051_262_195_642_814_6e8,
        -1.356_125_479_601_086_4e8,
        -6.557_839_961_925_971_5e7,
    );

    let vel_expct_km_s = Vector3::new(
        3.605_137_428_051_132_5e1,
        4.888_902_461_954_414_5e1,
        2.070_293_379_779_953e1,
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
        .translate_from_to_geometric(LUNA_J2000, VENUS_J2000, epoch)
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
    let path = "./data/de440s.bsp";
    let buf = file2heap!(path).unwrap();
    let spk = SPK::parse(buf).unwrap();
    let ctx = Almanac::from_spk(spk).unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    // Earth Moon Barycenter to Earth Moon

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de440s.bsp')
    >>> sp.furnsh('../../hifitime/naif0012.txt')
    >>> et = sp.utc2et('2002 FEB 07 00:00:00')
    >>> ['{:.16e}'.format(x) for x in sp.spkez(3, et, "J2000", "NONE", 301)[0]] # Target = 3; Obs = 301
    ['8.1576590498004080e+04',
    '3.4547568919842143e+05',
    '1.4439185936206434e+05',
    '-9.6071184502255447e-01',
    '2.0358322489248903e-01',
    '1.8380551484083130e-01']
    */

    let state = ctx
        .translate_from_to(
            EARTH_MOON_BARYCENTER_J2000,
            LUNA_J2000,
            epoch,
            Aberration::None,
        )
        .unwrap();

    // Check that we correctly set the output frame
    assert_eq!(state.frame, LUNA_J2000);

    let pos_expct_km = Vector3::new(
        8.157_659_049_800_408e4,
        3.454_756_891_984_214_3e5,
        1.443_918_593_620_643_4e5,
    );

    let vel_expct_km_s = Vector3::new(
        -9.607_118_450_225_545e-1,
        2.035_832_248_924_890_3e-1,
        1.838_055_148_408_313e-1,
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
fn spk_hermite_type13_verif() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de440s.bsp";
    let buf = file2heap!(path).unwrap();
    let spk = SPK::parse(buf).unwrap();

    let buf = file2heap!("data/gmat-hermite.bsp").unwrap();
    let spacecraft = SPK::parse(buf).unwrap();

    let ctx = Almanac::from_spk(spk)
        .unwrap()
        .with_spk(spacecraft)
        .unwrap();

    let epoch = Epoch::from_gregorian_hms(2000, 1, 1, 14, 0, 0, TimeScale::UTC);

    let my_sc_j2k = Frame::from_ephem_j2000(-10000001);

    let state = ctx
        .translate_from_to_geometric(my_sc_j2k, EARTH_J2000, epoch)
        .unwrap();
    println!("{state:?}");

    // Check that we correctly set the output frame
    assert_eq!(state.frame, EARTH_J2000);

    let pos_expct_km = Vector3::new(
        2.592_009_077_500_681e3,
        6.746_927_386_252_019e3,
        1.383_255_342_128_272_3e3,
    );

    let vel_expct_km_s = Vector3::new(
        -6.668_845_721_035_875,
        2.774_347_087_031_804_5,
        -8.583_249_702_745_147e-1,
    );

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

#[test]
fn multithread_query() {
    use core::str::FromStr;
    use rayon::prelude::*;
    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de440s.bsp";
    let buf = file2heap!(path).unwrap();
    let spk = SPK::parse(buf).unwrap();
    let ctx = Almanac::from_spk(spk).unwrap();

    let start_epoch = Epoch::from_str("2000-01-01T00:00:00 ET").unwrap();

    let end_epoch = start_epoch + 105.days();

    let time_it = TimeSeries::exclusive(start_epoch, end_epoch, 2.hours());

    let start = Epoch::now().unwrap();

    let epochs: Vec<Epoch> = time_it.collect();
    epochs.into_par_iter().for_each(|epoch| {
        let state = ctx
            .translate_from_to_geometric(LUNA_J2000, EARTH_MOON_BARYCENTER_J2000, epoch)
            .unwrap();
        println!("{state:?}");
    });

    let delta_t = Epoch::now().unwrap() - start;
    println!("Took {delta_t}");
}

#[test]
fn hermite_query() {
    use anise::naif::kpl::parser::convert_tpc;

    let traj = SPK::load("./data/gmat-hermite.bsp").unwrap();
    let summary = traj.data_summaries().unwrap()[0];
    println!("{}", summary);

    let mut ctx = Almanac::from_spk(traj).unwrap();
    // Also load the plantery data
    ctx.planetary_data = convert_tpc("data/pck00008.tpc", "data/gm_de431.tpc").unwrap();

    let summary_from_ctx = ctx.spk_summary_from_name("SPK_SEGMENT").unwrap().0;

    // The UIDs of the frames match.
    assert_eq!(
        summary.center_frame_uid(),
        summary_from_ctx.center_frame_uid()
    );

    // And the summaries match
    assert_eq!(&summary, summary_from_ctx);

    let summary_duration = summary.end_epoch() - summary.start_epoch();

    // Query in the middle to the parent, since we don't have anything else loaded.
    let state = ctx
        .translate_from_to(
            summary.target_frame(),
            summary.center_frame(),
            summary.start_epoch() + summary_duration * 0.5,
            Aberration::None,
        )
        .unwrap();

    // This tests that we've loaded the frame info from the Almanac, otherwise we cannot compute the orbital elements.
    assert_eq!(format!("{state:x}"), "[Earth J2000] 2000-01-01T13:39:27.999998123 UTC\tsma = 7192.041350 km\tecc = 0.024628\tinc = 12.851841 deg\traan = 306.170038 deg\taop = 315.085528 deg\tta = 96.135384 deg");

    // Fetch the state at the start of this spline to make sure we don't glitch.
    assert!(ctx
        .translate_from_to(
            summary.target_frame(),
            summary.center_frame(),
            summary.start_epoch(),
            Aberration::None,
        )
        .is_ok());

    // The very last state may fail because of a rounding difference in hifitime when going to/from TDB
    // For example, in this exact case, the end_epoch is shown to be 12032.183931521 seconds but the epoch
    // data in the BSP is 12032.1839315118(27), or 30 ns. This fix is in progress in hifitime v4.
    // assert!(ctx
    //     .translate_from_to(
    //         summary.target_frame(),
    //         to_frame,
    //         summary.end_epoch(),
    //         Aberration::None,
    //     )
    //     .is_ok());
}
