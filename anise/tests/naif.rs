/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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
        daf::{datatypes::Type2ChebyshevSet, NAIFDataSet, DAF},
        pck::BPCSummaryRecord,
        spk::summary::SPKSummaryRecord,
        Endian,
    },
    prelude::*,
};

#[test]
fn test_binary_pck_load() {
    let _ = pretty_env_logger::try_init();

    // Using the DE421 as demo because the correct data is in the DAF documentation
    let filename = "../data/earth_latest_high_prec.bpc";
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
    let _ = pretty_env_logger::try_init();

    // Using the DE421 as demo because the correct data is in the DAF documentation
    let bytes = file2heap!("../data/de421.bsp").unwrap();

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
    let spice = default_almanac.with_spk(de421).unwrap();
    assert_eq!(spice.num_loaded_spk(), 1);
    assert_eq!(default_almanac.num_loaded_spk(), 0);

    // Now load another DE file
    // NOTE: Rust has strict lifetime requirements, and the Spice Context is set up such that loading another dataset will return a new context with that data set loaded in it.
    {
        let bytes = file2heap!("../data/de440.bsp").unwrap();
        let de440 = DAF::<SPKSummaryRecord>::parse(bytes).unwrap();
        let spice = spice.with_spk(de440).unwrap();

        // And another
        let bytes = file2heap!("../data/de440s.bsp").unwrap();
        let de440 = DAF::<SPKSummaryRecord>::parse(bytes).unwrap();
        let spice = spice.with_spk(de440).unwrap();

        // NOTE: Because everything is a pointer, the size on the stack remains constant at 521 bytes.
        println!("{}", size_of_val(&spice));
        assert_eq!(spice.num_loaded_spk(), 3);
        assert_eq!(default_almanac.num_loaded_spk(), 0);
    }

    // NOTE: Because everything is a pointer, the size on the stack remains constant at 521 bytes.
    println!("{}", size_of_val(&spice));
}

#[test]
fn test_invalid_load() {
    let _ = pretty_env_logger::try_init();

    // Check that it doesn't fail if the file does not exist
    assert!(BPC::load("i_dont_exist.bpc").is_err());
    // Check that a file that's too small does not panic
    assert!(BPC::load("../.gitattributes").is_err());
}

#[test]
fn test_spk_mut_summary_name() {
    let _ = pretty_env_logger::try_init();

    let path = "../data/variable-seg-size-hermite.bsp";
    let output_path = "../target/rename-test.bsp";

    let mut my_spk = SPK::load(path).unwrap().to_mutable();

    let summary_size = my_spk.file_record().unwrap().summary_size();

    let mut name_rcrd = my_spk.name_record().unwrap();

    // Rename all summaries
    for sno in 0..my_spk.data_summaries().unwrap().len() {
        name_rcrd.set_nth_name(
            sno,
            summary_size,
            &format!("Renamed #{sno} (ANISE by Nyx Space)"),
        );
    }
    my_spk.set_name_record(name_rcrd).unwrap();

    my_spk.persist(output_path).unwrap();

    // Check that the written file is correct.
    let reloaded = SPK::load(output_path).unwrap();
    assert_eq!(
        reloaded.name_record().unwrap().nth_name(0, summary_size),
        "Renamed #0 (ANISE by Nyx Space)"
    );
}

#[test]
fn test_spk_truncate_cheby() {
    let _ = pretty_env_logger::try_init();

    let path = "../data/de440s.bsp";

    let my_spk = SPK::load(path).unwrap();

    // Check that we correctly throw an error if the nth data does not exist.
    assert!(my_spk.nth_data::<Type2ChebyshevSet>(100).is_err());

    let idx = 10;

    let summary = my_spk.data_summaries().unwrap()[idx];
    let segment = my_spk.nth_data::<Type2ChebyshevSet>(idx).unwrap();

    let orig_init_epoch = segment.init_epoch;

    let new_start = summary.start_epoch() + Unit::Day * 16;

    let updated_segment = segment.truncate(&summary, Some(new_start), None).unwrap();

    assert!(
        updated_segment.init_epoch - orig_init_epoch <= Unit::Day * 16,
        "truncated too much data"
    );

    assert_ne!(
        updated_segment.init_epoch, orig_init_epoch,
        "truncated too little data"
    );

    // Now we can grab a mutable version of the SPK and modify it.
    let mut my_spk_trunc = my_spk.to_mutable();
    assert!(my_spk_trunc
        .set_nth_data(idx, updated_segment, new_start, summary.end_epoch())
        .is_ok());

    // Serialize the data into a new BSP and confirm that we've updated everything.
    let output_path = "../target/truncated-de440s.bsp";
    my_spk_trunc.persist(output_path).unwrap();

    let reloaded = SPK::load(output_path).unwrap();
    let summary = reloaded.data_summaries().unwrap()[idx];
    assert_eq!(summary.start_epoch(), new_start);

    // Test that we can remove segments all togethet
    let mut my_spk_rm = my_spk.to_mutable();
    assert!(my_spk_rm.delete_nth_data(idx).is_ok());

    // Serialize the data into a new BSP and confirm that we've updated everything.
    let output_path = "../target/rm-de440s.bsp";
    my_spk_rm.persist(output_path).unwrap();

    let reloaded = SPK::load(output_path).unwrap();
    assert!(
        reloaded.summary_from_id(301).is_err(),
        "summary 301 not removed"
    );
}
