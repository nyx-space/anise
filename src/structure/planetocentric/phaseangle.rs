/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use super::MAX_NUT_PREC_ANGLES;
use core::fmt;
use der::{Decode, Encode, Reader, Writer};
use hifitime::{Epoch, Unit};

/// Angle data is represented as a polynomial of an angle, exactly like in SPICE PCK.
/// In fact, the following documentation is basically copied from [the required PCK reading](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/pck.html).
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct PhaseAngle {
    /// The fixed offset of the angular data
    pub offset_deg: f64,
    /// The rate of change of this angle per T, where T represents then number of centuries since J2000 TDB for right ascension and declination, and days since J2000 TDB for the axis twist.
    pub rate_deg: f64,
    /// The acceleration of this angle per T (same definition as above).
    pub accel_deg: f64,
    /// Number of nutation / precession angle coefficients
    pub coeffs_count: u8,
    pub coeffs: [f64; MAX_NUT_PREC_ANGLES],
}

impl PhaseAngle {
    pub fn maybe_new(data: &[f64]) -> Option<Self> {
        if data.len() < 3 {
            None
        } else {
            let mut coeffs = [0.0; MAX_NUT_PREC_ANGLES];
            for (i, coeff) in data.iter().skip(3).enumerate() {
                coeffs[i] = *coeff;
            }
            Some(Self {
                offset_deg: data[0],
                rate_deg: data[1],
                accel_deg: data[2],
                coeffs_count: (data.len() as u8).saturating_sub(3),
                coeffs,
            })
        }
    }

    /// Evaluates this phase angle in degrees provided the epoch
    pub fn evaluate_deg(&self, epoch: Epoch, rate_unit: Unit) -> f64 {
        let factor = epoch.to_tdb_duration().to_unit(rate_unit);

        self.offset_deg + self.rate_deg * factor + self.accel_deg * factor.powi(2)
    }
}

impl Encode for PhaseAngle {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.offset_deg.encoded_len()?
            + self.rate_deg.encoded_len()?
            + self.accel_deg.encoded_len()?
            + self.coeffs_count.encoded_len()?
            + self.coeffs.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.offset_deg.encode(encoder)?;
        self.rate_deg.encode(encoder)?;
        self.accel_deg.encode(encoder)?;
        self.coeffs_count.encode(encoder)?;
        self.coeffs.encode(encoder)
    }
}

impl<'a> Decode<'a> for PhaseAngle {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            offset_deg: decoder.decode()?,
            rate_deg: decoder.decode()?,
            accel_deg: decoder.decode()?,
            coeffs_count: decoder.decode()?,
            coeffs: decoder.decode()?,
        })
    }
}

impl fmt::Display for PhaseAngle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.accel_deg.abs() > 0.0 {
            write!(
                f,
                "{} + {} x + {} x^2",
                self.offset_deg, self.rate_deg, self.accel_deg
            )
        } else {
            write!(f, "{} + {} x", self.offset_deg, self.rate_deg)
        }
    }
}
