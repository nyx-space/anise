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
use crate::naif::pck::BPCSummaryRecord;
use crate::naif::BPC;
use log::error;

use super::{Context, MAX_LOADED_BPCS};

impl<'a: 'b, 'b> Context<'a> {
    /// Loads a Binary Planetary Constants kernel.
    pub fn load_bpc(&self, bpc: &'b BPC) -> Result<Context<'b>, AniseError> {
        // This is just a bunch of pointers so it doesn't use much memory.
        let mut me = self.clone();
        let mut data_idx = MAX_LOADED_BPCS;
        for (idx, item) in self.bpc_data.iter().enumerate() {
            if item.is_none() {
                data_idx = idx;
                break;
            }
        }
        if data_idx == MAX_LOADED_BPCS {
            return Err(AniseError::StructureIsFull);
        }
        me.bpc_data[data_idx] = Some(bpc);
        Ok(me)
    }

    pub fn num_loaded_bpc(&self) -> usize {
        let mut count = 0;
        for maybe in self.bpc_data {
            if maybe.is_none() {
                break;
            } else {
                count += 1;
            }
        }

        count
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch and the BPC where this name was found to be valid at that epoch.
    pub fn bpc_summary_from_name_at_epoch(
        &self,
        name: &str,
        epoch: Epoch,
    ) -> Result<(&BPCSummaryRecord, usize, usize), AniseError> {
        for (no, maybe_bpc) in self
            .bpc_data
            .iter()
            .take(self.num_loaded_bpc())
            .rev()
            .enumerate()
        {
            let bpc = maybe_bpc.unwrap();
            if let Ok((summary, idx_in_bpc)) = bpc.summary_from_name_at_epoch(name, epoch) {
                return Ok((summary, no, idx_in_bpc));
            }
        }

        // If we're reached this point, there is no relevant summary at this epoch.
        error!("Context: No summary {name} valid at epoch {epoch}");
        Err(AniseError::MissingInterpolationData(epoch))
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch
    pub fn bpc_summary_at_epoch(
        &self,
        id: i32,
        epoch: Epoch,
    ) -> Result<(&BPCSummaryRecord, usize, usize), AniseError> {
        // TODO: Consider a return type here
        for (no, maybe_bpc) in self
            .bpc_data
            .iter()
            .take(self.num_loaded_bpc())
            .rev()
            .enumerate()
        {
            let bpc = maybe_bpc.unwrap();
            if let Ok((summary, idx_in_bpc)) = bpc.summary_from_id_at_epoch(id, epoch) {
                // NOTE: We're iterating backward, so the correct BPC number is "total loaded" minus "current iteration".
                return Ok((summary, self.num_loaded_bpc() - no - 1, idx_in_bpc));
            }
        }

        error!("Context: No summary {id} valid at epoch {epoch}");
        // If we're reached this point, there is no relevant summary at this epoch.
        Err(AniseError::MissingInterpolationData(epoch))
    }

    /// Returns the summary given the name of the summary record.
    pub fn bpc_summary_from_name(
        &self,
        name: &str,
    ) -> Result<(&BPCSummaryRecord, usize, usize), AniseError> {
        for (bpc_no, maybe_bpc) in self
            .bpc_data
            .iter()
            .take(self.num_loaded_bpc())
            .rev()
            .enumerate()
        {
            let bpc = maybe_bpc.unwrap();
            if let Ok((summary, idx_in_bpc)) = bpc.summary_from_name(name) {
                return Ok((summary, bpc_no, idx_in_bpc));
            }
        }

        // If we're reached this point, there is no relevant summary at this epoch.
        error!("Context: No summary {name} valid");
        Err(AniseError::NoInterpolationData)
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch
    pub fn bpc_summary(&self, id: i32) -> Result<(&BPCSummaryRecord, usize, usize), AniseError> {
        // TODO: Consider a return type here
        for (no, maybe_bpc) in self
            .bpc_data
            .iter()
            .take(self.num_loaded_bpc())
            .rev()
            .enumerate()
        {
            let bpc = maybe_bpc.unwrap();
            if let Ok((summary, idx_in_bpc)) = bpc.summary_from_id(id) {
                // NOTE: We're iterating backward, so the correct BPC number is "total loaded" minus "current iteration".
                return Ok((summary, self.num_loaded_bpc() - no - 1, idx_in_bpc));
            }
        }

        error!("Context: No summary {id} valid");
        // If we're reached this point, there is no relevant summary
        Err(AniseError::NoInterpolationData)
    }
}
