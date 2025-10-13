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

use super::DataSet;

#[derive(Clone, StaticType, Serialize, Deserialize)]
pub struct LocationDhallSet {
    pub key: (Option<NaifId>, Option<String>),
    pub value: Location,
}

impl LocationDhallSet {
    pub fn from_dhall(repr: &str) -> Result<LocationDataSet, String> {
        let many_me: Vec<Self> = serde_dhall::from_str(repr)
            .static_type_annotation()
            .parse()
            .map_err(|e| e.to_string())?;

        let mut dataset = DataSet::default();

        for me in &many_me {
            dataset
                .push(
                    me.value.clone(),
                    me.key.0,
                    match me.key.1.as_ref() {
                        Some(s) => Some(s.as_str()),
                        None => None,
                    },
                )
                .map_err(|e| e.to_string())?;
        }

        Ok(dataset)
    }
}

impl LocationDataSet {
    pub fn to_dhall(&self) -> Result<String, String> {
        let mut many_me = BTreeMap::new();

        for (id, pos) in &self.lut.by_id {
            many_me.insert(
                pos,
                LocationDhallSet {
                    key: (Some(*id), None),
                    value: self.get_by_id(*id).unwrap(),
                },
            );
        }

        for (name, pos) in &self.lut.by_name {
            if let Some(entry) = many_me.get_mut(&pos) {
                entry.key.1 = Some(name.to_string());
            } else {
                many_me.insert(
                    pos,
                    LocationDhallSet {
                        key: (None, Some(name.clone())),
                        value: self.get_by_name(name).unwrap(),
                    },
                );
            }
        }

        // The BTreeMap ensures that everything is organized in the same way as in the dataset.
        let many_me_vec = many_me.values().cloned().collect::<Vec<LocationDhallSet>>();

        serde_dhall::serialize(&many_me_vec)
            .static_type_annotation()
            .to_string()
            .map_err(|e| e.to_string())
    }
}
