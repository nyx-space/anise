/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::constants::frames::EARTH_J2000;
use anise::prelude::*;
use hifitime::{TimeSeries, TimeUnits};
use log::error;

use crate::framework::ephemeris::*;
use crate::framework::Validator;

const NUM_QUERIES_PER_PAIR: f64 = 1_000.0;

/// Warning: this test _will_ leak all of the loaded BSP data.
/// The other option would be to call `include_bytes!` at compile time, but that would make the binary super big.
struct HermiteType13<'a> {
    sc_naif_id: i32,
    start_epoch: Epoch,
    time_it: TimeSeries,
    ctx: Context<'a>,
}

impl<'a> Validator<'a> for HermiteType13<'a> {
    type Data = EphemValData;

    fn validate(&self, df: polars::prelude::LazyFrame) {
        assert!(true);
    }

    fn setup(files: &[String], ctx: Context<'a>) -> Self {
        let sc_naif_id = -10000001;
        // let hermite_path = format!("/home/chris/Downloads/DefaultLEOSatelliteType13Hermite.bsp");
        // let sc_naif_id = -200000;

        // SPICE load
        for file_path in files {
            spice::furnsh(file_path);
        }

        // Query the ephemeris data for a bunch of different times.
        let summary = ctx.spk_summary_from_id(sc_naif_id).unwrap().0;
        let start_epoch = summary.start_epoch();
        let end_epoch = summary.end_epoch();

        let time_step = ((end_epoch - start_epoch).to_seconds() / NUM_QUERIES_PER_PAIR).seconds();

        let time_it = TimeSeries::exclusive(start_epoch, end_epoch - time_step, time_step);

        Self {
            sc_naif_id,
            start_epoch,
            time_it,
            ctx,
        }
    }

    fn teardown(self) {
        drop(self.ctx);
        spice::unload("data/gmat-hermite.bsp");
    }
}

impl<'a> Iterator for HermiteType13<'a> {
    type Item = EphemValData;

    fn next(&mut self) -> Option<Self::Item> {
        let src_frame = "Earth".to_string();
        let dst_frame = format!("{}", self.sc_naif_id);

        match self.time_it.next() {
            Some(epoch) => {
                let epoch_offset = (epoch - self.start_epoch).to_seconds();

                match self.ctx.translate_from_to_km_s_geometric(
                    EARTH_J2000,
                    Frame::from_ephem_j2000(self.sc_naif_id),
                    epoch,
                ) {
                    Ok(state) => {
                        // Perform the same query in SPICE
                        let (spice_state, _) = spice::spkezr(
                            "EARTH",
                            epoch.to_et_seconds(),
                            "J2000",
                            "NONE",
                            &format!("{}", self.sc_naif_id),
                        );

                        let data = EphemValData {
                            src_frame,
                            dst_frame,
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

                        Some(data)
                    }

                    Err(e) => {
                        error!("At epoch {epoch:E}: {e}");
                        Some(EphemValData::error(src_frame, dst_frame, epoch_offset))
                    }
                }
            }
            None => None,
        }
    }
}

#[test]
fn validate_hermite_type13_from_gmat() {
    let mut validator = EphemerisValidator {
        output_file_name: "type13-validation-test-results".to_string(),
        input_file_names: vec![
            "data/de440.bsp".to_string(),
            "data/gmat-hermite.bsp".to_string(),
        ],
        ..Default::default()
    };

    validator.setup();

    validator.execute::<HermiteType13>();
}
