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
use hifitime::Epoch;

pub const MAX_NUT_PREC_ANGLES: usize = 16;

use super::{phaseangle::PhaseAngle, trigangle::TrigAngle};
use crate::structure::{common::InterpolationKind, spline::Splines};

/// ANISE supports two different kinds of orientation data. High precision, with spline based interpolations, and constants right ascension, declination, and prime meridian, typically used for planetary constant data.
#[derive(Clone, Debug, PartialEq)]
pub enum OrientationData<'a> {
    PlanetaryConstant {
        pole_right_ascension: PhaseAngle,
        pole_declination: PhaseAngle,
        prime_meridian: PhaseAngle,
        nut_prec_angles: SequenceOf<TrigAngle, MAX_NUT_PREC_ANGLES>,
    },
    HighPrecision {
        ref_epoch: Epoch,
        backward: bool,
        interpolation_kind: InterpolationKind,
        splines: Splines<'a>,
    },
}

impl<'a> Encode for OrientationData<'a> {
    fn encoded_len(&self) -> der::Result<der::Length> {
        match &self {
            OrientationData::PlanetaryConstant {
                pole_right_ascension,
                pole_declination,
                prime_meridian,
                nut_prec_angles,
            } => {
                pole_right_ascension.encoded_len()?
                    + pole_declination.encoded_len()?
                    + prime_meridian.encoded_len()?
                    + nut_prec_angles.encoded_len()?
            }
            OrientationData::HighPrecision {
                ref_epoch,
                backward,
                interpolation_kind,
                splines,
            } => {
                ref_epoch.encoded_len()?
                    + backward.encoded_len()?
                    + interpolation_kind.encoded_len()?
                    + splines.encoded_len()?
            }
        }
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        match &self {
            OrientationData::PlanetaryConstant {
                pole_right_ascension,
                pole_declination,
                prime_meridian,
                nut_prec_angles,
            } => {
                pole_right_ascension.encode(encoder)?;
                pole_declination.encode(encoder)?;
                prime_meridian.encode(encoder)?;
                nut_prec_angles.encode(encoder)
            }
            OrientationData::HighPrecision {
                ref_epoch,
                backward,
                interpolation_kind,
                splines,
            } => {
                ref_epoch.encode(encoder)?;
                backward.encode(encoder)?;
                interpolation_kind.encode(encoder)?;
                splines.encode(encoder)
            }
        }
    }
}

impl<'a> Decode<'a> for OrientationData<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        // This is encoded as a CHOICE, so let's try to decode the first field as a PhaseAngle, and if it isn't
        // we'll try as an Epoch.

        match decoder.decode::<PhaseAngle>() {
            Ok(pole_right_ascension) => Ok(Self::PlanetaryConstant {
                pole_right_ascension,
                pole_declination: decoder.decode()?,
                prime_meridian: decoder.decode()?,
                nut_prec_angles: decoder.decode()?,
            }),
            Err(_) => {
                // Hopefully this is a high precision orientation, otherwise the error will rise up.
                Ok(Self::HighPrecision {
                    ref_epoch: decoder.decode()?,
                    backward: decoder.decode()?,
                    interpolation_kind: decoder.decode()?,
                    splines: decoder.decode()?,
                })
            }
        }
    }
}
