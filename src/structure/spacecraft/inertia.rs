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

use crate::NaifId;

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
