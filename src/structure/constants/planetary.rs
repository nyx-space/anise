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

use super::ellipsoid::Ellipsoid;

/// Planetary constants can store the same data as the SPICE textual PCK files
#[derive(Clone, Debug, PartialEq)]
pub struct PlanetaryConstants<'a> {
    /// Name is used as the input for the hashing function.
    pub name: &'a str,
    /// Generic comments field
    pub comments: &'a str,
    /// Gravitational parameter (μ) of this planetary object.
    pub mu_km3_s2: f64,
    /// The shape is always a tri axial ellipsoid
    pub shape: Option<Ellipsoid>,
}

impl<'a> Encode for PlanetaryConstants<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        Utf8StringRef::new(self.name)?.encoded_len()?
            + Utf8StringRef::new(self.comments)?.encoded_len()?
            + self.mu_km3_s2.encoded_len()?
            + self.shape.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        Utf8StringRef::new(self.name)?.encode(encoder)?;
        Utf8StringRef::new(self.comments)?.encode(encoder)?;
        self.mu_km3_s2.encode(encoder)?;
        self.shape.encode(encoder)
    }
}

impl<'a> Decode<'a> for PlanetaryConstants<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let name: Utf8StringRef = decoder.decode()?;
        let comments: Utf8StringRef = decoder.decode()?;

        Ok(Self {
            name: name.as_str(),
            comments: comments.as_str(),
            mu_km3_s2: decoder.decode()?,
            shape: Some(decoder.decode()?),
        })
    }
}
