/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::Epoch;

use crate::errors::AniseError;
use crate::naif::spk::summary::SPKSummaryRecord;
use crate::naif::{BPC, SPK};
use log::error;

pub const MAX_LOADED_FILES: usize = 32;

/// A SPICE context contains all of the loaded SPICE data.
///
/// # Limitations
/// You may only load up to 32 SPICE files of each kind.
/// The stack space does _not_ depend on how much data is loaded at any given time.
#[derive(Clone, Default)]
pub struct Context<'a> {
    // TODO: Add ANISE context here too.
    pub spk_data: [Option<&'a SPK<'a>>; MAX_LOADED_FILES],
    pub bpc_data: [Option<&'a BPC<'a>>; MAX_LOADED_FILES],
}

impl<'a: 'b, 'b> Context<'a> {
    pub fn from_spk(spk: &'a SPK<'a>) -> Result<Context<'a>, AniseError> {
        let me = Self::default();
        me.load_spk(spk)
    }

    /// Loads a new SPK file into a new context.
    /// This new context is needed to satisfy the unloading of files. In fact, to unload a file, simply let the newly loaded context drop out of scope and Rust will clean it up.
    pub fn load_spk(&self, spk: &'b SPK<'b>) -> Result<Context<'b>, AniseError> {
        // This is just a bunch of pointers so it doesn't use much memory.
        let mut me = self.clone();
        // Parse as SPK and place into the SPK list if there is room
        let mut data_idx = MAX_LOADED_FILES;
        for (idx, item) in self.spk_data.iter().enumerate() {
            if item.is_none() {
                data_idx = idx;
                break;
            }
        }
        if data_idx == MAX_LOADED_FILES {
            return Err(AniseError::StructureIsFull);
        }
        me.spk_data[data_idx] = Some(spk);
        Ok(me)
    }

    /// Loads a Binary Planetary Constants kernel.
    pub fn load_bpc(&self, bpc: &'b BPC<'a>) -> Result<Context<'b>, AniseError> {
        // This is just a bunch of pointers so it doesn't use much memory.
        let mut me = self.clone();
        // Parse as SPK and place into the SPK list if there is room
        let mut data_idx = MAX_LOADED_FILES;
        for (idx, item) in self.bpc_data.iter().enumerate() {
            if item.is_none() {
                data_idx = idx;
                break;
            }
        }
        if data_idx == MAX_LOADED_FILES {
            return Err(AniseError::StructureIsFull);
        }
        me.bpc_data[data_idx] = Some(bpc);
        Ok(me)
    }

    pub fn num_loaded_spk(&self) -> usize {
        let mut count = 0;
        for maybe in self.spk_data {
            if maybe.is_none() {
                break;
            } else {
                count += 1;
            }
        }

        count
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch and the SPK where this name was found to be valid at that epoch.
    pub fn spk_summary_from_name_at_epoch(
        &self,
        name: &str,
        epoch: Epoch,
    ) -> Result<(&SPKSummaryRecord, usize, usize), AniseError> {
        for (spkno, maybe_spk) in self
            .spk_data
            .iter()
            .take(self.num_loaded_spk())
            .rev()
            .enumerate()
        {
            let spk = maybe_spk.unwrap();
            if let Ok((summary, idx_in_spk)) = spk.summary_from_name_at_epoch(name, epoch) {
                return Ok((summary, spkno, idx_in_spk));
            }
        }

        // If we're reached this point, there is no relevant summary at this epoch.
        error!("Context: No summary {name} valid at epoch {epoch}");
        Err(AniseError::MissingInterpolationData(epoch))
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch
    pub fn spk_summary_from_id_at_epoch(
        &self,
        id: i32,
        epoch: Epoch,
    ) -> Result<(&SPKSummaryRecord, usize, usize), AniseError> {
        // TODO: Consider a return type here
        for (spkno, maybe_spk) in self
            .spk_data
            .iter()
            .take(self.num_loaded_spk())
            .rev()
            .enumerate()
        {
            let spk = maybe_spk.unwrap();
            if let Ok((summary, idx_in_spk)) = spk.summary_from_id_at_epoch(id, epoch) {
                return Ok((summary, spkno, idx_in_spk));
            } else {
                error!("Nothing in {spkno}");
            }
        }

        error!("Context: No summary {id} valid at epoch {epoch}");
        // If we're reached this point, there is no relevant summary at this epoch.
        Err(AniseError::MissingInterpolationData(epoch))
    }
}
