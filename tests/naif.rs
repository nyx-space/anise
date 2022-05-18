/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::convert::TryInto;

use anise::{
    asn1::{root::TrajectoryFile, spline::SplineKind},
    naif::{
        daf::{Endianness, DAF},
        spk::SPK,
    },
    prelude::*,
};
use der::Decode;

#[test]
fn test_spk_load() {
    // Using the DE421 as demo because the correct data is in the DAF documentation
    let bsp_path = "data/de421.bsp";
    // let filename = "data/de440.bsp";
    let bytes = file_mmap!(bsp_path).unwrap();

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

    let (seg, meta) = spk.segment_ptr(301).unwrap();
    assert_eq!(
        seg.start_idx, 944041,
        "Invalid start of coeff index for DE421"
    );
    assert_eq!(
        meta.interval_length, 345600,
        "Invalid interval length (in seconds) for DE421"
    );
    assert_eq!(meta.rsize, 41, "Invalid rsize for DE421");
    assert_eq!(
        meta.num_records_in_seg, 14080,
        "Invalid num_records_in_seg for DE421"
    );
    assert!(
        (meta.init_s_past_j2k - -3169195200.0).abs() < 2e-16,
        "Invalid start time"
    );

    spk.all_coefficients(301).unwrap();
    // Build the ANISE file
    // TODO: Compute the checksum and make sure it's correct
    let filename_anis = "de421.anis";
    // spk.to_anise(filename, filename_anis);
    spk.to_anise(bsp_path, filename_anis);
    // Load this ANIS file and make sure that it matches the original DE421 data.

    let bytes = file_mmap!(filename_anis).unwrap();
    let ctx = TrajectoryFile::from_der(&bytes).unwrap();
    assert_eq!(
        ctx.ephemeris_lut.hashes.len(),
        spk.segments.len(),
        "Incorrect number of ephem in map"
    );
    assert_eq!(
        ctx.ephemeris_lut.indexes.len(),
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

    for (eidx, ephem) in ctx.ephemeris_data.iter().enumerate() {
        let splt = ephem.name.split("#").collect::<Vec<&str>>();
        let seg_target_id = str::parse::<i32>(splt[1]).unwrap();
        // Fetch the SPK segment
        let (seg, meta, all_seg_data) = spk.all_coefficients(seg_target_id).unwrap();
        if all_seg_data.is_empty() {
            continue;
        }
        assert_eq!(
            all_seg_data.len(),
            seg_len[eidx],
            "wrong number of segments for {}",
            eidx
        );
        assert_eq!(seg.name, splt[0].trim(), "incorrect name");

        let splines = &ephem.splines;
        match splines.kind {
            SplineKind::FixedWindow { window_duration_s } => {
                assert_eq!(
                    window_duration_s, meta.interval_length as f64,
                    "incorrect interval duration"
                );
            }
            _ => panic!("wrong spline kind"),
        };

        assert_eq!(splines.config.num_position_coeffs, 3);
        assert_eq!(splines.config.num_position_dt_coeffs, 0);
        assert_eq!(splines.config.num_velocity_coeffs, 0);
        assert_eq!(splines.config.num_velocity_dt_coeffs, 0);
        assert_eq!(splines.config.num_epochs, 0);

        for (sidx, seg_data) in all_seg_data.iter().enumerate() {
            for (cidx, x_truth) in seg_data.x_coeffs.iter().enumerate() {
                assert_eq!(
                    splines
                        .fetch(sidx, cidx, anise::asn1::spline::Coefficient::X)
                        .unwrap(),
                    *x_truth
                );
            }

            for (cidx, y_truth) in seg_data.y_coeffs.iter().enumerate() {
                assert_eq!(
                    splines
                        .fetch(sidx, cidx, anise::asn1::spline::Coefficient::Y)
                        .unwrap(),
                    *y_truth
                );
            }

            for (cidx, z_truth) in seg_data.z_coeffs.iter().enumerate() {
                assert_eq!(
                    splines
                        .fetch(sidx, cidx, anise::asn1::spline::Coefficient::Z)
                        .unwrap(),
                    *z_truth
                );
            }

            // assert_eq!(
            //     x_coeffs.len(),
            //     seg_data.x_coeffs.len(),
            //     "invalid number of X coeffs for target {}, spline idx {}",
            //     seg_target_id,
            //     sidx
            // );
            // Check that the data strictly matches
            // for (cidx, x) in x_coeffs.iter().enumerate() {
            //     spk_f64 = all_seg_data[sidx].x_coeffs[cidx]
            //     assert!((*x - all_seg_data[sidx].x_coeffs[cidx]).abs() < f64::EPSILON);
            // }
            // idx += 1;
            // let y_coeffs = &spline[idx * degree..(idx + 1) * degree];
            // assert_eq!(
            //     y_coeffs.len(),
            //     seg_data.y_coeffs.len(),
            //     "invalid number of y coeffs for target {}, spline idx {}",
            //     seg_target_id,
            //     sidx
            // );
            // idx += 1;
            // let z_coeffs = &spline[idx * degree..(idx + 1) * degree];
            // assert_eq!(
            //     z_coeffs.len(),
            //     seg_data.z_coeffs.len(),
            //     "invalid number of z coeffs for target {}, spline idx {}",
            //     seg_target_id,
            //     sidx
            // );
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
