/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

/// For the DE440 and DE438s files, load them in CSPICE (using rust-spice) and in ANISE, compute NUM_QUERIES_PER_PAIR states between each pair of targets in each file (equally spaced for the duration of the ephemeris),
/// store all of the data in a parquet file, and analyze it to make sure that the errors in computation between CSPICE and ANISE are within bounds.
/// WARNING: This is an O(N*log(N)*K) test, where N is the number of items in each SPK file and K is the number of queries per pair, so it's ignored by default.
/// On my computer, in release mode, this test runs in 5.15 seconds.
#[test]
#[cfg(feature = "std")]
fn validate_jplde_translation() {
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
    use std::sync::Arc;

    // If the error is larger than this, we should fail immediately.
    const FAIL_POS_KM: f64 = 1e2;
    const FAIL_VEL_KM_S: f64 = 1e-1;
    // Number of queries we should do per pair of ephemerides
    const NUM_QUERIES_PER_PAIR: f64 = 1_000.0;
    const BATCH_SIZE: usize = 10_000;

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
        Field::new("File delta T (s)", DataType::Float64, false),
        Field::new("absolute error (km)", DataType::Float64, false),
        Field::new("relative error (km)", DataType::Float64, false),
    ]);

    let file = File::create("target/validation-test-results.parquet").unwrap();

    // Default writer properties
    let props = WriterProperties::builder().build();

    let mut writer = ArrowWriter::try_new(file, Arc::new(schema), Some(props)).unwrap();

    let mut num_comparisons: usize = 0;

    for de_name in &["de438s", "de440"] {
        let path = format!("data/{de_name}.bsp");
        // SPICE load
        spice::furnsh(&path);

        // ANISE load
        let buf = file_mmap!(path).unwrap();
        let spk = SPK::parse(&buf).unwrap();
        let ctx = Context::from_spk(&spk).unwrap();

        // We only have one SPK loaded, so we know what summary to go through
        let first_spk = ctx.spk_data[0].unwrap();

        let mut pairs = Vec::new();

        for ephem1 in first_spk.data_summaries {
            let j2000_ephem1 = Frame::from_ephem_j2000(ephem1.target_id);

            for ephem2 in first_spk.data_summaries {
                if ephem1.target_id == ephem2.target_id {
                    continue;
                }

                let key = if ephem1.target_id < ephem2.target_id {
                    (ephem1.target_id, ephem2.target_id)
                } else {
                    (ephem2.target_id, ephem1.target_id)
                };

                if pairs.contains(&key) {
                    // We're already handled that pair
                    continue;
                }
                pairs.push(key);

                let j2000_ephem2 = Frame::from_ephem_j2000(ephem2.target_id);

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

                let component = ["X", "Y", "Z", "VX", "VY", "VZ"];

                let mut batch_de_name = Vec::with_capacity(BATCH_SIZE);
                let mut batch_src_frm = Vec::with_capacity(BATCH_SIZE);
                let mut batch_dest_frm = Vec::with_capacity(BATCH_SIZE);
                let mut batch_comp = Vec::with_capacity(BATCH_SIZE);
                let mut batch_epoch = Vec::with_capacity(BATCH_SIZE);
                let mut batch_hops = Vec::with_capacity(BATCH_SIZE);
                let mut batch_abs = Vec::with_capacity(BATCH_SIZE);
                let mut batch_rel = Vec::with_capacity(BATCH_SIZE);

                for (i, epoch) in time_it.enumerate() {
                    let hops = ctx
                        .common_ephemeris_path(j2000_ephem1, j2000_ephem2, epoch)
                        .unwrap()
                        .0 as u8;

                    match ctx.translate_from_to_km_s_geometric(j2000_ephem1, j2000_ephem2, epoch) {
                        Ok(state) => {
                            num_comparisons += 1;
                            // Perform the same query in SPICE
                            let (spice_state, _) = spice::spkezr(
                                ephem1.spice_name().unwrap(),
                                epoch.to_et_seconds(),
                                "J2000",
                                "NONE",
                                ephem2.spice_name().unwrap(),
                            );

                            // Check component by component instead of rebuilding a Vector3 from the SPICE data
                            for i in 0..6 {
                                let (anise_value, max_err) = if i < 3 {
                                    (state.radius_km[i], FAIL_POS_KM)
                                } else {
                                    (state.velocity_km_s[i - 3], FAIL_VEL_KM_S)
                                };

                                // We don't look at the absolute error here, that's for the stats to show any skewness
                                let abs_err = abs_diff(anise_value, spice_state[i]);
                                let rel_err = rel_diff(anise_value, spice_state[i], EPSILON);

                                if !relative_eq!(anise_value, spice_state[i], epsilon = max_err) {
                                    // Always save the parquet file
                                    writer.close().unwrap();

                                    panic!(
                                        "{epoch:E}\t{}got = {:.16}\texp = {:.16}\terr = {:.16}",
                                        component[i], anise_value, spice_state[i], rel_err
                                    );
                                }

                                // Update data

                                batch_de_name.push(de_name.clone());
                                batch_src_frm.push(j2000_ephem1.to_string());
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
                                                "DE file",
                                                Arc::new(StringArray::from(batch_de_name.clone()))
                                                    as ArrayRef,
                                            ),
                                            (
                                                "source frame",
                                                Arc::new(StringArray::from(batch_src_frm.clone()))
                                                    as ArrayRef,
                                            ),
                                            (
                                                "destination frame",
                                                Arc::new(StringArray::from(batch_dest_frm.clone()))
                                                    as ArrayRef,
                                            ),
                                            (
                                                "# hops",
                                                Arc::new(UInt8Array::from(batch_hops.clone()))
                                                    as ArrayRef,
                                            ),
                                            (
                                                "component",
                                                Arc::new(StringArray::from(batch_comp.clone()))
                                                    as ArrayRef,
                                            ),
                                            (
                                                "File delta T (s)",
                                                Arc::new(Float64Array::from(batch_epoch.clone()))
                                                    as ArrayRef,
                                            ),
                                            (
                                                "absolute error (km)",
                                                Arc::new(Float64Array::from(batch_abs.clone()))
                                                    as ArrayRef,
                                            ),
                                            (
                                                "relative error (km)",
                                                Arc::new(Float64Array::from(batch_rel.clone()))
                                                    as ArrayRef,
                                            ),
                                        ])
                                        .unwrap(),
                                    )
                                    .unwrap();
                                // Re-init all of the vectors
                                batch_de_name = Vec::with_capacity(BATCH_SIZE);
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
                                "File delta T (s)",
                                Arc::new(Float64Array::from(batch_epoch)) as ArrayRef,
                            ),
                            (
                                "absolute error (km)",
                                Arc::new(Float64Array::from(batch_abs)) as ArrayRef,
                            ),
                            (
                                "relative error (km)",
                                Arc::new(Float64Array::from(batch_rel)) as ArrayRef,
                            ),
                        ])
                        .unwrap(),
                    )
                    .unwrap();

                // Regularly flush to not lose data
                writer.flush().unwrap();
            }

            info!("[{de_name}] done with {}", j2000_ephem1);
        }
        // Unload SPICE (note that this is not needed for ANISE because it falls out of scope)
        spice::unload(&format!("data/{de_name}.bsp"));
    }
    // Save the parquet file
    writer.close().unwrap();

    info!("done with {num_comparisons} comparisons");

    // And now, analyze the parquet file!

    let df = LazyFrame::scan_parquet("target/validation-test-results.parquet", Default::default())
        .unwrap();

    const TYPICAL_REL_ERR_KM: f64 = 1e-7; // Allow up to 100 micrometers of error.
    const MAX_REL_ERR_KM: f64 = 1e-5; // Allow up to 1 centimeter of error.

    let rel_errors = df
        .clone()
        .select([
            min("relative error (km)").alias("min rel err (km) OK"),
            col("relative error (km)")
                .quantile(0.25, QuantileInterpolOptions::Higher)
                .alias("q25 rel err (km) OK"),
            col("relative error (km)")
                .mean()
                .alias("mean rel err (km) OK"),
            col("relative error (km)")
                .median()
                .alias("median rel err (km) OK"),
            col("relative error (km)")
                .quantile(0.75, QuantileInterpolOptions::Higher)
                .alias("q75 rel err (km) OK"),
            col("relative error (km)")
                .quantile(0.99, QuantileInterpolOptions::Higher)
                .alias("q99 rel err (km) OK"),
            max("relative error (km)").alias("max rel err (km) OK"),
        ])
        .collect()
        .unwrap();
    println!("{}", rel_errors);

    let rel_errors_ok = df
        .clone()
        .select([
            min("relative error (km)")
                .alias("min rel err (km) OK")
                .lt(TYPICAL_REL_ERR_KM),
            col("relative error (km)")
                .quantile(0.25, QuantileInterpolOptions::Higher)
                .alias("q25 rel err (km) OK")
                .lt(TYPICAL_REL_ERR_KM),
            col("relative error (km)")
                .mean()
                .alias("mean rel err (km) OK")
                .lt(TYPICAL_REL_ERR_KM),
            col("relative error (km)")
                .median()
                .alias("median rel err (km) OK")
                .lt(TYPICAL_REL_ERR_KM),
            col("relative error (km)")
                .quantile(0.75, QuantileInterpolOptions::Higher)
                .alias("q75 rel err (km) OK")
                .lt(TYPICAL_REL_ERR_KM),
            col("relative error (km)")
                .quantile(0.99, QuantileInterpolOptions::Higher)
                .alias("q99 rel err (km) OK")
                .lt(MAX_REL_ERR_KM),
            max("relative error (km)")
                .alias("max rel err (km) OK")
                .lt(MAX_REL_ERR_KM),
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
            min("absolute error (km)").alias("min abs err (km)"),
            col("absolute error (km)")
                .quantile(0.25, QuantileInterpolOptions::Higher)
                .alias("q25 abs err (km)"),
            col("absolute error (km)").mean().alias("mean abs err (km)"),
            col("absolute error (km)")
                .median()
                .alias("median abs err (km)"),
            col("absolute error (km)")
                .quantile(0.75, QuantileInterpolOptions::Higher)
                .alias("q75 abs err (km)"),
            col("absolute error (km)")
                .quantile(0.99, QuantileInterpolOptions::Higher)
                .alias("q99 abs err (km)"),
            max("absolute error (km)").alias("max abs err (km)"),
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
        .filter(col("absolute error (km)").gt(lit(q99_abs)))
        .select([
            col("absolute error (km)"),
            col("relative error (km)"),
            col("File delta T (s)"),
            col("DE file"),
            col("source frame"),
            col("destination frame"),
            max("component"),
        ])
        .collect()
        .unwrap();
    println!("{}", outliers);

    let outfile = "target/validation-outliers.parquet";
    let mut file = std::fs::File::create(outfile).unwrap();
    ParquetWriter::new(&mut file).finish(&mut outliers).unwrap();

    info!("saved outliers to {outfile}");
}
