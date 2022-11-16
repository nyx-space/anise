/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{asn1::SequenceOf, Decode, Encode, Reader, Writer};

use super::{lookuptable::LookUpTable, metadata::Metadata, records::Record};

pub const MAX_RECORDS: usize = 32;

/// A DataSet is the core structure shared by all ANISE binary data.
#[derive(Clone, Default)]
pub struct DataSet<'a, R: Record<'a>> {
    pub metadata: Metadata<'a>,
    /// All datasets have LookUpTable (LUT) that stores the mapping between a key and its index in the ephemeris list.
    pub lut: LookUpTable<'a>,
    /// All datasets have up Records that store the actual data.
    pub data: SequenceOf<R, MAX_RECORDS>,
}

impl<'a, R: Record<'a>> Encode for DataSet<'a, R> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.metadata.encoded_len()? + self.lut.encoded_len()? + self.data.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.metadata.encode(encoder)?;
        self.lut.encode(encoder)?;
        self.data.encode(encoder)
    }
}

impl<'a, R: Record<'a>> Decode<'a> for DataSet<'a, R> {
    fn decode<D: Reader<'a>>(decoder: &mut D) -> der::Result<Self> {
        Ok(Self {
            metadata: decoder.decode()?,
            lut: decoder.decode()?,
            data: decoder.decode()?,
        })
    }
}
