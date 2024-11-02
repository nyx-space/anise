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

use crate::ephemerides::NoEphemerisLoadedSnafu;
use crate::naif::daf::DAFError;
use crate::naif::daf::NAIFSummaryRecord;
use crate::naif::spk::summary::SPKSummaryRecord;
use crate::naif::SPK;
use crate::{ephemerides::EphemerisError, NaifId};
use log::error;

use super::{Almanac, MAX_LOADED_SPKS};

impl Almanac {
    pub fn from_spk(spk: SPK) -> Result<Almanac, EphemerisError> {
        let me = Self::default();
        me.with_spk(spk)
    }

    /// Loads a new SPK file into a new context.
    /// This new context is needed to satisfy the unloading of files. In fact, to unload a file, simply let the newly loaded context drop out of scope and Rust will clean it up.
    pub fn with_spk(&self, spk: SPK) -> Result<Self, EphemerisError> {
        // This is just a bunch of pointers so it doesn't use much memory.
        let mut me = self.clone();
        // Parse as SPK and place into the SPK list if there is room
        let mut data_idx = MAX_LOADED_SPKS;
        for (idx, item) in self.spk_data.iter().enumerate() {
            if item.is_none() {
                data_idx = idx;
                break;
            }
        }
        if data_idx == MAX_LOADED_SPKS {
            return Err(EphemerisError::StructureIsFull {
                max_slots: MAX_LOADED_SPKS,
            });
        }
        me.spk_data[data_idx] = Some(spk);
        Ok(me)
    }
}

