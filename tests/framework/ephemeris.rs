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

use arrow::{
    array::{ArrayRef, Float64Array, StringArray, UInt8Array},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use parquet::{arrow::ArrowWriter, file::properties::WriterProperties};
use std::{fs::File, sync::Arc};
use test_context::TestContext;

const BATCH_SIZE: usize = 10_000;

pub struct EphemValData {
    pub src_frame: String,
    pub dst_frame: String,
    pub component: String,
    pub epoch_offset: f64,
    pub spice_val: f64,
    pub anise_val: f64,
}

pub struct EphemerisValidator<V: Validator<Data = EphemValData>> {
    pub validator: V,
    pub batch_src_frame: Vec<String>,
    pub batch_dst_frame: Vec<String>,
    pub batch_component: Vec<String>,
    pub batch_epoch_offset: Vec<f64>,
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
            batch_epoch_offset: Vec::with_capacity(BATCH_SIZE),
            batch_spice_val: Vec::with_capacity(BATCH_SIZE),
            batch_anise_val: Vec::with_capacity(BATCH_SIZE),
        }
    }
}

impl<V: Validator<Data = EphemValData>> EphemerisValidator<V> {
    /// Executes this ephemeris validation
    pub fn execute(&mut self) {
        // Build the schema
        let schema = Schema::new(vec![
            Field::new("source frame", DataType::Utf8, false),
            Field::new("destination frame", DataType::Utf8, false),
            Field::new("component", DataType::Utf8, false),
            Field::new("File delta T (s)", DataType::Float64, false),
            Field::new("SPICE value", DataType::Float64, false),
            Field::new("ANISE value", DataType::Float64, false),
        ]);

        let file = File::create(format!(
            "target/{}.parquet",
            self.validator.output_file_name()
        ))
        .unwrap();

        // Default writer properties
        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, Arc::new(schema), Some(props)).unwrap();

        // Enumeration on the validator shall return the next item.
        for (i, data) in (&mut self.validator).enumerate() {
            self.batch_src_frame.push(data.src_frame);
            self.batch_dst_frame.push(data.dst_frame);
            self.batch_component.push(data.component);
            self.batch_epoch_offset.push(data.epoch_offset);
            self.batch_spice_val.push(data.spice_val);
            self.batch_anise_val.push(data.anise_val);

            // Consider writing the batch
            if i % BATCH_SIZE == 0 {
                self.persist();
            }
        }
    }

    fn persist(&mut self) {
        // TODO: Move the write into `self`.
        writer
            .write(
                &RecordBatch::try_from_iter(vec![
                    (
                        "source frame",
                        Arc::new(StringArray::from(self.batch_src_frame.clone())) as ArrayRef,
                    ),
                    (
                        "destination frame",
                        Arc::new(StringArray::from(self.batch_dst_frame.clone())) as ArrayRef,
                    ),
                    (
                        "component",
                        Arc::new(StringArray::from(self.batch_component.clone())) as ArrayRef,
                    ),
                    (
                        "File delta T (s)",
                        Arc::new(Float64Array::from(self.batch_epoch_offset.clone())) as ArrayRef,
                    ),
                    (
                        "SPICE value",
                        Arc::new(Float64Array::from(self.batch_spice_val.clone())) as ArrayRef,
                    ),
                    (
                        "ANISE value",
                        Arc::new(Float64Array::from(self.batch_anise_val.clone())) as ArrayRef,
                    ),
                ])
                .unwrap(),
            )
            .unwrap();

        // Re-init all of the vectors
        self.batch_src_frame = Vec::with_capacity(BATCH_SIZE);
        self.batch_dst_frame = Vec::with_capacity(BATCH_SIZE);
        self.batch_component = Vec::with_capacity(BATCH_SIZE);
        self.batch_epoch_offset = Vec::with_capacity(BATCH_SIZE);
        self.batch_spice_val = Vec::with_capacity(BATCH_SIZE);
        self.batch_anise_val = Vec::with_capacity(BATCH_SIZE);
    }
}
