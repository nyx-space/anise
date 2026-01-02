/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{Decode, Encode, Reader, Writer};

use crate::math::Vector3;

use super::{FovShape, Instrument};

impl FovShape {
    fn variant(&self) -> u8 {
        match self {
            Self::Conical { .. } => 0,
            Self::Rectangular { .. } => 1,
        }
    }
}

impl Encode for FovShape {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.variant().encoded_len()?
            + match self {
                Self::Conical { half_angle_deg } => {
                    (half_angle_deg.encoded_len()? + f64::NAN.encoded_len()?)?
                }
                Self::Rectangular {
                    x_half_angle_deg,
                    y_half_angle_deg,
                } => (x_half_angle_deg.encoded_len()? + y_half_angle_deg.encoded_len()?)?,
            }
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.variant().encode(encoder)?;
        match self {
            Self::Conical { half_angle_deg } => {
                half_angle_deg.encode(encoder)?;
                f64::NAN.encode(encoder)
            }
            Self::Rectangular {
                x_half_angle_deg,
                y_half_angle_deg,
            } => {
                x_half_angle_deg.encode(encoder)?;
                y_half_angle_deg.encode(encoder)
            }
        }
    }
}

impl<'a> Decode<'a> for FovShape {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let variant: u8 = decoder.decode()?;

        Ok(match variant {
            0 => {
                let half_angle_deg = decoder.decode()?;
                // Decode the f64::NAN we added for padding.
                let _: f64 = decoder.decode()?;
                Self::Conical { half_angle_deg }
            }
            1 => {
                let x_half_angle_deg = decoder.decode()?;
                let y_half_angle_deg = decoder.decode()?;
                Self::Rectangular {
                    x_half_angle_deg,
                    y_half_angle_deg,
                }
            }
            _ => Self::default(),
        })
    }
}

impl Encode for Instrument {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.mounting_rotation.encoded_len()?
            + self.mounting_translation.x.encoded_len()?
            + self.mounting_translation.y.encoded_len()?
            + self.mounting_translation.z.encoded_len()?
            + self.fov.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.mounting_rotation.encode(encoder)?;
        self.mounting_translation.x.encode(encoder)?;
        self.mounting_translation.y.encode(encoder)?;
        self.mounting_translation.z.encode(encoder)?;
        self.fov.encode(encoder)
    }
}

impl<'a> Decode<'a> for Instrument {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let mounting_rotation = decoder.decode()?;
        let x = decoder.decode()?;
        let y = decoder.decode()?;
        let z = decoder.decode()?;
        let fov = decoder.decode()?;

        Ok(Self {
            mounting_rotation,
            mounting_translation: Vector3::new(x, y, z),
            fov,
        })
    }
}

#[cfg(test)]
mod instrument_encdec {
    use crate::math::rotation::EulerParameter;

    use super::*;

    #[test]
    fn conical() {
        let repr = Instrument {
            mounting_rotation: EulerParameter::about_x(core::f64::consts::FRAC_2_SQRT_PI, 1, 2),
            mounting_translation: Vector3::new(1.0, 2.0, 3.0),
            fov: FovShape::Conical {
                half_angle_deg: 12.3,
            },
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = Instrument::from_der(&buf).unwrap();

        assert_eq!(repr_dec, repr);
    }

    #[test]
    fn rect() {
        let repr = Instrument {
            mounting_rotation: EulerParameter::about_x(core::f64::consts::FRAC_2_SQRT_PI, 1, 2),
            mounting_translation: Vector3::new(1.0, 2.0, 3.0),
            fov: FovShape::Rectangular {
                x_half_angle_deg: 12.3,
                y_half_angle_deg: 13.9,
            },
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = Instrument::from_der(&buf).unwrap();

        assert_eq!(repr_dec, repr);
    }
}
