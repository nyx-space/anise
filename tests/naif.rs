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
    // Using the DE421 as demo because the correct data is in the DAF documentation
    let filename = "data/de421.bsp";
    // let filename = "data/de440.bsp";
    let bytes = file_mmap!(filename).unwrap();

    let de421 = DAF::parse(&bytes).unwrap();
    assert_eq!(de421.nd, 2);
    assert_eq!(de421.ni, 6);
    assert_eq!(de421.idword, "DAF/SPK");
    assert_eq!(de421.internal_filename, "NIO2SPK");
    assert_eq!(de421.fwrd, 4);
    assert_eq!(de421.bwrd, 4);
    assert_eq!(de421.endianness, Endianness::Little);

    let comments = de421.comments();
    assert_eq!(comments.len(), 1379);
    de421.summaries();
}
