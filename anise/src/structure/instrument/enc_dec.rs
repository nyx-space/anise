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

/// The field of view half angles feed `to_radians()` and then `sin_cos`/`tan`/`normalize` in the
/// instrument FOV helpers, so a non-finite angle read from an untrusted instrument dataset turns
/// `generate_fov_boundary_vectors` into NaN rays and `fov_margin_deg` into a NaN margin, silently
/// corrupting footprint and in-FOV results. Reject it on decode like the other ANISE decoders
/// reject their out-of-range floats.
fn reject_non_finite_angle(angle_deg: f64) -> der::Result<()> {
    if angle_deg.is_finite() {
        Ok(())
    } else {
        Err(der::Error::new(
            der::ErrorKind::Value {
                tag: der::Tag::Real,
            },
            der::Length::ONE,
        ))
    }
}

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
                let half_angle_deg: f64 = decoder.decode()?;
                // Decode the f64::NAN we added for padding (intentionally non-finite, not validated).
                let _: f64 = decoder.decode()?;
                reject_non_finite_angle(half_angle_deg)?;
                Self::Conical { half_angle_deg }
            }
            1 => {
                let x_half_angle_deg: f64 = decoder.decode()?;
                let y_half_angle_deg: f64 = decoder.decode()?;
                reject_non_finite_angle(x_half_angle_deg)?;
                reject_non_finite_angle(y_half_angle_deg)?;
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
        self.q_to_i.encoded_len()?
            + self.offset_i.x.encoded_len()?
            + self.offset_i.y.encoded_len()?
            + self.offset_i.z.encoded_len()?
            + self.fov.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.q_to_i.encode(encoder)?;
        self.offset_i.x.encode(encoder)?;
        self.offset_i.y.encode(encoder)?;
        self.offset_i.z.encode(encoder)?;
        self.fov.encode(encoder)
    }
}

impl<'a> Decode<'a> for Instrument {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let q_to_i = decoder.decode()?;
        let x = decoder.decode()?;
        let y = decoder.decode()?;
        let z = decoder.decode()?;
        let fov = decoder.decode()?;

        Ok(Self {
            q_to_i,
            offset_i: Vector3::new(x, y, z),
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
            q_to_i: EulerParameter::about_x(core::f64::consts::FRAC_2_SQRT_PI, 1, 2),
            offset_i: Vector3::new(1.0, 2.0, 3.0),
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
            q_to_i: EulerParameter::about_x(core::f64::consts::FRAC_2_SQRT_PI, 1, 2),
            offset_i: Vector3::new(1.0, 2.0, 3.0),
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

    #[test]
    fn reject_non_finite_fov_angles() {
        // A crafted instrument dataset carrying a non-finite FOV half angle decodes fine without
        // this guard, then normalize()/tan() in the FOV helpers turn it into NaN rays and margins.
        for bad in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            let conical = FovShape::Conical {
                half_angle_deg: bad,
            };
            let mut buf = vec![];
            conical.encode_to_vec(&mut buf).unwrap();
            assert!(
                FovShape::from_der(&buf).is_err(),
                "non-finite conical half angle {bad} must be rejected"
            );

            for rect in [
                FovShape::Rectangular {
                    x_half_angle_deg: bad,
                    y_half_angle_deg: 12.3,
                },
                FovShape::Rectangular {
                    x_half_angle_deg: 12.3,
                    y_half_angle_deg: bad,
                },
            ] {
                let mut buf = vec![];
                rect.encode_to_vec(&mut buf).unwrap();
                assert!(
                    FovShape::from_der(&buf).is_err(),
                    "non-finite rectangular half angle {bad} must be rejected"
                );
            }
        }

        // A valid finite FOV still round-trips.
        let good = FovShape::Conical {
            half_angle_deg: 12.3,
        };
        let mut buf = vec![];
        good.encode_to_vec(&mut buf).unwrap();
        assert_eq!(FovShape::from_der(&buf).unwrap(), good);
    }
}
