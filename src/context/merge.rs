/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::log::{info, trace};
use crate::{structure::context::AniseContext, errors::AniseError};

impl<'a> AniseContext<'a> {
    /// Clones this context and merges it with the other.
    ///
    /// # Warning
    /// Cloning is an expensive operation.
    pub fn merge(&self, other: &'a Self) -> Result<Self, AniseError> {
        let mut me = self.clone();
        me.merge_mut(other)?;
        Ok(me)
    }

    /// Merges another Anise context into this one.
    ///
    /// # Implementation details
    /// + The creation date is set to the newest of the two creation dates
    /// + If the originators are not the same, the other originator is appended to the current one.
    /// + The metadata URI for FAIR compliance is unset in the resulting file
    ///
    /// # Potential errors
    /// + The resulting file would have too many trajectories compared to the maximum number of trajectories
    /// + Two trajectories have the same name but different contents
    /// + Incomatible versions: the versions of self and other must match
    pub fn merge_mut(&mut self, other: &'a Self) -> Result<(usize, usize), AniseError> {
        // Check the versions match (eventually, we need to make sure that the versions are compatible)
        if self.metadata.anise_version != other.metadata.anise_version {
            return Err(AniseError::IncompatibleVersion {
                got: other.metadata.anise_version,
                exp: self.metadata.anise_version,
            });
        }
        // Update the creation date
        if self.metadata.creation_date > other.metadata.creation_date {
            self.metadata.creation_date = other.metadata.creation_date;
            info!(
                "[merge] new creation data set to {}",
                self.metadata.creation_date
            );
        }
        // Append the Ephemeris data tables
        let mut num_ephem_added = 0;
        for new_hash in other.ephemeris_lut.hashes.iter() {
            let data_idx = other.ephemeris_lut.index_for_hash(new_hash)?.into();
            trace!("[merge] fetching ephemeris idx={data_idx} for hash {new_hash}");
            let other_e = other.try_ephemeris_data(data_idx)?;
            if self.append_ephemeris_mut(*other_e)? {
                num_ephem_added += 1;
            }
        }

        // Append the Orientation data tables
        let mut num_orientation_added = 0;
        for new_hash in other.orientation_lut.hashes.iter() {
            let data_idx = other.orientation_lut.index_for_hash(new_hash)?.into();
            trace!("[merge] fetching orientation idx={data_idx} for hash {new_hash}");
            let other_o = other.try_orientation_data(data_idx)?;
            if self.append_orientation_mut(*other_o)? {
                num_orientation_added += 1;
            }
        }
        Ok((num_ephem_added, num_orientation_added))
    }
}
