/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use bytes::Bytes;
use hifitime::TimeScale;
use log::info;
use snafu::ResultExt;
use zerocopy::FromBytes;

use crate::ephemerides::SPKSnafu;
use crate::errors::{
    AlmanacError, AlmanacResult, EphemerisSnafu, LoadingSnafu, OrientationSnafu, TLDataSetSnafu,
};
use crate::file2heap;
use crate::naif::daf::{FileRecord, NAIFRecord};
use crate::naif::pretty_print::NAIFPrettyPrint;
use crate::naif::{BPC, SPK};
use crate::orientations::BPCSnafu;
use crate::structure::dataset::DataSetType;
use crate::structure::metadata::Metadata;
use crate::structure::{EulerParameterDataSet, PlanetaryDataSet, SpacecraftDataSet};
use core::fmt;

// TODO: Switch these to build constants so that it's configurable when building the library.
pub const MAX_LOADED_SPKS: usize = 32;
pub const MAX_LOADED_BPCS: usize = 8;
pub const MAX_SPACECRAFT_DATA: usize = 16;
pub const MAX_PLANETARY_DATA: usize = 128;

pub mod aer;
pub mod bpc;
pub mod eclipse;
pub mod planetary;
pub mod solar;
pub mod spk;
pub mod transform;

#[cfg(feature = "metaload")]
pub mod metaload;

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "embed_ephem")]
#[cfg_attr(docsrs, doc(cfg(feature = "embed_ephem")))]
mod embed;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// An Almanac contains all of the loaded SPICE and ANISE data. It is the context for all computations.
///
/// :type path: str
/// :rtype: Almanac
#[derive(Clone, Default)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
pub struct Almanac {
    /// NAIF SPK is kept unchanged
    pub spk_data: [Option<SPK>; MAX_LOADED_SPKS],
    /// NAIF BPC is kept unchanged
    pub bpc_data: [Option<BPC>; MAX_LOADED_BPCS],
    /// Dataset of planetary data
    pub planetary_data: PlanetaryDataSet,
    /// Dataset of spacecraft data
    pub spacecraft_data: SpacecraftDataSet,
    /// Dataset of euler parameters
    pub euler_param_data: EulerParameterDataSet,
}

impl fmt::Display for Almanac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Almanac: #SPK = {}\t#BPC = {}",
            self.num_loaded_spk(),
            self.num_loaded_bpc()
        )?;
        if !self.planetary_data.lut.by_id.is_empty() {
            write!(f, "\t{}", self.planetary_data)?;
        }
        if !self.spacecraft_data.lut.by_id.is_empty() {
            write!(f, "\t{}", self.spacecraft_data)?;
        }
        if !self.euler_param_data.lut.by_id.is_empty() {
            write!(f, "\t{}", self.euler_param_data)?;
        }
        Ok(())
    }
}

impl Almanac {
    /// Initializes a new Almanac from the provided file path, guessing at the file type
    pub fn new(path: &str) -> AlmanacResult<Self> {
        Self::default().load(path)
    }

    /// Loads the provided spacecraft data into a clone of this original Almanac.
    pub fn with_spacecraft_data(&self, spacecraft_data: SpacecraftDataSet) -> Self {
        let mut me = self.clone();
        me.spacecraft_data = spacecraft_data;
        me
    }

    /// Loads the provided Euler parameter data into a clone of this original Almanac.
    pub fn with_euler_parameters(&self, ep_dataset: EulerParameterDataSet) -> Self {
        let mut me = self.clone();
        me.euler_param_data = ep_dataset;
        me
    }

    /// Loads the provides bytes as one of the data types supported in ANISE.
    pub fn load_from_bytes(&self, bytes: Bytes) -> AlmanacResult<Self> {
        self._load_from_bytes(bytes, None)
    }

