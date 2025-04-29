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

impl TryFrom<u8> for DataSetType {
    type Error = &'static str;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(DataSetType::NotApplicable),
            1 => Ok(DataSetType::SpacecraftData),
            2 => Ok(DataSetType::PlanetaryData),
            3 => Ok(DataSetType::EulerParameterData),
            _ => Err("Invalid value for DataSetType"),
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
        DataSetType::try_from(asu8).map_err(|_| {
            der::Error::new(
                der::ErrorKind::Value {
                    tag: der::Tag::Integer,
                },
                der::Length::ONE,
            )
        })
    }
}
