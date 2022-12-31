/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use polars::prelude::LazyFrame;
use test_context::TestContext;

/// All validation of ANISE computations compared to SPICE must implement the Validator.
///
/// This allows running the validation, outputting all of the data into a Parquet file for post-analysis, and also validating the input.
pub trait Validator: TestContext + Iterator<Item = Self::Data> {
    type Data;
    fn output_file_name<'a>(&self) -> &'a str;
    fn validate(&self, df: LazyFrame);
}

pub mod ephemeris;

#[test]
fn demo() {}
