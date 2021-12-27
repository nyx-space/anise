/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::convert::TryInto;

use anise::{
    naif::{
        daf::{Endianness, DAF},
        spk::SPK,
    },
    prelude::*,
};

#[test]
fn test_spk_load() {
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
    assert_eq!(de421.comments().len(), 1379);
    // Convert to SPK
    let spk: SPK = (&de421).try_into().unwrap();
    println!("{}", spk);

    let (seg_coeff_idx, (init_s_past_j2k, interval_length, rsize, num_records_in_seg)) =
        spk.segment_ptr(301).unwrap();
    assert_eq!(
        seg_coeff_idx, 944041,
        "Invalid start of coeff index for DE421"
    );
    assert_eq!(
        interval_length, 345600,
        "Invalid interval length (in seconds) for DE421"
    );
    assert_eq!(rsize, 41, "Invalid rsize for DE421");
    assert_eq!(
        num_records_in_seg, 14080,
        "Invalid num_records_in_seg for DE421"
    );
    assert!(
        (init_s_past_j2k - -3169195200.0).abs() < 2e-16,
        "Invalid start time"
    );
}

#[ignore]
#[test]
fn test_binary_pck_load() {
    // Using the DE421 as demo because the correct data is in the DAF documentation
    let filename = "data/earth_old_high_prec.bpc";
    let bytes = file_mmap!(filename).unwrap();

    let high_prec = DAF::parse(&bytes).unwrap();
    println!("{}", high_prec.comments());
    high_prec.summaries();
}
