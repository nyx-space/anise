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
    NaifId,
    ephemerides::{EphemerisError, SPKWritingSnafu},
    naif::{
        SPK,
        daf::{FileRecord, NAIFRecord, NameRecord, RCRD_LEN, SummaryRecord, data_types::DataType},
        spk::summary::SPKSummaryRecord,
    },
};
use log::warn;
use snafu::ensure;
use std::{fs::File, io::Write};
use zerocopy::IntoBytes;

use crate::prelude::Almanac;

use super::Ephemeris;

impl Ephemeris {
    pub fn to_spice_bsp(
        &self,
        naif_id: NaifId,
        data_type: Option<DataType>,
    ) -> Result<SPK, EphemerisError> {
        let segments = self.segment_views();
        if segments.is_empty() {
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

        for segment in &segments {
            ensure!(
                segment.useable_start < segment.useable_end,
                SPKWritingSnafu {
                    details: "SPK segment start epoch must be strictly before its end epoch"
                        .to_string()
                }
            );
            // The Hermite branch below stores (degree - 1) / 2 as the window size, so a
            // zero degree underflows and cannot describe an interpolation window.
            ensure!(
                segment.degree > 0,
                SPKWritingSnafu {
                    details: "interpolation degree must be strictly positive".to_string()
                }
            );
            if data_type.is_none() {
                ensure!(
                    [
                        DataType::Type13HermiteUnequalStep,
                        DataType::Type9LagrangeUnequalStep
                    ]
                    .contains(&segment.interpolation),
                    SPKWritingSnafu {
                        details:
                            "segment data type must be either Type 13 Hermite or Type 9 Lagrange"
                                .to_string()
                    }
                );
            }
        }

        if self.includes_covariance() {
            warn!("ephemeris contains covariance, which is NOT copied to the SPICE BSP file");
        }

        let mut bytes = vec![];
        let mut file_rcrd = FileRecord::spk("Nyx Space ANISE");

        const SUMMARIES_PER_RECORD: usize =
            (RCRD_LEN - SummaryRecord::SIZE) / SPKSummaryRecord::SIZE;
        const WORDS_PER_RECORD: usize = RCRD_LEN / size_of::<f64>();
        let summary_block_count = segments.len().div_ceil(SUMMARIES_PER_RECORD);
        let first_data_word = (1usize
            .checked_add(summary_block_count.checked_mul(2).ok_or_else(|| {
                EphemerisError::SPKWritingError {
                    details: "too many OEM segments for DAF record addressing".to_string(),
                }
            })?)
            .and_then(|records| records.checked_mul(WORDS_PER_RECORD))
            .and_then(|words| words.checked_add(1)))
        .ok_or_else(|| EphemerisError::SPKWritingError {
            details: "too many OEM segments for DAF word addressing".to_string(),
        })?;

        let mut next_word = first_data_word;
        let mut summaries = Vec::with_capacity(segments.len());
        let mut segment_data = Vec::with_capacity(segments.len());
        let interpolation_almanac = Almanac::default();
        for segment in &segments {
            let interpolation = data_type.unwrap_or(segment.interpolation);
            let required_raw_states = match interpolation {
                DataType::Type9LagrangeUnequalStep => {
                    ensure!(
                        segment.degree <= 15,
                        SPKWritingSnafu {
                            details: "SPK Type 9 interpolation degree must not exceed 15"
                                .to_string()
                        }
                    );
                    segment.degree.checked_add(1).ok_or_else(|| {
                        EphemerisError::SPKWritingError {
                            details: "Lagrange interpolation degree is too large".to_string(),
                        }
                    })?
                }
                DataType::Type13HermiteUnequalStep => {
                    ensure!(
                        segment.degree % 2 == 1,
                        SPKWritingSnafu {
                            details: "Hermite interpolation degree must be odd".to_string()
                        }
                    );
                    ensure!(
                        segment.degree <= 27,
                        SPKWritingSnafu {
                            details: "SPK Hermite interpolation degree must not exceed 27"
                                .to_string()
                        }
                    );
                    segment.degree.div_ceil(2)
                }
                _ => unreachable!(),
            };
            ensure!(
                segment.state_data.len() >= required_raw_states,
                SPKWritingSnafu {
                    details: format!(
                        "segment has {} raw states but interpolation degree {} requires at least {required_raw_states}",
                        segment.state_data.len(),
                        segment.degree
                    )
                }
            );

            let raw_start = *segment
                .state_data
                .first_key_value()
                .expect("empty segment is never constructed")
                .0;
            let raw_end = *segment
                .state_data
                .last_key_value()
                .expect("empty segment is never constructed")
                .0;
            let mut spk_state_data = segment.state_data.to_owned();
            let mut output_view = *segment;
            output_view.interpolation = interpolation;
            // SPK Type 9/13 require their descriptor bounds to lie within the
            // stored epoch array. OEM useable bounds can extend beyond the first
            // or last raw support node, so materialize only those boundary states
            // from the owning segment's interpolation before serialization.
            if segment.useable_start < raw_start {
                let orbit = Self::orbit_at_in_with_window(
                    output_view,
                    segment.useable_start,
                    required_raw_states,
                    &interpolation_almanac,
                )?;
                spk_state_data.insert(
                    segment.useable_start,
                    super::EphemerisRecord { orbit, covar: None },
                );
            }
            if segment.useable_end > raw_end {
                let orbit = Self::orbit_at_in_with_window(
                    output_view,
                    segment.useable_end,
                    required_raw_states,
                    &interpolation_almanac,
                )?;
                spk_state_data.insert(
                    segment.useable_end,
                    super::EphemerisRecord { orbit, covar: None },
                );
            }

            let num_states = spk_state_data.len();
            let mut data_bytes = Vec::with_capacity(num_states * 7 * 8);
            let mut epoch_bytes = Vec::with_capacity(num_states * 8);
            let mut registry_bytes = Vec::with_capacity(num_states.saturating_sub(1) / 100 * 8);

            for (idx, entry) in spk_state_data.values().enumerate() {
                let orbit = entry.orbit;
                for value in [
                    orbit.radius_km.x,
                    orbit.radius_km.y,
                    orbit.radius_km.z,
                    orbit.velocity_km_s.x,
                    orbit.velocity_km_s.y,
                    orbit.velocity_km_s.z,
                ] {
                    data_bytes.extend_from_slice(&value.to_ne_bytes());
                }
                epoch_bytes.extend_from_slice(&orbit.epoch.to_et_seconds().to_ne_bytes());
                if idx > 0 && (idx + 1) % 100 == 0 && idx + 1 < num_states {
                    registry_bytes.extend_from_slice(&orbit.epoch.to_et_seconds().to_ne_bytes());
                }
            }
            data_bytes.extend_from_slice(&epoch_bytes);
            data_bytes.extend_from_slice(&registry_bytes);
            let samples_m1 = match interpolation {
                DataType::Type9LagrangeUnequalStep => segment.degree,
                DataType::Type13HermiteUnequalStep => (segment.degree - 1) / 2,
                _ => segment.degree,
            };
            data_bytes.extend_from_slice(&(samples_m1 as f64).to_ne_bytes());
            data_bytes.extend_from_slice(&(num_states as f64).to_ne_bytes());

            let word_count = data_bytes.len() / size_of::<f64>();
            let end_word = next_word
                .checked_add(word_count)
                .and_then(|word| word.checked_sub(1))
                .ok_or_else(|| EphemerisError::SPKWritingError {
                    details: "SPK segment data address overflow".to_string(),
                })?;
            let start_idx =
                i32::try_from(next_word).map_err(|_| EphemerisError::SPKWritingError {
                    details: "SPK segment start address exceeds i32".to_string(),
                })?;
            let end_idx = i32::try_from(end_word).map_err(|_| EphemerisError::SPKWritingError {
                details: "SPK segment end address exceeds i32".to_string(),
            })?;
            let first_frame = segment
                .state_data
                .first_key_value()
                .expect("empty segment is never constructed")
                .1
                .orbit
                .frame;
            summaries.push(SPKSummaryRecord {
                start_epoch_et_s: segment.useable_start.to_et_seconds(),
                end_epoch_et_s: segment.useable_end.to_et_seconds(),
                target_id: naif_id,
                center_id: first_frame.ephemeris_id,
                frame_id: first_frame.orientation_id,
                data_type_i: interpolation.into(),
                start_idx,
                end_idx,
            });
            segment_data.push(data_bytes);
            next_word = end_word + 1;
        }

        file_rcrd.backward = u32::try_from(summary_block_count * 2).map_err(|_| {
            EphemerisError::SPKWritingError {
                details: "DAF backward record pointer exceeds u32".to_string(),
            }
        })?;
        file_rcrd.free_addr =
            u32::try_from(next_word).map_err(|_| EphemerisError::SPKWritingError {
                details: "DAF free address exceeds u32".to_string(),
            })?;
        let summary_size = file_rcrd.summary_size();
        place_in_rcrd(file_rcrd.as_bytes(), &mut bytes);

        let segment_name = format!("{} (converted by Nyx Space ANISE)", self.object_id);
        for (block_idx, chunk) in summaries.chunks(SUMMARIES_PER_RECORD).enumerate() {
            let record_number = 2 + block_idx * 2;
            let daf_summary = SummaryRecord {
                next_record: if block_idx + 1 < summary_block_count {
                    (record_number + 2) as f64
                } else {
                    0.0
                },
                prev_record: if block_idx == 0 {
                    0.0
                } else {
                    (record_number - 2) as f64
                },
                num_summaries: chunk.len() as f64,
            };
            let mut summary_record = Vec::with_capacity(RCRD_LEN);
            summary_record.extend_from_slice(daf_summary.as_bytes());
            for summary in chunk {
                summary_record.extend_from_slice(summary.as_bytes());
            }
            place_in_rcrd(&summary_record, &mut bytes);

            // DAF name records are blank-padded; keep unused slots as spaces too,
            // matching the original single-record writer and NAIF convention.
            let mut name_record = NameRecord {
                raw_names: [0x20; RCRD_LEN],
            };
            for idx in 0..chunk.len() {
                name_record.set_nth_name(idx, summary_size, &segment_name);
            }
            place_in_rcrd(name_record.as_bytes(), &mut bytes);
        }

        for data in segment_data {
            bytes.extend_from_slice(&data);
        }
        bytes.resize(bytes.len().div_ceil(RCRD_LEN) * RCRD_LEN, 0);

        // Parsing the generated bytes builds the chronological ID index, so a shared
        // useable endpoint consistently selects the later, higher-priority segment.
        let mut spk = SPK::parse(&bytes[..]).map_err(|error| EphemerisError::SPKWritingError {
            details: format!("generated SPK could not be parsed: {error}"),
        })?;
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
                });
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
