/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{Decode, Encode, Reader, Writer};

/// Re-export hifitime's units as TimeUnit.
pub use hifitime::Unit as TimeUnit;

/// Defines the distance units supported by ANISE. This notably allows storing interpolation information from instruments to comets.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum DistanceUnit {
    Micrometer,
    Millimeter,
    Meter,
    Kilometer,
    Megameter,
}

impl Default for DistanceUnit {
    fn default() -> Self {
        Self::Kilometer
    }
}

/// Allows conversion of a Distance into a u8 with the following mapping.
/// Mapping: Micrometer: 0; Millimeter: 1; Meter: 2; Kilometer: 3; Megameter: 4.
impl From<DistanceUnit> for u8 {
    fn from(dist: DistanceUnit) -> Self {
        match dist {
            DistanceUnit::Micrometer => 0,
            DistanceUnit::Millimeter => 1,
            DistanceUnit::Meter => 2,
            DistanceUnit::Kilometer => 3,
            DistanceUnit::Megameter => 4,
        }
    }
}

/// Allows conversion of a Distance into a u8 with the following mapping.
/// Mapping: Micrometer: 0; Millimeter: 1; Meter: 2; Kilometer: 3; Megameter: 4.
impl From<&DistanceUnit> for u8 {
    fn from(dist: &DistanceUnit) -> Self {
        u8::from(*dist)
    }
}

/// Allows conversion of a u8 into a Distance.
/// Mapping: 0: Micrometer; 1: Millimeter; 2: Meter; 4: Megameter; 3 or any other value is considered kilometer
impl From<u8> for DistanceUnit {
    fn from(val: u8) -> Self {
        match val {
            0 => DistanceUnit::Micrometer,
            1 => DistanceUnit::Millimeter,
            2 => DistanceUnit::Meter,
            4 => DistanceUnit::Megameter,
            _ => DistanceUnit::Kilometer,
        }
    }
}

impl Encode for DistanceUnit {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let converted: u8 = self.into();
        converted.encoded_len()
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let converted: u8 = self.into();
        converted.encode(encoder)
    }
}

impl<'a> Decode<'a> for DistanceUnit {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let converted: u8 = decoder.decode()?;
        Ok(Self::from(converted))
    }
}
