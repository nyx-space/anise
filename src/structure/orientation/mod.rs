/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{asn1::Utf8StringRef, Decode, Encode, Reader, Writer};

use crate::NaifId;
pub const MAX_NUT_PREC_ANGLES: usize = 16;

use self::orient_data::OrientationData;

pub mod orient_data;
pub mod phaseangle;
pub mod trigangle;

#[derive(Clone, Debug, PartialEq)]
pub struct Orientation<'a> {
    pub name: &'a str,
    pub parent_orientation_hash: NaifId,
    pub orientation_data: OrientationData<'a>,
}

impl<'a> Encode for Orientation<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        Utf8StringRef::new(self.name)?.encoded_len()?
            + self.parent_orientation_hash.encoded_len()?
            + self.orientation_data.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        Utf8StringRef::new(self.name)?.encode(encoder)?;
        self.parent_orientation_hash.encode(encoder)?;
        self.orientation_data.encode(encoder)
    }
}

impl<'a> Decode<'a> for Orientation<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let name: Utf8StringRef = decoder.decode()?;

        Ok(Self {
            name: name.as_str(),
            parent_orientation_hash: decoder.decode()?,
            orientation_data: decoder.decode()?,
        })
    }
}
