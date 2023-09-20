/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::mem::size_of_val;

use anise::{
    file2heap,
    naif::{
        daf::DAF,
        pck::BPCSummaryRecord,
        spk::{datatypes::Type2ChebyshevSet, summary::SPKSummaryRecord},
        Endian,
    },
    prelude::*,
};

#[test]
fn test_binary_pck_load() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // Using the DE421 as demo because the correct data is in the DAF documentation
    let filename = "data/earth_latest_high_prec.bpc";
    let bytes = file2heap!(filename).unwrap();

    let high_prec = DAF::<BPCSummaryRecord>::parse(bytes).unwrap();

    assert_eq!(high_prec.crc32(), 0x97bca34c);
    assert!(high_prec.scrub().is_ok());

    let name_rcrd = high_prec.name_record().unwrap();
    let summary_size = high_prec.file_record().unwrap().summary_size();
    for idx in 0..name_rcrd.num_entries(summary_size) {
        let summary = &high_prec.data_summaries().unwrap()[idx];
        if summary.is_empty() {
            break;
        }
        let name = name_rcrd.nth_name(idx, summary_size);
        println!("{} -> {:?}", name, summary);
    }
}

#[test]
fn test_spk_load_bytes() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // Using the DE421 as demo because the correct data is in the DAF documentation
    let bytes = file2heap!("data/de421.bsp").unwrap();

    let de421 = DAF::<SPKSummaryRecord>::parse(bytes).unwrap();

    assert_eq!(de421.crc32(), 0x5c78bc13);
    assert!(de421.scrub().is_ok());

    assert_eq!(de421.file_record().unwrap().nd, 2);
    assert_eq!(de421.file_record().unwrap().ni, 6);
    assert_eq!(
        de421.file_record().unwrap().identification().unwrap(),
        "SPK"
    );
    assert_eq!(
        de421.file_record().unwrap().internal_filename().unwrap(),
        "NIO2SPK"
    );
    assert_eq!(de421.file_record().unwrap().forward, 4);
    assert_eq!(de421.file_record().unwrap().backward, 4);
    assert_eq!(
        de421.file_record().unwrap().endianness().unwrap(),
        Endian::Little
    );
    assert_eq!(de421.daf_summary().unwrap().num_summaries(), 15);
    assert_eq!(de421.daf_summary().unwrap().next_record(), 0);
    assert_eq!(de421.daf_summary().unwrap().prev_record(), 0);

    println!("{}", de421.comments().unwrap().unwrap());

    // From Python jplephem, an inspection of the coefficients of the DE421 file shows the number of segments we should have.
    // // So let's add it here as a test.
    // // >>> from jplephem.spk import SPK
    // // >>> de421 = SPK.open('../anise.rs/data/de421.bsp')
    // // >>> [c.load_array()[2].shape[1] for c in de421.segments]
    let seg_len: &[usize] = &[
        7040, 3520, 3520, 1760, 1760, 1760, 1760, 1760, 1760, 3520, 14080, 14080, 1, 1, 1,
    ];

    let name_rcrd = de421.name_record().unwrap();
    let summary_size = de421.file_record().unwrap().summary_size();

    for (n, segment) in seg_len
        .iter()
        .enumerate()
        .take(de421.daf_summary().unwrap().num_summaries())
    {
        let name = name_rcrd.nth_name(n, summary_size);
        let summary = &de421.data_summaries().unwrap()[n];

        println!("{} -> {}", name, summary);
        // We know that the DE421 data is all in Type 2
        let data_set = de421.nth_data::<Type2ChebyshevSet>(n).unwrap();
        assert_eq!(data_set.num_records, *segment);
        if summary.target_id == 301 {
            assert_eq!(
                summary.start_idx, 944041,
                "Invalid start of coeff index for DE421"
            );
            assert!(
                (summary.start_epoch_et_s - -3169195200.0).abs() < 2e-16,
                "Invalid start time"
            );
            assert_eq!(
                data_set.interval_length,
                345600.0.seconds(),
                "Invalid interval length (in seconds) for DE421"
            );
            assert_eq!(data_set.rsize, 41, "Invalid rsize for DE421");
            assert_eq!(
                data_set.init_epoch,
                Epoch::from_et_seconds(-3169195200.0),
                "Invalid start time"
            );
        }
    }

    // Try to grab some data here!
    let data_set = de421.nth_data::<Type2ChebyshevSet>(3).unwrap();
    println!("{data_set}");

    // Put this in a context
    let default_almanac = Almanac::default();
    let spice = default_almanac.load_spk(de421).unwrap();
    assert_eq!(spice.num_loaded_spk(), 1);
    assert_eq!(default_almanac.num_loaded_spk(), 0);

    // Now load another DE file
    // NOTE: Rust has strict lifetime requirements, and the Spice Context is set up such that loading another dataset will return a new context with that data set loaded in it.
    {
        let bytes = file2heap!("data/de440.bsp").unwrap();
        let de440 = DAF::<SPKSummaryRecord>::parse(bytes).unwrap();
        let spice = spice.load_spk(de440).unwrap();

        // And another
        let bytes = file2heap!("data/de440s.bsp").unwrap();
        let de440 = DAF::<SPKSummaryRecord>::parse(bytes).unwrap();
        let spice = spice.load_spk(de440).unwrap();

        // NOTE: Because everything is a pointer, the size on the stack remains constant at 521 bytes.
        println!("{}", size_of_val(&spice));
        assert_eq!(spice.num_loaded_spk(), 3);
        assert_eq!(default_almanac.num_loaded_spk(), 0);
    }

    // NOTE: Because everything is a pointer, the size on the stack remains constant at 521 bytes.
    println!("{}", size_of_val(&spice));
}

// The `load` function copies the bytes, so it's only available with std

#[test]
fn test_spk_rename_summary() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    let path = "data/variable-seg-size-hermite.bsp";

    let example_data = SPK::load(path).unwrap();

    example_data.name_record().unwrap().set_nth_name(
        0,
        example_data.file_record().unwrap().summary_size(),
        "BLAH BLAH",
    );

    dbg!(example_data
        .name_record()
        .unwrap()
        .nth_name(0, example_data.file_record().unwrap().summary_size()));

    example_data.persist("target/rename-test.bsp").unwrap();
}
