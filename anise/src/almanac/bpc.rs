/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::collections::HashMap;

use hifitime::Epoch;

#[cfg(feature = "python")]
use pyo3::prelude::*;
use snafu::ensure;

use crate::naif::daf::NAIFSummaryRecord;
use crate::naif::pck::BPCSummaryRecord;
use crate::naif::BPC;
use crate::orientations::{NoOrientationsLoadedSnafu, OrientationError};
use crate::{naif::daf::DAFError, NaifId};
use log::{error, warn};

use super::Almanac;

impl Almanac {
    pub fn from_bpc(bpc: BPC) -> Self {
        let me = Self::default();
        me.with_bpc(bpc)
    }

    /// Loads a new Binary Planetary Constants (BPC) kernel into a new context, using the system time as the alias. If the time is not availble, then 0 TAI is used.
    /// This new context is needed to satisfy the unloading of files. In fact, to unload a file, simply let the newly loaded context drop out of scope and Rust will clean it up.
    pub fn with_bpc(self, bpc: BPC) -> Self {
        self.with_bpc_as(bpc, None)
    }

    /// Loads a new Binary Planetary Constant (BPC) file into a new context, naming it with the provided alias or the current system time.
    /// To unload a file, call bpc_unload.
    pub fn with_bpc_as(mut self, bpc: BPC, alias: Option<String>) -> Self {
        // For lifetime reasons, we format the message using a ref first
        let alias = alias.unwrap_or(Epoch::now().unwrap_or_default().to_string());
        let msg = format!("unloading BPC `{alias}`");
        if self.bpc_data.insert(alias, bpc).is_some() {
            warn!("{msg}");
        }
        self
    }

