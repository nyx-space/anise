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

/// Only the Triaxial Ellipsoid shape model is currently supported by ANISE.
/// This is directly inspired from SPICE PCK.
/// > For each body, three radii are listed: The first number is
/// > the largest equatorial radius (the length of the semi-axis
/// > containing the prime meridian), the second number is the smaller
/// > equatorial radius, and the third is the polar radius.
///
/// Example: Radii of the Earth.
///
///    BODY399_RADII     = ( 6378.1366   6378.1366   6356.7519 )
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TriaxialEllipsoid {
    pub largest_equatorial_radius_km: Option<f64>,
    pub smallest_equatorial_radius_km: Option<f64>,
    pub polar_radius_km: Option<f64>,
}

impl Encode for TriaxialEllipsoid {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.largest_equatorial_radius_km.encoded_len()?
            + self.smallest_equatorial_radius_km.encoded_len()?
            + self.polar_radius_km.encoded_len()?
    }

    fn encode(&self, encoder: &mut dyn Writer) -> der::Result<()> {
        self.largest_equatorial_radius_km.encode(encoder)?;
        self.smallest_equatorial_radius_km.encode(encoder)?;
        self.polar_radius_km.encode(encoder)
    }
}

impl<'a> Decode<'a> for TriaxialEllipsoid {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            largest_equatorial_radius_km: decoder.decode()?,
            smallest_equatorial_radius_km: decoder.decode()?,
            polar_radius_km: decoder.decode()?,
        })
    }
}
