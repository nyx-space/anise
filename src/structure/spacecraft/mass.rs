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

/// Defines a spacecraft mass a the sum of the dry (structural) mass and the fuel mass, both in kilogram
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mass {
    /// Structural mass of the spacecraft in kg
    pub dry_mass_kg: f64,
    /// Usable fuel mass of the spacecraft in kg
    pub usable_fuel_mass_kg: f64,
    /// Unusable fuel mass of the spacecraft in kg
    pub unusable_fuel_mass_kg: f64,
}

impl Encode for Mass {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.dry_mass_kg.encoded_len()?
            + self.usable_fuel_mass_kg.encoded_len()?
            + self.unusable_fuel_mass_kg.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.dry_mass_kg.encode(encoder)?;
        self.usable_fuel_mass_kg.encode(encoder)?;
        self.unusable_fuel_mass_kg.encode(encoder)
    }
}

impl<'a> Decode<'a> for Mass {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            dry_mass_kg: decoder.decode()?,
            usable_fuel_mass_kg: decoder.decode()?,
            unusable_fuel_mass_kg: decoder.decode()?,
        })
    }
}
