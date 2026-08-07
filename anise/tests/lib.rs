/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

#[macro_use]
extern crate approx;
mod almanac;
mod astro;
mod ephemerides;
mod frames;
#[cfg(feature = "analysis")]
mod instrument;
mod orientations;

/// Single query for timing purposes
#[test]
fn flamegraph() {
    use anise::constants::frames::{MOON_J2000, SUN_J2000};
    use anise::prelude::Almanac;
    use hifitime::{Epoch, TimeSeries, Unit};
    let almanac = Almanac::new("../data/de440s.bsp").unwrap();
    let epoch = Epoch::from_gregorian_utc_at_noon(2024, 2, 29);
    for epoch in TimeSeries::inclusive(epoch, epoch + Unit::Hour * 1, Unit::Second * 1) {
        let orbit = almanac
            .transform(SUN_J2000, MOON_J2000, epoch, None)
            .unwrap();
        println!("{orbit}")
    }
}
