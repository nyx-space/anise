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
use nalgebra::Matrix3;

use crate::NaifId;

/// Inertial tensor definition
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Inertia {
    /// Inertia tensor reference frame hash
    pub orientation_id: NaifId,
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

impl Inertia {
    pub fn tensor_kgm2(&self) -> Matrix3<f64> {
        Matrix3::new(
            self.i_11_kgm2,
            self.i_12_kgm2,
            self.i_13_kgm2,
            self.i_12_kgm2,
            self.i_22_kgm2,
            self.i_23_kgm2,
            self.i_13_kgm2,
            self.i_23_kgm2,
            self.i_33_kgm2,
        )
    }
}

impl Encode for Inertia {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.orientation_id.encoded_len()?
            + self.i_11_kgm2.encoded_len()?
            + self.i_22_kgm2.encoded_len()?
            + self.i_33_kgm2.encoded_len()?
            + self.i_12_kgm2.encoded_len()?
            + self.i_13_kgm2.encoded_len()?
            + self.i_23_kgm2.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.orientation_id.encode(encoder)?;
        self.i_11_kgm2.encode(encoder)?;
        self.i_22_kgm2.encode(encoder)?;
        self.i_33_kgm2.encode(encoder)?;
        self.i_12_kgm2.encode(encoder)?;
        self.i_13_kgm2.encode(encoder)?;
        self.i_23_kgm2.encode(encoder)
    }
}

impl<'a> Decode<'a> for Inertia {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            orientation_id: decoder.decode()?,
            i_11_kgm2: decoder.decode()?,
            i_22_kgm2: decoder.decode()?,
            i_33_kgm2: decoder.decode()?,
            i_12_kgm2: decoder.decode()?,
            i_13_kgm2: decoder.decode()?,
            i_23_kgm2: decoder.decode()?,
        })
    }
}

#[cfg(test)]
mod inertia_ut {
    use super::{Decode, Encode, Inertia, Matrix3};
    #[test]
    fn example_repr() {
        let repr = Inertia {
            // Spacecraft IDs are typically negative
            orientation_id: -20,
            i_11_kgm2: 120.0,
            i_22_kgm2: 180.0,
            i_33_kgm2: 220.0,
            i_12_kgm2: 20.0,
            i_13_kgm2: -15.0,
            i_23_kgm2: 30.0,
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = Inertia::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        let tensor = Matrix3::new(120.0, 20.0, -15.0, 20.0, 180.0, 30.0, -15.0, 30.0, 220.0);
        assert_eq!(tensor, repr.tensor_kgm2());
    }

    #[test]
    fn default_repr() {
        let repr = Inertia::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = Inertia::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }
}
