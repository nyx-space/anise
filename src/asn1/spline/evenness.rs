/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{Decode, Encode, Reader, Tag, Writer};

use crate::DBL_SIZE;

/// Splice Space defines whether this is an equal-time step interpolation spline (called `Even` splines in ANISE) or an unequal-time step spline (called `Uneven`).
///
/// # Even splines
///
/// These store data like what would typically be stored in the NAIF SPK Types 2, 3, 8, and 12. The interpolation of the trajectory is done over a fixed time window, e.g. 16 days.
/// In ANISE, a single interpolation spline must be less than 4000 years because the window duration is stored in nanoseconds on an unsigned integer.
///
/// ## Querying
/// To query the set of coefficients needed for a given interpolation, the following algorithm applies given the desired epoch `epoch` as an input parameter.
/// 1. Compute `delta_ns`: the difference in nanoseconds between `epoch` and the ephemeris start epoch (making sure that both are in the same time system).
/// 2. Compute the spline index `spl_idx` by performing the integer division between `delta_ns` and the spline duration `duration_ns` (defined in the meta data of the splines).
/// 3. Seek through the byte string of the spline data by chunks of the spline length, which depends on the kind of data stored (Position, etc.) and the existence of not of covariance information.
///
/// Defines the two kinds of spacing splines supports: equal time steps (fixed sized interpolation) or unequal time steps (also called sliding window)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Evenness {
    Even {
        duration_ns: u64,
    },
    Uneven {
        /// Unevenly spaced window ephemerides may only span five centuries to constraint stack size
        indexes: [i16; 5], // TODO: Consider 10? Or just enough for DE in full.
    },
}

impl Evenness {
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length in octets that precedes the spline coefficient data.
    ///
    /// For example, if this is set to 16, it means that the spline coefficient data starts at an offset of 16 compared to the start of the spline itself.
    pub const fn len(&self) -> usize {
        DBL_SIZE
            * match self {
                Self::Even { duration_ns: _ } => 1,
                Self::Uneven { indexes: _ } => 2,
            }
    }
}

impl Default for Evenness {
    fn default() -> Self {
        Self::Even { duration_ns: 0 }
    }
}

impl Encode for Evenness {
    fn encoded_len(&self) -> der::Result<der::Length> {
        match self {
            Self::Even { duration_ns } => (*duration_ns).encoded_len(),
            Self::Uneven { indexes: _indexes } => {
                todo!()
            }
        }
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        match self {
            Self::Even { duration_ns } => (*duration_ns).encode(encoder),
            Self::Uneven { indexes: _indexes } => {
                todo!()
            }
        }
    }
}

impl<'a> Decode<'a> for Evenness {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        // Check the header tag to decode this CHOICE
        if decoder.peek_tag()? == Tag::Real {
            Ok(Self::Even {
                duration_ns: decoder.decode()?,
            })
        } else {
            decoder.sequence(|sdecoder| {
                let indexes = sdecoder.decode()?;
                Ok(Self::Uneven { indexes })
            })
        }
    }
}
