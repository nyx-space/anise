/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{asn1::OctetStringRef, Decode, Encode, Reader, Writer};
use indexmap::IndexMap;
use log::warn;
use snafu::prelude::*;

use crate::NaifId;

/// Maximum length of a look up table name string
pub const KEY_NAME_LEN: usize = 32;

#[derive(Debug, Snafu, PartialEq)]
#[snafu(visibility(pub(crate)))]
#[non_exhaustive]
pub enum LutError {
    #[snafu(display("must provide either an ID or a name for a loop up, but provided neither"))]
    NoKeyProvided,
    #[snafu(display("ID {id} not in look up table"))]
    UnknownId { id: NaifId },
    #[snafu(display("name {name} not in look up table"))]
    UnknownName { name: String },
    #[snafu(display("Look up table index is not in dataset"))]
    InvalidIndex { index: u32 },
}

/// A LookUpTable allows finding the [u32] ("NaifId") associated with either an ID or a name.
///
/// # Note
/// _Both_ the IDs and the name MUST be unique in the look up table.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct LookUpTable {
    /// Unique IDs of each item in the LUT
    pub by_id: IndexMap<NaifId, u32>,
    /// Corresponding index for each hash
    pub by_name: IndexMap<String, u32>,
}

impl LookUpTable {
    pub fn append(&mut self, id: i32, name: &str, index: u32) -> Result<(), LutError> {
        self.by_id.insert(id, index);
        self.by_name.insert(name.to_string(), index);
        Ok(())
    }

    pub fn append_id(&mut self, id: i32, index: u32) -> Result<(), LutError> {
        self.by_id.insert(id, index);
        Ok(())
    }

    pub fn append_name(&mut self, name: &str, index: u32) -> Result<(), LutError> {
        self.by_name.insert(name.to_string(), index);
        Ok(())
    }

    /// Returns the list of entries of this LUT.
    ///
    /// Performance: O(n+m) where n is the number of IDs and m number of names.
    pub fn entries(&self) -> IndexMap<u32, (Option<NaifId>, Option<String>)> {
        let mut rtn = IndexMap::with_capacity(self.by_id.len() + self.by_name.len());

        for (id, entry) in &self.by_id {
            // IDs are unique, and this is the first iteration, so we can't be overwriting anything
            rtn.insert(*entry, (Some(*id), None));
        }

        // Now map to the names
        for (name, entry) in &self.by_name {
            if !rtn.contains_key(entry) {
                rtn.insert(*entry, (None, Some(name.clone())));
            } else {
                let val = rtn.get_mut(entry).unwrap();
                val.1 = Some(name.clone());
            }
        }

        rtn
    }

    /// Change the ID of a given entry to the new ID
    ///
    /// This will return an error if the current ID is not in the LUT, or if the new ID is already in the LUT.
    pub fn reid(&mut self, current_id: i32, new_id: i32) -> Result<(), LutError> {
        if let Some(entry) = self.by_id.swap_remove(&current_id) {
            // We can unwrap the insertion because we just removed something.
            self.by_id.insert(new_id, entry);
            Ok(())
        } else {
            Err(LutError::UnknownId { id: current_id })
        }
    }

    /// Removes this ID from the LUT if it's present.
    ///
    /// If this item was inserted with a name, it will rename accessible by the name.
    pub fn rmid(&mut self, id: i32) -> Result<(), LutError> {
        if self.by_id.swap_remove(&id).is_none() {
            Err(LutError::UnknownId { id })
        } else {
            Ok(())
        }
    }

    /// Change the ID of a given entry to the new ID
    ///
    /// This will return an error if the current ID is not in the LUT, or if the new ID is already in the LUT.
    pub fn rename(&mut self, current_name: &str, new_name: &str) -> Result<(), LutError> {
        if let Some(entry) = self.by_name.swap_remove(current_name) {
            // We can unwrap the insertion because we just removed something.
            self.by_name.insert(new_name.to_string(), entry);
            Ok(())
        } else {
            Err(LutError::UnknownName {
                name: current_name.to_string(),
            })
        }
    }

