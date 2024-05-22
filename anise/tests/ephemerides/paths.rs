/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::str::FromStr;

use anise::constants::celestial_objects::{EARTH_MOON_BARYCENTER, SOLAR_SYSTEM_BARYCENTER};
use anise::constants::frames::*;
use anise::file2heap;
use anise::prelude::*;

/// Tests that direct path computations match what SPICE returned to within good precision.
#[test]
fn common_root_verif() {
    let _ = pretty_env_logger::try_init();

    // SLS Launch epoch!!! IT'S LIIIIVEE!!
    let epoch = Epoch::from_str("2022-11-15T23:47:36+06:00").unwrap();

    // Load the context
    // Check that this test works for DE430, DE438s (short), and DE440
    for path in [
        "../data/de430.bsp",
        "../data/de440s.bsp",
        "../data/de440.bsp",
    ] {
        let buf = file2heap!(path).unwrap();
        let spk = SPK::parse(buf).unwrap();
        let ctx = Almanac::from_spk(spk).unwrap();

        // The root of all these files should be the SSB
        assert_eq!(
            ctx.try_find_ephemeris_root().unwrap(),
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
                ctx.common_ephemeris_path(*planet_ctr, MOON_J2000, epoch)
                    .unwrap()
                    .2,
                SOLAR_SYSTEM_BARYCENTER
            );

            assert_eq!(
                ctx.common_ephemeris_path(MOON_J2000, *planet_ctr, epoch)
                    .unwrap()
                    .2,
                SOLAR_SYSTEM_BARYCENTER
            );
        }

        // Common root between Earth and Moon should be EMB
        assert_eq!(
            ctx.common_ephemeris_path(MOON_J2000, EARTH_J2000, epoch)
                .unwrap()
                .2,
            EARTH_MOON_BARYCENTER
        );
        assert_eq!(
            ctx.common_ephemeris_path(EARTH_J2000, MOON_J2000, epoch)
                .unwrap()
                .2,
            EARTH_MOON_BARYCENTER
        );

        // Common root between EMB and Moon should be EMB
        assert_eq!(
            ctx.common_ephemeris_path(MOON_J2000, EARTH_MOON_BARYCENTER_J2000, epoch)
                .unwrap()
                .2,
            EARTH_MOON_BARYCENTER
        );
        assert_eq!(
            ctx.common_ephemeris_path(EARTH_MOON_BARYCENTER_J2000, MOON_J2000, epoch)
                .unwrap()
                .2,
            EARTH_MOON_BARYCENTER
        );
    }
}
