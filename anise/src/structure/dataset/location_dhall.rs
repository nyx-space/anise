/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::collections::BTreeMap;

use crate::NaifId;
use crate::{astro::Location, structure::LocationDataSet};
use serde::{Deserialize, Serialize};
use serde_dhall::StaticType;

use super::{DataSet, DataSetType};

#[derive(Clone, Debug, StaticType, Serialize, Deserialize, PartialEq)]
pub struct LocationDhallSetEntry {
    pub id: Option<NaifId>,
    pub alias: Option<String>,
    pub value: Location,
}

#[derive(Clone, Debug, StaticType, Serialize, Deserialize, PartialEq)]
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
