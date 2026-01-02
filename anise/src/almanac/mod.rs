/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use bytes::{BufMut, BytesMut};
use hifitime::{Epoch, TimeScale};
use indexmap::IndexMap;
use log::{info, warn};
use snafu::ResultExt;
use zerocopy::FromBytes;

use crate::ephemerides::SPKSnafu;
use crate::errors::{
    AlmanacError, AlmanacResult, EphemerisSnafu, InputOutputError, LoadingSnafu, OrientationSnafu,
    TLDataSetSnafu,
};
use crate::math::rotation::EulerParameter;
use crate::naif::daf::{FileRecord, NAIFRecord};
use crate::naif::pretty_print::NAIFPrettyPrint;
use crate::naif::{BPC, SPK};
use crate::orientations::{BPCSnafu, OrientationError};
use crate::structure::dataset::{DataSetError, DataSetType};
use crate::structure::lookuptable::LutError;
use crate::structure::metadata::Metadata;
use crate::structure::{
    EulerParameterDataSet, InstrumentDataSet, LocationDataSet, PlanetaryDataSet, SpacecraftDataSet,
};
use crate::NaifId;
use core::fmt;

pub mod aer;
pub mod bpc;
pub mod eclipse;
pub mod instrument;
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
    pub spk_data: IndexMap<String, SPK>,
    /// NAIF BPC is kept unchanged
    pub bpc_data: IndexMap<String, BPC>,
    /// Dataset of planetary data
    pub planetary_data: IndexMap<String, PlanetaryDataSet>,
    /// Dataset of spacecraft data
    pub spacecraft_data: IndexMap<String, SpacecraftDataSet>,
    /// Dataset of euler parameters
    pub euler_param_data: IndexMap<String, EulerParameterDataSet>,
    /// Dataset of locations
    pub location_data: IndexMap<String, LocationDataSet>,
    /// Dataset of instruments
    pub instrument_data: IndexMap<String, InstrumentDataSet>,
}

impl fmt::Display for Almanac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Almanac: #SPK = {}\t#BPC = {}",
            self.num_loaded_spk(),
            self.num_loaded_bpc()
        )?;
        if !self.planetary_data.is_empty() {
            write!(f, "\t#Planetary kernels = {}", self.planetary_data.len())?;
        }
        if !self.spacecraft_data.is_empty() {
            write!(f, "\t#Spacecraft kernels = {}", self.spacecraft_data.len())?;
        }
        if !self.euler_param_data.is_empty() {
            write!(
                f,
                "\t#Euler param kernels = {}",
                self.euler_param_data.len()
            )?;
        }
        if !self.location_data.is_empty() {
            write!(f, "\t#Location kernels = {}", self.location_data.len())?;
        }
        Ok(())
    }
}

impl Almanac {
    /// Initializes a new Almanac from the provided file path, guessing at the file type
    pub fn new(path: &str) -> AlmanacResult<Self> {
        Self::default().load(path)
    }

    /// Loads the provided spacecraft data.
    pub fn with_spacecraft_data(self, spacecraft_data: SpacecraftDataSet) -> Self {
        self.with_spacecraft_data_as(spacecraft_data, None)
    }

    /// Loads the provided spacecraft data.
    pub fn with_spacecraft_data_as(
        mut self,
        spacecraft_data: SpacecraftDataSet,
        alias: Option<String>,
    ) -> Self {
        let alias = alias.unwrap_or(Epoch::now().unwrap_or_default().to_string());
        let msg = format!("unloading spacecraft data `{alias}`");
        if self
            .spacecraft_data
            .insert(alias, spacecraft_data)
            .is_some()
        {
            warn!("{msg}");
        }
        self
    }

    /// Loads the provided Euler parameter data into a clone of this original Almanac.
    pub fn with_euler_parameters(self, ep_dataset: EulerParameterDataSet) -> Self {
        self.with_euler_parameters_as(ep_dataset, None)
    }

