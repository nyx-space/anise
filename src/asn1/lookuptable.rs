/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use crc32fast::hash;
use der::{asn1::SequenceOf, Decode, Encode, Reader, Writer};

use crate::prelude::AniseError;

use super::MAX_TRAJECTORIES;

/// A LookUpTable allows looking up the data given the hash.
///
/// # Note
/// In this version of ANISE, the look up is O(N) due to a limitation in the ASN1 library used.
/// Eventually, the specification will require the hashes will be ordered for a binary search on the index,
/// thereby greatly reducing the search time for each data, from O(N) to O(log N).
#[derive(Clone, Default, PartialEq, Eq)]
pub struct LookUpTable {
    /// Hashes of the general hashing algorithm
    pub hashes: SequenceOf<u32, MAX_TRAJECTORIES>,
    /// Corresponding index for each hash
    pub indexes: SequenceOf<u16, MAX_TRAJECTORIES>,
}

impl LookUpTable {
    /// Searches the lookup table for the requested hash
    /// Returns Ok with the index for the requested hash
    /// Returns Err with an ItemNotFound if the item isn't found
    /// Returns Err with an IndexingError if the index is not present but the hash is present.
    ///
    /// NOTE: Until https://github.com/anise-toolkit/anise.rs/issues/18 is addressed
    /// this function has a time complexity of O(N)
    pub fn index_for_hash(&self, hash: &u32) -> Result<u16, AniseError> {
        for (idx, item) in self.hashes.iter().enumerate() {
            if item == hash {
                return match self.indexes.get(idx) {
                    Some(item_index) => Ok(*item_index),
                    None => Err(AniseError::IndexingError),
                };
            }
        }
        Err(AniseError::ItemNotFound)
    }

    /// Searches the lookup table for the requested key
    /// Returns Ok with the index for the hash of the requested key
    /// Returns Err with an ItemNotFound if the item isn't found
    /// Returns Err with an IndexingError if the index is not present but the hash of the name is present.
    ///
    /// NOTE: Until https://github.com/anise-toolkit/anise.rs/issues/18 is addressed
    /// this function has a time complexity of O(N)
    pub fn index_for_key(&self, key: &str) -> Result<u16, AniseError> {
        self.index_for_hash(&hash(key.as_bytes()))
    }
}

impl Encode for LookUpTable {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.hashes.encoded_len()? + self.indexes.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.hashes.encode(encoder)?;
        self.indexes.encode(encoder)
    }
}

impl<'a> Decode<'a> for LookUpTable {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            hashes: decoder.decode()?,
            indexes: decoder.decode()?,
        })
    }
}
