/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
extern crate der;

use crate::{
    asn1::{context::AniseContext, semver::Semver, ANISE_VERSION, MAX_TRAJECTORIES},
    errors::{AniseError, InternalErrorKind},
};
use der::Decode;
use std::convert::TryFrom;

impl<'a> AniseContext<'a> {
    /// Try to load an Anise file from a pointer of bytes
    pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, AniseError> {
        match Self::from_der(&bytes) {
            Ok(ctx) => Ok(ctx),
            Err(e) => {
                // If we can't load the file, let's try to load the version only to be helpful
                match Semver::from_der(&bytes[0..5]) {
                    Ok(file_version) => {
                        if file_version == ANISE_VERSION {
                            // Versions match but the rest of the file is corrupted
                            Err(AniseError::DecodingError(e))
                        } else {
                            Err(AniseError::IncompatibleVersion {
                                got: file_version,
                                exp: ANISE_VERSION,
                            })
                        }
                    }
                    Err(_) => Err(AniseError::DecodingError(e)),
                }
            }
        }
    }

    /// Forces to load an Anise file from a pointer of bytes.
    /// **Panics** if the bytes cannot be interpreted as an Anise file.
    pub fn from_bytes(buf: &'a [u8]) -> Self {
        Self::try_from_bytes(buf).unwrap()
    }

    /// Clones this context and merges it with the other.
    ///
    /// # Warning
    /// Cloning is an expensive operation.
    pub fn merge(&self, other: Self) -> Result<Self, AniseError> {
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
    pub fn merge_mut(&mut self, other: Self) -> Result<(), AniseError> {
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
        }
        // Append the Ephemeris data tables
        let mut num_added = 0;
        for new_hash in other.ephemeris_lut.hashes.iter() {
            let data_idx = other.ephemeris_lut.index_for_hash(*new_hash)?.into();
            if let Ok(self_idx) = self.ephemeris_lut.index_for_hash(*new_hash) {
                if self.ephemeris_data.get(self_idx.into()) != other.ephemeris_data.get(data_idx) {
                    // The ephemeris data differ but the name is the same
                    return Err(AniseError::IntegrityError);
                }
            } else {
                // This hash does not exist, let's append it.

                // Check that we can add one item
                if self.ephemeris_lut.indexes.len() == MAX_TRAJECTORIES {
                    // We're full!
                    return Err(AniseError::IndexingError);
                }

                self.ephemeris_lut
                    .hashes
                    .add(*new_hash)
                    .or_else(|_| Err(InternalErrorKind::LUTAppendFailure))?;
                // Push the index too
                self.ephemeris_lut
                    .indexes
                    .add(
                        (self.ephemeris_lut.indexes.len() + num_added)
                            .try_into()
                            .unwrap(),
                    )
                    .or_else(|_| Err(InternalErrorKind::LUTAppendFailure))?;
                // Add the ephemeris data
                self.ephemeris_data
                    .add(
                        *other
                            .ephemeris_data
                            .get(data_idx)
                            .ok_or(AniseError::IntegrityError)?,
                    )
                    .or_else(|_| Err(InternalErrorKind::LUTAppendFailure))?;
                // Increment the number of added items
                num_added += 1;
            }
        }

        // Append the Orientation data tables
        let mut num_added = 0;
        for new_hash in other.orientation_lut.hashes.iter() {
            let data_idx = other.orientation_lut.index_for_hash(*new_hash)?.into();
            if let Ok(self_idx) = self.orientation_lut.index_for_hash(*new_hash) {
                if self.orientation_data.get(self_idx.into())
                    != other.orientation_data.get(data_idx)
                {
                    // The orientation data differ but the name is the same
                    return Err(AniseError::IntegrityError);
                }
            } else {
                // This hash does not exist, let's append it.

                // Check that we can add one item
                if self.orientation_lut.indexes.len() == MAX_TRAJECTORIES {
                    // We're full!
                    return Err(AniseError::IndexingError);
                }

                self.orientation_lut
                    .hashes
                    .add(*new_hash)
                    .or_else(|_| Err(InternalErrorKind::LUTAppendFailure))?;
                // Push the index too
                self.orientation_lut
                    .indexes
                    .add(
                        (self.orientation_lut.indexes.len() + num_added)
                            .try_into()
                            .unwrap(),
                    )
                    .or_else(|_| Err(InternalErrorKind::LUTAppendFailure))?;
                // Add the orientation data
                self.orientation_data
                    .add(
                        *other
                            .orientation_data
                            .get(data_idx)
                            .ok_or(AniseError::IntegrityError)?,
                    )
                    .or_else(|_| Err(InternalErrorKind::LUTAppendFailure))?;
                // Increment the number of added items
                num_added += 1;
            }
        }
        Ok(())
    }

    pub fn rename_ephemeris_traj_mut(&mut self) {}
    pub fn rename_orientation_traj_mut(&mut self) {}
}

impl<'a> TryFrom<&'a [u8]> for AniseContext<'a> {
    type Error = AniseError;

    fn try_from(buf: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_bytes(buf)
    }
}
