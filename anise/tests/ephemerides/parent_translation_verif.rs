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

use anise::constants::frames::{LUNA_J2000, VENUS_J2000};
use anise::file2heap;
use anise::math::Vector3;
use anise::prelude::*;

const ZEROS: &[u8] = &[0; 2048];
/// Test that we can load data from a static pointer to it.
#[test]
fn invalid_load_from_static() {
    assert!(SPK::from_static(&ZEROS).is_err());
}

#[test]
fn de440s_parent_translation_verif() {
    let _ = pretty_env_logger::try_init();

    let bytes = file2heap!("../data/de440s.bsp").unwrap();
    let de438s = SPK::parse(bytes).unwrap();
    let ctx = Almanac::from_spk(de438s).unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de440s.bsp')
    >>> sp.furnsh('../../hifitime/naif0012.txt')
    >>> et = sp.utc2et('2002 FEB 07 00:00:00')
    >>> et
    66312064.18493876
    >>> ['{:.16e}'.format(x) for x in sp.spkez(2, et, "J2000", "NONE", 0)[0]]
    ['9.5205530594596043e+07', '-4.6160758818180226e+07', '-2.6779476581501361e+07', '1.6612048969243794e+01', '2.8272067093941200e+01', '1.1668575714409423e+01']
    */

    let state = ctx
        .translate_to_parent(VENUS_J2000, epoch, Aberration::NONE)
        .unwrap();

    let pos_km = state.radius_km;
    let vel_km_s = state.velocity_km_s;

    let pos_expct_km = Vector3::new(
        9.520_553_059_459_604e7,
        -4.616_075_881_818_022_6e7,
        -2.677_947_658_150_136e7,
    );

    let vel_expct_km_s = Vector3::new(
        1.661_204_896_924_379_4e1,
        2.827_206_709_394_12e1,
        1.166_857_571_440_942_3e1,
    );

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        (pos_km - pos_expct_km).norm() < EPSILON,
        "got {} but want {}",
        pos_km,
        pos_expct_km
    );

    assert!(
        (vel_km_s - vel_expct_km_s).norm() < EPSILON,
        "got {} but want {}",
        vel_km_s,
        vel_expct_km_s
    );
}

/// This tests that the rotation from Moon to Earth matches SPICE with different aberration corrections.
/// We test Moon->Earth Moon Barycenter (instead of Venus->SSB as above) because there is no stellar correction possible
/// when the parent is the solar system barycenter.
#[test]
fn de440s_parent_translation_verif_aberrations() {
    let _ = pretty_env_logger::try_init();

    let bytes = file2heap!("../data/de440s.bsp").unwrap();
    let de438s = SPK::parse(bytes).unwrap();
    let ctx = Almanac::from_spk(de438s).unwrap();

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

    struct TestCase {
        correction: Option<Aberration>,
        pos_expct_km: Vector3,
        vel_expct_km_s: Vector3,
    }

    let cases = [
        TestCase {
            correction: Aberration::LT,
            pos_expct_km: Vector3::new(
                -8.1551741540104151e+04,
                -3.4544933489888906e+05,
                -1.4438031089871377e+05,
            ),
            vel_expct_km_s: Vector3::new(
                9.6070843890026225e-01,
                -2.0357817054602378e-01,
                -1.8380326019667059e-01,
            ),
        },
        TestCase {
            correction: Aberration::LT_S,
            pos_expct_km: Vector3::new(
                -8.1570721849324545e+04,
                -3.4544537500374130e+05,
                -1.4437906334030110e+05,
            ),
            vel_expct_km_s: Vector3::new(
                9.6061748706693784e-01,
                -2.0361038608395909e-01,
                -1.8380826287127400e-01,
            ),
        },
        TestCase {
            correction: Aberration::CN,
            pos_expct_km: Vector3::new(
                -8.1551743705525994e+04,
                -3.4544933719548583e+05,
                -1.4438031190508604e+05,
            ),
            vel_expct_km_s: Vector3::new(
                9.6070843946986884e-01,
                -2.0357817069716688e-01,
                -1.8380326026637128e-01,
            ),
        },
        TestCase {
            correction: Aberration::CN_S,
            pos_expct_km: Vector3::new(
                -8.1570724014738982e+04,
                -3.4544537730026408e+05,
                -1.4437906434664151e+05,
            ),
            vel_expct_km_s: Vector3::new(
                9.6061748763649357e-01,
                -2.0361038623448113e-01,
                -1.8380826294069577e-01,
            ),
        },
        TestCase {
            correction: Aberration::XLT,
            pos_expct_km: Vector3::new(
                -8.1601439447537065e+04,
                -3.4550204350015521e+05,
                -1.4440340782643855e+05,
            ),
            vel_expct_km_s: Vector3::new(
                9.6071525662101465e-01,
                -2.0358827342129260e-01,
                -1.8380776693460277e-01,
            ),
        },
        TestCase {
            correction: Aberration::XLT_S,
            pos_expct_km: Vector3::new(
                -8.1582459098574356e+04,
                -3.4550600420432026e+05,
                -1.4440465574480488e+05,
            ),
            vel_expct_km_s: Vector3::new(
                9.6080620884495171e-01,
                -2.0355606455727215e-01,
                -1.8380276724235226e-01,
            ),
        },
        TestCase {
            correction: Aberration::XCN,
            pos_expct_km: Vector3::new(
                -8.1601441613525152e+04,
                -3.4550204579737782e+05,
                -1.4440340883307904e+05,
            ),
            vel_expct_km_s: Vector3::new(
                9.6071525719129625e-01,
                -2.0358827357191700e-01,
                -1.8380776700407786e-01,
            ),
        },
        TestCase {
            correction: Aberration::XCN_S,
            pos_expct_km: Vector3::new(
                -8.1582461264569836e+04,
                -3.4550600650161679e+05,
                -1.4440465675147722e+05,
            ),
            vel_expct_km_s: Vector3::new(
                9.6080620941528405e-01,
                -2.0355606470851764e-01,
                -1.8380276731210626e-01,
            ),
        },
    ];

    for case in cases {
        let state = ctx
            .translate_to_parent(LUNA_J2000, epoch, case.correction)
            .unwrap();

        let pos_km = state.radius_km;
        let vel_km_s = state.velocity_km_s;

        println!("{state}");

        // We expect exactly the same output as SPICE to machine precision.
        assert!(
            (pos_km - case.pos_expct_km).norm() < EPSILON,
            "got {} but want {} with {}",
            pos_km,
            case.pos_expct_km,
            case.correction.unwrap()
        );

        assert!(
            (vel_km_s - case.vel_expct_km_s).norm() < EPSILON,
            "got {} but want {} with {}",
            vel_km_s,
            case.vel_expct_km_s,
            case.correction.unwrap()
        );
    }
}
