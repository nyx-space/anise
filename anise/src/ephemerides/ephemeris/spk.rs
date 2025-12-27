/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{
    ephemerides::{EphemerisError, SPKWritingSnafu},
    naif::{
        daf::{data_types::DataType, FileRecord, NameRecord, SummaryRecord, RCRD_LEN},
        spk::summary::SPKSummaryRecord,
        SPK,
    },
    NaifId, DBL_SIZE,
};
use bytes::BytesMut;
use log::warn;
use snafu::ensure;
use std::{fs::File, io::Write};
use zerocopy::IntoBytes;

use super::Ephemeris;

impl Ephemeris {
    pub fn to_spice_bsp(
        &self,
        naif_id: NaifId,
        data_type: Option<DataType>,
    ) -> Result<SPK, EphemerisError> {
        if self.state_data.is_empty() {
            return Err(EphemerisError::SPKWritingError {
                details: "ephemeris file contains no state data".to_string(),
            });
        }

        if let Some(data_type) = data_type {
            ensure!(
                [
                    DataType::Type13HermiteUnequalStep,
                    DataType::Type9LagrangeUnequalStep
                ]
                .contains(&data_type),
                SPKWritingSnafu {
                    details:
                        ("provided data type must be either Type 13 Hermite or Type 9 Lagrange")
                            .to_string()
                }
            );
        }

        if self.includes_covariance() {
            warn!("ephemeris contains covariance, which is NOT copied to the SPICE BSP file");
        }

        let mut bytes = vec![];
        let mut statedata_bytes = vec![];
        // Create the FileRecord, mutable because we need to set the addresses later.
        let mut file_rcrd = FileRecord::spk("Nyx Space ANISE");
        // Write this object name in the name record, there will be only one summary/segment for the whole ephem.
        // The names are trimmed so we initialize the bytes with spacex (0x20).
        let mut raw_names = [0x20; RCRD_LEN];
        for (dest, src) in raw_names
            .iter_mut()
            .zip(format!("{} (converted by Nyx Space ANISE)", self.object_id).as_bytes())
        {
            *dest = *src;
        }

        let name_rcrd = NameRecord { raw_names };

        let interpolation = match data_type {
            None => self.interpolation,
            Some(desired_type) => desired_type,
        };

        // Build the SPK Summary
        let first_orbit = self.state_data.first_key_value().unwrap().1.orbit;
        let first_frame = first_orbit.frame;
        let last_orbit = self.state_data.last_key_value().unwrap().1.orbit;
        let spk_summary = SPKSummaryRecord {
            start_epoch_et_s: first_orbit.epoch.to_et_seconds(),
            end_epoch_et_s: last_orbit.epoch.to_et_seconds(),
            target_id: naif_id,
            center_id: first_frame.ephemeris_id,
            frame_id: first_frame.orientation_id,
            data_type_i: interpolation.into(),
            start_idx: 0,
            end_idx: (self.state_data.len() * 7 * DBL_SIZE) as i32,
        };

        // Build a single Summary record
        let daf_summary = SummaryRecord {
            next_record: 0.0,
            prev_record: 0.0,
            num_summaries: 1.0,
        };

        // Build the data records. Both Lagrange and Hermite use the same structure.
        let mut state_data = Vec::with_capacity(self.state_data.len() * 7);
        let mut epoch_data = Vec::with_capacity(self.state_data.len());
        let mut epoch_registry = Vec::with_capacity(self.state_data.len() % 100 + 1);
        for (idx, (_, entry)) in self.state_data.iter().enumerate() {
            let orbit = entry.orbit;
            state_data.extend_from_slice(&[
                orbit.radius_km.x.to_ne_bytes(),
                orbit.radius_km.y.to_ne_bytes(),
                orbit.radius_km.z.to_ne_bytes(),
                orbit.velocity_km_s.x.to_ne_bytes(),
                orbit.velocity_km_s.y.to_ne_bytes(),
                orbit.velocity_km_s.z.to_ne_bytes(),
            ]);
            epoch_data.push(orbit.epoch.to_et_seconds().to_ne_bytes());
            if idx % 100 == 0 {
                epoch_registry.push(orbit.epoch.to_et_seconds().to_ne_bytes());
            }
        }

        // Now, manually build the HermiteSetType13 since we have nearly everything in the correct order and format.
        statedata_bytes.extend_from_slice(&state_data);
        statedata_bytes.extend_from_slice(&epoch_data);
        statedata_bytes.extend_from_slice(&epoch_registry);
        statedata_bytes.push((self.degree as f64).to_ne_bytes());
        statedata_bytes.push(((self.state_data.len() - 1) as f64).to_ne_bytes());

        // Update the file record
        file_rcrd.free_addr = statedata_bytes.len() as u32;

        // Write the bytes in order.
        place_in_rcrd(file_rcrd.as_bytes(), &mut bytes);
        // The SPK summary immediately follows the DAF summary for each summary!
        let summaries = [daf_summary.as_bytes(), spk_summary.as_bytes()].concat();
        place_in_rcrd(&summaries, &mut bytes);
        place_in_rcrd(name_rcrd.as_bytes(), &mut bytes);
        bytes.extend_from_slice(statedata_bytes.as_bytes());

        let u8_bytes = bytes.as_bytes();

        // Finally, builds the DAF!
        let mut spk = SPK {
            bytes: BytesMut::from(u8_bytes),
            crc32: None,
            _daf_type: std::marker::PhantomData,
        };
        spk.set_crc32();
        Ok(spk)
    }

    /// Converts this ephemeris to SPICE BSP/SPK file in the provided data type, saved to the provided output_fname.
    pub fn write_spice_bsp(
        &self,
        naif_id: NaifId,
        output_fname: &str,
        data_type: Option<DataType>,
    ) -> Result<(), EphemerisError> {
        let spk = self.to_spice_bsp(naif_id, data_type)?;

        match File::create(output_fname) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&spk.bytes) {
                    return Err(EphemerisError::SPKWritingError {
                        details: format!("{e}"),
                    });
                };
            }
            Err(e) => {
                return Err(EphemerisError::SPKWritingError {
                    details: format!("{e}"),
                })
            }
        };

        Ok(())
    }
}

fn place_in_rcrd(input_bytes: &[u8], output_bytes: &mut Vec<u8>) {
    let mut rcrd_bytes = [0x0; RCRD_LEN];
    for (dest, src) in rcrd_bytes.iter_mut().zip(input_bytes) {
        *dest = *src;
    }

    output_bytes.extend_from_slice(&rcrd_bytes);
}
