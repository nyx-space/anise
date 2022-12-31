use super::Validator;

/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use arrow::datatypes::{DataType, Field, Schema};
use parquet::{arrow::ArrowWriter, file::properties::WriterProperties};
use std::{fs::File, sync::Arc};
use test_context::TestContext;

const BATCH_SIZE: usize = 10_000;

pub struct EphemValData {
    pub src_frame: String,
    pub dst_frame: String,
    pub component: String,
    pub spice_val: f64,
    pub anise_val: f64,
}

pub struct EphemerisValidator<V: Validator<Data = EphemValData>> {
    pub validator: V,
    pub batch_src_frame: Vec<String>,
    pub batch_dst_frame: Vec<String>,
    pub batch_component: Vec<String>,
    pub batch_spice_val: Vec<f64>,
    pub batch_anise_val: Vec<f64>,
}

impl<V: Validator<Data = EphemValData>> TestContext for EphemerisValidator<V> {
    fn setup() -> Self {
        let validator = V::setup();
        Self {
            validator,
            batch_src_frame: Vec::with_capacity(BATCH_SIZE),
            batch_dst_frame: Vec::with_capacity(BATCH_SIZE),
            batch_component: Vec::with_capacity(BATCH_SIZE),
            batch_spice_val: Vec::with_capacity(BATCH_SIZE),
            batch_anise_val: Vec::with_capacity(BATCH_SIZE),
        }
    }
}

impl<V: Validator<Data = EphemValData>> EphemerisValidator<V> {
    /// Executes this ephemeris validation
    pub fn execute(&self) {
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

        let file = File::create(format!(
            "target/{}.parquet",
            self.validator.output_file_name()
        ))
        .unwrap();

        // Default writer properties
        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, Arc::new(schema), Some(props)).unwrap();
    }
}
