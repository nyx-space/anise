/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use polars::{lazy::dsl::Expr, prelude::*};

#[derive(Debug, Default)]
pub struct Validation {
    pub file_name: String,
    pub max_q75_err: f64,
    pub max_q99_err: f64,
    pub max_abs_err: f64,
}

impl Validation {
    /// Computes the quantiles of the absolute errors in the Parquet file and asserts these are within the bounds of the validation.
    pub fn validate(&self) {
        // Open the parquet file with all the data
        let df = LazyFrame::scan_parquet(
            format!("../target/{}.parquet", self.file_name),
            Default::default(),
        )
        .unwrap();

        let abs_errors = df
            .clone()
            .select([
                // Absolute difference
                min("Absolute difference").alias("min abs err"),
                col("Absolute difference")
                    .quantile(
                        Expr::Literal(polars::prelude::LiteralValue::Float64(0.25)),
                        QuantileInterpolOptions::Higher,
                    )
                    .alias("q25 abs err"),
                col("Absolute difference").mean().alias("mean abs err"),
                col("Absolute difference").median().alias("median abs err"),
                col("Absolute difference")
                    .quantile(
                        Expr::Literal(polars::prelude::LiteralValue::Float64(0.75)),
                        QuantileInterpolOptions::Higher,
                    )
                    .alias("q75 abs err"),
                col("Absolute difference")
                    .quantile(
                        Expr::Literal(polars::prelude::LiteralValue::Float64(0.99)),
                        QuantileInterpolOptions::Higher,
                    )
                    .alias("q99 abs err"),
                max("Absolute difference").alias("max abs err"),
            ])
            .collect()
            .unwrap();
        println!("{}", abs_errors);

        // Validate results

        // q75
        let err = match abs_errors.get_row(0).unwrap().0[4] {
            AnyValue::Float64(val) => val,
            _ => unreachable!(),
        };

        assert!(
            err <= self.max_q75_err,
            "q75 of absolute error is {err} > {}",
            self.max_q75_err
        );

        // q99
        let err = match abs_errors.get_row(0).unwrap().0[5] {
            AnyValue::Float64(val) => val,
            _ => unreachable!(),
        };

        assert!(
            err <= self.max_q99_err,
            "q99 of absolute error is {err} > {}",
            self.max_q99_err
        );

        // max abs err
        let err = match abs_errors.get_row(0).unwrap().0[6] {
            AnyValue::Float64(val) => val,
            _ => unreachable!(),
        };

        assert!(
            err <= self.max_abs_err,
            "maximum absolute error is {err} > {}",
            self.max_abs_err
        );
    }
}
