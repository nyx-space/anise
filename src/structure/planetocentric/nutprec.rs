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
use hifitime::{Epoch, Unit};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// This structure is only used to store the nutation and precession angle data.
#[derive(Copy, Clone, Debug, Default, PartialEq, AsBytes, FromZeroes, FromBytes)]
#[repr(C)]
pub struct NutationPrecessionAngle {
    offset_deg: f64,
    rate_deg: f64,
}

impl NutationPrecessionAngle {
    /// Evaluates this nutation precession angle at the given epoch
    pub fn evaluate_deg(&self, epoch: Epoch) -> f64 {
        // SPICE actually uses ET not TDB, so we use that too.
        let d = epoch.to_tdb_duration().to_unit(Unit::Century);
        self.offset_deg + self.rate_deg * d
    }
}

impl Encode for NutationPrecessionAngle {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.offset_deg.encoded_len()? + self.rate_deg.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.offset_deg.encode(encoder)?;
        self.rate_deg.encode(encoder)
    }
}

impl<'a> Decode<'a> for NutationPrecessionAngle {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            offset_deg: decoder.decode()?,
            rate_deg: decoder.decode()?,
        })
    }
}

#[cfg(test)]
mod nut_prec_ut {
    use super::{Decode, Encode, Epoch, NutationPrecessionAngle};
    #[test]
    fn zero_repr() {
        let repr = NutationPrecessionAngle {
            ..Default::default()
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = NutationPrecessionAngle::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);
    }

    #[test]
    fn example_repr() {
        let repr = NutationPrecessionAngle {
            offset_deg: 125.045,
            rate_deg: -0.052992,
        };

        let mut buf = vec![];
        repr.encode_to_vec(&mut buf).unwrap();

        let repr_dec = NutationPrecessionAngle::from_der(&buf).unwrap();

        assert_eq!(repr, repr_dec);

        // Ensure that at zero, we have only an offset.
        assert_eq!(repr.evaluate_deg(Epoch::from_tdb_seconds(0.0)), 125.045);
    }
}