impl Almanac {
    pub fn num_loaded_spk(&self) -> usize {
        let mut count = 0;
        for maybe in &self.spk_data {
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
    ) -> Result<(&SPKSummaryRecord, usize, usize), EphemerisError> {
        for (spk_no, maybe_spk) in self
            .spk_data
            .iter()
            .take(self.num_loaded_spk())
            .rev()
            .enumerate()
        {
            let spk = maybe_spk.as_ref().unwrap();
            if let Ok((summary, idx_in_spk)) = spk.summary_from_name_at_epoch(name, epoch) {
                return Ok((summary, spk_no, idx_in_spk));
            }
        }

        // If we're reached this point, there is no relevant summary at this epoch.
        error!("Almanac: No summary {name} valid at epoch {epoch}");
        Err(EphemerisError::SPK {
            action: "searching for SPK summary",
            source: DAFError::SummaryNameAtEpochError {
                kind: "SPK",
                name: name.to_string(),
                epoch,
            },
        })
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch
    pub fn spk_summary_at_epoch(
        &self,
        id: i32,
        epoch: Epoch,
    ) -> Result<(&SPKSummaryRecord, usize, usize), EphemerisError> {
        for (spk_no, maybe_spk) in self
            .spk_data
            .iter()
            .take(self.num_loaded_spk())
            .rev()
            .enumerate()
        {
            let spk = maybe_spk.as_ref().unwrap();
            if let Ok((summary, idx_in_spk)) = spk.summary_from_id_at_epoch(id, epoch) {
                // NOTE: We're iterating backward, so the correct SPK number is "total loaded" minus "current iteration".
                return Ok((summary, self.num_loaded_spk() - spk_no - 1, idx_in_spk));
            }
        }

        error!("Almanac: No summary {id} valid at epoch {epoch}");
        // If we're reached this point, there is no relevant summary at this epoch.
        Err(EphemerisError::SPK {
            action: "searching for SPK summary",
            source: DAFError::SummaryIdAtEpochError {
                kind: "SPK",
                id,
                epoch,
            },
        })
    }

    /// Returns the most recently loaded summary by its name, if any with that ID are available
    pub fn spk_summary_from_name(
        &self,
        name: &str,
    ) -> Result<(&SPKSummaryRecord, usize, usize), EphemerisError> {
        for (spk_no, maybe_spk) in self
            .spk_data
            .iter()
            .take(self.num_loaded_spk())
            .rev()
            .enumerate()
        {
            let spk = maybe_spk.as_ref().unwrap();
            if let Ok((summary, idx_in_spk)) = spk.summary_from_name(name) {
                return Ok((summary, spk_no, idx_in_spk));
            }
        }

        // If we're reached this point, there is no relevant summary at this epoch.
        error!("Almanac: No summary {name} valid");

        Err(EphemerisError::SPK {
            action: "searching for SPK summary",
            source: DAFError::SummaryNameError {
                kind: "SPK",
                name: name.to_string(),
            },
        })
    }

    /// Returns the most recently loaded summary by its ID, if any with that ID are available
    pub fn spk_summary(
        &self,
        id: i32,
    ) -> Result<(&SPKSummaryRecord, usize, usize), EphemerisError> {
        for (spk_no, maybe_spk) in self
            .spk_data
            .iter()
            .take(self.num_loaded_spk())
            .rev()
            .enumerate()
        {
            let spk = maybe_spk.as_ref().unwrap();
            if let Ok((summary, idx_in_spk)) = spk.summary_from_id(id) {
                // NOTE: We're iterating backward, so the correct SPK number is "total loaded" minus "current iteration".
                return Ok((summary, self.num_loaded_spk() - spk_no - 1, idx_in_spk));
            }
        }

        error!("Almanac: No summary {id} valid");
        // If we're reached this point, there is no relevant summary
        Err(EphemerisError::SPK {
            action: "searching for SPK summary",
            source: DAFError::SummaryIdError { kind: "SPK", id },
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
    pub fn spk_summaries(&self, id: NaifId) -> Result<Vec<SPKSummaryRecord>, EphemerisError> {
        let mut summaries = vec![];
        for maybe_spk in self.spk_data.iter().take(self.num_loaded_spk()).rev() {
            let spk = maybe_spk.as_ref().unwrap();
            if let Ok(these_summaries) = spk.data_summaries() {
                for summary in these_summaries {
                    if summary.id() == id {
                        summaries.push(*summary);
                    }
                }
            }
        }

        if summaries.is_empty() {
            error!("Almanac: No summary {id} valid");
            // If we're reached this point, there is no relevant summary
            Err(EphemerisError::SPK {
                action: "searching for SPK summary",
                source: DAFError::SummaryIdError { kind: "SPK", id },
            })
        } else {
            Ok(summaries)
        }
    }

    /// Returns the applicable domain of the request id, i.e. start and end epoch that the provided id has loaded data.
    ///
    /// :type id: int
    /// :rtype: typing.Tuple
    pub fn spk_domain(&self, id: NaifId) -> Result<(Epoch, Epoch), EphemerisError> {
        let summaries = self.spk_summaries(id)?;

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

    /// Returns a map of each loaded SPK ID to its domain validity.
    ///
    /// # Warning
    /// This function performs a memory allocation.
    ///
    /// :rtype: typing.Dict
    pub fn spk_domains(&self) -> Result<HashMap<NaifId, (Epoch, Epoch)>, EphemerisError> {
        ensure!(self.num_loaded_spk() > 0, NoEphemerisLoadedSnafu);

        let mut domains = HashMap::new();
        for maybe_spk in self.spk_data.iter().take(self.num_loaded_spk()).rev() {
            let spk = maybe_spk.as_ref().unwrap();
            if let Ok(these_summaries) = spk.data_summaries() {
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
mod ut_almanac_spk {
    use crate::{
        constants::frames::{EARTH_J2000, MOON_J2000},
        prelude::{Almanac, Epoch},
    };

    #[test]
    fn summaries_nothing_loaded() {
        let almanac = Almanac::default();
        let e = Epoch::now().unwrap();

        assert!(
            almanac.spk_summary(0).is_err(),
            "empty Almanac should report an error"
        );
        assert!(
            almanac.spk_summary_at_epoch(0, e).is_err(),
            "empty Almanac should report an error"
        );
        assert!(
            almanac.spk_summary_from_name("invalid name").is_err(),
            "empty Almanac should report an error"
        );
        assert!(
            almanac
                .spk_summary_from_name_at_epoch("invalid name", e)
                .is_err(),
            "empty Almanac should report an error"
        );
    }

    #[test]
    fn queries_nothing_loaded() {
        let almanac = Almanac::default();
        let e = Epoch::now().unwrap();

        assert!(
            almanac.try_find_ephemeris_root().is_err(),
            "empty Almanac should report an error"
        );

        assert!(
            almanac.ephemeris_path_to_root(MOON_J2000, e).is_err(),
            "empty Almanac should report an error"
        );

        assert!(
            almanac
                .common_ephemeris_path(MOON_J2000, EARTH_J2000, e)
                .is_err(),
            "empty Almanac should report an error"
        );
    }
}
