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
use serde_derive::{Deserialize, Serialize};
use std::ops::Sub;

/// Defines a spacecraft mass a the sum of the dry (structural) mass and the propellant mass, both in kilogram
#[derive(Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mass {
    /// Structural mass of the spacecraft, in kg
    pub dry_mass_kg: f64,
    /// Propellant mass of the spacecraft, in kg
    pub prop_mass_kg: f64,
    /// Extra mass like unusable propellant mass of the spacecraft, in kg
    pub extra_mass_kg: f64,
}

impl Mass {
    /// Creates a new spacecraft data structure where all mass is considered usable.
    pub fn from_dry_and_prop_masses(dry_mass_kg: f64, prop_mass_kg: f64) -> Self {
        Self {
            dry_mass_kg,
            prop_mass_kg,
            extra_mass_kg: 0.0,
        }
    }
    /// Creates a new spacecraft data structure from its dry mass (prop and extra set to zero).
    pub fn from_dry_mass(dry_mass_kg: f64) -> Self {
        Self {
            dry_mass_kg,
            prop_mass_kg: 0.0,
            extra_mass_kg: 0.0,
        }
    }
    /// Returns the total mass in kg
    pub fn total_mass_kg(&self) -> f64 {
        self.dry_mass_kg + self.prop_mass_kg + self.extra_mass_kg
    }

    /// Returns true if all the masses are greater or equal to zero
    pub fn is_valid(&self) -> bool {
        self.dry_mass_kg >= 0.0 && self.prop_mass_kg >= 0.0 && self.extra_mass_kg >= 0.0
    }

    /// Returns a Mass structure that is guaranteed to be physically correct
    pub fn abs(self) -> Self {
        if self.is_valid() {
            self
        } else {
            Self {
                dry_mass_kg: self.dry_mass_kg.abs(),
                prop_mass_kg: self.prop_mass_kg.abs(),
                extra_mass_kg: self.extra_mass_kg.abs(),
            }
        }
    }
}

impl Sub for Mass {
    type Output = Mass;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            dry_mass_kg: self.dry_mass_kg - rhs.dry_mass_kg,
            prop_mass_kg: self.prop_mass_kg - rhs.prop_mass_kg,
            extra_mass_kg: self.extra_mass_kg - rhs.extra_mass_kg,
        }
    }
}

impl Encode for Mass {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.dry_mass_kg.encoded_len()?
            + self.prop_mass_kg.encoded_len()?
            + self.extra_mass_kg.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.dry_mass_kg.encode(encoder)?;
        self.prop_mass_kg.encode(encoder)?;
        self.extra_mass_kg.encode(encoder)
    }
}

impl<'a> Decode<'a> for Mass {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            dry_mass_kg: decoder.decode()?,
            prop_mass_kg: decoder.decode()?,
            extra_mass_kg: decoder.decode()?,
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
            prop_mass_kg: 15.7,
            extra_mass_kg: 0.3,
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = Mass::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        assert_eq!(repr_dec.total_mass_kg(), 66.0);
    }
}
