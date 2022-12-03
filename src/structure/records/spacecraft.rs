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

use crate::NaifId;

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
    pub inertia_tensor: Option<InertiaTensor>,
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

/// Defines a spacecraft mass a the sum of the dry (structural) mass and the fuel mass, both in kilogram
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mass {
    /// Structural mass of the spacecraft in kg
    pub dry_mass_kg: f64,
    /// Total fuel mass of the spacecraft in kg
    pub fuel_mass_kg: f64,
}

impl Encode for Mass {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.dry_mass_kg.encoded_len()? + self.fuel_mass_kg.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.dry_mass_kg.encode(encoder)?;
        self.fuel_mass_kg.encode(encoder)
    }
}

impl<'a> Decode<'a> for Mass {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            dry_mass_kg: decoder.decode()?,
            fuel_mass_kg: decoder.decode()?,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SRPData {
    /// Solar radiation pressure area in m^2
    pub area_m2: f64,
    /// Solar radiation pressure coefficient of reflectivity (C_r)
    pub coeff_reflectivity: f64,
}

impl<'a> Encode for SRPData {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.area_m2.encoded_len()? + self.coeff_reflectivity.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.area_m2.encode(encoder)?;
        self.coeff_reflectivity.encode(encoder)
    }
}

impl<'a> Decode<'a> for SRPData {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            area_m2: decoder.decode()?,
            coeff_reflectivity: decoder.decode()?,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DragData {
    /// Atmospheric drag area in m^2
    pub area_m2: Option<f64>,
    /// Drag coefficient (C_d)
    pub coeff_drag: Option<f64>,
}

impl<'a> Encode for DragData {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.area_m2.encoded_len()? + self.coeff_drag.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.area_m2.encode(encoder)?;
        self.coeff_drag.encode(encoder)
    }
}

impl<'a> Decode<'a> for DragData {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            area_m2: decoder.decode()?,
            coeff_drag: decoder.decode()?,
        })
    }
}

/// Inertial tensor definition
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InertiaTensor {
    /// Inertia tensor reference frame hash
    pub orientation_hash: NaifId,
    /// Moment of inertia about the 1-axis
    pub i_11_kgm2: f64,
    /// Moment of inertia about the 2-axis
    pub i_22_kgm2: f64,
    /// Moment of inertia about the 3-axis
    pub i_33_kgm2: f64,
    /// Inertia cross product of the 1 and 2 axes
    pub i_12_kgm2: f64,
    /// Inertia cross product of the 1 and 2 axes
    pub i_13_kgm2: f64,
    /// Inertia cross product of the 2 and 3 axes
    pub i_23_kgm2: f64,
}

impl<'a> Encode for InertiaTensor {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.orientation_hash.encoded_len()?
            + self.i_11_kgm2.encoded_len()?
            + self.i_22_kgm2.encoded_len()?
            + self.i_33_kgm2.encoded_len()?
            + self.i_12_kgm2.encoded_len()?
            + self.i_13_kgm2.encoded_len()?
            + self.i_23_kgm2.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.orientation_hash.encode(encoder)?;
        self.i_11_kgm2.encode(encoder)?;
        self.i_22_kgm2.encode(encoder)?;
        self.i_33_kgm2.encode(encoder)?;
        self.i_12_kgm2.encode(encoder)?;
        self.i_13_kgm2.encode(encoder)?;
        self.i_23_kgm2.encode(encoder)
    }
}

impl<'a> Decode<'a> for InertiaTensor {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            orientation_hash: decoder.decode()?,
            i_11_kgm2: decoder.decode()?,
            i_22_kgm2: decoder.decode()?,
            i_33_kgm2: decoder.decode()?,
            i_12_kgm2: decoder.decode()?,
            i_13_kgm2: decoder.decode()?,
            i_23_kgm2: decoder.decode()?,
        })
    }
}
