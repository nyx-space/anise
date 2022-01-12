/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{convert::TryInto, f64::EPSILON};

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

    let (seg, (init_s_past_j2k, interval_length, rsize, num_records_in_seg)) =
        spk.segment_ptr(301).unwrap();
    assert_eq!(
        seg.start_idx, 944041,
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

    spk.all_coefficients(301).unwrap();
    // Build the ANISE file
    // TODO: Compute the checksum and make sure it's correct
    let filename_anis = "de421.anis";
    spk.to_anise(filename, filename_anis);
    // Load this ANIS file and make sure that it matches the original DE421 data.

    let bytes = file_mmap!(filename_anis).unwrap();
    let ctx = Anise::from_bytes(&bytes);
    assert_eq!(
        ctx.ephemeris_map().unwrap().hash().unwrap().len(),
        spk.segments.len(),
        "Incorrect number of ephem in map"
    );
    assert_eq!(
        ctx.ephemeris_map().unwrap().index().unwrap().len(),
        spk.segments.len(),
        "Incorrect number of ephem in map"
    );
    assert_eq!(
        ctx.ephemerides().unwrap().len(),
        spk.segments.len(),
        "Incorrect number of ephem in map"
    );

    // From Python jplephem, an inspection of the coefficients of the DE421 file shows the number of segments we should have.
    // So let's add it here as a test.
    // >>> from jplephem.spk import SPK
    // >>> de421 = SPK.open('../anise.rs/data/de421.bsp')
    // >>> [c.load_array()[2].shape[1] for c in de421.segments]
    let seg_len: &[usize] = &[
        7040, 3520, 3520, 1760, 1760, 1760, 1760, 1760, 1760, 3520, 14080, 14080, 1, 1, 1,
    ];

    for (eidx, ephem) in ctx.ephemerides().unwrap().iter().enumerate() {
        let splt = ephem.name().split("#").collect::<Vec<&str>>();
        let seg_target_id = str::parse::<i32>(splt[1]).unwrap();
        // Fetch the SPK segment
        let (seg, seg_data) = spk.all_coefficients(seg_target_id).unwrap();
        assert_eq!(
            seg_data.len(),
            seg_len[eidx],
            "wrong number of segments for {}",
            eidx
        );
        assert_eq!(seg.name, splt[0].trim(), "incorrect name");
        let eqts = ephem.interpolator_as_equal_time_steps().unwrap();

        for (sidx, spline) in eqts.splines().unwrap().iter().enumerate() {
            let anise_x = spline.x().unwrap();
            assert_eq!(
                anise_x.len(),
                seg_data[sidx].x_coeffs.len(),
                "invalid number of X coeffs for target {}, spline idx {}",
                seg_target_id,
                sidx
            );
            // Check that the data strictly matches
            for (cidx, x) in anise_x.iter().enumerate() {
                assert!((x - seg_data[sidx].x_coeffs[cidx]).abs() < EPSILON);
            }

            // Repeat for Y
            let anise_y = spline.y().unwrap();
            assert_eq!(
                anise_y.len(),
                seg_data[sidx].y_coeffs.len(),
                "invalid number of Y coeffs for target {}, spline idx {}",
                seg_target_id,
                sidx
            );
            // Check that the data strictly matches
            for (cidx, y) in anise_y.iter().enumerate() {
                assert!((y - seg_data[sidx].y_coeffs[cidx]).abs() < EPSILON);
            }

            // Repeat for Z
            let anise_z = spline.z().unwrap();
            assert_eq!(
                anise_z.len(),
                seg_data[sidx].z_coeffs.len(),
                "invalid number of Z coeffs for target {}, spline idx {}",
                seg_target_id,
                sidx
            );
            // Check that the data strictly matches
            for (cidx, z) in anise_z.iter().enumerate() {
                assert!((z - seg_data[sidx].z_coeffs[cidx]).abs() < EPSILON);
            }
        }
    }
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
