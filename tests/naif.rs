/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use anise::{
    naif::daf::{Endianness, DAF},
    prelude::*,
};

#[test]
fn test_de438s_load() {
    let filename = "data/de440.bsp";
    let bytes = file_mmap!(filename).unwrap();

    let de440 = DAF::parse(&bytes).unwrap();
    assert_eq!(de440.nd, 2);
    assert_eq!(de440.ni, 6);
    assert_eq!(de440.fwrd, 62);
    assert_eq!(de440.bwrd, 62);
    assert_eq!(de440.endianness, Endianness::Little);
}
