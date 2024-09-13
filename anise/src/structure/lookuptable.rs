/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{
    asn1::{OctetStringRef, SequenceOf},
    Decode, Encode, Reader, Writer,
};
use heapless::{FnvIndexMap, String};
use log::warn;
use snafu::prelude::*;

use crate::NaifId;

/// Maximum length of a look up table name string
pub const KEY_NAME_LEN: usize = 32;

#[derive(Debug, Snafu, PartialEq)]
#[snafu(visibility(pub(crate)))]
pub enum LutError {
    #[snafu(display(
        "ID LUT is full with all {max_slots} taken (increase ENTRIES at build time)"
    ))]
    IdLutFull { max_slots: usize },
    #[snafu(display(
        "Names LUT is full with all {max_slots} taken (increase ENTRIES at build time)"
    ))]
    NameLutFull { max_slots: usize },
    #[snafu(display("must provide either an ID or a name for a loop up, but provided neither"))]
    NoKeyProvided,
    #[snafu(display("ID {id} not in look up table"))]
    UnknownId { id: NaifId },
    #[snafu(display("name {name} not in look up table"))]
    UnknownName { name: String<KEY_NAME_LEN> },
    #[snafu(display("Look up table index is not in dataset"))]
    InvalidIndex { index: u32 },
}

/// A LookUpTable allows finding the [u32] ("NaifId") associated with either an ID or a name.
///
/// # Note
/// _Both_ the IDs and the name MUST be unique in the look up table.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct LookUpTable<const ENTRIES: usize> {
    /// Unique IDs of each item in the
    pub by_id: FnvIndexMap<NaifId, u32, ENTRIES>,
    /// Corresponding index for each hash
    pub by_name: FnvIndexMap<String<32>, u32, ENTRIES>,
}

impl<const ENTRIES: usize> LookUpTable<ENTRIES> {
    pub fn append(&mut self, id: i32, name: &str, index: u32) -> Result<(), LutError> {
        self.by_id
            .insert(id, index)
            .map_err(|_| LutError::IdLutFull { max_slots: ENTRIES })?;
        self.by_name
            .insert(name.try_into().unwrap(), index)
            .map_err(|_| LutError::NameLutFull { max_slots: ENTRIES })?;
        Ok(())
    }

    pub fn append_id(&mut self, id: i32, index: u32) -> Result<(), LutError> {
        self.by_id
            .insert(id, index)
            .map_err(|_| LutError::IdLutFull { max_slots: ENTRIES })?;
        Ok(())
    }

    pub fn append_name(&mut self, name: &str, index: u32) -> Result<(), LutError> {
        self.by_name
            .insert(name.try_into().unwrap(), index)
            .map_err(|_| LutError::NameLutFull { max_slots: ENTRIES })?;
        Ok(())
    }

