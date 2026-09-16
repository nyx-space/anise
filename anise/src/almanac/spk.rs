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
use crate::naif::SPK;
use crate::naif::daf::DAFError;
use crate::naif::daf::NAIFSummaryRecord;
use crate::naif::spk::summary::SPKSummaryRecord;
use crate::{NaifId, ephemerides::EphemerisError};
use log::{error, warn};

use super::Almanac;

impl Almanac {
    pub fn from_spk(spk: SPK) -> Self {
        let me = Self::default();
        me.with_spk(spk)
    }

    /// Loads a new SPK file into a new context, using the system time as the alias. If the time is not availble, then 0 TAI is used.
    /// This new context is needed to satisfy the unloading of files. In fact, to unload a file, simply let the newly loaded context drop out of scope and Rust will clean it up.
    pub fn with_spk(self, spk: SPK) -> Self {
        self.with_spk_as(spk, None)
    }

    /// Loads a new SPK file into a new context, naming it with the provided alias, or the current system time if no alias is provided.
    /// To unload a file, call spk_unload.
    pub fn with_spk_as(mut self, spk: SPK, alias: Option<String>) -> Self {
        // For lifetime reasons, we format the message using a ref first.
        // This message is only displayed if there was something with that name before.
        let alias = alias.unwrap_or(Epoch::now().unwrap_or_default().to_string());
        let msg = format!("unloading SPK `{alias}`");
        if self.spk_data.insert(alias, spk).is_some() {
            warn!("{msg}");
        }
        self
    }

    /// Unloads the SPK with the provided alias.
    /// **WARNING:** This causes the order of the loaded files to be perturbed, which may be an issue if several SPKs with the same IDs are loaded.
    pub fn spk_unload(&mut self, alias: &str) -> Result<(), EphemerisError> {
        if self.spk_data.swap_remove(alias).is_none() {
            Err(EphemerisError::AliasNotFound {
                alias: alias.to_string(),
                action: "unload ephemeris",
            })
        } else {
            Ok(())
        }
    }
}

impl Almanac {
    pub fn num_loaded_spk(&self) -> usize {
        self.spk_data.len()
    }

