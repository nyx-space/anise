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
    assert_eq!(format!("{SSB_ICRS}"), "Solar System Barycenter ICRS");

    assert_eq!(format!("{SUN_ICRS}"), "Sun ICRS");

    assert_eq!(format!("{MERCURY_ICRS}"), "Mercury Barycenter ICRS");

    assert_eq!(format!("{VENUS_ICRS}"), "Venus Barycenter ICRS");

    assert_eq!(
        format!("{EARTH_MOON_BARYCENTER_ICRS}"),
        "Earth-Moon Barycenter ICRS"
    );

    assert_eq!(format!("{EARTH_ICRS}"), "Earth ICRS");

    assert_eq!(format!("{MOON_ICRS}"), "Moon ICRS");

    assert_eq!(format!("{MARS_ICRS}"), "Mars ICRS");

    assert_eq!(format!("{MARS_BARYCENTER_ICRS}"), "Mars Barycenter ICRS");

    assert_eq!(
        format!("{JUPITER_BARYCENTER_ICRS}"),
        "Jupiter Barycenter ICRS"
    );

    assert_eq!(
        format!("{SATURN_BARYCENTER_ICRS}"),
        "Saturn Barycenter ICRS"
    );

    assert_eq!(
        format!("{URANUS_BARYCENTER_ICRS}"),
        "Uranus Barycenter ICRS"
    );

    assert_eq!(
        format!("{NEPTUNE_BARYCENTER_ICRS}"),
        "Neptune Barycenter ICRS"
    );

    assert_eq!(format!("{PLUTO_BARYCENTER_ICRS}"), "Pluto Barycenter ICRS");
}
