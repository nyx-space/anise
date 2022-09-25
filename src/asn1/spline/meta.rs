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

use crate::{
    asn1::units::{DistanceUnit, TimeUnit},
    prelude::AniseError,
    DBL_SIZE,
};

use super::{covkind::CovKind, evenness::Evenness, statekind::StateKind, Field};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]

pub struct SplineMeta {
    /// Defines whether this is an evenly or unevenly timed spline
    pub evenness: Evenness,
    /// Defines what kind of state data is stored in this spline
    pub state_kind: StateKind,
    /// Defines what kind of covariance data is stored in this spline
    pub cov_kind: CovKind,
    /// Defines the numerator unit of the state data (e.g. "kilometer", the default)
    pub numerator_unit: DistanceUnit,
    /// Defines the denominator unit of the state data (e.g. "second", the default)
    pub denominator_unit: TimeUnit,
}

impl SplineMeta {
    /// Returns the offset (in bytes) in the octet string
    pub fn spline_offset(&self, idx: usize) -> usize {
        idx * self.len()
    }

    /// Returns the offset of this field in the spline given how this spline is set up.
    /// This may return an error when requesting a field that is not available.
    pub fn field_offset(&self, field: Field, coeff_idx: usize) -> Result<usize, AniseError> {
        if self.cov_kind.is_empty() {
            if field >= Field::CovXX {
                Err(AniseError::ParameterNotSpecified)
            } else {
                // Padding from header (e.g. one double for even splines, two for uneven splines).
                let header_padding = self.evenness.len();
                // Offset from the requested field (e.g. coefficients for X are stored before those for Y components).
                let field_offset = match field {
                    Field::X => 0,
                    Field::Y => 1,
                    Field::Z => 2,
                    Field::Vx => 3,
                    Field::Vy => 4,
                    Field::Vz => 5,
                    _ => unreachable!(),
                };

                // Offset to reach the correct coefficient given the index, e.g. to get the 3rd Y component,
                // the total offset in the spline should be header_padding + 1 * num of coeffs + coefficient index.

                Ok(header_padding
                    + field_offset * (self.state_kind.degree() as usize) * DBL_SIZE
                    + coeff_idx * DBL_SIZE)
            }
        } else {
            todo!()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of a spline in bytes
    pub fn len(&self) -> usize {
        self.evenness.len() + self.state_kind.len() + self.cov_kind.len()
    }
}

impl Encode for SplineMeta {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.evenness.encoded_len()?
            + self.state_kind.encoded_len()?
            + self.cov_kind.encoded_len()?
            + self.numerator_unit.encoded_len()?
            + self.denominator_unit.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.evenness.encode(encoder)?;
        self.state_kind.encode(encoder)?;
        self.cov_kind.encode(encoder)?;
        self.numerator_unit.encode(encoder)?;
        self.denominator_unit.encode(encoder)
    }
}

impl<'a> Decode<'a> for SplineMeta {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let spacing = decoder.decode()?;
        let state_kind = decoder.decode()?;
        let cov_kind = decoder.decode()?;
        let numerator_unit = decoder.decode()?;
        let denominator_unit = decoder.decode()?;

        Ok(Self {
            evenness: spacing,
            state_kind,
            cov_kind,
            numerator_unit,
            denominator_unit,
        })
    }
}

/*
    + All Spline data has both the start epoch of the spline and the duration: this will be 11 and 10 octets each! Hopefully that isn't too large.
    + If it is too large, if spline space is set to evenly spaced, then remove the duration ==> that means the first entry should be duration and not epoch
        => it's OK to remove the first item or the last, weird to remove any other one.
    + For the index, consider only storing the centuries as i16. Then, for a given time T, check the century. to get the first mini-segment?
    And then store N epochs a u64 offset in nanoseconds from that century? The only issue: how to fetch the N-th mini-segment?
        => Maybe, in the time index, store the century and total length in bytes of what's encoded for that century? That should work but it'll be a pain to create especially for an interpolation overlapping two centuries.
    + ALSO! Shouldn't the window duration for evenly spaced splines be a single u64 of nanoseconds? Same size as f64, but more precise, and can have up to 4 centuries: not bad!
*/
