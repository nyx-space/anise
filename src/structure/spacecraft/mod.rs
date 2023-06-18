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

mod drag;
mod inertia;
mod mass;
mod srp;

pub use drag::DragData;
pub use inertia::Inertia;
pub use mass::Mass;
pub use srp::SRPData;

/// Spacecraft constants can store the same spacecraft constant data as the CCSDS Orbit Parameter Message (OPM) and CCSDS Attitude Parameter Messages (APM)
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct SpacecraftConstants<'a> {
    /// Name is used as the input for the hashing function
    pub name: &'a str,
    /// Generic comments field
    pub comments: &'a str,
    /// Mass of the spacecraft in kg
    pub mass_kg: Option<Mass>,
    /// Solar radiation pressure data
    pub srp_data: Option<SRPData>,
    /// Atmospheric drag data
    pub drag_data: Option<DragData>,
    // Inertia tensor
    pub inertia_tensor: Option<Inertia>,
}

impl<'a> Encode for SpacecraftConstants<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        Utf8StringRef::new(self.name)?.encoded_len()?
            + Utf8StringRef::new(self.comments)?.encoded_len()?
            + self.mass_kg.encoded_len()?
            + self.srp_data.encoded_len()?
            + self.drag_data.encoded_len()?
            + self.inertia_tensor.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        Utf8StringRef::new(self.name)?.encode(encoder)?;
        Utf8StringRef::new(self.comments)?.encode(encoder)?;
        self.mass_kg.encode(encoder)?;
        self.srp_data.encode(encoder)?;
        self.drag_data.encode(encoder)?;
        self.inertia_tensor.encode(encoder)
    }
}

impl<'a> Decode<'a> for SpacecraftConstants<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let name: Utf8StringRef = decoder.decode()?;
        let comments: Utf8StringRef = decoder.decode()?;

        Ok(Self {
            name: name.as_str(),
            comments: comments.as_str(),
            mass_kg: Some(decoder.decode()?),
            srp_data: Some(decoder.decode()?),
            drag_data: Some(decoder.decode()?),
            inertia_tensor: Some(decoder.decode()?),
        })
    }
}