    /// Returns the list of entries of this LUT
    pub fn entries(&self) -> FnvIndexMap<u32, (Option<NaifId>, Option<String<32>>), ENTRIES> {
        let mut rtn = FnvIndexMap::default();

        for (id, entry) in &self.by_id {
            // IDs are unique, and this is the first iteration, so we can't be overwriting anything
            rtn.insert(*entry, (Some(*id), None)).unwrap();
        }

        // Now map to the names
        for (name, entry) in &self.by_name {
            if !rtn.contains_key(entry) {
                rtn.insert(*entry, (None, Some(name.clone()))).unwrap();
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
            self.by_id.insert(new_id, entry).unwrap();
            Ok(())
        } else {
            Err(LutError::UnknownId { id: current_id })
        }
    }

    /// Removes this ID from the LUT if it's present.
    ///
    /// If this item was inserted with a name, it will rename accessible by the name.
    pub fn rmid(&mut self, id: i32) -> Result<(), LutError> {
        if self.by_id.remove(&id).is_none() {
            Err(LutError::UnknownId { id })
        } else {
            Ok(())
        }
    }

    /// Change the ID of a given entry to the new ID
    ///
    /// This will return an error if the current ID is not in the LUT, or if the new ID is already in the LUT.
    pub fn rename(&mut self, current_name: &str, new_name: &str) -> Result<(), LutError> {
        if let Some(entry) = self.by_name.swap_remove(&current_name.try_into().unwrap()) {
            // We can unwrap the insertion because we just removed something.
            self.by_name
                .insert(new_name.try_into().unwrap(), entry)
                .unwrap();
            Ok(())
        } else {
            Err(LutError::UnknownName {
                name: current_name.try_into().unwrap(),
            })
        }
    }

    /// Removes this ID from the LUT if it's present.
    ///
    /// If this item was inserted with a name, it will rename accessible by the name.
    pub fn rmname(&mut self, name: &str) -> Result<(), LutError> {
        if self.by_name.remove(&name.try_into().unwrap()).is_none() {
            Err(LutError::UnknownName {
                name: name.try_into().unwrap(),
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
    ///
    /// # Note
    /// The list of entries might be duplicated if all items have both a name and an ID.
    fn der_encoding(
        &self,
    ) -> (
        SequenceOf<i32, ENTRIES>,
        SequenceOf<u32, ENTRIES>,
        SequenceOf<OctetStringRef, ENTRIES>,
        SequenceOf<u32, ENTRIES>,
    ) {
        // Build the list of entries
        let mut id_entries = SequenceOf::<u32, ENTRIES>::new();
        let mut name_entries = SequenceOf::<u32, ENTRIES>::new();

        // Build the list of keys
        let mut ids = SequenceOf::<i32, ENTRIES>::new();
        for (id, index) in &self.by_id {
            ids.add(*id).unwrap();
            id_entries.add(*index).unwrap();
        }
        // Build the list of names
        let mut names = SequenceOf::<OctetStringRef, ENTRIES>::new();
        for (name, index) in &self.by_name {
            names
                .add(OctetStringRef::new(name.as_bytes()).unwrap())
                .unwrap();

            name_entries.add(*index).unwrap();
        }

        (ids, id_entries, names, name_entries)
    }
}

impl<const ENTRIES: usize> Encode for LookUpTable<ENTRIES> {
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

impl<'a, const ENTRIES: usize> Decode<'a> for LookUpTable<ENTRIES> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        // Decode as sequences and use that to build the look up table.
        let mut lut = Self::default();
        let ids: SequenceOf<i32, ENTRIES> = decoder.decode()?;
        let id_entries: SequenceOf<u32, ENTRIES> = decoder.decode()?;
        let names: SequenceOf<OctetStringRef, ENTRIES> = decoder.decode()?;
        let name_entries: SequenceOf<u32, ENTRIES> = decoder.decode()?;

        for (id, index) in ids.iter().zip(id_entries.iter()) {
            lut.by_id.insert(*id, *index).unwrap();
        }

        for (name, entry) in names.iter().zip(name_entries.iter()) {
            let key = core::str::from_utf8(name.as_bytes()).unwrap();
            lut.by_name
                .insert(
                    key[..KEY_NAME_LEN.min(key.len())].try_into().unwrap(),
                    *entry,
                )
                .unwrap();
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
        let repr = LookUpTable::<2>::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = LookUpTable::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        dbg!(repr);
        assert_eq!(core::mem::size_of::<LookUpTable<64>>(), 4368);
    }

    #[test]
    fn repr_ids_only() {
        let mut repr = LookUpTable::<32>::default();
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
        const LUT_SIZE: usize = 32;
        // Create a vector to store the strings and declare it before repr for borrow checker
        let mut names = Vec::new();
        let mut repr = LookUpTable::<LUT_SIZE>::default();

        for i in 0..LUT_SIZE {
            names.push(format!("Name{}", i));
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
        let mut lut = LookUpTable::<8>::default();
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