    fn _load_from_bytes(&self, bytes: Bytes, path: Option<&str>) -> AlmanacResult<Self> {
        // Try to load as a SPICE DAF first (likely the most typical use case)

        // Load the header only
        if let Some(file_record_bytes) = bytes.get(..FileRecord::SIZE) {
            let file_record = FileRecord::read_from_bytes(file_record_bytes).unwrap();
            if let Ok(fileid) = file_record.identification() {
                return match fileid {
                    "PCK" => {
                        info!("Loading {} as DAF/PCK", path.unwrap_or("bytes"));
                        let bpc = BPC::parse(bytes)
                            .context(BPCSnafu {
                                action: "parsing bytes",
                            })
                            .context(OrientationSnafu {
                                action: "from generic loading",
                            })?;
                        self.with_bpc(bpc).context(OrientationSnafu {
                            action: "adding BPC file to context",
                        })
                    }
                    "SPK" => {
                        info!("Loading {} as DAF/SPK", path.unwrap_or("bytes"));
                        let spk = SPK::parse(bytes)
                            .context(SPKSnafu {
                                action: "parsing bytes",
                            })
                            .context(EphemerisSnafu {
                                action: "from generic loading",
                            })?;
                        self.with_spk(spk).context(EphemerisSnafu {
                            action: "adding SPK file to context",
                        })
                    }
                    fileid => Err(AlmanacError::GenericError {
                        err: format!("DAF/{fileid} is not yet supported"),
                    }),
                };
            }
            // Fall through to try to load as an ANISE file
        }

        if let Ok(metadata) = Metadata::decode_header(&bytes) {
            // Use `try_from` to validate the dataset type
            let dataset_type =
                DataSetType::try_from(metadata.dataset_type as u8).map_err(|err| {
                    AlmanacError::GenericError {
                        err: format!("Invalid dataset type: {err}"),
                    }
                })?;

            // Now, we can load this depending on the kind of data that it is
            match dataset_type {
                DataSetType::NotApplicable => {
                    // Not something that can be decoded
                    Err(AlmanacError::GenericError {
                        err: format!("Malformed dataset type in {}", path.unwrap_or("bytes")),
                    })
                }
                DataSetType::SpacecraftData => {
                    // Decode as spacecraft data
                    let dataset = SpacecraftDataSet::try_from_bytes(bytes).context({
                        TLDataSetSnafu {
                            action: "loading as spacecraft data",
                        }
                    })?;
                    info!(
                        "Loading {} as ANISE spacecraft data",
                        path.unwrap_or("bytes")
                    );
                    Ok(self.with_spacecraft_data(dataset))
                }
                DataSetType::PlanetaryData => {
                    // Decode as planetary data
                    let dataset = PlanetaryDataSet::try_from_bytes(bytes).context({
                        TLDataSetSnafu {
                            action: "loading as planetary data",
                        }
                    })?;
                    info!("Loading {} as ANISE/PCA", path.unwrap_or("bytes"));
                    Ok(self.with_planetary_data(dataset))
                }
                DataSetType::EulerParameterData => {
                    // Decode as euler parameter data
                    let dataset = EulerParameterDataSet::try_from_bytes(bytes).context({
                        TLDataSetSnafu {
                            action: "loading Euler parameters",
                        }
                    })?;
                    info!("Loading {} as ANISE/EPA", path.unwrap_or("bytes"));
                    Ok(self.with_euler_parameters(dataset))
                }
            }
        } else {
            Err(AlmanacError::GenericError {
                err: "Provided file cannot be inspected loaded directly in ANISE and may need a conversion first".to_string(),
            })
        }
    }

    /// Generic function that tries to load the provided path guessing to the file type.
    pub fn load(&self, path: &str) -> AlmanacResult<Self> {
        // Load the data onto the heap
        let bytes = file2heap!(path).context(LoadingSnafu {
            path: path.to_string(),
        })?;

        self._load_from_bytes(bytes, Some(path))
            .map_err(|e| match e {
                AlmanacError::GenericError { err } => {
                    // Add the path to the error
                    AlmanacError::GenericError {
                        err: format!("with {path}: {err}"),
                    }
                }
                _ => e,
            })
    }

    /// Pretty prints the description of this Almanac, showing everything by default. Default time scale is TDB.
    /// If any parameter is set to true, then nothing other than that will be printed.
    pub fn describe(
        &self,
        spk: Option<bool>,
        bpc: Option<bool>,
        planetary: Option<bool>,
        eulerparams: Option<bool>,
        time_scale: Option<TimeScale>,
        round_time: Option<bool>,
    ) {
        let print_any = spk.unwrap_or(false) || bpc.unwrap_or(false) || planetary.unwrap_or(false);

        if spk.unwrap_or(!print_any) {
            for (spk_no, maybe_spk) in self
                .spk_data
                .iter()
                .take(self.num_loaded_spk())
                .rev()
                .enumerate()
            {
                let spk = maybe_spk.as_ref().unwrap();
                println!(
                    "=== SPK #{spk_no} ===\n{}",
                    spk.describe_in(time_scale.unwrap_or(TimeScale::TDB), round_time)
                );
            }
        }

        if bpc.unwrap_or(!print_any) {
            for (bpc_no, maybe_bpc) in self
                .bpc_data
                .iter()
                .take(self.num_loaded_bpc())
                .rev()
                .enumerate()
            {
                let bpc = maybe_bpc.as_ref().unwrap();
                println!(
                    "=== BPC #{bpc_no} ===\n{}",
                    bpc.describe_in(time_scale.unwrap_or(TimeScale::TDB), round_time)
                );
            }
        }

        if planetary.unwrap_or(!print_any) {
            println!("=== PLANETARY DATA ==\n{}", self.planetary_data.describe());
        }

        if eulerparams.unwrap_or(!print_any) {
            println!(
                "=== EULER PARAMETER DATA ==\n{}",
                self.euler_param_data.describe()
            );
        }
    }
}
