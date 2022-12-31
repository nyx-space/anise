/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

#[test]
#[cfg(feature = "std")]
fn validate_hermite_translation() {
    use anise::constants::frames::EARTH_J2000;
    // ANISE imports
    use anise::file_mmap;
    use anise::math::utils::{abs_diff, rel_diff};
    use anise::prelude::*;
    // Test imports
    use arrow::array::{ArrayRef, Float64Array, StringArray, UInt8Array};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use core::f64::EPSILON;
    use hifitime::{TimeSeries, TimeUnits};
    use log::error;
    use log::info;
    use parquet::arrow::arrow_writer::ArrowWriter;
    use parquet::file::properties::WriterProperties;
    use polars::prelude::*;
    use spice;
    use std::fs::File;
    use std::sync::Arc;

    use crate::ephemerides::consts::{
        MAX_ABS_POS_ERR_KM, MAX_ABS_VEL_ERR_KM_S, MAX_REL_POS_ERR_KM, TYPICAL_REL_POS_ERR_KM,
    };

    // Number of queries we should do per pair of ephemerides
    const NUM_QUERIES_PER_PAIR: f64 = 1_000.0;
    const BATCH_SIZE: usize = 10_000;

    if pretty_env_logger::try_init().is_err() {
        println!("could not init env_logger");
    }

    // Output parquet file

    // Build the schema
    let schema = Schema::new(vec![
        Field::new("source frame", DataType::Utf8, false),
        Field::new("destination frame", DataType::Utf8, false),
        Field::new("# hops", DataType::UInt8, false),
        Field::new("component", DataType::Utf8, false),
        Field::new("File delta T (s)", DataType::Float64, false),
        Field::new("absolute error", DataType::Float64, false),
        Field::new("relative error", DataType::Float64, false),
    ]);

    let file = File::create("target/type13-validation-test-results.parquet").unwrap();

    // Default writer properties
    let props = WriterProperties::builder().build();

    let mut writer = ArrowWriter::try_new(file, Arc::new(schema), Some(props)).unwrap();

    let mut num_comparisons: usize = 0;

    let de_path = format!("data/de440.bsp");
    let hermite_path = format!("data/gmat-hermite.bsp");
    let sc_naif_id = -10000001;
    // let hermite_path = format!("/home/chris/Downloads/DefaultLEOSatelliteType13Hermite.bsp");
    // let sc_naif_id = -200000;
    // SPICE load
    spice::furnsh(&hermite_path.clone());

    // ANISE load
    let de_path2 = de_path.clone();
    let hermite_path2 = hermite_path.clone();
    let buf = file_mmap!(de_path2).unwrap();
    let de_spk = SPK::parse(&buf).unwrap();
    let hermite_buf = file_mmap!(hermite_path2).unwrap();
    let hermite_spk = SPK::parse(&hermite_buf).unwrap();
    let ctx = Context::from_spk(&de_spk)
        .unwrap()
        .load_spk(&hermite_spk)
        .unwrap();
    println!("{ctx}");

    // We loaded the spacecraft second, so we're just grabbing that directly, but really, users should not be doing that.
    let sc_spk = ctx.spk_data[1]
        .unwrap()
        .summary_from_id(sc_naif_id)
        .unwrap()
        .0;

    let j2000_spacecraft = Frame::from_ephem_j2000(sc_naif_id); // From GMAT setup

    let j2000_ephem2 = EARTH_J2000;

    // Query the ephemeris data for a bunch of different times.
    let start_epoch = sc_spk.start_epoch();

    let end_epoch = sc_spk.end_epoch();

    let time_step = ((end_epoch - start_epoch).to_seconds() / NUM_QUERIES_PER_PAIR).seconds();

    let time_it = TimeSeries::exclusive(start_epoch, end_epoch - time_step, time_step);

    let component = ["X", "Y", "Z", "VX", "VY", "VZ"];

    let mut batch_src_frm = Vec::with_capacity(BATCH_SIZE);
    let mut batch_dest_frm = Vec::with_capacity(BATCH_SIZE);
    let mut batch_comp = Vec::with_capacity(BATCH_SIZE);
    let mut batch_epoch = Vec::with_capacity(BATCH_SIZE);
    let mut batch_hops = Vec::with_capacity(BATCH_SIZE);
    let mut batch_abs = Vec::with_capacity(BATCH_SIZE);
    let mut batch_rel = Vec::with_capacity(BATCH_SIZE);

    for (i, epoch) in time_it.enumerate() {
        let hops = ctx
            .common_ephemeris_path(j2000_ephem2, j2000_spacecraft, epoch)
            .unwrap()
            .0 as u8;

        match ctx.translate_from_to_km_s_geometric(j2000_ephem2, j2000_spacecraft, epoch) {
            Ok(state) => {
                num_comparisons += 1;
                // Perform the same query in SPICE
                let (spice_state, _) = spice::spkezr(
                    "EARTH",
                    epoch.to_et_seconds(),
                    "J2000",
                    "NONE",
                    &format!("{}", sc_naif_id),
                );

                // Check component by component instead of rebuilding a Vector3 from the SPICE data
                for i in 0..6 {
                    let (anise_value, max_err) = if i < 3 {
                        (state.radius_km[i], MAX_ABS_POS_ERR_KM)
                    } else {
                        (state.velocity_km_s[i - 3], MAX_ABS_VEL_ERR_KM_S)
                    };

                    // We don't look at the absolute error here, that's for the stats to show any skewness
                    let abs_err = abs_diff(anise_value, spice_state[i]);
                    let rel_err = rel_diff(anise_value, spice_state[i], EPSILON);

                    if !relative_eq!(anise_value, spice_state[i], epsilon = max_err) {
                        // Always save the parquet file
                        // writer.close().unwrap();

                        println!(
                            "{epoch:E}\t{}got = {:.16}\texp = {:.16}\terr = {:.16}",
                            component[i], anise_value, spice_state[i], rel_err
                        );
                    }

                    // Update data

                    batch_src_frm.push(j2000_spacecraft.to_string());
                    batch_dest_frm.push(j2000_ephem2.to_string());
                    batch_hops.push(hops);
                    batch_comp.push(component[i]);
                    batch_epoch.push((epoch - start_epoch).to_seconds());
                    batch_abs.push(abs_err);
                    batch_rel.push(rel_err);
                }

                // Consider writing the batch
                if i % BATCH_SIZE == 0 {
                    writer
                        .write(
                            &RecordBatch::try_from_iter(vec![
                                (
                                    "source frame",
                                    Arc::new(StringArray::from(batch_src_frm.clone())) as ArrayRef,
                                ),
                                (
                                    "destination frame",
                                    Arc::new(StringArray::from(batch_dest_frm.clone())) as ArrayRef,
                                ),
                                (
                                    "# hops",
                                    Arc::new(UInt8Array::from(batch_hops.clone())) as ArrayRef,
                                ),
                                (
                                    "component",
                                    Arc::new(StringArray::from(batch_comp.clone())) as ArrayRef,
                                ),
                                (
                                    "File delta T (s)",
                                    Arc::new(Float64Array::from(batch_epoch.clone())) as ArrayRef,
                                ),
                                (
                                    "absolute error",
                                    Arc::new(Float64Array::from(batch_abs.clone())) as ArrayRef,
                                ),
                                (
                                    "relative error",
                                    Arc::new(Float64Array::from(batch_rel.clone())) as ArrayRef,
                                ),
                            ])
                            .unwrap(),
                        )
                        .unwrap();
                    // Re-init all of the vectors
                    batch_src_frm = Vec::with_capacity(BATCH_SIZE);
                    batch_dest_frm = Vec::with_capacity(BATCH_SIZE);
                    batch_comp = Vec::with_capacity(BATCH_SIZE);
                    batch_epoch = Vec::with_capacity(BATCH_SIZE);
                    batch_hops = Vec::with_capacity(BATCH_SIZE);
                    batch_abs = Vec::with_capacity(BATCH_SIZE);
                    batch_rel = Vec::with_capacity(BATCH_SIZE);
                }
            }
            Err(e) => {
                // Always save the parquet file
                // writer.close().unwrap();
                error!("At epoch {epoch:E}: {e}");
            }
        };
    }

    writer
        .write(
            &RecordBatch::try_from_iter(vec![
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
                    "File delta T (s)",
                    Arc::new(Float64Array::from(batch_epoch)) as ArrayRef,
                ),
                (
                    "absolute error",
                    Arc::new(Float64Array::from(batch_abs)) as ArrayRef,
                ),
                (
                    "relative error",
                    Arc::new(Float64Array::from(batch_rel)) as ArrayRef,
                ),
            ])
            .unwrap(),
        )
        .unwrap();

    // Regularly flush to not lose data
    writer.flush().unwrap();

    // Unload SPICE (note that this is not needed for ANISE because it falls out of scope)
    // spice::unload(&format!("data/{hermite_path}.bsp"));
    spice::unload(&de_path);
    spice::unload(&hermite_path);

    // Save the parquet file
    writer.close().unwrap();

    info!("done with {num_comparisons} comparisons");

    // And now, analyze the parquet file!

    let df = LazyFrame::scan_parquet(
        "target/type13-validation-test-results.parquet",
        Default::default(),
    )
    .unwrap();

    let rel_errors = df
        .clone()
        .select([
            min("relative error").alias("min rel err (km) OK"),
            col("relative error")
                .quantile(0.25, QuantileInterpolOptions::Higher)
                .alias("q25 rel err (km) OK"),
            col("relative error").mean().alias("mean rel err (km) OK"),
            col("relative error")
                .median()
                .alias("median rel err (km) OK"),
            col("relative error")
                .quantile(0.75, QuantileInterpolOptions::Higher)
                .alias("q75 rel err (km) OK"),
            col("relative error")
                .quantile(0.99, QuantileInterpolOptions::Higher)
                .alias("q99 rel err (km) OK"),
            max("relative error").alias("max rel err (km) OK"),
        ])
        .collect()
        .unwrap();
    println!("{}", rel_errors);

    let rel_errors_ok = df
        .clone()
        .select([
            min("relative error")
                .alias("min rel err (km) OK")
                .lt(TYPICAL_REL_POS_ERR_KM),
            col("relative error")
                .quantile(0.25, QuantileInterpolOptions::Higher)
                .alias("q25 rel err (km) OK")
                .lt(TYPICAL_REL_POS_ERR_KM),
            col("relative error")
                .mean()
                .alias("mean rel err (km) OK")
                .lt(TYPICAL_REL_POS_ERR_KM),
            col("relative error")
                .median()
                .alias("median rel err (km) OK")
                .lt(TYPICAL_REL_POS_ERR_KM),
            col("relative error")
                .quantile(0.75, QuantileInterpolOptions::Higher)
                .alias("q75 rel err (km) OK")
                .lt(TYPICAL_REL_POS_ERR_KM),
            col("relative error")
                .quantile(0.99, QuantileInterpolOptions::Higher)
                .alias("q99 rel err (km) OK")
                .lt(MAX_REL_POS_ERR_KM),
            max("relative error")
                .alias("max rel err (km) OK")
                .lt(MAX_REL_POS_ERR_KM),
        ])
        .collect()
        .unwrap();

    for item in rel_errors_ok.get_row(0).0 {
        match item {
            AnyValue::Boolean(val) => assert!(val),
            _ => panic!("expected a boolean in the DataFrame column"),
        }
    }

    let abs_errors = df
        .clone()
        .select([
            // Absolute error
            min("absolute error").alias("min abs err (km)"),
            col("absolute error")
                .quantile(0.25, QuantileInterpolOptions::Higher)
                .alias("q25 abs err (km)"),
            col("absolute error").mean().alias("mean abs err (km)"),
            col("absolute error").median().alias("median abs err (km)"),
            col("absolute error")
                .quantile(0.75, QuantileInterpolOptions::Higher)
                .alias("q75 abs err (km)"),
            col("absolute error")
                .quantile(0.99, QuantileInterpolOptions::Higher)
                .alias("q99 abs err (km)"),
            max("absolute error").alias("max abs err (km)"),
        ])
        .collect()
        .unwrap();
    println!("{}", abs_errors);

    // For debugging purposes, print all of the q99 errors
    let q99_abs = match abs_errors.get_row(0).0[5] {
        AnyValue::Float64(val) => val,
        _ => unreachable!(),
    };

    let mut outliers = df
        .filter(col("absolute error").gt(lit(q99_abs)))
        .select([
            col("absolute error"),
            col("relative error"),
            col("File delta T (s)"),
            col("source frame"),
            col("destination frame"),
            max("component"),
        ])
        .collect()
        .unwrap();
    println!("{}", outliers);

    let outfile = "target/type13-validation-outliers.parquet";
    let mut file = std::fs::File::create(outfile).unwrap();
    ParquetWriter::new(&mut file).finish(&mut outliers).unwrap();

    info!("saved outliers to {outfile}");
}