    /// Loads the provided Euler parameter data.
    pub fn with_euler_parameters_as(
        mut self,
        ep_dataset: EulerParameterDataSet,
        alias: Option<String>,
    ) -> Self {
        let alias = alias.unwrap_or(Epoch::now().unwrap_or_default().to_string());
        let msg = format!("unloading Euler parameter data `{alias}`");
        if self.euler_param_data.insert(alias, ep_dataset).is_some() {
            warn!("{msg}");
        }
        self
    }

    /// Loads the provided location data.
    pub fn with_location_data(self, loc_dataset: LocationDataSet) -> Self {
        self.with_location_data_as(loc_dataset, None)
    }

    /// Loads the provided location data.
    pub fn with_location_data_as(
        mut self,
        loc_dataset: LocationDataSet,
        alias: Option<String>,
    ) -> Self {
        let alias = alias.unwrap_or(Epoch::now().unwrap_or_default().to_string());
        let msg = format!("unloading location data `{alias}`");
        if self.location_data.insert(alias, loc_dataset).is_some() {
            warn!("{msg}");
        }
        self
    }

    /// Loads the provides bytes as one of the data types supported in ANISE.
    pub fn load_from_bytes(self, bytes: BytesMut) -> AlmanacResult<Self> {
        self._load_from_bytes(bytes, None)
    }

