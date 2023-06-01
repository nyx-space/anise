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

use super::ellipsoid::Ellipsoid;
use super::{phaseangle::PhaseAngle, trigangle::TrigAngle};
use crate::structure::array::DataArray;
use crate::NaifId;

/// ANISE supports two different kinds of orientation data. High precision, with spline based interpolations, and constants right ascension, declination, and prime meridian, typically used for planetary constant data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlanetaryConstant<'a> {
    /// The NAIF ID of this object
    pub object_id: NaifId,
    /// Gravitational parameter (μ) of this planetary object.
    pub mu_km3_s2: f64,
    /// The shape is always a tri axial ellipsoid
    pub shape: Option<Ellipsoid>,
    ///     TODO: Create a PoleOrientation structure which is optional. If defined, it includes the stuff below, and none optional (DataArray can be empty).
    pub pole_right_ascension: Option<PhaseAngle>,
    pub pole_declination: Option<PhaseAngle>,
    pub prime_meridian: Option<PhaseAngle>,
    pub nut_prec_angles: Option<DataArray<'a, TrigAngle>>,
}

impl<'a> Encode for PlanetaryConstant<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.object_id.encoded_len()?
            + self.mu_km3_s2.encoded_len()?
            + self.shape.encoded_len()?
            + self.pole_right_ascension.encoded_len()?
            + self.pole_declination.encoded_len()?
            + self.prime_meridian.encoded_len()?
            + self.nut_prec_angles.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.object_id.encode(encoder)?;
        self.mu_km3_s2.encode(encoder)?;
        self.shape.encode(encoder)?;
        self.pole_right_ascension.encode(encoder)?;
        self.pole_declination.encode(encoder)?;
        self.prime_meridian.encode(encoder)?;
        self.nut_prec_angles.encode(encoder)
    }
}

impl<'a> Decode<'a> for PlanetaryConstant<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let object_id: NaifId = decoder.decode()?;
        let mu_km3_s2: f64 = decoder.decode()?;
        // TODO: I don't think this works because the data may be empty
        Ok(Self {
            object_id,
            mu_km3_s2,
            shape: Some(decoder.decode()?),
            pole_right_ascension: Some(decoder.decode()?),
            pole_declination: Some(decoder.decode()?),
            prime_meridian: Some(decoder.decode()?),
            nut_prec_angles: Some(decoder.decode()?),
        })
    }
}
