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

use crate::frames::FrameUid;

use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "analysis")]
use serde_dhall::StaticType;

#[cfg(feature = "python")]
use pyo3::exceptions::PyException;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyType;

use super::dataset::DataSetT;

/// Location is defined by its latitude, longitude, height above the geoid, mean angular rotation of the geoid, and a frame UID.
/// If the location includes a terrain mask, it will be used for obstruction checks when computing azimuth and elevation.
/// **Note:** The mean Earth angular velocity is `0.004178079012116429` deg/s.
///
///
/// :type latitude_deg: float
/// :type longitude_deg: float
/// :type height_km: float
/// :type frame: FrameUid
/// :type terrain_mask: list
/// :type terrain_mask_ignored: bool
///
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "analysis", derive(StaticType))]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct Location {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub height_km: f64,
    /// Frame on which this location rests
    pub frame: FrameUid,
    /// Mask due to the terrain; vector is assumed to be pre-sorted by azimuth (or the mask will not work correctly)
    pub terrain_mask: Vec<TerrainMask>,
    /// If set to True and the terrain mask hides the object, then the AER computation will still return the AER data instead of NaNs.
    pub terrain_mask_ignored: bool,
}

#[cfg(feature = "analysis")]
impl Location {
    /// Rebuild a Location from its Dhall representation
    pub fn from_dhall(repr: &str) -> Result<Self, String> {
        serde_dhall::from_str(repr)
            .static_type_annotation()
            .parse::<Self>()
            .map_err(|e| e.to_string())
    }

    /// Returns the Dhall representation of this Location
    pub fn to_dhall(&self) -> Result<String, String> {
        serde_dhall::serialize(&self)
            .static_type_annotation()
            .to_string()
            .map_err(|e| e.to_string())
    }

    /// Ensures that the terrain mask is ordered by azimuth, and remove duplicate azimuths
    pub fn sanitize_mask(&mut self) {
        use std::cmp::Ordering;
        self.terrain_mask.sort_by(|mask1, mask2| {
            mask1
                .azimuth_deg
                .partial_cmp(&mask2.azimuth_deg)
                .unwrap_or(Ordering::Less)
        });
        self.terrain_mask.dedup_by_key(|mask| mask.azimuth_deg);
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Location {
    /// Returns the elevation mask at the provided azimuth, does NOT account for whether the mask is ignored or not.
    ///
    /// :type azimuth_deg: float
    /// :rtype: float
    pub fn elevation_mask_at_azimuth_deg(&self, azimuth_deg: f64) -> f64 {
        if self.terrain_mask.is_empty() {
            return 0.0;
        }
        let idx = self
            .terrain_mask
            .partition_point(|mask| mask.azimuth_deg <= azimuth_deg.rem_euclid(360.0));
        if idx == 0 {
            return self
                .terrain_mask
                .last()
                .map_or(0.0, |mask| mask.elevation_mask_deg);
        }
        self.terrain_mask
            .get(idx - 1)
            .map_or(0.0, |mask| mask.elevation_mask_deg)
    }

    /// Returns the Dhall representation of this Location
    #[cfg(feature = "python")]
    #[pyo3(name = "to_dhall")]
    pub fn py_to_dhall(&self) -> Result<String, PyErr> {
        self.to_dhall().map_err(PyException::new_err)
    }

    #[cfg(feature = "python")]
    #[classmethod]
    #[pyo3(name = "from_dhall", signature=(repr))]
    fn py_from_dhall(_cls: Bound<'_, PyType>, repr: &str) -> Result<Self, PyErr> {
        Self::from_dhall(repr).map_err(PyException::new_err)
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl Location {
    #[new]
    fn py_new(
        latitude_deg: f64,
        longitude_deg: f64,
        height_km: f64,
        frame: FrameUid,
        terrain_mask: Vec<TerrainMask>,
        terrain_mask_ignored: bool,
    ) -> Self {
        Self {
            latitude_deg,
            longitude_deg,
            height_km,
            frame,
            terrain_mask,
            terrain_mask_ignored,
        }
    }

    /// :rtype: float
    #[getter]
    fn get_latitude_deg(&self) -> f64 {
        self.latitude_deg
    }
    /// :type latitude_deg: float
    #[setter]
    fn set_latitude_deg(&mut self, latitude_deg: f64) {
        self.latitude_deg = latitude_deg;
    }
    /// :rtype: float
    #[getter]
    fn get_longitude_deg(&self) -> f64 {
        self.longitude_deg
    }
    /// :type longitude_deg: float
    #[setter]
    fn set_longitude_deg(&mut self, longitude_deg: f64) {
        self.longitude_deg = longitude_deg
    }
    /// :rtype: float
    #[getter]
    fn get_height_km(&self) -> f64 {
        self.height_km
    }
    /// :type altitude_km: float
    #[setter]
    fn set_height_km(&mut self, height_km: f64) {
        self.height_km = height_km;
    }
    /// :rtype: list
    #[getter]
    fn get_terrain_mask(&self) -> Vec<TerrainMask> {
        self.terrain_mask.clone()
    }
    /// :type terrain_mask: list
    #[setter]
    fn set_terrain_mask(&mut self, terrain_mask: Vec<TerrainMask>) {
        self.terrain_mask = terrain_mask;
    }
    /// :rtype: bool
    #[getter]
    fn get_terrain_mask_ignored(&self) -> bool {
        self.terrain_mask_ignored
    }
    /// :type terrain_mask_ignored: bool
    #[setter]
    fn set_terrain_mask_ignored(&mut self, terrain_mask_ignored: bool) {
        self.terrain_mask_ignored = terrain_mask_ignored;
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

impl DataSetT for Location {
    const NAME: &'static str = "location data";
}

/// TerrainMask is used to compute obstructions during AER calculations.
///
/// :type azimuth_deg: float
/// :type elevation_mask_deg: float
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "analysis", derive(StaticType))]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
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

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl TerrainMask {
    #[getter]
    fn get_azimuth_deg(&self) -> f64 {
        self.azimuth_deg
    }

    #[setter]
    fn set_azimuth_deg(&mut self, azimuth_deg: f64) {
        self.azimuth_deg = azimuth_deg;
    }

    #[getter]
    fn get_elevation_mask_deg(&self) -> f64 {
        self.elevation_mask_deg
    }

    #[setter]
    fn set_elevation_mask_deg(&mut self, elevation_mask_deg: f64) {
        self.elevation_mask_deg = elevation_mask_deg;
    }

    #[new]
    fn py_new(azimuth_deg: f64, elevation_mask_deg: f64) -> Self {
        Self {
            azimuth_deg,
            elevation_mask_deg,
        }
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }

    fn __repr__(&self) -> String {
        format!("{self:?} @ {self:p}")
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
        self.latitude_deg.encoded_len()?
            + self.longitude_deg.encoded_len()?
            + self.height_km.encoded_len()?
            + self.frame.encoded_len()?
            + self.terrain_mask.encoded_len()?
            + self.terrain_mask_ignored.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.latitude_deg.encode(encoder)?;
        self.longitude_deg.encode(encoder)?;
        self.height_km.encode(encoder)?;
        self.frame.encode(encoder)?;
        self.terrain_mask.encode(encoder)?;
        self.terrain_mask_ignored.encode(encoder)
    }
}

impl<'a> Decode<'a> for Location {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            latitude_deg: decoder.decode()?,
            longitude_deg: decoder.decode()?,
            height_km: decoder.decode()?,
            frame: decoder.decode()?,
            terrain_mask: decoder.decode()?,
            terrain_mask_ignored: decoder.decode()?,
        })
    }
}

#[cfg(feature = "analysis")]
#[cfg(test)]
mod ut_loc {
    use super::Location;
    use super::{Decode, Encode};

