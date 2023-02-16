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

pub const MAX_NUT_PREC_ANGLES: usize = 16;

use super::{phaseangle::PhaseAngle, trigangle::TrigAngle};
use crate::structure::array::DataArray;

/// ANISE supports two different kinds of orientation data. High precision, with spline based interpolations, and constants right ascension, declination, and prime meridian, typically used for planetary constant data.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanetaryConstant<'a> {
    pub semi_major_radii_km: f64,
    pub semi_minor_radii_km: f64,
    pub polar_radii_km: f64,
    pub pole_right_ascension: PhaseAngle,
    pub pole_declination: PhaseAngle,
    pub prime_meridian: PhaseAngle,
    pub nut_prec_angles: DataArray<'a, TrigAngle>,
}

impl<'a> Encode for PlanetaryConstant<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.semi_major_radii_km.encoded_len()?
            + self.semi_minor_radii_km.encoded_len()?
            + self.polar_radii_km.encoded_len()?
            + self.pole_right_ascension.encoded_len()?
            + self.pole_declination.encoded_len()?
            + self.prime_meridian.encoded_len()?
            + self.nut_prec_angles.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.semi_major_radii_km.encode(encoder)?;
        self.semi_minor_radii_km.encode(encoder)?;
        self.polar_radii_km.encode(encoder)?;
        self.pole_right_ascension.encode(encoder)?;
        self.pole_declination.encode(encoder)?;
        self.prime_meridian.encode(encoder)?;
        self.nut_prec_angles.encode(encoder)
    }
}

impl<'a> Decode<'a> for PlanetaryConstant<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            semi_major_radii_km: decoder.decode()?,
            semi_minor_radii_km: decoder.decode()?,
            polar_radii_km: decoder.decode()?,
            pole_right_ascension: decoder.decode()?,
            pole_declination: decoder.decode()?,
            prime_meridian: decoder.decode()?,
            nut_prec_angles: decoder.decode()?,
        })
    }
}
