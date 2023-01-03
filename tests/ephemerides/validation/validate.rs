/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::ephemerides::consts::*;
use log::info;
use polars::prelude::*;

pub struct Validation {
    pub file_name: String,
}

impl Validation {
    pub fn validate(&self) {
        // Open the parquet file with all the data
        let df = LazyFrame::scan_parquet(
            format!("target/{}.parquet", self.file_name),
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
}
