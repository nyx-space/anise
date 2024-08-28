/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::constants::frames::{EARTH_J2000, EARTH_MOON_BARYCENTER_J2000, MOON_J2000, VENUS_J2000};
use anise::file2heap;
use anise::math::Vector3;
use anise::prelude::*;

// Corresponds to an error of 2e-5 meters, or 2e-2 millimeters, or 20 micrometers
const POSITION_EPSILON_KM: f64 = 2e-8;
// Corresponds to an error of 5e-6 meters per second, or 5.0 micrometers per second
const VELOCITY_EPSILON_KM_S: f64 = 5e-9;
// Light time velocity error is too large! Cf. https://github.com/nyx-space/anise/issues/157
const ABERRATION_VELOCITY_EPSILON_KM_S: f64 = 1e-4;

#[test]
fn de440s_translation_verif_venus2emb() {
    let _ = pretty_env_logger::try_init();

    // "Load" the file via a memory map (avoids allocations)
    let path = "../data/de440s.bsp";
    let spk = SPK::load(path).unwrap();
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
        .translate(
            VENUS_J2000,
            EARTH_MOON_BARYCENTER_J2000,
            epoch,
            Aberration::NONE,
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
        relative_eq!(state.radius_km, pos_expct_km, epsilon = f64::EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km - state.radius_km
    );

    assert!(
        relative_eq!(state.velocity_km_s, vel_expct_km_s, epsilon = f64::EPSILON),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s - state.velocity_km_s
    );

    // Test the opposite translation
    let state = ctx
        .translate_geometric(EARTH_MOON_BARYCENTER_J2000, VENUS_J2000, epoch)
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, -pos_expct_km, epsilon = f64::EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km + state.radius_km
    );

    assert!(
        relative_eq!(state.velocity_km_s, -vel_expct_km_s, epsilon = f64::EPSILON),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s + state.velocity_km_s
    );
}

#[test]
fn de438s_translation_verif_venus2moon() {
    let _ = pretty_env_logger::try_init();

    // "Load" the file via a memory map (avoids allocations)
    let path = "../data/de440s.bsp";
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
        .translate(VENUS_J2000, MOON_J2000, epoch, Aberration::NONE)
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
        relative_eq!(state.radius_km, pos_expct_km, epsilon = f64::EPSILON),
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
        .translate_geometric(MOON_J2000, VENUS_J2000, epoch)
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, -pos_expct_km, epsilon = f64::EPSILON),
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
fn de438s_translation_verif_emb2moon() {
    let _ = pretty_env_logger::try_init();

    // "Load" the file via a memory map (avoids allocations)
    let path = "../data/de440s.bsp";
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
        .translate(
            EARTH_MOON_BARYCENTER_J2000,
            MOON_J2000,
            epoch,
            Aberration::NONE,
        )
        .unwrap();

    // Check that we correctly set the output frame
    assert_eq!(state.frame, MOON_J2000);

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
        relative_eq!(state.radius_km, pos_expct_km, epsilon = f64::EPSILON),
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
        .translate(
            MOON_J2000,
            EARTH_MOON_BARYCENTER_J2000,
            epoch,
            Aberration::NONE,
        )
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, -pos_expct_km, epsilon = f64::EPSILON),
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
    let _ = pretty_env_logger::try_init().is_err();

    let ctx = Almanac::default()
        .load("../data/de440s.bsp")
        .and_then(|ctx| ctx.load("../data/gmat-hermite.bsp"))
        .unwrap();

    let epoch = Epoch::from_gregorian_hms(2000, 1, 1, 14, 0, 0, TimeScale::UTC);

    let my_sc_j2k = Frame::from_ephem_j2000(-10000001);

    let state = ctx
        .translate_geometric(my_sc_j2k, EARTH_J2000, epoch)
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
    let path = "../data/de440s.bsp";
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
            .translate_geometric(MOON_J2000, EARTH_MOON_BARYCENTER_J2000, epoch)
            .unwrap();
        println!("{state:?}");
    });

    let delta_t = Epoch::now().unwrap() - start;
    println!("Took {delta_t}");
}

#[test]
fn hermite_query() {
    use anise::naif::kpl::parser::convert_tpc;

    let traj = SPK::load("../data/gmat-hermite.bsp").unwrap();
    let summary = traj.data_summaries().unwrap()[0];
    println!("{}", summary);

    let mut ctx = Almanac::from_spk(traj).unwrap();
    // Also load the plantery data
    ctx.planetary_data = convert_tpc("../data/pck00008.tpc", "../data/gm_de431.tpc").unwrap();

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
        .translate(
            summary.target_frame(),
            summary.center_frame(),
            summary.start_epoch() + summary_duration * 0.5,
            Aberration::NONE,
        )
        .unwrap();

    // This tests that we've loaded the frame info from the Almanac, otherwise we cannot compute the orbital elements.
    assert_eq!(format!("{state:x}"), "[Earth J2000] 2000-01-01T13:40:32.183929398 ET\tsma = 7192.041350 km\tecc = 0.024628\tinc = 12.851841 deg\traan = 306.170038 deg\taop = 315.085528 deg\tta = 96.135384 deg");

    // Fetch the state at the start of this spline to make sure we don't glitch.
    assert!(ctx
        .translate(
            summary.target_frame(),
            summary.center_frame(),
            summary.start_epoch(),
            Aberration::NONE,
        )
        .is_ok());

    assert!(ctx
        .translate(
            summary.target_frame(),
            summary.center_frame(),
            summary.end_epoch(),
            Aberration::NONE,
        )
        .is_ok());
}

