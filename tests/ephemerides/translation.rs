/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::f64::EPSILON;

use anise::constants::frames::{EARTH_MOON_BARYCENTER_J2000, LUNA_J2000, VENUS_J2000};
use anise::file_mmap;
use anise::math::Vector3;
use anise::prelude::*;

// For the Earth Moon Barycenter to Luna, there velocity error is up to 3e-14 km/s, or 3e-11 m/s, or 13 picometers per second.
const VELOCITY_EPSILON_KM_S: f64 = 1e-13;

#[test]
fn de438s_translation_verif_venus2emb() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.anise";
    let buf = file_mmap!(path).unwrap();
    let ctx: AniseContext = (&buf).try_into().unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de438s.bsp')
    >>> sp.furnsh('../../hifitime/naif0012.txt')
    >>> et = sp.utc2et('2002 FEB 07 00:00:00')
    >>> ['{:.16e}'.format(x) for x in sp.spkez(2, et, "J2000", "NONE", 3)[0]]
    ['2.0504464298094124e+08', '-1.3595802361226091e+08', '-6.5722791535179183e+07', '3.7012086122583923e+01', '4.8685441396743641e+01', '2.0519128283382937e+01']
    */

    dbg!(ctx
        .common_ephemeris_path(VENUS_J2000, EARTH_MOON_BARYCENTER_J2000)
        .unwrap());

    let state = ctx
        .translate_from_to(
            VENUS_J2000,
            EARTH_MOON_BARYCENTER_J2000,
            epoch,
            Aberration::None,
            DistanceUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    let pos_expct_km = Vector3::new(
        2.0504464298094124e+08,
        -1.3595802361226091e+08,
        -6.5722791535179183e+07,
    );

    let vel_expct_km_s = Vector3::new(
        3.7012086122583923e+01,
        4.8685441396743641e+01,
        2.0519128283382937e+01,
    );

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, pos_expct_km, epsilon = EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km - state.radius_km
    );

    assert!(
        relative_eq!(state.velocity_km_s, vel_expct_km_s, epsilon = EPSILON),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s - state.velocity_km_s
    );

    // Test the opposite translation
    let state = ctx
        .translate_from_to_km_s_geometric(EARTH_MOON_BARYCENTER_J2000, VENUS_J2000, epoch)
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, -pos_expct_km, epsilon = EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km + state.radius_km
    );

    assert!(
        relative_eq!(state.velocity_km_s, -vel_expct_km_s, epsilon = EPSILON),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s + state.velocity_km_s
    );
}

#[test]
fn de438s_translation_verif_venus2luna() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.anise";
    let buf = file_mmap!(path).unwrap();
    let ctx: AniseContext = (&buf).try_into().unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    // Venus to Earth Moon

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de438s.bsp')
    >>> sp.furnsh('../../hifitime/naif0012.txt')
    >>> et = sp.utc2et('2002 FEB 07 00:00:00')
    >>> ['{:.16e}'.format(x) for x in sp.spkez(2, et, "J2000", "NONE", 3)[0]]
    ['2.0512621957198492e+08', '-1.3561254792311624e+08', '-6.5578399676164642e+07', '3.6051374278187268e+01', '4.8889024622166957e+01', '2.0702933800840963e+01']
    >>> ['{:.16e}'.format(x) for x in sp.spkez(3, et, "J2000", "NONE", 301)[0]]
    ['8.1576591043659311e+04', '3.4547568914467981e+05', '1.4439185901453768e+05', '-9.6071184439665624e-01', '2.0358322542331578e-01', '1.8380551745802590e-01']
    */

    let state = ctx
        .translate_from_to(
            VENUS_J2000,
            LUNA_J2000,
            epoch,
            Aberration::None,
            DistanceUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    let pos_expct_km = Vector3::new(
        2.0512621957198492e+08,
        -1.3561254792311624e+08,
        -6.5578399676164642e+07,
    );

    let vel_expct_km_s = Vector3::new(
        3.6051374278187268e+01,
        4.8889024622166957e+01,
        2.0702933800840963e+01,
    );

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, pos_expct_km, epsilon = EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km - state.radius_km
    );

    assert!(
        relative_eq!(
            state.velocity_km_s,
            vel_expct_km_s,
            epsilon = VELOCITY_EPSILON_KM_S
        ),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s - state.velocity_km_s
    );

    // Test the opposite translation
    let state = ctx
        .translate_from_to_km_s_geometric(LUNA_J2000, VENUS_J2000, epoch)
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, -pos_expct_km, epsilon = EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km + state.radius_km
    );

    assert!(
        relative_eq!(
            state.velocity_km_s,
            -vel_expct_km_s,
            epsilon = VELOCITY_EPSILON_KM_S
        ),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s + state.velocity_km_s
    );
}