    /// Unloads the BPC with the provided alias.
    /// **WARNING:** This causes the order of the loaded files to be perturbed, which may be an issue if several SPKs with the same IDs are loaded.
    pub fn bpc_unload(&mut self, alias: &str) -> Result<(), OrientationError> {
        if self.bpc_data.swap_remove(alias).is_none() {
            Err(OrientationError::AliasNotFound {
                alias: alias.to_string(),
                action: "unload BPC",
            })
        } else {
            Ok(())
        }
    }
    pub fn num_loaded_bpc(&self) -> usize {
        self.bpc_data.len()
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch and the BPC where this name was found to be valid at that epoch.
    pub fn bpc_summary_from_name_at_epoch(
        &self,
        name: &str,
        epoch: Epoch,
    ) -> Result<(&BPCSummaryRecord, usize, usize), OrientationError> {
        for (no, bpc) in self.bpc_data.values().rev().enumerate() {
            if let Ok((summary, idx_in_bpc)) = bpc.summary_from_name_at_epoch(name, epoch) {
                return Ok((summary, no, idx_in_bpc));
            }
        }

        // If we're reached this point, there is no relevant summary at this epoch.
        Err(OrientationError::BPC {
            action: "searching for BPC summary",
            source: DAFError::SummaryNameAtEpochError {
                kind: "BPC",
                name: name.to_string(),
                epoch,
            },
        })
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch
    pub fn bpc_summary_at_epoch(
        &self,
        id: i32,
        epoch: Epoch,
    ) -> Result<(&BPCSummaryRecord, usize, usize), OrientationError> {
        for (no, bpc) in self.bpc_data.values().rev().enumerate() {
            if let Ok((summary, idx_in_bpc)) = bpc.summary_from_id_at_epoch(id, epoch) {
                // NOTE: We're iterating backward, so the correct BPC number is "total loaded" minus "current iteration".
                return Ok((summary, self.num_loaded_bpc() - no - 1, idx_in_bpc));
            }
        }

        // If the ID is not present at all, bpc_domain will report it.
        let (start, end) = self.bpc_domain(id)?;
        error!("Almanac: summary {id} valid from {start} to {end} but not at requested {epoch}");
        // If we're reached this point, there is no relevant summary at this epoch.
        Err(OrientationError::BPC {
            action: "searching for SPK summary",
            source: DAFError::SummaryIdAtEpochError {
                kind: "BPC",
                id,
                epoch,
                start,
                end,
            },
        })
    }

    /// Returns the summary given the name of the summary record.
    pub fn bpc_summary_from_name(
        &self,
        name: &str,
    ) -> Result<(&BPCSummaryRecord, usize, usize), OrientationError> {
        for (bpc_no, bpc) in self.bpc_data.values().rev().enumerate() {
            if let Ok((summary, idx_in_bpc)) = bpc.summary_from_name(name) {
                return Ok((summary, bpc_no, idx_in_bpc));
            }
        }

        // If we're reached this point, there is no relevant summary at this epoch.
        Err(OrientationError::BPC {
            action: "searching for BPC summary",
            source: DAFError::SummaryNameError {
                kind: "BPC",
                name: name.to_string(),
            },
        })
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch
    pub fn bpc_summary(
        &self,
        id: i32,
    ) -> Result<(&BPCSummaryRecord, usize, usize), OrientationError> {
        for (no, bpc) in self.bpc_data.values().rev().enumerate() {
            if let Ok((summary, idx_in_bpc)) = bpc.summary_from_id(id) {
                // NOTE: We're iterating backward, so the correct BPC number is "total loaded" minus "current iteration".
                return Ok((summary, self.num_loaded_bpc() - no - 1, idx_in_bpc));
            }
        }

        // If we're reached this point, there is no relevant summary
        Err(OrientationError::BPC {
            action: "searching for BPC summary",
            source: DAFError::SummaryIdError { kind: "BPC", id },
        })
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Almanac {
    /// Returns a vector of the summaries whose ID matches the desired `id`, in the order in which they will be used, i.e. in reverse loading order.
    ///
    /// # Warning
    /// This function performs a memory allocation.
    ///
    /// :type id: int
    /// :rtype: typing.List
    pub fn bpc_summaries(&self, id: NaifId) -> Result<Vec<BPCSummaryRecord>, OrientationError> {
        let mut summaries = vec![];

        for bpc in self.bpc_data.values().rev() {
            if let Ok(these_summaries) = bpc.data_summaries() {
                for summary in these_summaries {
                    if summary.id() == id {
                        summaries.push(*summary);
                    }
                }
            }
        }

        if summaries.is_empty() {
            // If we're reached this point, there is no relevant summary
            Err(OrientationError::BPC {
                action: "searching for BPC summary",
                source: DAFError::SummaryIdError { kind: "BPC", id },
            })
        } else {
            Ok(summaries)
        }
    }

    /// Returns the applicable domain of the request id, i.e. start and end epoch that the provided id has loaded data.
    ///
    /// :type id: int
    /// :rtype: typing.Tuple
    pub fn bpc_domain(&self, id: NaifId) -> Result<(Epoch, Epoch), OrientationError> {
        let summaries = self.bpc_summaries(id)?;

        // We know that the summaries is non-empty because if it is, the previous function call returns an error.
        let start = summaries
            .iter()
            .min_by_key(|summary| summary.start_epoch())
            .unwrap()
            .start_epoch();

        let end = summaries
            .iter()
            .max_by_key(|summary| summary.end_epoch())
            .unwrap()
            .end_epoch();

        Ok((start, end))
    }

    /// Returns a map of each loaded BPC ID to its domain validity.
    ///
    /// # Warning
    /// This function performs a memory allocation.
    ///
    /// :rtype: typing.Dict
    pub fn bpc_domains(&self) -> Result<HashMap<NaifId, (Epoch, Epoch)>, OrientationError> {
        ensure!(self.num_loaded_bpc() > 0, NoOrientationsLoadedSnafu);

        let mut domains = HashMap::new();
        for bpc in self.bpc_data.values().rev() {
            if let Ok(these_summaries) = bpc.data_summaries() {
                for summary in these_summaries {
                    let this_id = summary.id();
                    match domains.get_mut(&this_id) {
                        Some((ref mut cur_start, ref mut cur_end)) => {
                            if *cur_start > summary.start_epoch() {
                                *cur_start = summary.start_epoch();
                            }
                            if *cur_end < summary.end_epoch() {
                                *cur_end = summary.end_epoch();
                            }
                        }
                        None => {
                            domains.insert(this_id, (summary.start_epoch(), summary.end_epoch()));
                        }
                    }
                }
            }
        }

        Ok(domains)
    }
}

#[cfg(test)]
mod ut_almanac_bpc {
    use crate::prelude::{Almanac, Epoch};

    #[test]
    fn summaries_nothing_loaded() {
        let almanac = Almanac::default();

        let e = Epoch::now().unwrap();

        assert!(
            almanac.bpc_summary(0).is_err(),
            "empty Almanac should report an error"
        );
        assert!(
            almanac.bpc_summary_at_epoch(0, e).is_err(),
            "empty Almanac should report an error"
        );
        assert!(
            almanac.bpc_summary_from_name("invalid name").is_err(),
            "empty Almanac should report an error"
        );
        assert!(
            almanac
                .bpc_summary_from_name_at_epoch("invalid name", e)
                .is_err(),
            "empty Almanac should report an error"
        );
    }
}
