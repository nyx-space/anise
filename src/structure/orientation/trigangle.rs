/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{asn1::OctetStringRef, Decode, Encode, Reader, Writer};

use super::phaseangle::PhaseAngle;
use zerocopy::{AsBytes, FromBytes};

/// Trigonometric angle polynomials are used almost exclusively for satellites.
/// This structure enables the PCK level support for this information while also enforcing the correct length.
/// In fact, the following documentation is **copied** from [the required PCK reading](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/pck.html).
///
/// Orientation models for natural satellites of planets are a little more complicated; in addition to polynomial terms, the RA, DEC, and W expressions include trigonometric terms. The arguments of the trigonometric terms are linear polynomials. These arguments are sometimes called ``phase angles.'' However, within CSPICE internal documentation, these quantities often are called ``nutation precession angles.'' That terminology is used here.
///
/// Expressions for the right ascension and declination of the north pole and the location of the prime meridian for any satellite of a given planet are as follows:
///
///                                 2      ____
///                            RA2*t       \
///    RA  = RA0  + RA1*t/T  + ------   +  /     a  * sin * theta
///                               2        ----   i              i
///                              T           i
///  
///                                  2     ____
///                            DEC2*t      \
///    DEC = DEC0 + DEC1*t/T + -------  +  /    d  * cos * theta
///                                2       ----  i              i
///                               T          i
///  
///                                2       ____
///                            W2*t        \
///    W   = W0   + W1*t/d   + -----    +  /     w  * sin * theta
///                               2        ----   i              i
///                              d           i
///
/// where
///
///    d = seconds/day
///    T = seconds/Julian century
///    t = ephemeris time, expressed as seconds past a reference epoch
///
/// RA0, RA1, DEC0, DEC1, W0, and W1 are constants specific to each satellite.
///
/// The nutation precession angles
///
///    theta
///         i
///
/// are specific to each planet. The coefficients
///
///    a ,  d ,  and w
///     i    i        i
///
/// are specific to each satellite.
#[derive(Copy, Clone, Debug, Default, PartialEq, AsBytes, FromBytes)]
#[repr(packed)]
pub struct TrigAngle {
    /// Right ascension angle factor for this trig polynomial
    pub right_ascension_deg: f64,
    /// Declination angle factor for this trig polynomial
    pub declination_deg: f64,
    /// Prime meridian angle factor for this trig polynomial
    pub prime_meridian_deg: f64,
    /// Nutation and precession phase angle data of each trigonometric polynomial, e.g. J1-J10 and Ja-Je for Jupiter data.
    pub nut_prec_angle: PhaseAngle,
}

impl Encode for TrigAngle {
    fn encoded_len(&self) -> der::Result<der::Length> {
        OctetStringRef::new(self.as_bytes()).unwrap().encoded_len()

        // self.right_ascension_deg.encoded_len()?
        //     + self.declination_deg.encoded_len()?
        //     + self.prime_meridian_deg.encoded_len()?
        //     + self.nut_prec_angle.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        OctetStringRef::new(self.as_bytes())
            .unwrap()
            .encode(encoder)
        // self.right_ascension_deg.encode(encoder)?;
        // self.declination_deg.encode(encoder)?;
        // self.prime_meridian_deg.encode(encoder)?;
        // self.nut_prec_angle.encode(encoder)
    }
}

impl<'a> Decode<'a> for TrigAngle {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            right_ascension_deg: decoder.decode()?,
            declination_deg: decoder.decode()?,
            prime_meridian_deg: decoder.decode()?,
            nut_prec_angle: decoder.decode()?,
        })
    }
}
