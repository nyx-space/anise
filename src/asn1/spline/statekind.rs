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

use crate::DBL_SIZE;

/// State Kind defines what kind of state is stored in the spline.
///
/// # Limitations
/// 1. The interpolation degree of all items in the state must be identical.
/// 2. A constant position must be encoded as degree 1 whose sole coefficient is the constant value.
///
/// ## Example
/// If the position is interpolated with an 11 degree polynomial, and the velocity must also be interpolated with an 11 degree polynomial.
///
/// # Encoding / decoding
/// The state kind is encoded along with its degree as a single field in the ASN1 encoding scheme.
/// The interpolation degrees are expressed on an 8-bit unsigned integer whose maximum value is 255 (2^8 - 1).
/// Hence, to encode both the state kind and the interpolation degree, a spacing of 255 is used between each state kind.
///
/// ASN1 encodes the tag and length as one octet each. Hence, position state metadata will always fit in exactly three octets: tag (1), length (1), degree (1).
/// Position and velocity data will fit in four octets: tag (1), length (1), data (2). And so on for each state kind.
/// Had the degree and state kind been stored as separate fields, we would be constantly using exactly six octets.
///
/// The other advantage is that a single ASN1 tag decoding will yield both the state kind and the degree: this allows the code to store both the state kind and the degree in the same enumerate structure.
///
/// ## Example
///
/// | Encoded value | State Kind | Degree |
/// | -- | -- | -- |
/// | 1 | Position only | 1
/// | 28 | Position only | 28
/// | 266 | Position and Velocity | 266-255 = 11
/// | 0 | None | _not applicable_
///
/// # Storage
///
/// Position data will always require three fields (x, y, z). Velocity adds another three (vx, vy, vz), and so does acceleration (ax, ay, az).
///
/// ## Example
/// Storing the position and velocity with an 11 degree polynomial will require 11*6 = 66 coefficient. Each coefficient is stored as packed structure of 8 octets floating point values in IEEE754 format.
/// Hence, this would require 66*8 = 528 octets per spline.
///
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum StateKind {
    /// No state data at all, i.e. this spline only has covariance information
    None,
    Position {
        degree: u8,
    },
    PositionVelocity {
        degree: u8,
    },
    PositionVelocityAcceleration {
        degree: u8,
    },
}

impl StateKind {
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length required to store this covariance information
    pub const fn len(&self) -> usize {
        let num_items = match self {
            StateKind::None => 0,
            StateKind::Position { degree } => degree * 3,
            StateKind::PositionVelocity { degree } => degree * 6,
            StateKind::PositionVelocityAcceleration { degree } => degree * 9,
        };
        DBL_SIZE * usize::from(num_items)
    }
}

impl Default for StateKind {
    fn default() -> Self {
        Self::None
    }
}

/// Allows conversion of the StateKind into a u8 with the following mapping.
impl From<StateKind> for u16 {
    fn from(kind: StateKind) -> Self {
        match kind {
            StateKind::None => 0,
            StateKind::Position { degree } => degree.into(),
            StateKind::PositionVelocity { degree } => (u8::MAX + degree).into(),
            StateKind::PositionVelocityAcceleration { degree } => {
                2_u16 * (u8::MAX as u16) + (degree as u16)
            }
        }
    }
}

impl From<&StateKind> for u16 {
    fn from(kind: &StateKind) -> Self {
        u16::from(*kind)
    }
}

/// Allows conversion of a u8 into a StateKind.
impl From<u16> for StateKind {
    fn from(val: u16) -> Self {
        if val == 0 {
            Self::None
        } else {
            // Prevents an overflow and coerces the degree to be within the bounds of a u8, as per the specs.
            let degree = (val % (u8::MAX as u16)) as u8;
            if val < u8::MAX.into() {
                Self::Position { degree }
            } else if val < 2 * (u8::MAX as u16) {
                Self::PositionVelocity { degree }
            } else {
                Self::PositionVelocityAcceleration { degree }
            }
        }
    }
}

impl Encode for StateKind {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let converted: u16 = self.into();
        converted.encoded_len()
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        let converted: u16 = self.into();
        converted.encode(encoder)
    }
}

impl<'a> Decode<'a> for StateKind {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let converted: u16 = decoder.decode()?;
        Ok(Self::from(converted))
    }
}