/// This tests that the rotation from Moon to Earth matches SPICE with different aberration corrections.
/// We test Moon->Earth Moon Barycenter (instead of Venus->SSB as above) because there is no stellar correction possible
/// when the parent is the solar system barycenter.
#[test]
fn de440s_translation_verif_aberrations() {
    let _ = pretty_env_logger::try_init();

    let ctx = Almanac::new("../data/de440s.bsp").unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de440s.bsp')
    >>> et = 66312064.18493876
    >>> ['{:.16e}'.format(x) for x in sp.spkez(301, et, "J2000", "LT", 3)[0]]:
    ['-8.1551741540104151e+04',
    '-3.4544933489888906e+05',
    '-1.4438031089871377e+05',
    '9.6070843890026225e-01',
    '-2.0357817054602378e-01',
    '-1.8380326019667059e-01']
    */

    struct AberrationCase {
        correction: Option<Aberration>,
        pos_expct_km: Vector3,
        vel_expct_km_s: Vector3,
    }

    let cases = [
        AberrationCase {
            correction: Aberration::LT,
            pos_expct_km: Vector3::new(
                -8.155_174_154_010_415e4,
                -3.454_493_348_988_890_6e5,
                -1.443_803_108_987_137_7e5,
            ),
            vel_expct_km_s: Vector3::new(
                9.607_084_389_002_623e-1,
                -2.035_781_705_460_237_8e-1,
                -1.838_032_601_966_706e-1,
            ),
        },
        AberrationCase {
            correction: Aberration::LT_S,
            pos_expct_km: Vector3::new(
                -8.157_072_184_932_455e4,
                -3.454_453_750_037_413e5,
                -1.443_790_633_403_011e5,
            ),
            vel_expct_km_s: Vector3::new(
                9.606_174_870_669_378e-1,
                -2.036_103_860_839_591e-1,
                -1.838_082_628_712_74e-1,
            ),
        },
        AberrationCase {
            correction: Aberration::CN,
            pos_expct_km: Vector3::new(
                -8.155_174_370_552_6e4,
                -3.454_493_371_954_858_3e5,
                -1.443_803_119_050_860_4e5,
            ),
            vel_expct_km_s: Vector3::new(
                9.607_084_394_698_688e-1,
                -2.035_781_706_971_668_8e-1,
                -1.838_032_602_663_712_8e-1,
            ),
        },
        AberrationCase {
            correction: Aberration::CN_S,
            pos_expct_km: Vector3::new(
                -8.157_072_401_473_898e4,
                -3.454_453_773_002_641e5,
                -1.443_790_643_466_415e5,
            ),
            vel_expct_km_s: Vector3::new(
                9.606_174_876_364_936e-1,
                -2.036_103_862_344_811_3e-1,
                -1.838_082_629_406_957_7e-1,
            ),
        },
        AberrationCase {
            correction: Aberration::XLT,
            pos_expct_km: Vector3::new(
                -8.160_143_944_753_706e4,
                -3.455_020_435_001_552e5,
                -1.444_034_078_264_385_5e5,
            ),
            vel_expct_km_s: Vector3::new(
                9.607_152_566_210_146e-1,
                -2.035_882_734_212_926e-1,
                -1.838_077_669_346_027_7e-1,
            ),
        },
        AberrationCase {
            correction: Aberration::XLT_S,
            pos_expct_km: Vector3::new(
                -8.158_245_909_857_436e4,
                -3.455_060_042_043_202_6e5,
                -1.444_046_557_448_048_8e5,
            ),
            vel_expct_km_s: Vector3::new(
                9.608_062_088_449_517e-1,
                -2.035_560_645_572_721_5e-1,
                -1.838_027_672_423_522_6e-1,
            ),
        },
        AberrationCase {
            correction: Aberration::XCN,
            pos_expct_km: Vector3::new(
                -8.160_144_161_352_515e4,
                -3.455_020_457_973_778e5,
                -1.444_034_088_330_790_4e5,
            ),
            vel_expct_km_s: Vector3::new(
                9.607_152_571_912_962e-1,
                -2.035_882_735_719_17e-1,
                -1.838_077_670_040_778_6e-1,
            ),
        },
        AberrationCase {
            correction: Aberration::XCN_S,
            pos_expct_km: Vector3::new(
                -8.158_246_126_456_984e4,
                -3.455_060_065_016_168e5,
                -1.444_046_567_514_772_2e5,
            ),
            vel_expct_km_s: Vector3::new(
                9.608_062_094_152_84e-1,
                -2.035_560_647_085_176_4e-1,
                -1.838_027_673_121_062_6e-1,
            ),
        },
    ];

    for (cno, case) in cases.iter().enumerate() {
        let state = ctx
            .translate(
                MOON_J2000,
                EARTH_MOON_BARYCENTER_J2000,
                epoch,
                case.correction,
            )
            .unwrap();

        let pos_km = state.radius_km;
        let vel_km_s = state.velocity_km_s;

        println!("{state}");

        // We expect exactly the same output as SPICE to machine precision.
        assert!(
            relative_eq!(pos_km, case.pos_expct_km, epsilon = f64::EPSILON),
            "got {} but want {} with {} (#{cno}) => err = {:.3e} km",
            pos_km,
            case.pos_expct_km,
            case.correction.unwrap(),
            (pos_km - case.pos_expct_km).norm()
        );

        assert!(
            relative_eq!(
                vel_km_s,
                case.vel_expct_km_s,
                epsilon = ABERRATION_VELOCITY_EPSILON_KM_S
            ),
            "got {} but want {} with {} (#{cno}) => err = {:.3e} km/s",
            vel_km_s,
            case.vel_expct_km_s,
            case.correction.unwrap(),
            (vel_km_s - case.vel_expct_km_s).norm()
        );

        println!(
            "got {} but want {} with {} (#{cno}) => err = {:.3e} km/s",
            vel_km_s,
            case.vel_expct_km_s,
            case.correction.unwrap(),
            (vel_km_s - case.vel_expct_km_s).norm()
        );
    }
}