#[test]
fn de438s_translation_verif_emb2luna() {
    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // "Load" the file via a memory map (avoids allocations)
    let path = "./data/de438s.anise";
    let buf = file_mmap!(path).unwrap();
    let ctx: AniseContext = (&buf).try_into().unwrap();

    let epoch = Epoch::from_gregorian_utc_at_midnight(2002, 2, 7);

    // Earth Moon Barycenter to Earth Moon

    /*
    Python code:
    >>> import spiceypy as sp
    >>> sp.furnsh('data/de438s.bsp')
    >>> sp.furnsh('../../hifitime/naif0012.txt')
    >>> et = sp.utc2et('2002 FEB 07 00:00:00')
    >>> ['{:.16e}'.format(x) for x in sp.spkez(3, et, "J2000", "NONE", 301)[0]] # Target = 3; Obs = 301
    ['8.1576591043659311e+04', '3.4547568914467981e+05', '1.4439185901453768e+05', '-9.6071184439665624e-01', '2.0358322542331578e-01', '1.8380551745802590e-01']
    */

    let state = ctx
        .translate_from_to(
            EARTH_MOON_BARYCENTER_J2000,
            LUNA_J2000,
            epoch,
            Aberration::None,
            DistanceUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    let pos_expct_km = Vector3::new(
        8.1576591043659311e+04,
        3.4547568914467981e+05,
        1.4439185901453768e+05,
    );

    let vel_expct_km_s = Vector3::new(
        -9.6071184439665624e-01,
        2.0358322542331578e-01,
        1.8380551745802590e-01,
    );

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, pos_expct_km, epsilon = EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km - state.radius_km
    );

    assert!(
        relative_eq!(
            state.velocity_km_s,
            vel_expct_km_s,
            epsilon = VELOCITY_EPSILON_KM_S
        ),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s - state.velocity_km_s
    );

    // Try the opposite
    let state = ctx
        .translate_from_to(
            LUNA_J2000,
            EARTH_MOON_BARYCENTER_J2000,
            epoch,
            Aberration::None,
            DistanceUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    // We expect exactly the same output as SPICE to machine precision.
    assert!(
        relative_eq!(state.radius_km, -pos_expct_km, epsilon = EPSILON),
        "pos = {}\nexp = {pos_expct_km}\nerr = {:e}",
        state.radius_km,
        pos_expct_km + state.radius_km
    );

    assert!(
        relative_eq!(
            state.velocity_km_s,
            -vel_expct_km_s,
            epsilon = VELOCITY_EPSILON_KM_S
        ),
        "vel = {}\nexp = {vel_expct_km_s}\nerr = {:e}",
        state.velocity_km_s,
        vel_expct_km_s + state.velocity_km_s
    );
}

#[test]
#[ignore]
#[cfg(feature = "std")]
fn validate_jplde_translation() {
    use anise::astro::Frame;
    use arrow::array::{ArrayRef, Float64Array, StringArray, UInt8Array};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use hifitime::{TimeSeries, TimeUnits};
    use log::info;
    use online_statistics::maximum::Max;
    use online_statistics::mean::Mean;
    use online_statistics::minimum::Min;
    use online_statistics::quantile::Quantile;
    use online_statistics::stats::Univariate;
    use parquet::arrow::arrow_writer::ArrowWriter;
    use parquet::file::properties::WriterProperties;
    use spice;
    use std::fs::File;
    use std::sync::Arc;

    // If the error is larger than this, we should fail immediately.
    const FAIL_POS_KM: f64 = 1e2;
    const FAIL_VEL_KM_S: f64 = 1e-1;
    // Number of queries we should do per pair of ephemerides
    const NUM_QUERIES_PER_PAIR: f64 = 1_000.0;

    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // Output parquet file

    // Build the schema
    let schema = Schema::new(vec![
        Field::new("DE file", DataType::Utf8, false),
        Field::new("source frame", DataType::Utf8, false),
        Field::new("destination frame", DataType::Utf8, false),
        Field::new("# hops", DataType::UInt8, false),
        Field::new("component", DataType::Utf8, false),
        Field::new("mean err", DataType::Float64, false),
        Field::new("min err", DataType::Float64, false),
        Field::new("q25 err", DataType::Float64, false),
        Field::new("q50 err", DataType::Float64, false),
        Field::new("q75 err", DataType::Float64, false),
        Field::new("max err", DataType::Float64, false),
    ]);

    let file = File::create("target/validation-test-results.parquet").unwrap();

    // Default writer properties
    let props = WriterProperties::builder().build();

    let mut writer = ArrowWriter::try_new(file, Arc::new(schema), Some(props)).unwrap();

    for de_name in &["de438s", "de440"] {
        // SPICE load
        spice::furnsh(&format!("data/{de_name}.bsp"));

        // ANISE load
        let path = format!("./data/{de_name}.anise");
        let buf = file_mmap!(path).unwrap();
        let ctx = AniseContext::from_bytes(&buf);

        for (idx1, ephem1) in ctx.ephemeris_data.iter().enumerate() {
            let j2000_ephem1 =
                Frame::from_ephem_j2000(*ctx.ephemeris_lut.hashes.get(idx1).unwrap());

            for (idx2, ephem2) in ctx.ephemeris_data.iter().enumerate() {
                if ephem1 == ephem2 {
                    continue;
                }

                let j2000_ephem2 =
                    Frame::from_ephem_j2000(*ctx.ephemeris_lut.hashes.get(idx2).unwrap());

                // Query the ephemeris data for a bunch of different times.
                let start_epoch = if ephem1.start_epoch() < ephem2.start_epoch() {
                    ephem2.start_epoch()
                } else {
                    ephem1.start_epoch()
                };

                let end_epoch = if ephem1.end_epoch() < ephem2.end_epoch() {
                    ephem1.end_epoch()
                } else {
                    ephem2.end_epoch()
                };

                let time_step =
                    ((end_epoch - start_epoch).to_seconds() / NUM_QUERIES_PER_PAIR).seconds();

                let time_it = TimeSeries::exclusive(start_epoch, end_epoch - time_step, time_step);

                let mut maxes = [
                    Max::new(),
                    Max::new(),
                    Max::new(),
                    Max::new(),
                    Max::new(),
                    Max::new(),
                ];

                let mut mins = [
                    Min::new(),
                    Min::new(),
                    Min::new(),
                    Min::new(),
                    Min::new(),
                    Min::new(),
                ];

                let mut means = [
                    Mean::new(),
                    Mean::new(),
                    Mean::new(),
                    Mean::new(),
                    Mean::new(),
                    Mean::new(),
                ];

                let mut meds = [
                    Quantile::default(),
                    Quantile::default(),
                    Quantile::default(),
                    Quantile::default(),
                    Quantile::default(),
                    Quantile::default(),
                ];

                let mut q25s = [
                    Quantile::new(0.25).unwrap(),
                    Quantile::new(0.25).unwrap(),
                    Quantile::new(0.25).unwrap(),
                    Quantile::new(0.25).unwrap(),
                    Quantile::new(0.25).unwrap(),
                    Quantile::new(0.25).unwrap(),
                ];

                let mut q75s = [
                    Quantile::new(0.75).unwrap(),
                    Quantile::new(0.75).unwrap(),
                    Quantile::new(0.75).unwrap(),
                    Quantile::new(0.75).unwrap(),
                    Quantile::new(0.75).unwrap(),
                    Quantile::new(0.75).unwrap(),
                ];

                let component = ["X", "Y", "Z", "VX", "VY", "VZ"];

                for epoch in time_it {
                    match ctx.translate_from_to_km_s_geometric(j2000_ephem1, j2000_ephem2, epoch) {
                        Ok(state) => {
                            // Perform the same query in SPICE
                            let (spice_state, _) = spice::spkezr(
                                match ephem1.name {
                                    "Luna" => "Moon",
                                    _ => ephem1.name,
                                },
                                epoch.to_et_seconds(),
                                "J2000",
                                "NONE",
                                match ephem2.name {
                                    "Luna" => "Moon",
                                    _ => ephem2.name,
                                },
                            );

                            // Check component by component instead of rebuilding a Vector3 from the SPICE data
                            for i in 0..6 {
                                let (anise_value, max_err) = if i < 3 {
                                    (state.radius_km[i], FAIL_POS_KM)
                                } else {
                                    (state.velocity_km_s[i - 3], FAIL_VEL_KM_S)
                                };

                                // We don't look at the absolute error here, that's for the stats to show any skewness
                                let err = anise_value - spice_state[i];

                                if !relative_eq!(anise_value, spice_state[i], epsilon = max_err) {
                                    // Always save the parquet file
                                    writer.close().unwrap();

                                    panic!(
                                        "{epoch:E}\t{}got = {:.16}\texp = {:.16}\terr = {:.16}",
                                        component[i], anise_value, spice_state[i], err
                                    );
                                }

                                // Update statistics
                                mins[i].update(err);
                                maxes[i].update(err);
                                q25s[i].update(err);
                                q75s[i].update(err);
                                meds[i].update(err);
                                means[i].update(err);
                            }
                        }
                        Err(e) => {
                            // Always save the parquet file
                            writer.close().unwrap();
                            panic!("At epoch {epoch:E}: {e}");
                        }
                    };
                }

                let mut batch_de_name = Vec::with_capacity(6);
                let mut batch_src_frm = Vec::with_capacity(6);
                let mut batch_dest_frm = Vec::with_capacity(6);
                let mut batch_comp = Vec::with_capacity(6);
                let mut batch_hops = Vec::with_capacity(6);
                let mut batch_mean = Vec::with_capacity(6);
                let mut batch_min = Vec::with_capacity(6);
                let mut batch_max = Vec::with_capacity(6);
                let mut batch_q25 = Vec::with_capacity(6);
                let mut batch_q50 = Vec::with_capacity(6);
                let mut batch_q75 = Vec::with_capacity(6);

                for i in 0..6 {
                    batch_de_name.push(de_name.clone());
                    batch_src_frm.push(j2000_ephem1.to_string());
                    batch_dest_frm.push(j2000_ephem2.to_string());
                    batch_comp.push(component[i]);
                    batch_hops.push(
                        ctx.common_ephemeris_path(j2000_ephem1, j2000_ephem2)
                            .unwrap()
                            .0 as u8,
                    );
                    batch_mean.push(means[i].get());
                    batch_min.push(mins[i].get());
                    batch_max.push(maxes[i].get());
                    batch_q50.push(meds[i].get());
                    batch_q25.push(q25s[i].get());
                    batch_q75.push(q75s[i].get());
                }

                writer
                    .write(
                        &RecordBatch::try_from_iter(vec![
                            (
                                "DE file",
                                Arc::new(StringArray::from(batch_de_name)) as ArrayRef,
                            ),
                            (
                                "source frame",
                                Arc::new(StringArray::from(batch_src_frm)) as ArrayRef,
                            ),
                            (
                                "destination frame",
                                Arc::new(StringArray::from(batch_dest_frm)) as ArrayRef,
                            ),
                            ("# hops", Arc::new(UInt8Array::from(batch_hops)) as ArrayRef),
                            (
                                "component",
                                Arc::new(StringArray::from(batch_comp)) as ArrayRef,
                            ),
                            (
                                "mean err",
                                Arc::new(Float64Array::from(batch_mean)) as ArrayRef,
                            ),
                            (
                                "min err",
                                Arc::new(Float64Array::from(batch_min)) as ArrayRef,
                            ),
                            (
                                "q25 err",
                                Arc::new(Float64Array::from(batch_q25)) as ArrayRef,
                            ),
                            (
                                "q50 err",
                                Arc::new(Float64Array::from(batch_q50)) as ArrayRef,
                            ),
                            (
                                "q75 err",
                                Arc::new(Float64Array::from(batch_q75)) as ArrayRef,
                            ),
                            (
                                "max err",
                                Arc::new(Float64Array::from(batch_max)) as ArrayRef,
                            ),
                        ])
                        .unwrap(),
                    )
                    .unwrap();

                // Regularly flush to not lose data
                writer.flush().unwrap();
            }

            info!("[{de_name}] done with {} ", j2000_ephem1);
        }
        // Unload SPICE (note that this is not needed for ANISE because it falls out of scope)
        spice::unload(&format!("data/{de_name}.bsp"));
    }
    // Always save the parquet file
    writer.close().unwrap();
}