    /// Returns the summary given the name of the summary record if that summary has data defined at the requested epoch and the SPK where this name was found to be valid at that epoch.
    pub fn spk_summary_from_name_at_epoch(
        &self,
        name: &str,
        epoch: Epoch,
    ) -> Result<(&SPKSummaryRecord, usize, Option<usize>, usize), EphemerisError> {
        for (spk_no, spk) in self.spk_data.values().rev().enumerate() {
            if let Ok((summary, daf_idx, idx_in_spk)) = spk.summary_from_name_at_epoch(name, epoch)
            {
                // NOTE: We're iterating backward, so the correct SPK number is "total loaded" minus "current iteration".
                return Ok((
                    summary,
                    self.num_loaded_spk() - spk_no - 1,
                    daf_idx,
                    idx_in_spk,
                ));
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
        epoch_et_s: f64,
    ) -> Result<(&SPKSummaryRecord, usize, Option<usize>, usize), EphemerisError> {
        for (spk_no, spk) in self.spk_data.values().rev().enumerate() {
            if let Ok((summary, daf_idx, idx_in_spk)) = spk.summary_from_id_at_epoch(id, epoch_et_s)
            {
                // NOTE: We're iterating backward, so the correct SPK number is "total loaded" minus "current iteration".
                return Ok((
                    summary,
                    self.num_loaded_spk() - spk_no - 1,
                    daf_idx,
                    idx_in_spk,
                ));
            }
        }

        // If the ID is not present at all, spk_domain_and_gap will report it.
        let (start, end, has_gap) = self.spk_domain_and_gap(id)?;
        let gap_str = if has_gap { " (with gaps)" } else { "" };
        error!(
            "Almanac: summary {id} valid from {start} to {end}{gap_str} but not at requested {epoch_et_s}"
        );
        // If we're reached this point, there is no relevant summary at this epoch.
        Err(EphemerisError::SPK {
            action: "searching for SPK summary",
            source: DAFError::SummaryIdAtEpochError {
                kind: "SPK",
                id,
                epoch: Epoch::from_et_seconds(epoch_et_s),
                start,
                end,
                has_gap,
            },
        })
    }

    /// Returns the most recently loaded summary by its name, if any with that ID are available
    pub fn spk_summary_from_name(
        &self,
        name: &str,
    ) -> Result<(&SPKSummaryRecord, usize, Option<usize>, usize), EphemerisError> {
        for (spk_no, spk) in self.spk_data.values().rev().enumerate() {
            if let Ok((summary, daf_idx, idx_in_spk)) = spk.summary_from_name(name) {
                // NOTE: We're iterating backward, so the correct SPK number is "total loaded" minus "current iteration".
                return Ok((
                    summary,
                    self.num_loaded_spk() - spk_no - 1,
                    daf_idx,
                    idx_in_spk,
                ));
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
    ) -> Result<(&SPKSummaryRecord, usize, Option<usize>, usize), EphemerisError> {
        for (spk_no, spk) in self.spk_data.values().rev().enumerate() {
            if let Ok((summary, daf_idx, idx_in_spk)) = spk.summary_from_id(id) {
                // NOTE: We're iterating backward, so the correct SPK number is "total loaded" minus "current iteration".
                return Ok((
                    summary,
                    self.num_loaded_spk() - spk_no - 1,
                    daf_idx,
                    idx_in_spk,
                ));
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
        for spk in self.spk_data.values().rev() {
            for these_summaries in spk.iter_summary_blocks().flatten() {
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
        let (start, end, _) = self.spk_domain_and_gap(id)?;
        Ok((start, end))
    }

    pub(crate) fn spk_domain_and_gap(
        &self,
        id: NaifId,
    ) -> Result<(Epoch, Epoch, bool), EphemerisError> {
        let mut summaries = self.spk_summaries(id)?;
        summaries.sort_by_key(|summary| summary.start_epoch());

        let start = summaries.first().expect("summaries is non-empty").start_epoch();
        let end = summaries
            .iter()
            .map(|s| s.end_epoch())
            .max()
            .expect("summaries is non-empty");

        let mut has_gap = false;
        let mut max_covered = summaries[0].end_epoch();
        for summary in summaries.iter().skip(1) {
            if summary.start_epoch() > max_covered {
                has_gap = true;
                break;
            }
            if summary.end_epoch() > max_covered {
                max_covered = summary.end_epoch();
            }
        }

        Ok((start, end, has_gap))
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
        for spk in self.spk_data.values().rev() {
            for these_summaries in spk.iter_summary_blocks().flatten() {
                for summary in these_summaries {
                    let this_id = summary.id();
                    match domains.get_mut(&this_id) {
                        Some((cur_start, cur_end)) => {
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
        constants::frames::{EARTH_ICRS, MOON_ICRS},
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
            almanac.spk_summary_at_epoch(0, e.to_et_seconds()).is_err(),
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
    fn spk_domain_gap_error_message() {
        use crate::constants::frames::EARTH_ICRS;
        use crate::ephemerides::ephemeris::Ephemeris;
        use crate::math::Vector6;
        use crate::prelude::Orbit;
        use hifitime::Unit;

        let start1 = Epoch::from_gregorian_utc_at_midnight(2020, 1, 1);
        let end1 = start1 + Unit::Day * 1;
        let start2 = start1 + Unit::Day * 10;
        let end2 = start2 + Unit::Day * 1;

        let mut ephem1 = Ephemeris::new("TEST1".to_string());
        ephem1.set_degree(1).unwrap();
        ephem1.insert_orbit(Orbit::from_cartesian_pos_vel(
            Vector6::zeros(),
            start1,
            EARTH_ICRS,
        ));
        ephem1.insert_orbit(Orbit::from_cartesian_pos_vel(
            Vector6::zeros(),
            end1,
            EARTH_ICRS,
        ));

        let mut ephem2 = Ephemeris::new("TEST2".to_string());
        ephem2.set_degree(1).unwrap();
        ephem2.insert_orbit(Orbit::from_cartesian_pos_vel(
            Vector6::zeros(),
            start2,
            EARTH_ICRS,
        ));
        ephem2.insert_orbit(Orbit::from_cartesian_pos_vel(
            Vector6::zeros(),
            end2,
            EARTH_ICRS,
        ));

        let spk1 = ephem1.to_spice_bsp(-159, None).unwrap();
        let spk2 = ephem2.to_spice_bsp(-159, None).unwrap();

        let almanac = Almanac::from_spk(spk1).with_spk(spk2);

        let query_epoch = start1 + Unit::Day * 5;
        let err = almanac.spk_summary_at_epoch(-159, query_epoch.to_et_seconds()).unwrap_err();
        let err_msg = format!("{err}");
        assert!(
            err_msg.contains("(with gaps)"),
            "Error message should indicate domain gap: {err_msg}"
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
            almanac
                .ephemeris_path_to_root(MOON_ICRS, e.to_et_seconds())
                .is_err(),
            "empty Almanac should report an error"
        );

        assert!(
            almanac
                .common_ephemeris_path(MOON_ICRS, EARTH_ICRS, e.to_et_seconds())
                .is_err(),
            "empty Almanac should report an error"
        );
    }
}
