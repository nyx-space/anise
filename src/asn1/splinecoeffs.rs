/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{Decode, Encode, Reader, Writer};

use crate::DBL_SIZE;

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct SplineCoeffCount {
    pub degree: u8,
    pub num_epochs: u8,
    pub num_position_coeffs: u8,
    pub num_position_dt_coeffs: u8,
    pub num_velocity_coeffs: u8,
    pub num_velocity_dt_coeffs: u8,
}

impl SplineCoeffCount {
    /// Returns the offset (in bytes) in the octet string
    pub const fn spline_offset(&self, idx: usize) -> usize {
        idx * self.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of a spline in bytes
    pub const fn len(&self) -> usize {
        let num_items = self.num_epochs
            + self.num_position_coeffs * self.degree
            + self.num_position_dt_coeffs * self.degree
            + self.num_velocity_coeffs * self.degree
            + self.num_velocity_dt_coeffs * self.degree;
        DBL_SIZE * (num_items as usize)
    }
}

impl Encode for SplineCoeffCount {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.degree.encoded_len()?
            + self.num_epochs.encoded_len()?
            + self.num_position_coeffs.encoded_len()?
            + self.num_position_dt_coeffs.encoded_len()?
            + self.num_velocity_coeffs.encoded_len()?
            + self.num_velocity_dt_coeffs.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.degree.encode(encoder)?;
        self.num_epochs.encode(encoder)?;
        self.num_position_coeffs.encode(encoder)?;
        self.num_position_dt_coeffs.encode(encoder)?;
        self.num_velocity_coeffs.encode(encoder)?;
        self.num_velocity_dt_coeffs.encode(encoder)
    }
}

impl<'a> Decode<'a> for SplineCoeffCount {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            degree: decoder.decode()?,
            num_epochs: decoder.decode()?,
            num_position_coeffs: decoder.decode()?,
            num_position_dt_coeffs: decoder.decode()?,
            num_velocity_coeffs: decoder.decode()?,
            num_velocity_dt_coeffs: decoder.decode()?,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Coefficient {
    X,
    Xdt,
    Y,
    Ydt,
    Z,
    Zdt,
    VX,
    VXdt,
    VY,
    VYdt,
    VZ,
    VZdt,
}
