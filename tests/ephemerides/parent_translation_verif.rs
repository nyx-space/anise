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
use anise::file_mmap;
use anise::math::Vector3;
use anise::prelude::*;

#[test]
fn de438s_parent_translation_verif() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.anise";
    let buf = file_mmap!(path).unwrap();
    let ctx = AniseContext::try_from_bytes(&buf).unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de438s.bsp')
    >>> sp.furnsh('../../hifitime/naif0012.txt')
    >>> et = sp.utc2et('2002 FEB 07 00:00:00')
    >>> et
    66312064.18493876
    >>> ['{:.16e}'.format(x) for x in sp.spkez(2, et, "J2000", "NONE", 0)[0]]
    ['9.5205638574810922e+07', '-4.6160711641080864e+07', '-2.6779481328088202e+07', '1.6612048965376893e+01', '2.8272067093357247e+01', '1.1668575733195270e+01']
    */

    let (pos, vel, acc, _) = ctx
        .translate_to_parent(
            VENUS_J2000,
            epoch,
            Aberration::None,
            DistanceUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    let pos_expct_km = Vector3::new(
        9.5205638574810922e+07,
        -4.6160711641080864e+07,
        -2.6779481328088202e+07,
    );

    let vel_expct_km_s = Vector3::new(
        1.6612048965376893e+01,
        2.8272067093357247e+01,
        1.1668575733195270e+01,
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
            DistanceUnit::Megameter,
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
