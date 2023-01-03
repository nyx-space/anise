/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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
use log::error;
use parquet::{arrow::ArrowWriter, file::properties::WriterProperties};
use std::{collections::HashMap, fs::File, io::Read, sync::Arc};

const COMPONENT: &[&'static str] = &["X", "Y", "Z", "VX", "VY", "VZ"];

const BATCH_SIZE: usize = 10_000;
const NUM_QUERIES_PER_PAIR: f64 = 10_000.0;

#[derive(Default)]
pub struct EphemValData {
    pub src_frame: String,
    pub dst_frame: String,
    pub epoch_offset: f64,
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
    pub fn error(src_frame: String, dst_frame: String, epoch_offset: f64) -> Self {
        Self {
            src_frame,
            dst_frame,
            epoch_offset,
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

pub struct CompareEphem {
    pub input_file_names: Vec<String>,
    pub output_file_name: String,
    pub writer: ArrowWriter<File>,
    pub batch_src_frame: Vec<String>,
    pub batch_dst_frame: Vec<String>,
    pub batch_component: Vec<String>,
    pub batch_epoch_offset: Vec<f64>,
    pub batch_spice_val: Vec<f64>,
    pub batch_anise_val: Vec<f64>,
}

impl CompareEphem {
    pub fn new(input_file_names: Vec<String>, output_file_name: String) -> Self {
        // Build the schema
        let schema = Schema::new(vec![
            Field::new("source frame", DataType::Utf8, false),
            Field::new("destination frame", DataType::Utf8, false),
            Field::new("component", DataType::Utf8, false),
            Field::new("File delta T (s)", DataType::Float64, false),
            Field::new("SPICE value", DataType::Float64, false),
            Field::new("ANISE value", DataType::Float64, false),
        ]);

        let file = File::create(format!("target/{}.parquet", output_file_name)).unwrap();

        // Default writer properties
        let props = WriterProperties::builder().build();
        let writer = ArrowWriter::try_new(file, Arc::new(schema), Some(props)).unwrap();

        let me = Self {
            output_file_name,
            input_file_names,
            writer,
            batch_src_frame: Vec::new(),
            batch_dst_frame: Vec::new(),
            batch_component: Vec::new(),
            batch_epoch_offset: Vec::new(),
            batch_spice_val: Vec::new(),
            batch_anise_val: Vec::new(),
        };

        me
    }

    /// Executes this ephemeris validation
    pub fn run(mut self) {
        // Load the context

        let mut ctx = Context::default();

        let mut buffers: Vec<Vec<u8>> = vec![Vec::new(); self.input_file_names.len()];
        let mut spks: Vec<SPK> = vec![SPK::default(); self.input_file_names.len()];

        for (i, path) in self.input_file_names.iter().enumerate() {
            // Open the SPK file
            let mut file = File::open(path).unwrap();
            file.read_to_end(&mut buffers[i]).unwrap();
            // Load the SPICE data too

            spice::furnsh(path);
        }

        // Build the pairs of the SPICE and ANISE queries at the same time as we create those instances.
        let mut pairs: HashMap<(i32, i32), (Frame, Frame, Epoch, Epoch)> = HashMap::new();

        for buf in &buffers {
            let spk = SPK::parse(buf).unwrap();

            for ephem1 in spk.data_summaries {
                if ephem1.is_empty() {
                    // We're reached the end of useful summaries.
                    break;
                }

                let from_frame = Frame::from_ephem_j2000(ephem1.target_id);

                for ephem2 in spk.data_summaries {
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
            spks.push(spk);
        }

        for spk in &spks {
            ctx = ctx.load_spk(spk).unwrap();
        }

        dbg!(&pairs);

        let mut i: usize = 0;
        for (from_frame, to_frame, start_epoch, end_epoch) in pairs.values() {
            let time_step =
                ((*end_epoch - *start_epoch).to_seconds() / NUM_QUERIES_PER_PAIR).seconds();

            let mut time_it =
                TimeSeries::exclusive(*start_epoch, *end_epoch - time_step, time_step);

            while let Some(epoch) = time_it.next() {
                let epoch_offset = (epoch - *start_epoch).to_seconds();

                let data = match ctx.translate_from_to_km_s_geometric(*from_frame, *to_frame, epoch)
                {
                    Ok(state) => {
                        // Find the SPICE names
                        let targ =
                            match SPKSummaryRecord::human_name_to_id(&format!("{from_frame:e}")) {
                                Ok(id) => {
                                    SPKSummaryRecord::id_to_human_name(id).unwrap().to_string()
                                }
                                Err(_) => format!("{from_frame:e}"),
                            };

                        let obs = match SPKSummaryRecord::human_name_to_id(&format!("{to_frame:e}"))
                        {
                            Ok(id) => SPKSummaryRecord::id_to_human_name(id).unwrap().to_string(),
                            Err(_) => format!("{from_frame:e}"),
                        };

                        // Perform the same query in SPICE
                        let (spice_state, _) =
                            spice::spkezr(&targ, epoch.to_et_seconds(), "J2000", "NONE", &obs);

                        let data = EphemValData {
                            src_frame: format!("{from_frame:e}"),
                            dst_frame: format!("{to_frame:e}"),
                            epoch_offset,
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
                        };

                        data
                    }

                    Err(e) => {
                        error!("At epoch {epoch:E}: {e}");
                        EphemValData::error(
                            format!("{from_frame:e}"),
                            format!("{to_frame:e}"),
                            epoch_offset,
                        )
                    }
                };

                for (j, component) in COMPONENT.iter().enumerate() {
                    self.batch_src_frame.push(data.src_frame.clone());
                    self.batch_dst_frame.push(data.dst_frame.clone());
                    self.batch_component.push(component.to_string());
                    self.batch_epoch_offset.push(data.epoch_offset);
                    let (spice_val, anise_val) = match j {
                        0 => (data.spice_val_x_km, data.anise_val_x_km),
                        1 => (data.spice_val_y_km, data.anise_val_y_km),
                        2 => (data.spice_val_z_km, data.anise_val_z_km),
                        3 => (data.spice_val_vy_km_s, data.anise_val_vy_km_s),
                        4 => (data.spice_val_vz_km_s, data.anise_val_vz_km_s),
                        5 => (data.spice_val_vz_km_s, data.anise_val_vz_km_s),
                        _ => unreachable!(),
                    };
                    self.batch_spice_val.push(spice_val);
                    self.batch_anise_val.push(anise_val);
                }

                // Consider writing the batch
                if i % BATCH_SIZE == 0 {
                    self.persist();
                }
                i += 1;
            }
        }

        // Test is finished, so let's close the writer, open it as a lazy dataframe, and pass it to the validation
        self.persist();
        self.writer.close().unwrap();
        // self.validate();
    }

    fn persist(&mut self) {
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

        // Regularly flush to not lose data
        self.writer.flush().unwrap();

        // Re-init all of the vectors
        self.batch_src_frame = Vec::with_capacity(BATCH_SIZE);
        self.batch_dst_frame = Vec::with_capacity(BATCH_SIZE);
        self.batch_component = Vec::with_capacity(BATCH_SIZE);
        self.batch_epoch_offset = Vec::with_capacity(BATCH_SIZE);
        self.batch_spice_val = Vec::with_capacity(BATCH_SIZE);
        self.batch_anise_val = Vec::with_capacity(BATCH_SIZE);
    }
}
