/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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

use crate::{errors::DecodingError, NaifId};

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
}

/// A lookup table entry contains the start and end indexes in the data array of the data that is sought after.
///
/// # Implementation note
/// This data is stored as a u32 to ensure that the same binary representation works on all platforms.
/// In fact, the size of the usize type varies based on whether this is a 32 or 64 bit platform.
#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    pub start_idx: u32,
    pub end_idx: u32,
}

impl Entry {
    pub(crate) fn as_range(&self) -> core::ops::Range<usize> {
        self.start_idx as usize..self.end_idx as usize
    }
    /// Returns a pre-populated decoding error
    pub(crate) fn decoding_error(&self) -> DecodingError {
        DecodingError::InaccessibleBytes {
            start: self.start_idx as usize,
            end: self.end_idx as usize,
            size: (self.end_idx - self.start_idx) as usize,
        }
    }
}

impl Encode for Entry {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.start_idx.encoded_len()? + self.end_idx.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.start_idx.encode(encoder)?;
        self.end_idx.encode(encoder)
    }
}

impl<'a> Decode<'a> for Entry {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            start_idx: decoder.decode()?,
            end_idx: decoder.decode()?,
        })
    }
}

/// A LookUpTable allows finding the [Entry] associated with either an ID or a name.
///
/// # Note
/// _Both_ the IDs and the name MUST be unique in the look up table.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct LookUpTable<const ENTRIES: usize> {
    /// Unique IDs of each item in the
    pub by_id: FnvIndexMap<NaifId, Entry, ENTRIES>,
    /// Corresponding index for each hash
    pub by_name: FnvIndexMap<String<32>, Entry, ENTRIES>,
}

impl<const ENTRIES: usize> LookUpTable<ENTRIES> {
    pub fn append(&mut self, id: i32, name: &str, entry: Entry) -> Result<(), LutError> {
        self.by_id
            .insert(id, entry)
            .map_err(|_| LutError::IdLutFull { max_slots: ENTRIES })?;
        self.by_name
            .insert(name.try_into().unwrap(), entry)
            .map_err(|_| LutError::NameLutFull { max_slots: ENTRIES })?;
        Ok(())
    }

    pub fn append_id(&mut self, id: i32, entry: Entry) -> Result<(), LutError> {
        self.by_id
            .insert(id, entry)
            .map_err(|_| LutError::IdLutFull { max_slots: ENTRIES })?;
        Ok(())
    }

    pub fn append_name(&mut self, name: &str, entry: Entry) -> Result<(), LutError> {
        self.by_name
            .insert(name.try_into().unwrap(), entry)
            .map_err(|_| LutError::NameLutFull { max_slots: ENTRIES })?;
        Ok(())
    }

    /// Returns the list of entries of this LUT
    pub fn entries(&self) -> FnvIndexMap<Entry, (Option<NaifId>, Option<String<32>>), ENTRIES> {
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
        SequenceOf<Entry, ENTRIES>,
        SequenceOf<OctetStringRef, ENTRIES>,
        SequenceOf<Entry, ENTRIES>,
    ) {
        // Build the list of entries
        let mut id_entries = SequenceOf::<Entry, ENTRIES>::new();
        let mut name_entries = SequenceOf::<Entry, ENTRIES>::new();

        // Build the list of keys
        let mut ids = SequenceOf::<i32, ENTRIES>::new();
        for (id, entry) in &self.by_id {
            ids.add(*id).unwrap();
            id_entries.add(*entry).unwrap();
        }
        // Build the list of names
        let mut names = SequenceOf::<OctetStringRef, ENTRIES>::new();
        for (name, entry) in &self.by_name {
            names
                .add(OctetStringRef::new(name.as_bytes()).unwrap())
                .unwrap();

            name_entries.add(*entry).unwrap();
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
        let id_entries: SequenceOf<Entry, ENTRIES> = decoder.decode()?;
        let names: SequenceOf<OctetStringRef, ENTRIES> = decoder.decode()?;
        let name_entries: SequenceOf<Entry, ENTRIES> = decoder.decode()?;

        for (id, entry) in ids.iter().zip(id_entries.iter()) {
            lut.by_id.insert(*id, *entry).unwrap();
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
    use super::{Decode, Encode, Entry, LookUpTable};
    #[test]
    fn zero_repr() {
        let repr = LookUpTable::<2>::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = LookUpTable::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        dbg!(repr);
        assert_eq!(core::mem::size_of::<LookUpTable<64>>(), 5136);
    }

    #[test]
    fn repr_ids_only() {
        let mut repr = LookUpTable::<32>::default();
        let num_bytes = 363;
        for i in 0..32 {
            let id = -20 - i;
            repr.append_id(
                id,
                Entry {
                    start_idx: (i * num_bytes) as u32,
                    end_idx: ((i + 1) * num_bytes) as u32,
                },
            )
            .unwrap();
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

        let num_bytes = 363;

        for i in 0..LUT_SIZE {
            names.push(format!("Name{}", i));
        }

        for (i, name) in names.iter().enumerate().take(LUT_SIZE) {
            repr.append_name(
                name,
                Entry {
                    start_idx: (i * num_bytes) as u32,
                    end_idx: ((i + 1) * num_bytes) as u32,
                },
            )
            .unwrap();
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

        lut.append(1, "a", Entry::default()).unwrap();
        assert!(lut.check_integrity()); // ID only, passes

        lut.append_name("a", Entry::default()).unwrap();
        assert!(lut.check_integrity()); // Name added, passes

        lut.append(2, "b", Entry::default()).unwrap();
        assert!(lut.check_integrity()); // Second ID, name missing, fails

        lut.append_name("b", Entry::default()).unwrap();
        assert!(lut.check_integrity()); // Name added, passes
    }
}
