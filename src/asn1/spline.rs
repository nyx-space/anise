/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{asn1::OctetStringRef, Decode, Encode, Length, Reader, Writer};

use super::{splinecoeffs::SplineCoeffCount, splinekind::SplineKind};

/// Maximum interpolation degree for splines. This is needed for encoding and decoding of Splines in ASN1 using the `der` library.
pub const MAX_INTERP_DEGREE: usize = 32;

// #[derive(Enumerated)]
// #[repr(u8)]
// pub enum TrunctationStrategy {
//     None = 0,
//     TruncateLow = 1,
//     TruncateHigh = 2,
// }

// WARNING: How do I specify the start and end epochs for variable sized windows where the duration in the window is needed to rebuild the state?
// Is that some kind of header? If so, what's its size? If it's a high precision epoch, it would be 80 bits, but more likely people will provide 64 bit floats.
// Also, I can't use an offset from the index because the splines are built separately from the index via multithreading, so that would be difficult to build (would need to mutate the spline prior to encoding)

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Splines<'a> {
    pub kind: SplineKind,
    pub config: SplineCoeffCount,
    /// Store the CRC32 checksum of the stored data. This should be checked prior to interpreting the data in the spline.
    pub data_checksum: u32,
    // TODO: Figure out how to properly add the covariance info, it's a bit hard because of the diag size
    // pub cov_position_coeff_len: u8,
    // pub cov_velocity_coeff_len: u8,
    // pub cov_acceleration_coeff_len: u8,
    pub data: &'a [u8],
}

impl<'a> Encode for Splines<'a> {
    fn encoded_len(&self) -> der::Result<Length> {
        self.kind.encoded_len()?
            + self.config.encoded_len()?
            + self.data_checksum.encoded_len()?
            + OctetStringRef::new(self.data).unwrap().encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.kind.encode(encoder)?;
        self.config.encode(encoder)?;
        self.data_checksum.encode(encoder)?;
        OctetStringRef::new(self.data).unwrap().encode(encoder)
    }
}

impl<'a> Decode<'a> for Splines<'a> {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let kind = decoder.decode()?;
        let config = decoder.decode()?;
        let data_checksum = decoder.decode()?;
        let data_bytes: OctetStringRef = decoder.decode()?;
        Ok(Self {
            kind,
            config,
            data_checksum,
            data: data_bytes.as_bytes(),
        })
    }
}
