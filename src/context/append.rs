/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::errors::IntegrityErrorKind;
use crate::log::{error, trace};
use crate::{
    asn1::{context::AniseContext, ephemeris::Ephemeris, MAX_TRAJECTORIES},
    errors::{AniseError, InternalErrorKind},
};
use crc32fast::hash;

impl<'a> AniseContext<'a> {
    /// Appends the provided ephemeris to this context
    ///
    /// # Implementation defails
    /// + If the provided ephemeris already exists in this context, nothing happens and this function returns Ok(false).
    /// This is an Ok to specify that no error happened and false to specify that nothing was appended
    /// + If the provided ephemeris' hash is already in the lookup table but the data do not match, this returns an Integrity Error
    /// specifying that nothing was added to prevent an integrity error.
    /// + If this ephemeris should have been added but there are already too many items compared to what the library supports,
    /// then this returns an Indexing Error.
    pub fn append_ephemeris_mut(&mut self, e: Ephemeris<'a>) -> Result<bool, AniseError> {
        let new_hash = hash(e.name.as_bytes());
        if let Ok(self_idx) = self.ephemeris_lut.index_for_hash(&new_hash) {
            if self
                .ephemeris_data
                .get(self_idx.into())
                .ok_or(AniseError::IntegrityError(IntegrityErrorKind::DataMissing))?
                != &e
            {
                error!(
                    "[append] ephemeris `{}` exists with hash {} but data differ",
                    e.name, new_hash
                );
                // The ephemeris data differ but the name is the same
                Err(AniseError::IntegrityError(
                    IntegrityErrorKind::DataMismatchOnMerge,
                ))
            } else {
                trace!(
                    "[append] nothing to do for ephemeris `{}` (data matches)",
                    e.name
                );
                // Data already exists and matches.
                Ok(false)
            }
        } else {
            // This hash does not exist, let's append it.

            // Check that we can add one item
            if self.ephemeris_lut.indexes.len() == MAX_TRAJECTORIES {
                error!("[append] cannnot append ephemeris, look up table is full");
                return Err(AniseError::IndexingError);
            }

            self.ephemeris_lut
                .hashes
                .add(new_hash)
                .map_err(|_| InternalErrorKind::LUTAppendFailure)?;
            // Push the index too
            self.ephemeris_lut
                .indexes
                .add((self.ephemeris_lut.indexes.len()).try_into().unwrap())
                .map_err(|_| InternalErrorKind::LUTAppendFailure)?;
            // Add the ephemeris data
            self.ephemeris_data
                .add(e)
                .map_err(|_| InternalErrorKind::LUTAppendFailure)?;
            trace!(
                "[append] added {} (hash={}) in position {}",
                e.name,
                new_hash,
                self.ephemeris_data.len()
            );
            Ok(true)
        }
    }

    /// Appends the provided ephemeris to this context
    ///
    /// # Implementation defails
    /// + If the provided ephemeris already exists in this context, nothing happens and this function returns Ok(false).
    /// This is an Ok to specify that no error happened and false to specify that nothing was appended
    /// + If the provided ephemeris' hash is already in the lookup table but the data do not match, this returns an Integrity Error
    /// specifying that nothing was added to prevent an integrity error.
    /// + If this ephemeris should have been added but there are already too many items compared to what the library supports,
    /// then this returns an Indexing Error.
    /// TODO: Change this from ephemeris to Orientation and update visibility
    pub(crate) fn append_orientation_mut(&mut self, o: Ephemeris<'a>) -> Result<bool, AniseError> {
        let new_hash = hash(o.name.as_bytes());
        if let Ok(self_idx) = self.orientation_lut.index_for_hash(&new_hash) {
            if self
                .orientation_data
                .get(self_idx.into())
                .ok_or(AniseError::IntegrityError(IntegrityErrorKind::DataMissing))?
                != &o
            {
                // The ephemeris data differ but the name is the same
                Err(AniseError::IntegrityError(
                    IntegrityErrorKind::DataMismatchOnMerge,
                ))
            } else {
                // Data already exists and matches.
                Ok(false)
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
                .add(new_hash)
                .map_err(|_| InternalErrorKind::LUTAppendFailure)?;
            // Push the index too
            self.orientation_lut
                .indexes
                .add((self.orientation_lut.indexes.len()).try_into().unwrap())
                .map_err(|_| InternalErrorKind::LUTAppendFailure)?;
            // Add the ephemeris data
            self.orientation_data
                .add(o)
                .map_err(|_| InternalErrorKind::LUTAppendFailure)?;
            // Increment the number of added items
            Ok(true)
        }
    }
}
