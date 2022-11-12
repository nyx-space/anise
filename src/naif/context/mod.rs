/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::naif::{daf::DAFBytes, pck::BPCSummaryRecord, spk::summary::SPKSummaryRecord};

use crate::errors::AniseError;

/// A SPICE context contains all of the loaded SPICE data.
///
/// # Limitations
/// You may only load up to 32 SPICE files of each kind.
/// The stack space does _not_ depend on how much data is loaded at any given time.
#[derive(Default)]
pub struct SpiceContext<'a> {
    pub spk_lut: [Option<&'a str>; 32],
    pub bpc_lut: [Option<&'a str>; 32],
    pub spk_data: [Option<&'a DAFBytes<'a, SPKSummaryRecord>>; 32],
    pub bpc_data: [Option<&'a DAFBytes<'a, BPCSummaryRecord>>; 32],
}

impl<'a> SpiceContext<'a> {
    pub fn furnsh_spk(
        &mut self,
        name: &'a str,
        spk: &'a DAFBytes<'a, SPKSummaryRecord>,
    ) -> Result<(), AniseError> {
        // Parse as SPK and place into the SPK list if there is room
        let mut data_idx = 32;
        for (idx, item) in self.spk_data.iter().enumerate() {
            if item.is_none() {
                data_idx = idx;
                break;
            }
        }
        if data_idx == 32 {
            return Err(AniseError::MaxTreeDepth);
        }
        self.spk_lut[data_idx] = Some(name);
        self.spk_data[data_idx] = Some(spk);
        Ok(())
    }

    pub fn furnsh_bpc(
        &mut self,
        name: &'a str,
        bpc: &'a DAFBytes<'a, BPCSummaryRecord>,
    ) -> Result<(), AniseError> {
        // Parse as SPK and place into the SPK list if there is room
        let mut data_idx = 32;
        for (idx, item) in self.bpc_data.iter().enumerate() {
            if item.is_none() {
                data_idx = idx;
                break;
            }
        }
        if data_idx == 32 {
            return Err(AniseError::MaxTreeDepth);
        }
        self.bpc_lut[data_idx] = Some(name);
        self.bpc_data[data_idx] = Some(bpc);
        Ok(())
    }

    pub fn unfurnsh_spk(&mut self, name: &'a str) -> Result<(), AniseError> {
        // Iterate through the LUT to find that name.
        let mut pos_idx = 0;
        for (idx, item) in self.spk_lut.iter().enumerate() {
            match item {
                None => return Err(AniseError::ItemNotFound), // Data is contiguous, so this mean we're found nothing
                Some(obj_name) => {
                    if &name == obj_name {
                        self.spk_lut[idx] = None;
                        self.spk_data[idx] = None;
                        pos_idx = idx;
                        break;
                    }
                }
            }
        }

        // Now move everything up.
        if pos_idx > 0 {
            // Find the first non-null
            let mut final_idx = 0;
            for (rev_idx, item) in self.spk_lut.iter().rev().enumerate() {
                if item.is_some() {
                    final_idx = rev_idx;
                    break;
                }
            }
            if final_idx > pos_idx {
                // Move everything up.
                for mov_idx in pos_idx..final_idx {
                    self.spk_lut[mov_idx] = self.spk_lut[mov_idx + 1];
                    self.spk_data[mov_idx] = self.spk_data[mov_idx + 1];
                }
            }
        }
        return Ok(());
    }

    pub fn unfurnsh_bpc(&mut self, name: &'a str) -> Result<(), AniseError> {
        // Ugh, I couldn't make it generic
        /*
                error[E0508]: cannot move out of type `[Option<DAFBytes<'_, R>>]`, a non-copy slice
           --> src/naif/context/mod.rs:168:33
            |
        168 |                 data[mov_idx] = data[mov_idx + 1];
            |                                 ^^^^^^^^^^^^^^^^^
            |                                 |
            |                                 cannot move out of here
            |                                 move occurs because `data[_]` has type `Option<DAFBytes<'_, R>>`, which does not implement the `Copy` trait

                */
        // Iterate through the LUT to find that name.
        let mut pos_idx = 0;
        for (idx, item) in self.bpc_lut.iter().enumerate() {
            match item {
                None => return Err(AniseError::ItemNotFound), // Data is contiguous, so this mean we're found nothing
                Some(obj_name) => {
                    if &name == obj_name {
                        self.bpc_lut[idx] = None;
                        self.bpc_data[idx] = None;
                        pos_idx = idx;
                        break;
                    }
                }
            }
        }

        // Now move everything up.
        if pos_idx > 0 {
            // Find the first non-null
            let mut final_idx = 0;
            for (rev_idx, item) in self.bpc_lut.iter().rev().enumerate() {
                if item.is_some() {
                    final_idx = rev_idx;
                    break;
                }
            }
            if final_idx > pos_idx {
                // Move everything up.
                for mov_idx in pos_idx..final_idx {
                    self.bpc_lut[mov_idx] = self.bpc_lut[mov_idx + 1];
                    self.bpc_data[mov_idx] = self.bpc_data[mov_idx + 1];
                }
            }
        }
        return Ok(());
    }
}
