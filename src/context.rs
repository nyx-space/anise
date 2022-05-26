/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::der::Decode;
use crate::errors::IntegrityErrorKind;
use crate::log::{error, info, trace};
use crate::{
    asn1::{
        context::AniseContext, ephemeris::Ephemeris, semver::Semver, ANISE_VERSION,
        MAX_TRAJECTORIES,
    },
    errors::{AniseError, InternalErrorKind},
};
use core::convert::TryFrom;
use crc32fast::hash;
use der::Encode;
use log::warn;
use std::fs::File;
use std::io::Write;
use std::path::Path;

impl<'a> AniseContext<'a> {
    /// Try to load an Anise file from a pointer of bytes
    pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, AniseError> {
        match Self::from_der(bytes) {
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
    pub fn merge_mut(&mut self, other: Self) -> Result<(usize, usize), AniseError> {
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
            let other_e = other
                .ephemeris_data
                .get(data_idx)
                .ok_or(AniseError::IntegrityError(IntegrityErrorKind::DataMissing))?;
            if self.append_ephemeris_mut(*other_e)? {
                num_ephem_added += 1;
            }
        }

        // Append the Orientation data tables
        let mut num_orientation_added = 0;
        for new_hash in other.orientation_lut.hashes.iter() {
            let data_idx = other.orientation_lut.index_for_hash(new_hash)?.into();
            trace!("[merge] fetching orientation idx={data_idx} for hash {new_hash}");
            let other_o = other
                .orientation_data
                .get(data_idx)
                .ok_or(AniseError::IntegrityError(IntegrityErrorKind::DataMissing))?;
            if self.append_orientation_mut(*other_o)? {
                num_orientation_added += 1;
            }
        }
        Ok((num_ephem_added, num_orientation_added))
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

    /// Saves this context in the providef filename.
    /// If overwrite is set to false, and the filename already exists, this function will return an error.
    ///
    /// TODO: This function should only be available with the alloc feature gate.
    pub fn save_as(&self, filename: &'a str, overwrite: bool) -> Result<(), AniseError> {
        match self.encoded_len() {
            Err(e) => Err(AniseError::InternalError(e.into())),
            Ok(length) => {
                let len: u32 = length.into();
                // Fill the vector with zeros
                let mut buf = vec![0x0; len as usize];
                self.save_as_via_buffer(filename, overwrite, &mut buf)
            }
        }
    }

    /// Saves this context in the providef filename.
    /// If overwrite is set to false, and the filename already exists, this function will return an error.
    pub fn save_as_via_buffer(
        &self,
        filename: &'a str,
        overwrite: bool,
        buf: &mut [u8],
    ) -> Result<(), AniseError> {
        if Path::new(filename).exists() {
            if !overwrite {
                return Err(AniseError::FileExists);
            } else {
                warn!("[save_as] overwriting {filename}");
            }
        }

        match File::create(filename) {
            Ok(mut file) => {
                if let Err(e) = self.encode_to_slice(buf) {
                    return Err(InternalErrorKind::Asn1Error(e).into());
                }
                if let Err(e) = file.write_all(buf) {
                    Err(e.kind().into())
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(e.kind().into()),
        }
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
