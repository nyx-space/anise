/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use der::{asn1::OctetStringRef, Decode, Encode, Length, Reader, Writer};
use zerocopy::{AsBytes, FromBytes, LayoutVerified};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Array<'a, T: FromBytes + AsBytes> {
    // use AsBytes / FromBytes from "zerocopy" crate to load the data ?
    /// Stores the CRC32 checksum of the data octet string.
    pub data_checksum: u32, // TODO: move the checksum into a CRC32DataArray to check integrity on load
    /// The data as an array of type T
    pub data: &'a [T],
}

impl<'a, T: FromBytes + AsBytes> Encode for Array<'a, T> {
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

impl<'a, T: FromBytes + AsBytes> Decode<'a> for Array<'a, T> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let data_checksum = decoder.decode()?;
        let data_bytes: &[u8] = decoder.decode::<OctetStringRef>()?.as_bytes();

        // TODO: Confirm checksum is correct here.

        Ok(Self {
            data_checksum,
            data: LayoutVerified::new_slice(data_bytes).unwrap().into_slice(),
        })
    }
}
