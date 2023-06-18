/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crc32fast::hash;
use der::{asn1::OctetStringRef, Decode, Encode, Length, Reader, Writer};
use log::error;
use zerocopy::{AsBytes, FromBytes, LayoutVerified};

use crate::{errors::IntegrityErrorKind, prelude::AniseError};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct DataArray<'a, T: Default + FromBytes + AsBytes> {
    /// Stores the CRC32 checksum of the data octet string.
    pub data_checksum: u32, // TODO: move the checksum into a CRC32DataArray to check integrity on load
    /// The data as an array of type T
    pub data: &'a [T],
}

impl<'a, T: Default + FromBytes + AsBytes> DataArray<'a, T> {
    /// Builds a new data array and sets its checksum.
    pub fn new(data: &'a [T]) -> Self {
        let mut me = Self {
            data_checksum: 0,
            data,
        };
        me.update_hash();
        me
    }

    /// Updates the hash of the bytes representation of this data.
    pub fn update_hash(&mut self) {
        self.data_checksum = hash(self.data.as_bytes());
    }

    // pub fn add(&mut self, item: T) {
    //     // TODO: Put behind an `alloc` crate feature
    //     let mut data: Vec<T> = Vec::from_iter(self.data.iter().cloned());
    //     data.push(item);
    //     self.set_data(&mut data);
    //     // info!("Now with {} items", self.da)
    //     self.update_hash();
    // }

    pub fn set_data(&mut self, backend: &'a [T]) {
        self.data = backend;
        self.update_hash();
    }

    pub const fn len(&self) -> usize {
        self.data.len()
    }

    pub fn check_integrity(&self) -> Result<(), AniseError> {
        // Ensure that the data is correctly decoded
        let computed_chksum = hash(self.data.as_bytes());
        if computed_chksum == self.data_checksum {
            Ok(())
        } else {
            error!(
                "[integrity] expected hash {} but computed {}",
                self.data_checksum, computed_chksum
            );
            Err(AniseError::IntegrityError(
                IntegrityErrorKind::ChecksumInvalid {
                    expected: self.data_checksum,
                    computed: computed_chksum,
                },
            ))
        }
    }
}

impl<'a, T: Default + FromBytes + AsBytes> Encode for DataArray<'a, T> {
    fn encoded_len(&self) -> der::Result<Length> {
        self.data_checksum.encoded_len()?
            + OctetStringRef::new(self.data.as_bytes())
                .unwrap()
                .encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.data_checksum.encode(encoder)?;
        OctetStringRef::new(self.data.as_bytes())
            .unwrap()
            .encode(encoder)
    }
}

impl<'a, T: Default + FromBytes + AsBytes> Decode<'a> for DataArray<'a, T> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let data_checksum = decoder.decode()?;
        let data_bytes: &[u8] = decoder.decode::<OctetStringRef>()?.as_bytes();

        // TODO: Try to find a way to compute the checksum here, but I don't know how to return a checksum error instead of a der::Result (which are quite limited)

        let me = Self {
            data_checksum,
            data: match LayoutVerified::new_slice(data_bytes) {
                Some(data) => data.into_slice(),
                None => &[T::default(); 0],
            },
        };
        Ok(me)
    }
}
