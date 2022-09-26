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

use anise::constants::celestial_objects::EARTH_MOON_BARYCENTER;
use anise::constants::celestial_objects::SOLAR_SYSTEM_BARYCENTER;
use anise::constants::frames::EARTH_J2000;
use anise::constants::frames::JUPITER_BARYCENTER_J2000;
use anise::constants::frames::LUNA_J2000;
use anise::constants::frames::MARS_BARYCENTER_J2000;
use anise::constants::frames::MERCURY_J2000;
use anise::constants::frames::NEPTUNE_BARYCENTER_J2000;
use anise::constants::frames::PLUTO_BARYCENTER_J2000;
use anise::constants::frames::SATURN_BARYCENTER_J2000;
use anise::constants::frames::URANUS_BARYCENTER_J2000;
use anise::constants::frames::VENUS_J2000;
use anise::constants::orientations::J2000;
use anise::frame::Frame;
use anise::math::Vector3;
use anise::prelude::AniseError;
use anise::prelude::File;
use anise::Epoch;
use anise::{file_mmap, prelude::AniseContext};

/// Tests the ephemeris computations from the de438s which don't require any frame transformation.
#[test]
fn de438s_zero_paths() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // Check that this test works for DE430, DE438s (short), and DE440
    for path in &[
        "./data/de430.anise",
        "./data/de438s.anise",
        "./data/de440.anise",
    ] {
        // "Load" the file via a memory map (avoids allocations)
        let buf = file_mmap!(path).unwrap();
        let ctx: AniseContext = (&buf).try_into().unwrap();

        // We know that these ephemerides files has exactly 14 ephemerides.
        assert_eq!(
            ctx.ephemeris_lut.hashes.len(),
            12,
            "DE438s should have 12 ephemerides"
        );

        // For all of the frames in this context, let's make sure that the translation between the same frames is always zero.
        for ephemeris_hash in ctx.ephemeris_lut.hashes.iter() {
            // Build a J2000 oriented frame with this ephemeris center
            let this_frame_j2k = Frame::from_ephem_orient(*ephemeris_hash, J2000);

            // Check that the common root between the same frame is that frame's hash.
            let root_ephem = ctx
                .find_common_ephemeris_node(this_frame_j2k, this_frame_j2k)
                .unwrap();

            assert_eq!(root_ephem, *ephemeris_hash);

            // Check that in these cases, the translation returns a zero vector in position and in velocity.

            let (delta_pos, delta_vel) = ctx
                .translate_from_to(this_frame_j2k, this_frame_j2k, Epoch::now().unwrap())
                .unwrap();
            assert!(delta_pos.norm() < EPSILON);
            assert!(delta_vel.norm() < EPSILON);
        }
    }

    // ctx.lt_translate_from_to(Earth, Moon, epoch, LTCorr) -> position and velocity of the Earth with respect to the Moon with light time correction at epoch
    // ctx.rotate_to_from() -> quaternion
}

/// Tests that direct path computations match what SPICE returned to within good precision.
#[test]
fn de438s_common_root_verif() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // Load the context
    // Check that this test works for DE430, DE438s (short), and DE440
    for path in &[
        "./data/de430.anise",
        "./data/de438s.anise",
        "./data/de440.anise",
    ] {
        let buf = file_mmap!(path).unwrap();
        let ctx: AniseContext = (&buf).try_into().unwrap();

        // The root of all these files should be the SSB
        assert_eq!(
            ctx.try_find_context_root().unwrap(),
            SOLAR_SYSTEM_BARYCENTER
        );

        // Common root between all planets (apart from Earth) and the Moon should be the solar system barycenter
        for planet_ctr in &[
            MERCURY_J2000,
            VENUS_J2000,
            MARS_BARYCENTER_J2000,
            JUPITER_BARYCENTER_J2000,
            SATURN_BARYCENTER_J2000,
            NEPTUNE_BARYCENTER_J2000,
            URANUS_BARYCENTER_J2000,
            PLUTO_BARYCENTER_J2000,
        ] {
            assert_eq!(
                ctx.find_common_ephemeris_node(*planet_ctr, LUNA_J2000)
                    .unwrap(),
                SOLAR_SYSTEM_BARYCENTER
            );

            assert_eq!(
                ctx.find_common_ephemeris_node(LUNA_J2000, *planet_ctr)
                    .unwrap(),
                SOLAR_SYSTEM_BARYCENTER
            );
        }

        // Common root between Earth and Moon should be EMB
        assert_eq!(
            ctx.find_common_ephemeris_node(LUNA_J2000, EARTH_J2000)
                .unwrap(),
            EARTH_MOON_BARYCENTER
        );
        assert_eq!(
            ctx.find_common_ephemeris_node(EARTH_J2000, LUNA_J2000)
                .unwrap(),
            EARTH_MOON_BARYCENTER
        );
    }
}

#[test]
fn de438s_translation_verif() {
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
    >>> et
    66312064.18493876
    >>> ['{:.16e}'.format(x) for x in sp.spkez(2, et, "J2000", "NONE", 0)[0]]
    ['9.5205638574810922e+07', '-4.6160711641080864e+07', '-2.6779481328088202e+07', '1.6612048965376893e+01', '2.8272067093357247e+01', '1.1668575733195270e+01']
    */

    let (pos, vel) = ctx.translate_to_parent(VENUS_J2000, epoch).unwrap();

    let pos_expct_km = Vector3::new(
        9.5205638574810922e+07,
        -4.6160711641080864e+07,
        -2.6779481328088202e+07,
    );

    let vel_expct = Vector3::new(
        1.6612048965376893e+01,
        2.8272067093357247e+01,
        1.1668575733195270e+01,
    );

    dbg!(vel, vel_expct);

    assert!(dbg!(pos - pos_expct_km).norm() < 1e-16);
    assert!(dbg!(vel - vel_expct).norm() < 1e-16);

    // assert!(dbg!(ven2ear_state.y - -1.356_125_479_230_852_7e8).abs() < 7e-4);
    // assert!(dbg!(ven2ear_state.z - -6.557_839_967_615_153e7).abs() < 4e-4);
    // assert!(dbg!(ven2ear_state.vy - 4.888_902_462_217_076_6e1).abs() < 1e-8);
    // assert!(dbg!(ven2ear_state.vz - 2.070_293_380_084_308_4e1).abs() < 1e-8);
}