#[cfg(feature = "metaload")]
#[test]
fn type9_lagrange_query() {
    use std::env;

    use anise::almanac::metaload::MetaFile;
    use anise::constants::frames::EARTH_J2000;
    use anise::prelude::Frame;

    if env::var("LAGRANGE_BSP").is_err() {
        // Skip this test if the env var is not defined.
        return;
    }

    let lagrange_meta = MetaFile {
        uri: "http://public-data.nyxspace.com/anise/ci/env:LAGRANGE_BSP".to_string(),
        crc32: None,
    };

    let almanac = Almanac::default()
        .load_from_metafile(lagrange_meta, true)
        .unwrap();

    let obj_id = -10;
    let obj_frame = Frame::from_ephem_j2000(obj_id);

    let (start, end) = almanac.spk_domain(obj_id).unwrap();

    // Query near the start, but somewhere where the state is not exactly defined
    let state = almanac
        .translate(
            obj_frame,
            EARTH_J2000,
            start + 1.159_f64.seconds(),
            Aberration::NONE,
        )
        .unwrap();

    let expected_pos_km = Vector3::new(-7338.44373643, 3159.7629953, 760.74472775);
    let expected_vel_km_s = Vector3::new(-7.21781188, -5.26834555, -4.12581558);

    dbg!(state.radius_km - expected_pos_km);
    dbg!(state.velocity_km_s - expected_vel_km_s);

    assert!(
        relative_eq!(state.radius_km, expected_pos_km, epsilon = 5e-6),
        "got {} but want {} => err = {:.3e} km",
        state.radius_km,
        expected_pos_km,
        (state.radius_km - expected_pos_km).norm()
    );

    assert!(
        relative_eq!(state.velocity_km_s, expected_vel_km_s, epsilon = 5e-9),
        "got {} but want {} => err = {:.3e} km/s",
        state.velocity_km_s,
        expected_vel_km_s,
        (state.velocity_km_s - expected_vel_km_s).norm()
    );

    // Query near the end, but not in the registry either
    let state = almanac
        .translate(
            obj_frame,
            EARTH_J2000,
            end - 1159.56_f64.seconds(),
            Aberration::NONE,
        )
        .unwrap();

    let expected_pos_km = Vector3::new(10106.15561792, -166810.06321791, -95547.93140678);
    let expected_vel_km_s = Vector3::new(0.43375448, -1.07193959, -0.55979951);

    dbg!(state.radius_km - expected_pos_km);
    dbg!(state.velocity_km_s - expected_vel_km_s);

    assert!(
        relative_eq!(state.radius_km, expected_pos_km, epsilon = 5e-6),
        "got {} but want {} => err = {:.3e} km",
        state.radius_km,
        expected_pos_km,
        (state.radius_km - expected_pos_km).norm()
    );

    assert!(
        relative_eq!(state.velocity_km_s, expected_vel_km_s, epsilon = 5e-9),
        "got {} but want {} => err = {:.3e} km/s",
        state.velocity_km_s,
        expected_vel_km_s,
        (state.velocity_km_s - expected_vel_km_s).norm()
    );
}
