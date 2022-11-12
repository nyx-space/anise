/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::naif::{daf::DAF, pck::BPCSummaryRecord, spk::summary::SPKSummaryRecord};

use crate::errors::AniseError;

/// A SPICE context contains all of the loaded SPICE data.
///
/// # Limitations
/// You may only load up to 32 SPICE files of each kind.
/// The stack space does _not_ depend on how much data is loaded at any given time.
#[derive(Clone, Default)]
pub struct SpiceContext<'a> {
    // TODO: Add ANISE context here too.
    pub spk_data: [Option<&'a DAF<'a, SPKSummaryRecord>>; 32],
    pub bpc_data: [Option<&'a DAF<'a, BPCSummaryRecord>>; 32],
}

impl<'a: 'b, 'b> SpiceContext<'a> {
    /// Loads a new SPK file into a new context.
    /// This new context is needed to satisfy the unloading of files. In fact, to unload a file, simply let the newly loaded context drop out of scope and Rust will clean it up.
    pub fn load_spk(
        &self,
        spk: &'b DAF<'b, SPKSummaryRecord>,
    ) -> Result<SpiceContext<'b>, AniseError> {
        // This is just a bunch of pointers so it doesn't use much memory.
        let mut me = self.clone();
        // Parse as SPK and place into the SPK list if there is room
        let mut data_idx = 32;
        for (idx, item) in self.spk_data.iter().enumerate() {
            if item.is_none() {
                data_idx = idx;
                break;
            }
        }
        if data_idx == 32 {
            return Err(AniseError::StructureIsFull);
        }
        me.spk_data[data_idx] = Some(spk);
        Ok(me)
    }

    /// Loads a Binary Planetary Constants kernel.
    pub fn load_bpc(
        &self,
        bpc: &'b DAF<'b, BPCSummaryRecord>,
    ) -> Result<SpiceContext<'b>, AniseError> {
        // This is just a bunch of pointers so it doesn't use much memory.
        let mut me = self.clone();
        // Parse as SPK and place into the SPK list if there is room
        let mut data_idx = 32;
        for (idx, item) in self.bpc_data.iter().enumerate() {
            if item.is_none() {
                data_idx = idx;
                break;
            }
        }
        if data_idx == 32 {
            return Err(AniseError::StructureIsFull);
        }
        me.bpc_data[data_idx] = Some(bpc);
        Ok(me)
    }
}
