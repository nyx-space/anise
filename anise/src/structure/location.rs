/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use core::fmt;
use der::{asn1::OctetStringRef, Decode, Encode, Error, ErrorKind, Length, Reader, Writer};

use crate::frames::FrameUid;

#[cfg(feature = "python")]
use pyo3::prelude::*;
/// Location is defined by its latitude, longitude, height above the geoid, mean angular rotation of the geoid, and a frame UID.
/// If the location includes a terrain mask, it will be used for obstruction checks when computing azimuth and elevation.
/// **Note:** The mean Earth angular velocity is `0.004178079012116429` deg/s.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct Location {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub height_km: f64,
    /// Mean angular rotation rate of the celestial object where this location lives.
    pub mean_angular_velocity_deg_s: f64,
    pub frame: FrameUid,
    pub mask: Vec<TerrainMask>,
}

/// TerrainMask is used to compute obstructions during AER calculations.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct TerrainMask {
    /// Azimuth where this elevation mask starts.
    pub start_azimuth_deg: f64,
    pub elevation_mask_deg: f64,
}

impl TerrainMask {
    /// Initializes a new flat terrain mask.
    pub fn from_flat_terrain(elevation_mask_deg: f64) -> Self {
        Self {
            start_azimuth_deg: 0.0,
            elevation_mask_deg,
        }
    }
}

/* impl Encode for Semver {
    fn encoded_len(&self) -> der::Result<der::Length> {
        let data: [u8; 3] = [self.major, self.minor, self.patch];
        let as_octet_string = OctetStringRef::new(&data).unwrap();
        as_octet_string.encoded_len()
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        let data: [u8; 3] = [self.major, self.minor, self.patch];
        let as_octet_string = OctetStringRef::new(&data).unwrap();
        as_octet_string.encode(encoder)
    }
}

impl<'a> Decode<'a> for Semver {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        let data: OctetStringRef = decoder.decode()?;
        if data.len() != Length::new(3) {
            return Err(Error::new(
                ErrorKind::Incomplete {
                    expected_len: Length::new(3),
                    actual_len: data.len(),
                },
                Length::new(0),
            ));
        }

        Ok(Self {
            major: data.as_bytes()[0],
            minor: data.as_bytes()[1],
            patch: data.as_bytes()[2],
        })
    }
}

impl fmt::Display for Semver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "ANISE version {}.{}.{}",
            self.major, self.minor, self.patch
        )
    }
} */
