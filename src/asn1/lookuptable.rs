use crc32fast::hash;
use der::{asn1::SequenceOf, Decode, Encode, Reader, Writer};

use crate::prelude::AniseError;

use super::MAX_TRAJECTORIES;

// TODO: Determine if this shouldn't be a single SeqOf with a tuple or a LUT Entry of {Hash, Index}
/// A LookUpTable enables O(1) access to any ephemeris data
#[derive(Clone, Default, PartialEq)]
pub struct LookUpTable {
    /// Hashes of the general hashing algorithm
    pub hashes: SequenceOf<u32, MAX_TRAJECTORIES>,
    /// Corresponding index for each hash, may only have 65_535 entries
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
    pub fn index_for_hash(&self, hash: u32) -> Result<u16, AniseError> {
        for (idx, item) in self.hashes.iter().enumerate() {
            if item == &hash {
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
        self.index_for_hash(hash(key.as_bytes()))
    }
}

impl<'a> Encode for LookUpTable {
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
