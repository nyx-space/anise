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

use self::planetary_constant::PlanetaryConstant;
use crate::NaifId;
pub mod phaseangle;
pub mod planetary_constant;
pub mod trigangle;

#[derive(Clone, Debug, PartialEq)]
pub struct PlanetaryData<'a> {
    pub name: &'a str,
    pub parent_orientation_hash: NaifId,
    pub constants: PlanetaryConstant<'a>,
}

impl<'a> Encode for PlanetaryData<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        Utf8StringRef::new(self.name)?.encoded_len()?
            + self.parent_orientation_hash.encoded_len()?
            + self.constants.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        Utf8StringRef::new(self.name)?.encode(encoder)?;
        self.parent_orientation_hash.encode(encoder)?;
        self.constants.encode(encoder)
    }
}

impl<'a> Decode<'a> for PlanetaryData<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let name: Utf8StringRef = decoder.decode()?;

        Ok(Self {
            name: name.as_str(),
            parent_orientation_hash: decoder.decode()?,
            constants: decoder.decode()?,
        })
    }
}
