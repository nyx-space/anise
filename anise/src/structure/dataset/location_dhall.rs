/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::NaifId;
use crate::{astro::Location, structure::LocationDataSet};
use serde::{Deserialize, Serialize};
use serde_dhall::StaticType;
use std::collections::BTreeMap;

#[cfg(feature = "python")]
use crate::file2heap;
#[cfg(feature = "python")]
use pyo3::exceptions::PyException;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyType;
#[cfg(feature = "python")]
use std::path::PathBuf;

use super::{DataSet, DataSetType};

/// Entry of a Location Dhall set
///
/// :type id: int, optional
/// :type alias: string, optional
/// :type value: Location
#[derive(Clone, Debug, StaticType, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
pub struct LocationDhallSetEntry {
    pub id: Option<NaifId>,
    pub alias: Option<String>,
    pub value: Location,
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl LocationDhallSetEntry {
    #[new]
    #[pyo3(signature=(value, id=None, alias=None))]
    fn py_new(value: Location, id: Option<NaifId>, alias: Option<String>) -> Self {
        Self { id, alias, value }
    }

    /// :rtype: int
    #[getter]
    fn get_id(&self) -> Option<NaifId> {
        self.id
    }
    /// :type id: int
    #[setter]
    fn set_id(&mut self, id: Option<NaifId>) {
        self.id = id;
    }
    /// :rtype: str
    #[getter]
    fn get_alias(&self) -> Option<String> {
        self.alias.clone()
    }
    /// :type alias: str
    #[setter]
    fn set_alias(&mut self, alias: Option<String>) {
        self.alias = alias;
    }
    /// :rtype: Location
    #[getter]
    fn get_value(&self) -> Location {
        self.value.clone()
    }
    /// :type value: Location
    #[setter]
    fn set_value(&mut self, value: Location) {
        self.value = value;
    }
}
/// A Dhall-serializable Location DataSet that serves as an optional intermediate to the LocationDataSet kernels.
///
/// :type data: list
#[derive(Clone, Debug, StaticType, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
pub struct LocationDhallSet {
    data: Vec<LocationDhallSetEntry>,
}

impl LocationDhallSet {
    /// Convert this Dhall representation of locations to a LocationDataSet kernel.
    ///
    /// Function is mutable because the terrain mask is sanitized prior to building the kernel.
    pub fn to_dataset(&mut self) -> Result<LocationDataSet, String> {
        let mut dataset = DataSet::default();
        dataset.metadata.dataset_type = DataSetType::LocationData;

        for e in &mut self.data {
            e.value.sanitize_mask();
            dataset
                .push(
                    e.value.clone(),
                    e.id,
                    match e.alias.as_ref() {
                        Some(s) => Some(s.as_str()),
                        None => None,
                    },
                )
                .map_err(|e| e.to_string())?;
        }

        Ok(dataset)
    }

    /// Deserialize the Dhall string of a Location data set into its Dhall representation structure.
    pub fn from_dhall(repr: &str) -> Result<Self, String> {
        let me: Self = serde_dhall::from_str(repr)
            .static_type_annotation()
            .parse()
            .map_err(|e| e.to_string())?;

        Ok(me)
    }

