/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use crate::{errors::IntegrityErrorKind, prelude::AniseError};
use der::{asn1::OctetStringRef, Decode, Encode, Reader, Writer};

use super::{lookuptable::LookUpTable, metadata::Metadata};

/// A DataSet is the core structure shared by all ANISE binary data.
#[derive(Clone, Default)]
pub struct DataSet<'a> {
    pub metadata: Metadata<'a>,
    /// All datasets have LookUpTable (LUT) that stores the mapping between a key and its index in the ephemeris list.
    pub lut: LookUpTable<'a>,
    pub crc32_checksum: u32,
    /// The actual data from the dataset
    pub bytes: &'a [u8],
}

impl<'a> DataSet<'a> {
    /// Compute the CRC32 of the underlying bytes
    pub fn crc32(&self) -> u32 {
        crc32fast::hash(&self.bytes)
    }

    /// Scrubs the data by computing the CRC32 of the bytes and making sure that it still matches the previously known hash
    pub fn scrub(&self) -> Result<(), AniseError> {
        if self.crc32() == self.crc32_checksum {
            Ok(())
        } else {
            // Compiler will optimize the double computation away
            Err(AniseError::IntegrityError(
                IntegrityErrorKind::ChecksumInvalid {
                    expected: self.crc32_checksum,
                    computed: self.crc32(),
                },
            ))
        }
    }
}

impl<'a> Encode for DataSet<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let as_byte_ref = OctetStringRef::new(&self.bytes)?;
        self.metadata.encoded_len()?
            + self.lut.encoded_len()?
            + self.crc32_checksum.encoded_len()?
            + as_byte_ref.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let as_byte_ref = OctetStringRef::new(&self.bytes)?;
        self.metadata.encode(encoder)?;
        self.lut.encode(encoder)?;
        self.crc32_checksum.encode(encoder)?;
        as_byte_ref.encode(encoder)
    }
}

impl<'a> Decode<'a> for DataSet<'a> {
    fn decode<D: Reader<'a>>(decoder: &mut D) -> der::Result<Self> {
        let metadata = decoder.decode()?;
        let lut = decoder.decode()?;
        let crc32_checksum = decoder.decode()?;
        let bytes: OctetStringRef = decoder.decode()?;
        Ok(Self {
            metadata,
            lut,
            crc32_checksum,
            bytes: bytes.as_bytes(),
        })
    }
}
