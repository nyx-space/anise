/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::{naif::spk::summary::SPKSummaryRecord, prelude::*};
use arrow::{
    array::{ArrayRef, Float64Array, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use log::{error, info};
use parquet::{arrow::ArrowWriter, file::properties::WriterProperties};
use std::{collections::HashMap, fs::File, sync::Arc};

const COMPONENT: &[&str] = &["X", "Y", "Z", "VX", "VY", "VZ"];

// Number of items to keep in memory before flushing to the parquet file
const BATCH_SIZE: usize = 10_000;

#[derive(Default)]
pub struct EphemValData {
    pub src_frame: String,
    pub dst_frame: String,
    pub epoch_et_s: f64,
    pub spice_val_x_km: f64,
    pub anise_val_x_km: f64,
    pub spice_val_y_km: f64,
    pub anise_val_y_km: f64,
    pub spice_val_z_km: f64,
    pub anise_val_z_km: f64,

    pub spice_val_vx_km_s: f64,
    pub anise_val_vx_km_s: f64,
    pub spice_val_vy_km_s: f64,
    pub anise_val_vy_km_s: f64,
    pub spice_val_vz_km_s: f64,
    pub anise_val_vz_km_s: f64,
}

impl EphemValData {
    pub fn error(src_frame: String, dst_frame: String, epoch_et_s: f64) -> Self {
        Self {
            src_frame,
            dst_frame,
            epoch_et_s,
            spice_val_x_km: f64::INFINITY,
            anise_val_x_km: f64::INFINITY,
            spice_val_y_km: f64::INFINITY,
            anise_val_y_km: f64::INFINITY,
            spice_val_z_km: f64::INFINITY,
            anise_val_z_km: f64::INFINITY,
            spice_val_vx_km_s: f64::INFINITY,
            anise_val_vx_km_s: f64::INFINITY,
            spice_val_vy_km_s: f64::INFINITY,
            anise_val_vy_km_s: f64::INFINITY,
            spice_val_vz_km_s: f64::INFINITY,
            anise_val_vz_km_s: f64::INFINITY,
        }
    }
}

/// An ephemeris comparison tool that writes the differences between ephemerides to a Parquet file.
pub struct CompareEphem {
    pub input_file_names: Vec<String>,
    pub num_queries_per_pair: usize,
    pub dry_run: bool,
    pub aberration: Option<Aberration>,
    pub writer: ArrowWriter<File>,
    pub batch_src_frame: Vec<String>,
    pub batch_dst_frame: Vec<String>,
    pub batch_component: Vec<String>,
    pub batch_epoch_et_s: Vec<f64>,
    pub batch_spice_val: Vec<f64>,
    pub batch_anise_val: Vec<f64>,
    pub batch_abs_diff: Vec<f64>,
}

impl CompareEphem {
    pub fn new(
        input_file_names: Vec<String>,
        output_file_name: String,
        num_queries_per_pair: usize,
        aberration: Option<Aberration>,
    ) -> Self {
        let _ = pretty_env_logger::try_init();

        // Build the schema
        let schema = Schema::new(vec![
            Field::new("source frame", DataType::Utf8, false),
            Field::new("destination frame", DataType::Utf8, false),
            Field::new("component", DataType::Utf8, false),
            Field::new("ET Epoch (s)", DataType::Float64, false),
            Field::new("SPICE value", DataType::Float64, false),
            Field::new("ANISE value", DataType::Float64, false),
            Field::new("Absolute difference", DataType::Float64, false),
        ]);

        let file = File::create(format!("../target/{}.parquet", output_file_name)).unwrap();

        // Default writer properties
        let props = WriterProperties::builder().build();
        let writer = ArrowWriter::try_new(file, Arc::new(schema), Some(props)).unwrap();

        Self {
            input_file_names,
            num_queries_per_pair,
            aberration,
            writer,
            dry_run: false,
            batch_src_frame: Vec::new(),
            batch_dst_frame: Vec::new(),
            batch_component: Vec::new(),
            batch_epoch_et_s: Vec::new(),
            batch_spice_val: Vec::new(),
            batch_anise_val: Vec::new(),
            batch_abs_diff: Vec::new(),
        }
    }

    /// Executes this ephemeris validation and return the number of querying errors
    #[must_use]
    pub fn run(mut self) -> usize {
        let mut spks: Vec<SPK> = Vec::new();
        // Load the context
        let mut ctx = Almanac::default();

        for path in &self.input_file_names {
            let spk = SPK::load(path).unwrap();
            spks.push(spk);

            // Load the SPICE data too
            spice::furnsh(path);
        }

        // If there is a light time correction, start after the epoch because the light time correction
        // will cause us to seek out of the definition bounds.

        let bound_offset = match self.aberration {
            None => 0.01_f64.microseconds(),
            Some(_) => 36.0_f64.hours(),
        };

        // Build the pairs of the SPICE and ANISE queries at the same time as we create those instances.
        let mut pairs: HashMap<(i32, i32), (Frame, Frame, Epoch, Epoch)> = HashMap::new();

        for spk in &spks {
            for ephem1 in spk.data_summaries().unwrap() {
                if ephem1.is_empty() {
                    // We're reached the end of useful summaries.
                    break;
                }

                let from_frame = Frame::from_ephem_j2000(ephem1.target_id);

                for ephem2 in spk.data_summaries().unwrap() {
                    if ephem1.target_id == ephem2.target_id {
                        continue;
                    }

                    if ephem2.is_empty() {
                        // We're reached the end of useful summaries.
                        break;
                    }

                    let key = if ephem1.target_id < ephem2.target_id {
                        (ephem1.target_id, ephem2.target_id)
                    } else {
                        (ephem2.target_id, ephem1.target_id)
                    };

                    if pairs.contains_key(&key) {
                        // We're already handled that pair
                        continue;
                    }

                    let to_frame = Frame::from_ephem_j2000(ephem2.target_id);

                    // Query the ephemeris data for a bunch of different times.
                    let start_epoch = ephem1.start_epoch().max(ephem2.start_epoch()) + bound_offset;

                    let end_epoch = ephem1.end_epoch().min(ephem2.end_epoch()) - bound_offset;

                    pairs.insert(key, (from_frame, to_frame, start_epoch, end_epoch));
                }

                // Insert the parent too
                let key = if ephem1.target_id < ephem1.center_id {
                    (ephem1.target_id, ephem1.center_id)
                } else {
                    (ephem1.center_id, ephem1.target_id)
                };

                if pairs.contains_key(&key) {
                    // We're already handled that pair
                    continue;
                }

                let to_frame = Frame::from_ephem_j2000(ephem1.center_id);

                pairs.insert(
                    key,
                    (
                        from_frame,
                        to_frame,
                        ephem1.start_epoch(),
                        ephem1.end_epoch(),
                    ),
                );
            }
        }

        for spk in spks {
            ctx = ctx.with_spk(spk).unwrap();
        }

        info!("Pairs in comparator: {:?}", &pairs);

        let mut i: usize = 0;
        let mut err_count: usize = 0;
        for (from_frame, to_frame, start_epoch, end_epoch) in pairs.values() {
            let time_step = ((*end_epoch - *start_epoch).to_seconds()
                / (self.num_queries_per_pair as f64))
                .seconds();

            let time_it = TimeSeries::exclusive(
                *start_epoch + bound_offset,
                *end_epoch - time_step - bound_offset,
                time_step,
            );

            info!("{time_it} for {from_frame} -> {to_frame} ");

            if self.dry_run {
                continue;
            }

            for epoch in time_it {
                let data = match ctx.translate(*from_frame, *to_frame, epoch, self.aberration) {
                    Ok(state) => {
                        // Find the SPICE names
                        let targ =
                            match SPKSummaryRecord::spice_name_to_id(&format!("{from_frame:e}")) {
                                Ok(id) => {
                                    SPKSummaryRecord::id_to_spice_name(id).unwrap().to_string()
                                }
                                Err(_) => format!("{from_frame:e}"),
                            };

                        let obs = match SPKSummaryRecord::spice_name_to_id(&format!("{to_frame:e}"))
                        {
                            Ok(id) => SPKSummaryRecord::id_to_spice_name(id).unwrap().to_string(),
                            Err(_) => format!("{to_frame:e}"),
                        };

                        // Perform the same query in SPICE
                        let spice_ab_corr = match self.aberration {
                            None => "NONE".to_string(),
                            Some(corr) => format!("{corr:?}"),
                        };

                        let (spice_state, _) = spice::spkezr(
                            &targ,
                            epoch.to_et_seconds(),
                            "J2000",
                            &spice_ab_corr,
                            &obs,
                        );

                        EphemValData {
                            src_frame: format!("{from_frame:e}"),
                            dst_frame: format!("{to_frame:e}"),
                            epoch_et_s: epoch.to_et_seconds(),
                            spice_val_x_km: spice_state[0],
                            spice_val_y_km: spice_state[1],
                            spice_val_z_km: spice_state[2],
                            spice_val_vx_km_s: spice_state[3],
                            spice_val_vy_km_s: spice_state[4],
                            spice_val_vz_km_s: spice_state[5],
                            anise_val_x_km: state.radius_km.x,
                            anise_val_y_km: state.radius_km.y,
                            anise_val_z_km: state.radius_km.z,
                            anise_val_vx_km_s: state.velocity_km_s.x,
                            anise_val_vy_km_s: state.velocity_km_s.y,
                            anise_val_vz_km_s: state.velocity_km_s.z,
                        }
                    }

                    Err(e) => {
                        error!("At epoch {epoch:E}: {e}");
                        err_count += 1;
                        EphemValData::error(
                            format!("{from_frame:e}"),
                            format!("{to_frame:e}"),
                            epoch.to_et_seconds(),
                        )
                    }
                };

                for (j, component) in COMPONENT.iter().enumerate() {
                    self.batch_src_frame.push(data.src_frame.clone());
                    self.batch_dst_frame.push(data.dst_frame.clone());
                    self.batch_component.push(component.to_string());
                    self.batch_epoch_et_s.push(data.epoch_et_s);
                    let (spice_val, anise_val) = match j {
                        0 => (data.spice_val_x_km, data.anise_val_x_km),
                        1 => (data.spice_val_y_km, data.anise_val_y_km),
                        2 => (data.spice_val_z_km, data.anise_val_z_km),
                        3 => (data.spice_val_vx_km_s, data.anise_val_vx_km_s),
                        4 => (data.spice_val_vy_km_s, data.anise_val_vy_km_s),
                        5 => (data.spice_val_vz_km_s, data.anise_val_vz_km_s),
                        _ => unreachable!(),
                    };
                    self.batch_spice_val.push(spice_val);
                    self.batch_anise_val.push(anise_val);
                    self.batch_abs_diff.push((anise_val - spice_val).abs());
                }

                // Consider writing the batch
                if i % BATCH_SIZE == 0 {
                    self.persist();
                }
                i += 1;
            }
        }

        info!("Done with all {i} comparisons");

        // Comparison is finished, let's persist the last batch, close the file, and return the number of querying errors.
        self.persist();
        self.writer.close().unwrap();
        err_count
    }

    fn persist(&mut self) {
        if self.dry_run {
            return;
        }

        self.writer
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
                        "ET Epoch (s)",
                        Arc::new(Float64Array::from(self.batch_epoch_et_s.clone())) as ArrayRef,
                    ),
                    (
                        "SPICE value",
                        Arc::new(Float64Array::from(self.batch_spice_val.clone())) as ArrayRef,
                    ),
                    (
                        "ANISE value",
                        Arc::new(Float64Array::from(self.batch_anise_val.clone())) as ArrayRef,
                    ),
                    (
                        "Absolute difference",
                        Arc::new(Float64Array::from(self.batch_abs_diff.clone())) as ArrayRef,
                    ),
                ])
                .unwrap(),
            )
            .unwrap();

        // Regularly flush to not lose data
        self.writer.flush().unwrap();

        // Re-init all of the vectors
        self.batch_src_frame = Vec::with_capacity(BATCH_SIZE);
        self.batch_dst_frame = Vec::with_capacity(BATCH_SIZE);
        self.batch_component = Vec::with_capacity(BATCH_SIZE);
        self.batch_epoch_et_s = Vec::with_capacity(BATCH_SIZE);
        self.batch_spice_val = Vec::with_capacity(BATCH_SIZE);
        self.batch_anise_val = Vec::with_capacity(BATCH_SIZE);
        self.batch_abs_diff = Vec::with_capacity(BATCH_SIZE);
    }
}
