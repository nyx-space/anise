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
use log::{error, info};
use polars::prelude::*;
use std::{collections::HashMap, fs::File};

const COMPONENT: &[&str] = &["X", "Y", "Z", "VX", "VY", "VZ"];

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
    pub output_file_name: String,
    pub num_queries_per_pair: usize,
    pub dry_run: bool,
    pub aberration: Option<Aberration>,
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

        Self {
            input_file_names,
            output_file_name,
            num_queries_per_pair,
            aberration,
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

    /// Executes this ephemeris validation and return the number of querying errors.
    ///
    /// NOTE: All results are accumulated in memory before being written to the Parquet file.
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
            for ephem1 in spk.data_summaries(None).unwrap() {
                if ephem1.is_empty() {
                    // We're reached the end of useful summaries.
                    break;
                }

                let from_frame = Frame::from_ephem_j2000(ephem1.target_id);

                for ephem2 in spk.data_summaries(None).unwrap() {
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
            ctx = ctx.with_spk(spk);
        }

        info!("Pairs in comparator (count: {}): {:?}", pairs.len(), &pairs);

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
                i += 1;
            }
        }

        info!("Done with all {i} comparisons");

        // Comparison is finished, let's persist the results, and return the number of querying errors.
        self.persist();
        err_count
    }

    fn persist(&mut self) {
        if self.dry_run || self.batch_src_frame.is_empty() {
            return;
        }

        let mut df = df!(
            "source frame" => &self.batch_src_frame,
            "destination frame" => &self.batch_dst_frame,
            "component" => &self.batch_component,
            "ET Epoch (s)" => &self.batch_epoch_et_s,
            "SPICE value" => &self.batch_spice_val,
            "ANISE value" => &self.batch_anise_val,
            "Absolute difference" => &self.batch_abs_diff,
        )
        .unwrap();

        let path = format!("../target/{}.parquet", self.output_file_name);
        let file = File::create(path).unwrap();
        ParquetWriter::new(file).finish(&mut df).unwrap();

        // Clear the batch vectors to prevent double-persistence if called multiple times.
        self.batch_src_frame.clear();
        self.batch_dst_frame.clear();
        self.batch_component.clear();
        self.batch_epoch_et_s.clear();
        self.batch_spice_val.clear();
        self.batch_anise_val.clear();
        self.batch_abs_diff.clear();
    }
}
