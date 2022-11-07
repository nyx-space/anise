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
}