    #[test]
    fn test_location() {
        use crate::{constants::frames::EARTH_ITRF93, structure::location::TerrainMask};

        let dss65 = Location {
            latitude_deg: 40.427_222,
            longitude_deg: 4.250_556,
            height_km: 0.834_939,
            frame: EARTH_ITRF93.into(),
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
            terrain_mask_ignored: false,
        };

        // Test Dhall serde
        let as_dhall = dss65.to_dhall().unwrap();

        let from_dhall = Location::from_dhall(&as_dhall).unwrap();
        assert_eq!(from_dhall, dss65);

        println!("{as_dhall}");
        // Test ASN.1 serde
        let mut buf = vec![];
        dss65.encode_to_vec(&mut buf).unwrap();
        dbg!(buf.len());

        let dss65_dec = Location::from_der(&buf).unwrap();

        assert_eq!(dss65, dss65_dec);

        assert!((dss65.elevation_mask_at_azimuth_deg(0.0) - 5.0).abs() < f64::EPSILON);
        assert!((dss65.elevation_mask_at_azimuth_deg(34.0) - 5.0).abs() < f64::EPSILON);
        assert!((dss65.elevation_mask_at_azimuth_deg(35.0) - 10.0).abs() < f64::EPSILON);
        assert!((dss65.elevation_mask_at_azimuth_deg(270.0) - 3.0).abs() < f64::EPSILON);
        assert!((dss65.elevation_mask_at_azimuth_deg(359.0) - 3.0).abs() < f64::EPSILON);
        // Check azimuth over 360 wraps around
        assert!((dss65.elevation_mask_at_azimuth_deg(361.0) - 5.0).abs() < f64::EPSILON);
        // Check azimuth below 0 wraps around
        assert!((dss65.elevation_mask_at_azimuth_deg(-1.0) - 3.0).abs() < f64::EPSILON);
    }
}
