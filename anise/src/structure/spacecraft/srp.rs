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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SRPData {
    /// Solar radiation pressure area in m^2
    pub area_m2: f64,
    /// Solar radiation pressure coefficient of reflectivity (C_r)
    pub coeff_reflectivity: f64,
}

impl Default for SRPData {
    fn default() -> Self {
        Self {
            area_m2: 0.0,
            coeff_reflectivity: 1.8,
        }
    }
}

impl Encode for SRPData {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.area_m2.encoded_len()? + self.coeff_reflectivity.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.area_m2.encode(encoder)?;
        self.coeff_reflectivity.encode(encoder)
    }
}

impl<'a> Decode<'a> for SRPData {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            area_m2: decoder.decode()?,
            coeff_reflectivity: decoder.decode()?,
        })
    }
}

#[cfg(test)]
mod srp_ut {
    use super::{Decode, Encode, SRPData};
    #[test]
    fn zero_repr() {
        let repr = SRPData {
            area_m2: Default::default(),
            coeff_reflectivity: Default::default(),
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SRPData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn default_repr() {
        let repr = SRPData::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = SRPData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }
}
