/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{asn1::SequenceOf, Decode, Encode, Reader, Writer};

use super::{
    constants::{PlanetaryConstants, SpacecraftConstants},
    ephemeris::Ephemeris,
    lookuptable::LookUpTable,
    metadata::Metadata,
    orientation::Orientation,
    MAX_TRAJECTORIES,
};

/// A Context is the core structure which stores all of the ANISE data.
/// All of the data stored in the context can be written to disk and read in the exact same way regardless of the endianness
/// of the platform in which the data is written or read. This is guaranteed thanks to the use of the ISO certified ASN1 Distinguished
/// Encoding Rules (DER).
///
/// # Design considerations
/// ## Requirements
/// Here are the requirements this context fulfils:
/// 1. Memory-mappable data, i.e. does not require any memory allocation for loading a file.
/// 2. Endianness agnostic: data shall be read in the same way regardless of the endianness of the platform on which it was serialized or on which it is read
/// 3. Small size (ANISE are about 5.5% _smaller_ than their equivalent SPICE BSP files)
/// 4. Specification enabled out-of-the-box parsing by other programs (SPICE files are notoriously non-trivial to parse)
#[derive(Clone, Default)]
pub struct AniseContext<'a> {
    pub metadata: Metadata<'a>,
    /// Ephemeris LookUpTable (LUT) stores the mapping between a given ephemeris' hash and its index in the ephemeris list.
    pub ephemeris_lut: LookUpTable,
    /// Orientation LookUpTable (LUT) stores the mapping between a given orientation's hash and its index in the ephemeris list.
    pub orientation_lut: LookUpTable,
    /// Spacecraft constants LookUpTable (LUT) stores the mapping between a given spacecraft's hash and its index in the ephemeris list.
    pub spacecraft_constant_lut: LookUpTable,
    /// Planetary constants LookUpTable (LUT) stores the mapping between a given planetary data's hash and its index in the ephemeris list.
    pub planetary_constant_lut: LookUpTable,
    /// List of ephemerides in this file, whose index is stored in the LUT.
    pub ephemeris_data: SequenceOf<Ephemeris<'a>, MAX_TRAJECTORIES>,
    // Orientation data, both for planetary constant data and high precision orientation data
    pub orientation_data: SequenceOf<Orientation<'a>, MAX_TRAJECTORIES>,
    /// List of spacecraft constants in this file, whose index is stored in the LUT.
    pub spacecraft_constant_data: SequenceOf<SpacecraftConstants<'a>, MAX_TRAJECTORIES>,
    /// List of spacecraft constants in this file, whose index is stored in the LUT.
    pub planetary_constant_data: SequenceOf<PlanetaryConstants<'a>, MAX_TRAJECTORIES>,
}

impl<'a> Encode for AniseContext<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.metadata.encoded_len()?
            + self.ephemeris_lut.encoded_len()?
            + self.orientation_lut.encoded_len()?
            + self.ephemeris_data.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.metadata.encode(encoder)?;
        self.ephemeris_lut.encode(encoder)?;
        self.orientation_lut.encode(encoder)?;
        self.ephemeris_data.encode(encoder)
    }
}

impl<'a> Decode<'a> for AniseContext<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            metadata: decoder.decode()?,
            ephemeris_lut: decoder.decode()?,
            orientation_lut: decoder.decode()?,
            ephemeris_data: decoder.decode()?,
            ..Default::default()
        })
    }
}