    /// Serializes to a Dhall string
    pub fn to_dhall(&self) -> Result<String, String> {
        serde_dhall::serialize(&self)
            .static_type_annotation()
            .to_string()
            .map_err(|e| e.to_string())
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl LocationDhallSet {
    #[new]
    fn py_new(data: Vec<LocationDhallSetEntry>) -> Self {
        Self { data }
    }

    /// :rtype: list
    #[getter]
    fn get_data(&self) -> Vec<LocationDhallSetEntry> {
        self.data.clone()
    }
    /// :type data: list
    #[setter]
    fn set_data(&mut self, data: Vec<LocationDhallSetEntry>) {
        self.data = data;
    }
    /// Returns the Dhall representation of this Location
    ///
    /// :rtype: str
    #[pyo3(name = "to_dhall")]
    fn py_to_dhall(&self) -> Result<String, PyErr> {
        self.to_dhall().map_err(PyException::new_err)
    }

    /// Loads thie Location dataset from its Dhall representation as a string
    ///
    /// :type repr: str
    /// :rtype: LocationDhallSet
    #[classmethod]
    #[pyo3(name = "from_dhall")]
    fn py_from_dhall(_cls: Bound<'_, PyType>, repr: &str) -> Result<Self, PyErr> {
        Self::from_dhall(repr).map_err(PyException::new_err)
    }

    /// Converts this location Dhall set into a Python-compatible Location DataSet.
    ///
    /// :rtype: LocationDataSet
    #[pyo3(name = "to_dataset")]
    fn py_to_dataset(&mut self) -> Result<PyLocationDataSet, PyErr> {
        Ok(PyLocationDataSet {
            inner: self
                .to_dataset()
                .map_err(|e| PyException::new_err(e.to_string()))?,
        })
    }
}

impl LocationDataSet {
    /// Converts a location dataset kernel into its Dhall representation struct
    pub fn to_dhallset(&self) -> Result<LocationDhallSet, String> {
        let mut many_me = BTreeMap::new();

        for (id, pos) in &self.lut.by_id {
            many_me.insert(
                pos,
                LocationDhallSetEntry {
                    id: Some(*id),
                    alias: None,
                    value: self.get_by_id(*id).unwrap(),
                },
            );
        }

        for (name, pos) in &self.lut.by_name {
            if let Some(entry) = many_me.get_mut(&pos) {
                entry.alias = Some(name.to_string());
            } else {
                many_me.insert(
                    pos,
                    LocationDhallSetEntry {
                        id: None,
                        alias: Some(name.clone()),
                        value: self.get_by_name(name).unwrap(),
                    },
                );
            }
        }

        // The BTreeMap ensures that everything is organized in the same way as in the dataset.
        let data = many_me
            .values()
            .cloned()
            .collect::<Vec<LocationDhallSetEntry>>();

        Ok(LocationDhallSet { data })
    }
}

/// A wrapper around a location dataset kernel (PyO3 does not handle type aliases).
/// Use this class to load and unload kernels. Manipulate using its LocationDhallSet representation.
#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
#[pyo3(name = "LocationDataSet")]
pub struct PyLocationDataSet {
    inner: LocationDataSet,
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl PyLocationDataSet {
    /// Loads a Location Dataset kernel from the provided path
    ///
    /// :type path: string
    /// :rtype: LocationDataSet
    #[classmethod]
    fn load(_cls: Bound<'_, PyType>, path: &str) -> Result<Self, PyErr> {
        let dataset = LocationDataSet::try_from_bytes(
            file2heap!(path).map_err(|e| PyException::new_err(e.to_string()))?,
        )
        .map_err(|e| PyException::new_err(e.to_string()))?;

        Ok(Self { inner: dataset })
    }

    /// Save this dataset as a kernel, optionally specifying whether to overwrite the existing file.
    ///
    /// :type path: string
    /// :type overwrite: bool, optional
    /// :rtype: None
    #[pyo3(signature=(path, overwrite=false))]
    fn save_as(&mut self, path: &str, overwrite: Option<bool>) -> Result<(), PyErr> {
        self.inner.set_crc32();
        self.inner
            .save_as(&PathBuf::from(path), overwrite.unwrap_or_default())
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    /// Converts this location dataset into a manipulable location Dhall set.
    ///
    /// :rtype: LocationDhallSet
    fn to_dhallset(&self) -> Result<LocationDhallSet, PyErr> {
        self.inner
            .to_dhallset()
            .map_err(|e| PyException::new_err(e.to_string()))
    }
}

#[cfg(test)]
mod ut_loc_dhall {

    use crate::{
        astro::{Location, TerrainMask},
        constants::frames::EARTH_ITRF93,
    };

    use super::{LocationDhallSet, LocationDhallSetEntry};

    #[test]
    fn test_location_dhallset() {
        let dss65 = Location {
            latitude_deg: 40.427,
            longitude_deg: 4.250,
            height_km: 0.834,
            frame: EARTH_ITRF93.into(),
            terrain_mask: vec![],
            terrain_mask_ignored: true,
        };
        let paris = Location {
            latitude_deg: 42.0,
            longitude_deg: 2.0,
            height_km: 0.4,
            frame: EARTH_ITRF93.into(),
            terrain_mask: vec![TerrainMask {
                azimuth_deg: 0.0,
                elevation_mask_deg: 15.9,
            }],
            terrain_mask_ignored: true,
        };

        let set = LocationDhallSet {
            data: vec![
                LocationDhallSetEntry {
                    id: Some(1),
                    alias: Some("DSS65".to_string()),
                    value: dss65,
                },
                LocationDhallSetEntry {
                    id: None,
                    alias: Some("Paris".to_string()),
                    value: paris,
                },
            ],
        };

        let as_dhall = set.to_dhall().unwrap();
        println!("{as_dhall}");

        let mut from_dhall = LocationDhallSet::from_dhall(&as_dhall).unwrap();

        assert_eq!(from_dhall, set);

        let to_dataset = from_dhall.to_dataset().unwrap();
        println!("{to_dataset}");
    }
}
