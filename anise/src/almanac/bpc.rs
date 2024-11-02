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

use super::{Almanac, MAX_LOADED_BPCS};

impl Almanac {
    pub fn from_bpc(bpc: BPC) -> Result<Almanac, OrientationError> {
        let me = Self::default();
        me.with_bpc(bpc)
    }

    /// Loads a Binary Planetary Constants kernel.
    pub fn with_bpc(&self, bpc: BPC) -> Result<Self, OrientationError> {
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
            return Err(OrientationError::StructureIsFull {
                max_slots: MAX_LOADED_BPCS,
            });
        }
        me.bpc_data[data_idx] = Some(bpc);
        Ok(me)
    }

    pub fn num_loaded_bpc(&self) -> usize {
        let mut count = 0;
        for maybe in &self.bpc_data {
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
    ) -> Result<(&BPCSummaryRecord, usize, usize), OrientationError> {
        for (no, maybe_bpc) in self
            .bpc_data
            .iter()
            .take(self.num_loaded_bpc())
            .rev()
            .enumerate()
        {
            let bpc = maybe_bpc.as_ref().unwrap();
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
        for (no, maybe_bpc) in self
            .bpc_data
            .iter()
            .take(self.num_loaded_bpc())
            .rev()
            .enumerate()
        {
            let bpc = maybe_bpc.as_ref().unwrap();
            if let Ok((summary, idx_in_bpc)) = bpc.summary_from_id_at_epoch(id, epoch) {
                // NOTE: We're iterating backward, so the correct BPC number is "total loaded" minus "current iteration".
                return Ok((summary, self.num_loaded_bpc() - no - 1, idx_in_bpc));
            }
        }

        // If we're reached this point, there is no relevant summary at this epoch.
        Err(OrientationError::BPC {
            action: "searching for BPC summary",
            source: DAFError::SummaryIdAtEpochError {
                kind: "BPC",
                id,
                epoch,
            },
        })
    }

    /// Returns the summary given the name of the summary record.
    pub fn bpc_summary_from_name(
        &self,
        name: &str,
    ) -> Result<(&BPCSummaryRecord, usize, usize), OrientationError> {
        for (bpc_no, maybe_bpc) in self
            .bpc_data
            .iter()
            .take(self.num_loaded_bpc())
            .rev()
            .enumerate()
        {
            let bpc = maybe_bpc.as_ref().unwrap();
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
        for (no, maybe_bpc) in self
            .bpc_data
            .iter()
            .take(self.num_loaded_bpc())
            .rev()
            .enumerate()
        {
            let bpc = maybe_bpc.as_ref().unwrap();
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

        for maybe_bpc in self.bpc_data.iter().take(self.num_loaded_bpc()).rev() {
            let bpc = maybe_bpc.as_ref().unwrap();
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
        for maybe_bpc in self.bpc_data.iter().take(self.num_loaded_bpc()).rev() {
            let bpc = maybe_bpc.as_ref().unwrap();
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
