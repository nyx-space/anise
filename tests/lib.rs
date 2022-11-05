/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::structure::orientation::{
    orient_data::OrientationData, phaseangle::PhaseAngle, trigangle::TrigAngle,
};
use der::asn1::SequenceOf;

#[macro_use]
extern crate approx;
mod astro;
mod ephemerides;
mod frames;

#[test]
fn size_test() {
    use anise::file_mmap;
    use anise::prelude::*;

    let path = "./data/de438s.anise";
    let buf = file_mmap!(path).unwrap();
    let ctx = AniseContext::try_from_bytes(&buf).unwrap();

    use std::mem::size_of_val;
    println!("{}", size_of_val(&ctx));
    println!("{}", size_of_val(&ctx.ephemeris_data));
    println!("{}", size_of_val(&ctx.orientation_data));
    let pa = PhaseAngle {
        offset_deg: 0.0,
        rate_deg: 0.0,
        accel_deg: 0.0,
    };
    println!("pa = {}", size_of_val(&pa));

    let ta = TrigAngle {
        right_ascension_deg: 0.0,
        declination_deg: 0.0,
        prime_meridian_deg: 0.0,
        nut_prec_angle: pa,
    };

    println!("ta = {}", size_of_val(&ta));

    let mut nut_prec_angles = SequenceOf::new();
    nut_prec_angles.add(ta).unwrap();

    println!("npa = {}", size_of_val(&nut_prec_angles));

    let pa_od = OrientationData::PlanetaryConstant {
        pole_right_ascension: pa,
        pole_declination: pa,
        prime_meridian: pa,
        nut_prec_angles: nut_prec_angles.clone(),
    };
    println!("pa_od = {}", size_of_val(&pa_od));

    struct PlanetaryConstant {
        pole_right_ascension: PhaseAngle,
        pole_declination: PhaseAngle,
        prime_meridian: PhaseAngle,
        nut_prec_angles: SequenceOf<TrigAngle, 16>,
    };
    let pa_od2 = PlanetaryConstant {
        pole_right_ascension: pa,
        pole_declination: pa,
        prime_meridian: pa,
        nut_prec_angles,
    };
    println!("pa_od2 = {}", size_of_val(&pa_od2));
}