    fn _load_from_bytes(self, bytes: BytesMut, path: Option<&str>) -> AlmanacResult<Self> {
        // Check if they forgot to run git lfs
        if let Some(lfs_header) = bytes.get(..8) {
            if lfs_header == "version".as_bytes() {
                return Err(AlmanacError::GenericError {
                    err: "file is a git lfs pointer, run `git lfs pull`".to_string(),
                });
            }
        }

        let path_str = path.map_or_else(|| None, |p| Some(p.to_string()));

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
                        Ok(self.with_bpc_as(bpc, path_str))
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
                        Ok(self.with_spk_as(spk, path_str))
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
                    Ok(self.with_spacecraft_data_as(dataset, path_str))
                }
                DataSetType::PlanetaryData => {
                    // Decode as planetary data
                    let dataset = PlanetaryDataSet::try_from_bytes(bytes).context({
                        TLDataSetSnafu {
                            action: "loading as planetary data",
                        }
                    })?;
                    info!("Loading {} as ANISE/PCA", path.unwrap_or("bytes"));
                    Ok(self.with_planetary_data_as(dataset, path_str))
                }
                DataSetType::EulerParameterData => {
                    // Decode as euler parameter data
                    let dataset = EulerParameterDataSet::try_from_bytes(bytes).context({
                        TLDataSetSnafu {
                            action: "loading Euler parameters",
                        }
                    })?;
                    info!("Loading {} as ANISE/EPA", path.unwrap_or("bytes"));
                    Ok(self.with_euler_parameters_as(dataset, path_str))
                }
                DataSetType::LocationData => {
                    let dataset = LocationDataSet::try_from_bytes(bytes).context({
                        TLDataSetSnafu {
                            action: "loading location data",
                        }
                    })?;
                    info!("Loading {} as ANISE/LDA", path.unwrap_or("bytes"));
                    Ok(self.with_location_data_as(dataset, path_str))
                }
            }
        } else {
            Err(AlmanacError::GenericError {
                err: "file cannot be inspected or loaded directly in ANISE".to_string(),
            })
        }
    }

    /// Generic function that tries to load the provided path guessing to the file type.
    pub fn load(self, path: &str) -> AlmanacResult<Self> {
        // Load the data onto the heap
        let bytes = match std::fs::read(path) {
            Err(e) => {
                return Err(AlmanacError::Loading {
                    path: path.to_string(),
                    source: InputOutputError::IOError { kind: e.kind() },
                })
            }
            Ok(bytes) => BytesMut::from(&bytes[..]),
        };

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
    #[allow(clippy::too_many_arguments)]
    pub fn describe(
        &self,
        spk: Option<bool>,
        bpc: Option<bool>,
        planetary: Option<bool>,
        spacecraft: Option<bool>,
        eulerparams: Option<bool>,
        locations: Option<bool>,
        time_scale: Option<TimeScale>,
        round_time: Option<bool>,
    ) {
        let print_any = spk.unwrap_or(false)
            || bpc.unwrap_or(false)
            || planetary.unwrap_or(false)
            || eulerparams.unwrap_or(false)
            || locations.unwrap_or(false);

        if spk.unwrap_or(!print_any) {
            for (spk_no, (alias, spk)) in self.spk_data.iter().rev().enumerate() {
                println!(
                    "=== SPK #{spk_no}: `{alias}` ===\n{}",
                    spk.describe_in(time_scale.unwrap_or(TimeScale::TDB), round_time)
                );
            }
        }

        if bpc.unwrap_or(!print_any) {
            for (bpc_no, (alias, bpc)) in self.bpc_data.iter().rev().enumerate() {
                println!(
                    "=== BPC #{bpc_no}: `{alias}` ===\n{}",
                    bpc.describe_in(time_scale.unwrap_or(TimeScale::TDB), round_time)
                );
            }
        }

        if planetary.unwrap_or(!print_any) {
            for (num, (alias, data)) in self.planetary_data.iter().rev().enumerate() {
                println!(
                    "=== PLANETARY DATA #{num}: `{alias}` ===\n{}",
                    data.describe()
                );
            }
        }

        if spacecraft.unwrap_or(!print_any) {
            for (num, (alias, data)) in self.spacecraft_data.iter().rev().enumerate() {
                println!(
                    "=== SPACECRAFT DATA #{num}: `{alias}` ===\n{}",
                    data.describe()
                );
            }
        }

        if eulerparams.unwrap_or(!print_any) {
            for (num, (alias, data)) in self.euler_param_data.iter().rev().enumerate() {
                println!(
                    "=== EULER PARAMETER DATA #{num}: `{alias}` ===\n{}",
                    data.describe()
                );
            }
        }

        if locations.unwrap_or(!print_any) {
            for (num, (alias, data)) in self.location_data.iter().rev().enumerate() {
                println!(
                    "=== LOCATIONS DATA #{num}: `{alias}` ===\n{}",
                    data.describe()
                );
            }
        }
    }

    /// Returns the list of loaded kernels
    pub fn list_kernels(
        &self,
        spk: Option<bool>,
        bpc: Option<bool>,
        planetary: Option<bool>,
        spacecraft: Option<bool>,
        eulerparams: Option<bool>,
        locations: Option<bool>,
    ) -> Vec<String> {
        let print_any = spk.unwrap_or(false)
            || bpc.unwrap_or(false)
            || planetary.unwrap_or(false)
            || eulerparams.unwrap_or(false)
            || locations.unwrap_or(false);

        let mut kernels = vec![];

        if spk.unwrap_or(!print_any) {
            kernels.extend(self.spk_data.keys().cloned());
        }

        if bpc.unwrap_or(!print_any) {
            kernels.extend(self.bpc_data.keys().cloned());
        }

        if planetary.unwrap_or(!print_any) {
            kernels.extend(self.planetary_data.keys().cloned());
        }

        if spacecraft.unwrap_or(!print_any) {
            kernels.extend(self.spacecraft_data.keys().cloned());
        }

        if eulerparams.unwrap_or(!print_any) {
            kernels.extend(self.euler_param_data.keys().cloned());
        }

        if locations.unwrap_or(!print_any) {
            kernels.extend(self.location_data.keys().cloned());
        }

        kernels
    }
    /// Set the CRC32 of all loaded DAF files
    pub fn set_crc32(&mut self) {
        for spk in self.spk_data.values_mut() {
            spk.set_crc32();
        }
        for bpc in self.bpc_data.values_mut() {
            bpc.set_crc32();
        }
    }

    /// Load a new DAF/SPK file in place of the one in the provided alias.
    ///
    /// This reuses the existing memory buffer, growing it only if the new file
    /// is larger than the previous capacity. This effectively adopts a
    /// "high watermark" memory strategy, where the memory usage for this slot
    /// is determined by the largest file ever loaded into it.
    pub fn spk_swap(
        &mut self,
        alias: &str,
        new_spk_path: &str,
        new_alias: String,
    ) -> Result<(), AlmanacError> {
        let mut file = std::fs::File::open(new_spk_path)
            .map_err(|e| InputOutputError::IOError { kind: e.kind() })
            .context(LoadingSnafu {
                path: new_spk_path.to_string(),
            })?;

        let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);

        let entry = self
            .spk_data
            .get_mut(alias)
            .ok_or(AlmanacError::GenericError {
                err: format!("no SPK alias `{alias}`"),
            })?;

        let buffer = &mut entry.bytes;

        buffer.clear(); // Sets len to 0, keeps capacity
        buffer.reserve(file_len as usize); // Ensure we have enough space to avoid re-allocs

        // Zero-Copy Read: Stream file directly into the BytesMut
        // .writer() adapts the BytesMut to implement std::io::Write
        let mut writer = buffer.writer();
        std::io::copy(&mut file, &mut writer)
            .map_err(|e| InputOutputError::IOError { kind: e.kind() })
            .context(LoadingSnafu {
                path: new_spk_path.to_string(),
            })?;

        // 5. Handle Renaming
        if alias != new_alias {
            // Use shift remove instead of swap remove to preserve loading order.
            if let Some(entry) = self.spk_data.shift_remove(alias) {
                self.spk_data.insert(new_alias, entry);
            }
        }

        Ok(())
    }

    /// Load a new DAF/BPC file in place of the one in the provided alias.
    ///
    /// This reuses the existing memory buffer, growing it only if the new file
    /// is larger than the previous capacity. This effectively adopts a
    /// "high watermark" memory strategy, where the memory usage for this slot
    /// is determined by the largest file ever loaded into it.
    pub fn bpc_swap(
        &mut self,
        alias: &str,
        new_bpc_path: &str,
        new_alias: String,
    ) -> Result<(), AlmanacError> {
        let mut file = std::fs::File::open(new_bpc_path)
            .map_err(|e| InputOutputError::IOError { kind: e.kind() })
            .context(LoadingSnafu {
                path: new_bpc_path.to_string(),
            })?;

        let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);

        let entry = self
            .bpc_data
            .get_mut(alias)
            .ok_or(AlmanacError::GenericError {
                err: format!("no BPC alias `{alias}`"),
            })?;

        let buffer = &mut entry.bytes;

        buffer.clear(); // Sets len to 0, keeps capacity
        buffer.reserve(file_len as usize); // Ensure we have enough space to avoid re-allocs

        // Zero-Copy Read: Stream file directly into the BytesMut
        // .writer() adapts the BytesMut to implement std::io::Write
        let mut writer = buffer.writer();
        std::io::copy(&mut file, &mut writer)
            .map_err(|e| InputOutputError::IOError { kind: e.kind() })
            .context(LoadingSnafu {
                path: new_bpc_path.to_string(),
            })?;

        // 5. Handle Renaming
        if alias != new_alias {
            // Use shift remove instead of swap remove to preserve loading order.
            if let Some(entry) = self.bpc_data.shift_remove(alias) {
                self.bpc_data.insert(new_alias, entry);
            }
        }

        Ok(())
    }

    /// Returns the Euler parameter from its ID, searching through all loaded EP datasets in reverse order.
    pub(crate) fn euler_param_from_id(
        &self,
        id: NaifId,
    ) -> Result<EulerParameter, OrientationError> {
        for data in self.euler_param_data.values().rev() {
            if let Ok(datum) = data.get_by_id(id) {
                return Ok(datum);
            }
        }

        Err(OrientationError::OrientationDataSet {
            source: DataSetError::DataSetLut {
                action: "fetching by ID",
                source: LutError::UnknownId { id },
            },
        })
    }
}
