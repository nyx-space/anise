/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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
use heapless::FnvIndexMap;
use log::warn;

use crate::{prelude::AniseError, NaifId};

pub const MAX_LUT_ENTRIES: usize = 32;

/// A lookup table entry contains the start and end indexes in the data array of the data that is sought after.
///
/// # Implementation note
/// This data is stored as a u32 to ensure that the same binary representation works on all platforms.
/// In fact, the size of the usize type varies based on whether this is a 32 or 64 bit platform.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    pub start_idx: u32,
    pub end_idx: u32,
}

impl Encode for Entry {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.start_idx.encoded_len()? + self.end_idx.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
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
pub struct LookUpTable<'a> {
    /// Unique IDs of each item in the
    pub by_id: FnvIndexMap<NaifId, Entry, MAX_LUT_ENTRIES>,
    /// Corresponding index for each hash
    pub by_name: FnvIndexMap<&'a str, Entry, MAX_LUT_ENTRIES>,
}

impl<'a> LookUpTable<'a> {
    pub fn append(&mut self, id: i32, name: &'a str, entry: Entry) -> Result<(), AniseError> {
        self.by_id
            .insert(id, entry)
            .map_err(|_| AniseError::StructureIsFull)?;
        self.by_name
            .insert(name, entry)
            .map_err(|_| AniseError::StructureIsFull)?;
        Ok(())
    }

    pub fn append_id(&mut self, id: i32, entry: Entry) -> Result<(), AniseError> {
        self.by_id
            .insert(id, entry)
            .map_err(|_| AniseError::StructureIsFull)?;
        Ok(())
    }

    pub fn append_name(&mut self, name: &'a str, entry: Entry) -> Result<(), AniseError> {
        self.by_name
            .insert(name, entry)
            .map_err(|_| AniseError::StructureIsFull)?;
        Ok(())
    }

    /// Builds the DER encoding of this look up table
    fn der_encoding(
        &self,
    ) -> (
        SequenceOf<i32, MAX_LUT_ENTRIES>,
        SequenceOf<OctetStringRef, MAX_LUT_ENTRIES>,
        SequenceOf<Entry, MAX_LUT_ENTRIES>,
    ) {
        // Decide whether to encode the entries from the ID iterator or the names iterator based on which has the most.
        let use_id = self.by_id.len() >= self.by_name.len();
        // Build the list of entries
        let mut entries = SequenceOf::<Entry, MAX_LUT_ENTRIES>::new();
        // Build the list of keys
        let mut ids = SequenceOf::<i32, MAX_LUT_ENTRIES>::new();
        for (id, entry) in &self.by_id {
            ids.add(*id).unwrap();
            if use_id {
                entries.add(*entry).unwrap();
            }
        }
        // Build the list of names
        let mut names = SequenceOf::<OctetStringRef, MAX_LUT_ENTRIES>::new();
        for (name, entry) in &self.by_name {
            names
                .add(OctetStringRef::new(name.as_bytes()).unwrap())
                .unwrap();
            if !use_id {
                entries.add(*entry).unwrap();
            }
        }

        (ids, names, entries)
    }
}

impl<'a> Encode for LookUpTable<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let (ids, names, entries) = self.der_encoding();
        ids.encoded_len()? + names.encoded_len()? + entries.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let (ids, names, entries) = self.der_encoding();
        ids.encode(encoder)?;
        names.encode(encoder)?;
        entries.encode(encoder)
    }
}

impl<'a> Decode<'a> for LookUpTable<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        // Decode as sequences and use that to build the look up table.
        let mut lut = Self::default();
        let ids: SequenceOf<i32, MAX_LUT_ENTRIES> = decoder.decode()?;
        let names: SequenceOf<OctetStringRef, MAX_LUT_ENTRIES> = decoder.decode()?;
        let entries: SequenceOf<Entry, MAX_LUT_ENTRIES> = decoder.decode()?;
        for (id, entry) in ids.iter().zip(entries.iter()) {
            lut.by_id.insert(*id, *entry).unwrap();
        }
        for (name, entry) in names.iter().zip(entries.iter()) {
            lut.by_name
                .insert(core::str::from_utf8(name.as_bytes()).unwrap(), *entry)
                .unwrap();
        }
        if lut.by_name.len() != lut.by_id.len() && !lut.by_id.is_empty() && !lut.by_name.is_empty()
        {
            warn!(
                "decoded lookup table inconsistent: {} names but {} ids",
                lut.by_name.len(),
                lut.by_id.len()
            );
        }
        Ok(lut)
    }
}

#[cfg(test)]
mod lut_ut {
    use super::{Decode, Encode, Entry, LookUpTable, MAX_LUT_ENTRIES};
    #[test]
    fn zero_repr() {
        let repr = LookUpTable::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = LookUpTable::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        dbg!(repr);
        dbg!(core::mem::size_of::<LookUpTable>());
    }

    #[test]
    fn repr_ids_only() {
        let mut repr = LookUpTable::default();
        let num_bytes = 363;
        for i in 0..(MAX_LUT_ENTRIES as u32) {
            let id = -20 - (i as i32);
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
}
