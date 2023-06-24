/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::naif::{BPC, SPK};
use crate::structure::dataset::DataSet;
use crate::structure::planetocentric::PlanetaryData;
use crate::structure::spacecraft::SpacecraftData;
use core::fmt;

// TODO: Switch these to build constants so that it's configurable when building the library.
pub const MAX_LOADED_SPKS: usize = 32;
pub const MAX_LOADED_BPCS: usize = 8;
pub const MAX_SPACECRAFT_DATA: usize = 16;
pub const MAX_PLANETARY_DATA: usize = 64;

pub mod bpc;
pub mod spk;

/// A SPICE context contains all of the loaded SPICE data.
///
/// # Limitations
/// The stack space required depends on the maximum number of each type that can be loaded.
#[derive(Clone, Default)]
pub struct Context<'a> {
    /// NAIF SPK is kept unchanged
    pub spk_data: [Option<&'a SPK>; MAX_LOADED_SPKS],
    /// NAIF BPC is kept unchanged
    pub bpc_data: [Option<&'a BPC>; MAX_LOADED_BPCS],
    /// Dataset of planetary data
    pub planetary_data: DataSet<'a, PlanetaryData, MAX_PLANETARY_DATA>,
    /// Dataset of spacecraft data
    pub spacecraft_data: DataSet<'a, SpacecraftData<'a>, MAX_SPACECRAFT_DATA>,
}

impl<'a> fmt::Display for Context<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Context: #SPK = {}\t#BPC = {}",
            self.num_loaded_spk(),
            self.num_loaded_bpc()
        )?;
        if !self.planetary_data.lut.by_id.is_empty() {
            write!(f, "\t{}", self.planetary_data)?;
        }
        if !self.spacecraft_data.lut.by_id.is_empty() {
            write!(f, "\t{}", self.spacecraft_data)?;
        }
        Ok(())
    }
}
