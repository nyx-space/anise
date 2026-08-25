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
use crate::structure::SpacecraftDataSet;
use crate::structure::spacecraft::SpacecraftData;
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

/// Entry of a Spacecraft Dhall set
///
/// :type id: int, optional
/// :type alias: str, optional
/// :type value: SpacecraftData
#[derive(Clone, Debug, StaticType, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "python", pyclass(from_py_object, get_all, set_all))]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
pub struct SpacecraftDhallSetEntry {
    pub id: Option<NaifId>,
    pub name: Option<String>,
    pub value: SpacecraftData,
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl SpacecraftDhallSetEntry {
    #[new]
    #[pyo3(signature=(value, id=None, name=None))]
    fn py_new(value: SpacecraftData, id: Option<NaifId>, name: Option<String>) -> Self {
        Self { id, name, value }
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
}
/// A Dhall-serializable Spacecraft DhallSet that serves as an optional intermediate to the SpacecraftDataSet kernels.
///
/// :type data: list
#[derive(Clone, Debug, StaticType, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "python", pyclass(from_py_object))]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
pub struct SpacecraftDhallSet {
    data: Vec<SpacecraftDhallSetEntry>,
}

impl SpacecraftDhallSet {
    /// Convert this Dhall representation of a spacecraft dhallset to a SpacecraftDataSet kernel.
    pub fn to_dataset(&self) -> Result<SpacecraftDataSet, String> {
        let mut dataset = DataSet::default();
        dataset.metadata.dataset_type = DataSetType::SpacecraftData;

        for e in &self.data {
            dataset
                .push(
                    e.value,
                    e.id,
                    match e.name.as_ref() {
                        Some(s) => Some(s.as_str()),
                        None => None,
                    },
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(dataset)
    }

    /// Deserialize the Dhall string of a Spacecraft data set into its Dhall representation structure.
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
impl SpacecraftDhallSet {
    #[new]
    fn py_new(data: Vec<SpacecraftDhallSetEntry>) -> Self {
        Self { data }
    }

    /// :rtype: list
    #[getter]
    fn get_data(&self) -> Vec<SpacecraftDhallSetEntry> {
        self.data.clone()
    }
    /// :type data: list
    #[setter]
    fn set_data(&mut self, data: Vec<SpacecraftDhallSetEntry>) {
        self.data = data;
    }

    /// Loads this Spacecraft dataset from its Dhall representation as a string. Equivalent to from_dhall.
    ///
    /// :type repr: str
    /// :rtype: SpacecraftDhallSet
    #[classmethod]
    fn loads(_cls: &Bound<'_, PyType>, repr: &str) -> Result<Self, PyErr> {
        Self::from_dhall(repr).map_err(PyException::new_err)
    }

    /// Returns the Dhall representation of this SpacecraftDhallSet. Equivalent to to_dhall.
    ///
    /// :rtype: str
    fn dumps(&self) -> Result<String, PyErr> {
        self.to_dhall().map_err(PyException::new_err)
    }

    /// Returns the Dhall representation of this Spacecraft
    ///
    /// :rtype: str
    #[pyo3(name = "to_dhall")]
    fn py_to_dhall(&self) -> Result<String, PyErr> {
        self.to_dhall().map_err(PyException::new_err)
    }

    /// Loads this Spacecraft dataset from its Dhall representation as a string
    ///
    /// :type repr: str
    /// :rtype: SpacecraftDhallSet
    #[classmethod]
    #[pyo3(name = "from_dhall")]
    fn py_from_dhall(_cls: Bound<'_, PyType>, repr: &str) -> Result<Self, PyErr> {
        Self::from_dhall(repr).map_err(PyException::new_err)
    }

    /// Converts this location Dhall set into a Python-compatible Spacecraft DataSet.
    ///
    /// :rtype: SpacecraftDataSet
    #[pyo3(name = "to_dataset")]
    fn py_to_dataset(&mut self) -> Result<PySpacecraftDataSet, PyErr> {
        Ok(PySpacecraftDataSet {
            inner: self
                .to_dataset()
                .map_err(|e| PyException::new_err(e.to_string()))?,
        })
    }
}

impl SpacecraftDataSet {
    /// Converts a location dataset kernel into its Dhall representation struct
    pub fn to_dhallset(&self) -> Result<SpacecraftDhallSet, String> {
        let mut many_me = BTreeMap::new();

        for (id, pos) in &self.lut.by_id {
            many_me.insert(
                pos,
                SpacecraftDhallSetEntry {
                    id: Some(*id),
                    name: None,
                    value: self.get_by_id(*id).map_err(|e| e.to_string())?,
                },
            );
        }

        for (name, pos) in &self.lut.by_name {
            if let Some(entry) = many_me.get_mut(&pos) {
                entry.name = Some(name.to_string());
            } else {
                many_me.insert(
                    pos,
                    SpacecraftDhallSetEntry {
                        id: None,
                        name: Some(name.clone()),
                        value: self.get_by_name(name).map_err(|e| e.to_string())?,
                    },
                );
            }
        }

        // The BTreeMap ensures that everything is organized in the same way as in the dataset.
        let data = many_me
            .values()
            .cloned()
            .collect::<Vec<SpacecraftDhallSetEntry>>();

        Ok(SpacecraftDhallSet { data })
    }
}

/// A wrapper around a location dataset kernel (PyO3 does not handle type aliases).
/// Use this class to load and unload kernels. Manipulate using its SpacecraftDhallSet representation.
#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
#[pyo3(name = "SpacecraftDataSet")]
pub struct PySpacecraftDataSet {
    inner: SpacecraftDataSet,
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl PySpacecraftDataSet {
    /// Loads a Spacecraft Dataset kernel from the provided path
    ///
    /// :type path: str
    /// :rtype: SpacecraftDataSet
    #[classmethod]
    fn load(_cls: Bound<'_, PyType>, path: &str) -> Result<Self, PyErr> {
        let dataset = SpacecraftDataSet::try_from_bytes(
            file2heap!(path).map_err(|e| PyException::new_err(e.to_string()))?,
        )
        .map_err(|e| PyException::new_err(e.to_string()))?;

        Ok(Self { inner: dataset })
    }

    /// Save this dataset as a kernel, optionally specifying whether to overwrite the existing file.
    ///
    /// :type path: str
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
    /// :rtype: SpacecraftDhallSet
    fn to_dhallset(&self) -> Result<SpacecraftDhallSet, PyErr> {
        self.inner
            .to_dhallset()
            .map_err(|e| PyException::new_err(e.to_string()))
    }
}

#[cfg(test)]
mod ut_sc_dhall {

    use crate::structure::spacecraft::{DragData, Inertia, Mass, SRPData, SpacecraftData};

    use super::{SpacecraftDhallSet, SpacecraftDhallSetEntry};

    #[test]
    fn test_spacecraft_dhallset() {
        let inertia = Inertia {
            orientation_id: -159,
            i_xx_kgm2: 15.0,
            i_yy_kgm2: 16.0,
            i_zz_kgm2: 17.0,
            ..Default::default()
        };
        let srp = SRPData {
            area_m2: 23.4,
            coeff_reflectivity: 1.23,
        };
        let drag = DragData {
            area_m2: 12.3,
            coeff_drag: 1.01,
        };
        let mass = Mass {
            dry_mass_kg: 15.9,
            prop_mass_kg: 75.3,
            extra_mass_kg: 45.6,
        };

        let all_set = SpacecraftData {
            mass: Some(mass),
            srp_data: Some(srp),
            drag_data: Some(drag),
            inertia: Some(inertia),
        };

        let no_inertia = SpacecraftData {
            mass: Some(mass),
            srp_data: Some(srp),
            drag_data: Some(drag),
            inertia: None,
        };

        let deep_space = SpacecraftData {
            mass: Some(mass),
            srp_data: Some(srp),
            drag_data: None,
            inertia: None,
        };

        let mass_only = SpacecraftData {
            mass: Some(mass),
            srp_data: None,
            drag_data: None,
            inertia: None,
        };

        let set = SpacecraftDhallSet {
            data: vec![
                SpacecraftDhallSetEntry {
                    id: Some(1),
                    name: Some("all".to_string()),
                    value: all_set,
                },
                SpacecraftDhallSetEntry {
                    id: None,
                    name: Some("3dof".to_string()),
                    value: no_inertia,
                },
                SpacecraftDhallSetEntry {
                    id: None,
                    name: Some("deep space".to_string()),
                    value: deep_space,
                },
                SpacecraftDhallSetEntry {
                    id: None,
                    name: Some("mass".to_string()),
                    value: mass_only,
                },
            ],
        };

        let as_dhall = set.to_dhall().unwrap();
        println!("{as_dhall}");

        let from_dhall = SpacecraftDhallSet::from_dhall(&as_dhall).unwrap();

        assert_eq!(from_dhall, set);

        let to_dataset = from_dhall.to_dataset().unwrap();
        println!("{to_dataset}");

        assert!(!to_dataset.lut.is_empty());
    }
}
