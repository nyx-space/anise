/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::constants::frames::*;

/// Tests the ephemeris computations from the de438s which don't require any frame transformation.
#[test]
fn format_frame() {
    assert_eq!(format!("{}", SSB_J2000), "Solar System Barycenter J2000");

    assert_eq!(format!("{}", SUN_J2000), "Sun J2000");

    assert_eq!(format!("{}", MERCURY_J2000), "Mercury J2000");

    assert_eq!(format!("{}", VENUS_J2000), "Venus J2000");

    assert_eq!(
        format!("{}", EARTH_MOON_BARYCENTER_J2000),
        "Earth-Moon Barycenter J2000"
    );

    assert_eq!(format!("{}", EARTH_J2000), "Earth J2000");

    assert_eq!(format!("{}", MOON_J2000), "Moon J2000");

    assert_eq!(
        format!("{}", MARS_BARYCENTER_J2000),
        "Mars Barycenter J2000"
    );

    assert_eq!(
        format!("{}", JUPITER_BARYCENTER_J2000),
        "Jupiter Barycenter J2000"
    );

    assert_eq!(
        format!("{}", SATURN_BARYCENTER_J2000),
        "Saturn Barycenter J2000"
    );

    assert_eq!(
        format!("{}", URANUS_BARYCENTER_J2000),
        "Uranus Barycenter J2000"
    );

    assert_eq!(
        format!("{}", NEPTUNE_BARYCENTER_J2000),
        "Neptune Barycenter J2000"
    );

    assert_eq!(
        format!("{}", PLUTO_BARYCENTER_J2000),
        "Pluto Barycenter J2000"
    );
}
