/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use der::{Decode, Encode, Reader, Writer};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum DataSetType {
    /// Used only if not encoding a dataset but some other structure
    NotApplicable,
    SpacecraftData,
    PlanetaryData,
    EulerParameterData,
}

impl From<u8> for DataSetType {
    fn from(val: u8) -> Self {
        match val {
            0 => DataSetType::NotApplicable,
            1 => DataSetType::SpacecraftData,
            2 => DataSetType::PlanetaryData,
            3 => DataSetType::EulerParameterData,
            _ => panic!("Invalid value for DataSetType {val}"),
        }
    }
}

impl From<DataSetType> for u8 {
    fn from(val: DataSetType) -> Self {
        val as u8
    }
}

impl Encode for DataSetType {
    fn encoded_len(&self) -> der::Result<der::Length> {
        (*self as u8).encoded_len()
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        (*self as u8).encode(encoder)
    }
}

impl<'a> Decode<'a> for DataSetType {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let asu8: u8 = decoder.decode()?;
        Ok(Self::from(asu8))
    }
}
