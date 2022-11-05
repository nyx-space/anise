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

use crate::{prelude::AniseError, DBL_SIZE};

use super::{covkind::CovKind, evenness::Evenness, statekind::StateKind, Field};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]

pub struct SplineMeta {
    /// Defines whether this is an evenly or unevenly timed spline
    pub evenness: Evenness,
    /// Defines what kind of state data is stored in this spline
    pub state_kind: StateKind,
    /// Defines what kind of covariance data is stored in this spline
    pub cov_kind: CovKind,
}

impl SplineMeta {
    /// Returns the offset (in bytes) in the octet string
    pub fn spline_offset(&self, idx: usize) -> usize {
        idx * self.len()
    }

    /// Returns the offset of this field in the spline given how this spline is set up.
    /// This may return an error when requesting a field that is not available.
    pub fn field_offset(&self, field: Field, coeff_idx: usize) -> Result<usize, AniseError> {
        // Make the field is valid in this spline.
        if (self.cov_kind.is_empty() && field.is_covariance())
            || (!field.is_covariance() && self.state_kind.is_empty())
        {
            Err(AniseError::ParameterNotSpecified)
        } else {
            // TODO Make sure the position data is also there.
            // Padding from header (e.g. one double for even splines, two for uneven splines).
            let header_padding = self.evenness.len();
            // Offset from the requested field (e.g. coefficients for X are stored before those for Y components).
            let field_offset = match field {
                Field::MidPoint => {
                    // Special case: the midpoint is always at the start of each spline.
                    return Ok(0);
                }
                Field::Duration => {
                    if header_padding == 2 {
                        // Special case: the duration of the spline is always the second item of each spline, if this spline type supports it
                        return Ok(DBL_SIZE);
                    } else {
                        return Err(AniseError::ParameterNotSpecified);
                    }
                }
                Field::X => 0,
                Field::Y => 1,
                Field::Z => 2,
                Field::Vx => 3,
                Field::Vy => 4,
                Field::Vz => 5,
                Field::Ax => 6,
                Field::Ay => 7,
                Field::Az => 8,
                _ => unreachable!(),
            };

            // Offset to reach the correct coefficient given the index, e.g. to get the 3rd Y component,
            // the total offset in the spline should be header_padding + 1 * num of coeffs + coefficient index.
            Ok(header_padding
                + field_offset * (self.state_kind.degree() as usize) * DBL_SIZE
                + coeff_idx * DBL_SIZE)
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
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.evenness.encode(encoder)?;
        self.state_kind.encode(encoder)?;
        self.cov_kind.encode(encoder)
    }
}

impl<'a> Decode<'a> for SplineMeta {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let spacing = decoder.decode()?;
        let state_kind = decoder.decode()?;
        let cov_kind = decoder.decode()?;

        Ok(Self {
            evenness: spacing,
            state_kind,
            cov_kind,
        })
    }
}
