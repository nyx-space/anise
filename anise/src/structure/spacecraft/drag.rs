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
pub struct DragData {
    /// Atmospheric drag area in m^2
    pub area_m2: f64,
    /// Drag coefficient (C_d)
    pub coeff_drag: f64,
}

impl Default for DragData {
    fn default() -> Self {
        Self {
            area_m2: 0.0,
            coeff_drag: 2.2,
        }
    }
}

impl Encode for DragData {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.area_m2.encoded_len()? + self.coeff_drag.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.area_m2.encode(encoder)?;
        self.coeff_drag.encode(encoder)
    }
}

impl<'a> Decode<'a> for DragData {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            area_m2: decoder.decode()?,
            coeff_drag: decoder.decode()?,
        })
    }
}

#[cfg(test)]
mod drag_ut {
    use super::{Decode, DragData, Encode};
    #[test]
    fn zero_repr() {
        let repr = DragData {
            area_m2: Default::default(),
            coeff_drag: Default::default(),
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = DragData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn default_repr() {
        let repr = DragData::default();

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = DragData::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }
}
