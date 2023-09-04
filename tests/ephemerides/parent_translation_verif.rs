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

use anise::constants::frames::VENUS_J2000;
use anise::file2heap;
use anise::math::Vector3;
use anise::prelude::*;

#[test]
fn de438s_parent_translation_verif() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    let bytes = file2heap!("data/de440s.bsp").unwrap();
    let de438s = SPK::parse(bytes).unwrap();
    let ctx = Almanac::from_spk(&de438s).unwrap();

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

    let (pos, vel, acc, _) = ctx
        .translate_to_parent(
            VENUS_J2000,
            epoch,
            Aberration::None,
            LengthUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    let pos_expct_km = Vector3::new(
        9.5205530594596043e+07,
        -4.6160758818180226e+07,
        -2.6779476581501361e+07,
    );

    let vel_expct_km_s = Vector3::new(
        1.6612048969243794e+01,
        2.8272067093941200e+01,
        1.1668575714409423e+01,
    );

    // We expect exactly the same output as SPICE to machine precision.
    assert!((pos - pos_expct_km).norm() < EPSILON);
    assert!((vel - vel_expct_km_s).norm() < EPSILON);
    assert!(acc.norm() < EPSILON);

    // Same thing but in Megameters per millisecond
    let (pos, vel, acc, _) = ctx
        .translate_to_parent(
            VENUS_J2000,
            epoch,
            Aberration::None,
            LengthUnit::Megameter,
            TimeUnit::Millisecond,
        )
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        (pos - pos_expct_km * 1e-3).norm() < EPSILON,
        "got {} but want {}",
        pos,
        pos_expct_km * 1e-3
    );

    // NOTE: km/s and Mm/ms correspond to the same number: times 1e3 for km -> Mm and times 1e-3 for s -> ms.
    assert!(
        (vel - vel_expct_km_s).norm() < EPSILON,
        "got {} but want {}",
        vel,
        vel_expct_km_s
    );
    assert!(acc.norm() < EPSILON);
}
