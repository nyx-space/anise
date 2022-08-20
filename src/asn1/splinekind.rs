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

/// Defines the two kinds of splines supports: equal time steps (fixed window) or unequal time steps (also called sliding window)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SplineKind {
    FixedWindow {
        window_duration_s: f64,
    },
    SlidingWindow {
        /// Sliding window ephemerides may only span one millenia to constraint stack size
        indexes: [TimeIndex; 10],
    },
}

impl Encode for SplineKind {
    fn encoded_len(&self) -> der::Result<der::Length> {
        match self {
            Self::FixedWindow { window_duration_s } => (*window_duration_s).encoded_len(),
            Self::SlidingWindow { indexes: _indexes } => {
                todo!()
            }
        }
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        match self {
            Self::FixedWindow { window_duration_s } => (*window_duration_s).encode(encoder),
            Self::SlidingWindow { indexes: _indexes } => {
                todo!()
            }
        }
    }
}

impl<'a> Decode<'a> for SplineKind {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        // Check the header tag to decode this CHOICE
        if decoder.peek_tag()? == Tag::Real {
            Ok(Self::FixedWindow {
                window_duration_s: decoder.decode()?,
            })
        } else {
            decoder.sequence(|sdecoder| {
                let indexes = sdecoder.decode()?;
                Ok(Self::SlidingWindow { indexes })
            })
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct TimeIndex {
    pub century: i16,
    pub nanoseconds: u64,
}

impl Encode for TimeIndex {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.century.encoded_len()? + self.nanoseconds.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.century.encode(encoder)?;
        self.nanoseconds.encode(encoder)
    }
}

impl<'a> Decode<'a> for TimeIndex {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            century: decoder.decode()?,
            nanoseconds: decoder.decode()?,
        })
    }
}
