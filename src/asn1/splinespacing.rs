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
use hifitime::Duration;

/// Defines the two kinds of splines supports: equal time steps (fixed window) or unequal time steps (also called sliding window)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SplineSpacing {
    Even {
        window_duration_s: Duration,
    },
    Uneven {
        /// Unevenly spaced window ephemerides may only span five centuries to constraint stack size
        indexes: [Duration; 5],
    },
}

impl Encode for SplineSpacing {
    fn encoded_len(&self) -> der::Result<der::Length> {
        match self {
            Self::Even { window_duration_s } => (*window_duration_s).encoded_len(),
            Self::Uneven { indexes: _indexes } => {
                todo!()
            }
        }
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        match self {
            Self::Even { window_duration_s } => (*window_duration_s).encode(encoder),
            Self::Uneven { indexes: _indexes } => {
                todo!()
            }
        }
    }
}

impl<'a> Decode<'a> for SplineSpacing {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        // Check the header tag to decode this CHOICE
        if decoder.peek_tag()? == Tag::Real {
            Ok(Self::Even {
                window_duration_s: decoder.decode()?,
            })
        } else {
            decoder.sequence(|sdecoder| {
                let indexes = sdecoder.decode()?;
                Ok(Self::Uneven { indexes })
            })
        }
    }
}
