/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use der::{Decode, Encode, Reader, Writer};

use crate::NaifId;

use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "analysis")]
use serde_dhall::StaticType;

#[cfg(feature = "python")]
use pyo3::prelude::*;

use super::dataset::DataSetT;

/// Location is defined by its latitude, longitude, height above the geoid, mean angular rotation of the geoid, and a frame UID.
/// If the location includes a terrain mask, it will be used for obstruction checks when computing azimuth and elevation.
/// **Note:** The mean Earth angular velocity is `0.004178079012116429` deg/s.
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "analysis", derive(StaticType))]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct Location {
    pub loc_latitude_deg: f64,
    pub loc_longitude_deg: f64,
    pub loc_height_km: f64,
    /// ephemeris ID of the celestial object on which this location tests
    pub frame_ephemeris_id: NaifId,
    /// orientation ID of the body-fixed frame on which this location rests
    pub frame_orientation_id: NaifId,
    /// Mask due to the terrain; vector is assumed to be pre-sorted by azimuth (or the mask will not work correctly)
    pub terrain_mask: Vec<TerrainMask>,
}

impl Location {
    pub fn elevation_mask_from_azimuth_deg(&self, azimuth_deg: f64) -> f64 {
        let idx = self
            .terrain_mask
            .partition_point(|mask| mask.azimuth_deg < azimuth_deg.rem_euclid(360.0));
        self.terrain_mask
            .get(idx)
            .map_or(0.0, |mask| mask.elevation_mask_deg)
    }
}

impl DataSetT for Location {
    const NAME: &'static str = "location data";
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

impl Encode for Location {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.loc_latitude_deg.encoded_len()?
            + self.loc_longitude_deg.encoded_len()?
            + self.loc_height_km.encoded_len()?
            + self.frame_ephemeris_id.encoded_len()?
            + self.frame_orientation_id.encoded_len()?
            + self.terrain_mask.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.loc_latitude_deg.encode(encoder)?;
        self.loc_longitude_deg.encode(encoder)?;
        self.loc_height_km.encode(encoder)?;
        self.frame_ephemeris_id.encode(encoder)?;
        self.frame_orientation_id.encode(encoder)?;
        self.terrain_mask.encode(encoder)
    }
}

impl<'a> Decode<'a> for Location {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            loc_latitude_deg: decoder.decode()?,
            loc_longitude_deg: decoder.decode()?,
            loc_height_km: decoder.decode()?,
            frame_ephemeris_id: decoder.decode()?,
            frame_orientation_id: decoder.decode()?,
            terrain_mask: decoder.decode()?,
        })
    }
}

#[cfg(test)]
mod ut_loc {
    use super::Location;
    use super::{Decode, Encode};
    use crate::constants::celestial_objects::EARTH;
    use crate::constants::orientations::ITRF93;

    #[cfg(feature = "analysis")]
    #[test]
    fn test_location() {
        use serde_dhall::{from_str, serialize};

        use crate::structure::location::TerrainMask;

        let dss65 = Location {
            loc_latitude_deg: 40.427_222,
            loc_longitude_deg: 4.250_556,
            loc_height_km: 0.834_939,
            frame_ephemeris_id: EARTH,
            frame_orientation_id: ITRF93,
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

        // Test Dhall serde
        let as_dhall = serialize(&dss65)
            .static_type_annotation()
            .to_string()
            .unwrap();

        let from_dhall: Location = from_str(&as_dhall)
            .static_type_annotation()
            .parse()
            .unwrap();

        assert_eq!(from_dhall, dss65);

        println!("{as_dhall}");
        // Test ASN.1 serde
        let mut buf = vec![];
        dss65.encode_to_vec(&mut buf).unwrap();
        dbg!(buf.len());

        let dss65_dec = Location::from_der(&buf).unwrap();

        assert_eq!(dss65, dss65_dec);

        println!("{}", dss65.elevation_mask_from_azimuth_deg(0.1));
        dbg!(34.0_f64.rem_euclid(360.0));

        assert!((dss65.elevation_mask_from_azimuth_deg(0.0) - 5.0).abs() < f64::EPSILON);
        assert!((dss65.elevation_mask_from_azimuth_deg(34.0) - 5.0).abs() < f64::EPSILON);
        assert!((dss65.elevation_mask_from_azimuth_deg(35.0) - 15.0).abs() < f64::EPSILON);
        assert!((dss65.elevation_mask_from_azimuth_deg(270.0) - 3.0).abs() < f64::EPSILON);
        assert!((dss65.elevation_mask_from_azimuth_deg(359.0) - 3.0).abs() < f64::EPSILON);
        assert!((dss65.elevation_mask_from_azimuth_deg(361.0) - 5.0).abs() < f64::EPSILON);
    }
}
