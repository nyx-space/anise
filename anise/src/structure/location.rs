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
use der::{Decode, Encode, Reader, Writer};

use crate::NaifId;

use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "analysis")]
use serde_dhall::StaticType;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Location is defined by its latitude, longitude, height above the geoid, mean angular rotation of the geoid, and a frame UID.
/// If the location includes a terrain mask, it will be used for obstruction checks when computing azimuth and elevation.
/// **Note:** The mean Earth angular velocity is `0.004178079012116429` deg/s.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "analysis", derive(StaticType))]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct Location {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub height_km: f64,
    /// ephemeris ID of the celestial object on which this location tests
    pub ephemeris_id: NaifId,
    /// orientation ID of the body-fixed frame on which this location rests
    pub orientation_id: NaifId,
    /// Mask due to the terrain; vector is assumed to be pre-sorted by azimuth (or the mask will not work correctly)
    pub terrain_mask: Vec<TerrainMask>,
}

/// TerrainMask is used to compute obstructions during AER calculations.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "analysis", derive(StaticType))]
pub struct TerrainMask {
    /// Azimuth where this elevation mask starts.
    pub azimuth_deg: f64,
    pub elevation_mask_deg: f64,
}

impl TerrainMask {
    /// Initializes a new flat terrain mask.
    pub fn from_flat_terrain(elevation_mask_deg: f64) -> Self {
        Self {
            azimuth_deg: 0.0,
            elevation_mask_deg,
        }
    }
}

impl Encode for TerrainMask {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.azimuth_deg.encoded_len()? + self.elevation_mask_deg.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.azimuth_deg.encode(encoder)?;
        self.elevation_mask_deg.encode(encoder)
    }
}

impl<'a> Decode<'a> for TerrainMask {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            azimuth_deg: decoder.decode()?,
            elevation_mask_deg: decoder.decode()?,
        })
    }
}

#[cfg(test)]
mod ut_loc {
    use super::Location;
    use crate::constants::celestial_objects::EARTH;
    use crate::constants::orientations::ITRF93;
    #[cfg(feature = "analysis")]
    #[test]
    fn test_dhall_location() {
        use serde_dhall::serialize;

        use crate::structure::location::TerrainMask;

        let dss65 = Location {
            latitude_deg: 40.427_222,
            longitude_deg: 4.250_556,
            height_km: 0.834_939,
            ephemeris_id: EARTH,
            orientation_id: ITRF93,
            terrain_mask: vec![
                TerrainMask {
                    azimuth_deg: 0.0,
                    elevation_mask_deg: 5.0,
                },
                TerrainMask {
                    azimuth_deg: 35.0,
                    elevation_mask_deg: 10.0,
                },
                TerrainMask {
                    azimuth_deg: 270.0,
                    elevation_mask_deg: 3.0,
                },
            ],
        };

        println!(
            "{}",
            serialize(&dss65)
                .static_type_annotation()
                .to_string()
                .unwrap()
        );
    }
}
