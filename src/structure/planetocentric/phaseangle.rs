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
use zerocopy::{AsBytes, FromBytes};

/// Angle data is represented as a polynomial of an angle, exactly like in SPICE PCK.
/// In fact, the following documentation is basically copied from [the required PCK reading](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/pck.html).
#[derive(Copy, Clone, Debug, Default, PartialEq, AsBytes, FromBytes)]
#[repr(C)]
pub struct PhaseAngle {
    /// The fixed offset of the angular data
    pub offset_deg: f64,
    /// The rate of change of this angle per T, where T represents then number of centuries since J2000 TDB for right ascension and declination, and days since J2000 TDB for the axis twist.
    pub rate_deg: f64,
    /// The acceleration of this angle per T (same definition as above).
    pub accel_deg: f64,
}

// TODO: These are not offset, rate, accel!

/*
Based on the provided information, these quantities seem to be related to time and rates of time change, not acceleration. Let's break them down:

    d = seconds/day: This is a rate, specifically the rate of time passage per day. It is equivalent to the number of seconds in one day.

    T = seconds/Julian century: This is also a rate, indicating the number of seconds in a Julian century. It is used to describe a long span of time that is often used in astronomical calculations.

    t = ephemeris time, expressed as seconds past a reference epoch: This is not a rate, but a specific point in time. It is the measure of elapsed time from a certain reference point, known as an epoch.

None of these measures are related to acceleration, which is a measure of change in velocity over time (typically measured in units such as meters per second squared, m/s²). They all deal with time or the passage of time in different contexts, especially as related to astronomy.
 */

impl PhaseAngle {
    pub fn maybe_new(data: &[f64]) -> Option<Self> {
        if data.len() != 3 {
            None
        } else {
            Some(Self {
                offset_deg: data[0],
                rate_deg: data[1],
                accel_deg: data[2],
            })
        }
    }
}

impl Encode for PhaseAngle {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.offset_deg.encoded_len()?
            + self.rate_deg.encoded_len()?
            + self.accel_deg.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.offset_deg.encode(encoder)?;
        self.rate_deg.encode(encoder)?;
        self.accel_deg.encode(encoder)
    }
}

impl<'a> Decode<'a> for PhaseAngle {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            offset_deg: decoder.decode()?,
            rate_deg: decoder.decode()?,
            accel_deg: decoder.decode()?,
        })
    }
}
