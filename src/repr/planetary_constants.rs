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

/// Planetary constants can store the same data as the SPICE textual PCK files
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlanetaryConstants<'a> {
    /// Name is used as the input for the hashing function.
    pub name: &'a str,
    /// Generic comments field
    pub comments: &'a str,
    /// Structural mass of the spacecraft in kg
    pub dry_mass_kg: Option<f64>,
    /// Total fuel mass of the spacecraft in kg
    pub fuel_mass_kg: Option<f64>,
    /// Solar radiation pressure area in m^2
    pub srp_area_m2: Option<f64>,
    /// Solar radiation pressure coefficient of reflectivity (C_r)
    pub srp_coeff_reflectivity: Option<f64>,
    /// Atmospheric drag area in m^2
    pub drag_area_m2: Option<f64>,
    /// Drag coefficient (C_d)
    pub drag_coeff: Option<f64>,
}

impl<'a> Encode for PlanetaryConstants<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        Utf8StringRef::new(self.name)?.encoded_len()?
            + Utf8StringRef::new(self.comments)?.encoded_len()?
            + self.dry_mass_kg.encoded_len()?
            + self.fuel_mass_kg.encoded_len()?
            + self.srp_area_m2.encoded_len()?
            + self.srp_coeff_reflectivity.encoded_len()?
            + self.drag_area_m2.encoded_len()?
            + self.drag_coeff.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        Utf8StringRef::new(self.name)?.encode(encoder)?;
        Utf8StringRef::new(self.comments)?.encode(encoder)?;
        self.dry_mass_kg.encode(encoder)?;
        self.fuel_mass_kg.encode(encoder)?;
        self.srp_area_m2.encode(encoder)?;
        self.srp_coeff_reflectivity.encode(encoder)?;
        self.drag_area_m2.encode(encoder)?;
        self.drag_coeff.encode(encoder)
    }
}

impl<'a> Decode<'a> for PlanetaryConstants<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let name: Utf8StringRef = decoder.decode()?;
        let comments: Utf8StringRef = decoder.decode()?;

        Ok(Self {
            name: name.as_str(),
            comments: comments.as_str(),
            dry_mass_kg: decoder.decode()?,
            fuel_mass_kg: decoder.decode()?,
            srp_area_m2: decoder.decode()?,
            srp_coeff_reflectivity: decoder.decode()?,
            drag_area_m2: decoder.decode()?,
            drag_coeff: decoder.decode()?,
        })
    }
}
