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

/// Defines a spacecraft mass a the sum of the dry (structural) mass and the fuel mass, both in kilogram
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Mass {
    /// Structural mass of the spacecraft in kg
    pub dry_mass_kg: f64,
    /// Usable fuel mass of the spacecraft in kg
    pub usable_fuel_mass_kg: f64,
    /// Unusable fuel mass of the spacecraft in kg
    pub unusable_fuel_mass_kg: f64,
}

impl Mass {
    /// Creates a new spacecraft constant structure where all mass is considered usable.
    pub fn from_dry_and_fuel_masses(dry_mass_kg: f64, fuel_mass_kg: f64) -> Self {
        Self {
            dry_mass_kg,
            usable_fuel_mass_kg: fuel_mass_kg,
            unusable_fuel_mass_kg: 0.0,
        }
    }
    /// Returns the total mass in kg
    pub fn total_mass_kg(&self) -> f64 {
        self.dry_mass_kg + self.usable_fuel_mass_kg + self.unusable_fuel_mass_kg
    }
}

impl Encode for Mass {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.dry_mass_kg.encoded_len()?
            + self.usable_fuel_mass_kg.encoded_len()?
            + self.unusable_fuel_mass_kg.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
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

#[cfg(test)]
mod mass_ut {
    use super::{Decode, Encode, Mass};
    #[test]
    fn zero_repr() {
        let repr = Mass::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = Mass::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn example_repr() {
        let repr = Mass {
            dry_mass_kg: 50.0,
            usable_fuel_mass_kg: 15.7,
            unusable_fuel_mass_kg: 0.3,
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = Mass::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        assert_eq!(repr_dec.total_mass_kg(), 66.0);
    }
}
