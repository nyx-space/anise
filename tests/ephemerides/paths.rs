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

use anise::constants::celestial_objects::{EARTH_MOON_BARYCENTER, SOLAR_SYSTEM_BARYCENTER};
use anise::constants::frames::*;
use anise::constants::orientations::J2000;
use anise::prelude::Frame;
use anise::prelude::*;
use anise::{file_mmap, prelude::AniseContext};

/// Tests the ephemeris computations from the de438s which don't require any frame transformation.
#[test]
fn zero_translations() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // Check that this test works for DE421, DE430, DE438s (short), and DE440
    for path in [
        "./data/de421.anise",
        "./data/de430.anise",
        "./data/de438s.anise",
        "./data/de440.anise",
    ] {
        // "Load" the file via a memory map (avoids allocations)
        let buf = file_mmap!(path).unwrap();
        let ctx = AniseContext::try_from_bytes(&buf).unwrap();

        // We know that these ephemerides files has exactly 14 ephemerides.
        assert_eq!(
            ctx.ephemeris_lut.hashes.data.len(),
            12,
            "DE438s should have 12 ephemerides"
        );

        // For all of the frames in this context, let's make sure that the translation between the same frames is always zero.
        for ephemeris_hash in ctx.ephemeris_lut.hashes.data.iter() {
            // Build a J2000 oriented frame with this ephemeris center
            let this_frame_j2k = Frame::from_ephem_orient(*ephemeris_hash, J2000);

            // Check that the common root between the same frame is that frame's hash.
            let root_ephem = ctx
                .common_ephemeris_path(this_frame_j2k, this_frame_j2k)
                .unwrap()
                .2;

            assert_eq!(root_ephem, *ephemeris_hash);

            // Check that in these cases, the translation returns a zero vector in position and in velocity.

            let delta_state = ctx
                .translate_from_to_km_s(
                    this_frame_j2k,
                    this_frame_j2k,
                    Epoch::now().unwrap(),
                    Aberration::None,
                )
                .unwrap();
            assert!(delta_state.rmag_km() < EPSILON);
            assert!(delta_state.vmag_km_s() < EPSILON);
            assert!(delta_state.acceleration_km_s2.unwrap().norm() < EPSILON);
        }
    }

    // ctx.lt_translate_from_to(Earth, Moon, epoch, LTCorr) -> position and velocity of the Earth with respect to the Moon with light time correction at epoch
    // ctx.rotate_to_from() -> quaternion
}

/// Tests that direct path computations match what SPICE returned to within good precision.
#[test]
fn common_root_verif() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // Load the context
    // Check that this test works for DE430, DE438s (short), and DE440
    for path in [
        "./data/de430.anise",
        "./data/de438s.anise",
        "./data/de440.anise",
    ] {
        let buf = file_mmap!(path).unwrap();
        let ctx = AniseContext::try_from_bytes(&buf).unwrap();

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
                ctx.common_ephemeris_path(*planet_ctr, LUNA_J2000)
                    .unwrap()
                    .2,
                SOLAR_SYSTEM_BARYCENTER
            );

            assert_eq!(
                ctx.common_ephemeris_path(LUNA_J2000, *planet_ctr)
                    .unwrap()
                    .2,
                SOLAR_SYSTEM_BARYCENTER
            );
        }

        // Common root between Earth and Moon should be EMB
        assert_eq!(
            ctx.common_ephemeris_path(LUNA_J2000, EARTH_J2000)
                .unwrap()
                .2,
            EARTH_MOON_BARYCENTER
        );
        assert_eq!(
            ctx.common_ephemeris_path(EARTH_J2000, LUNA_J2000)
                .unwrap()
                .2,
            EARTH_MOON_BARYCENTER
        );

        // Common root between EMB and Moon should be EMB
        assert_eq!(
            ctx.common_ephemeris_path(LUNA_J2000, EARTH_MOON_BARYCENTER_J2000)
                .unwrap()
                .2,
            EARTH_MOON_BARYCENTER
        );
        assert_eq!(
            ctx.common_ephemeris_path(EARTH_MOON_BARYCENTER_J2000, LUNA_J2000)
                .unwrap()
                .2,
            EARTH_MOON_BARYCENTER
        );
    }
}
