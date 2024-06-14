/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::constants::frames::VENUS_J2000;
use anise::file2heap;
use anise::math::Vector3;
use anise::prelude::*;

const ZEROS: &[u8] = &[0; 256];
/// Test that we can load data from a static pointer to it, even if there is less than one record length
#[test]
fn invalid_load_from_static() {
    assert!(SPK::from_static(&ZEROS).is_err());
}

#[test]
fn de400_domain() {
    let almanac = Almanac::new("../data/de440s.bsp").unwrap();

    assert!(almanac.spk_domain(-1012).is_err());
    assert!(almanac.spk_domain(399).is_ok());
    assert!(almanac.spk_domains().is_ok());

    // No BPC loaded, so it should error.
    assert!(almanac.bpc_domain(-1).is_err());
    assert!(almanac.bpc_domain(399).is_err());

    assert!(almanac.bpc_domains().is_err());
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

    let state = ctx.translate_to_parent(VENUS_J2000, epoch).unwrap();

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
        (pos_km - pos_expct_km).norm() < f64::EPSILON,
        "got {} but want {}",
        pos_km,
        pos_expct_km
    );

    assert!(
        (vel_km_s - vel_expct_km_s).norm() < f64::EPSILON,
        "got {} but want {}",
        vel_km_s,
        vel_expct_km_s
    );
}