    /// Removes this ID from the LUT if it's present.
    ///
    /// If this item was inserted with a name, it will rename accessible by the name.
    pub fn rmname(&mut self, name: &str) -> Result<(), LutError> {
        if self.by_name.swap_remove(name).is_none() {
            Err(LutError::UnknownName {
                name: name.to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Returns the length of the LONGEST of the two look up indexes
    pub fn len(&self) -> usize {
        self.by_id.len().max(self.by_name.len())
    }

    /// Returns whether this dataset is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn check_integrity(&self) -> bool {
        if self.by_id.is_empty() || self.by_name.is_empty() {
            // If either map is empty, the LUT is integral because there cannot be
            // any inconsistencies between both maps
            true
        } else if self.by_id.len() != self.by_name.len() {
            // Mismatched lengths, integrity check failed
            false
        } else {
            // Iterate through each item in by_id
            for entry in self.by_id.values() {
                // Check if the entry exists in by_name
                if !self.by_name.values().any(|name_entry| name_entry == entry) {
                    return false;
                }
            }
            true
        }
    }

    /// Builds the DER encoding of this look up table.
    fn der_encoding(&self) -> (Vec<i32>, Vec<u32>, Vec<OctetStringRef<'_>>, Vec<u32>) {
        // Build the list of entries
        let mut id_entries = Vec::<u32>::with_capacity(self.by_id.len());
        let mut name_entries = Vec::<u32>::with_capacity(self.by_name.len());

        // Build the list of keys
        let mut ids = Vec::<i32>::with_capacity(self.by_id.len());
        for (id, index) in &self.by_id {
            ids.push(*id);
            id_entries.push(*index);
        }
        // Build the list of names
        let mut names = Vec::<OctetStringRef>::with_capacity(self.by_name.len());
        for (name, index) in &self.by_name {
            names.push(OctetStringRef::new(name.as_bytes()).unwrap());

            name_entries.push(*index);
        }

        (ids, id_entries, names, name_entries)
    }
}

impl Encode for LookUpTable {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let (ids, names, id_entries, name_entries) = self.der_encoding();
        ids.encoded_len()?
            + names.encoded_len()?
            + id_entries.encoded_len()?
            + name_entries.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        let (ids, names, id_entries, name_entries) = self.der_encoding();
        ids.encode(encoder)?;
        names.encode(encoder)?;
        id_entries.encode(encoder)?;
        name_entries.encode(encoder)
    }
}

impl<'a> Decode<'a> for LookUpTable {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        // Decode as sequences and use that to build the look up table.
        let mut lut = Self::default();
        let ids: Vec<i32> = decoder.decode()?;
        let id_entries: Vec<u32> = decoder.decode()?;
        let names: Vec<OctetStringRef> = decoder.decode()?;
        let name_entries: Vec<u32> = decoder.decode()?;

        for (id, index) in ids.iter().zip(id_entries.iter()) {
            lut.by_id.insert(*id, *index);
        }

        for (name, entry) in names.iter().zip(name_entries.iter()) {
            let key = core::str::from_utf8(name.as_bytes())?;
            lut.by_name
                .insert(key[..KEY_NAME_LEN.min(key.len())].to_string(), *entry);
        }

        if !lut.check_integrity() {
            warn!(
                "decoded lookup table is not integral: {} names but {} ids",
                lut.by_name.len(),
                lut.by_id.len()
            );
        }
        Ok(lut)
    }
}

#[cfg(test)]
mod lut_ut {
    use super::{Decode, Encode, LookUpTable};
    #[test]
    fn zero_repr() {
        let repr = LookUpTable::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = LookUpTable::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        dbg!(repr);
        assert_eq!(core::mem::size_of::<LookUpTable>(), 144);
    }

    #[test]
    fn repr_ids_only() {
        let mut repr = LookUpTable::default();
        for i in 0..32 {
            let id = -20 - i;
            repr.append_id(id, 0).unwrap();
        }

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = LookUpTable::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn repr_names_only() {
        // Create a vector to store the strings and declare it before repr for borrow checker
        const LUT_SIZE: usize = 32;
        let mut names = Vec::new();
        let mut repr = LookUpTable::default();

        for i in 0..LUT_SIZE {
            names.push(format!("Name{i}"));
        }

        for (i, name) in names.iter().enumerate().take(LUT_SIZE) {
            repr.append_name(name, i as u32).unwrap();
        }

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = LookUpTable::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn test_integrity_checker() {
        let mut lut = LookUpTable::default();
        assert!(lut.check_integrity()); // Empty, passes

        lut.append(1, "a", 0).unwrap();
        assert!(lut.check_integrity()); // ID only, passes

        lut.append_name("a", 0).unwrap();
        assert!(lut.check_integrity()); // Name added, passes

        lut.append(2, "b", 11).unwrap();
        assert!(lut.check_integrity()); // Second ID, name missing, fails

        lut.append_name("b", 11).unwrap();
        assert!(lut.check_integrity()); // Name added, passes
    }
}
