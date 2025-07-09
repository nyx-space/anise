/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

// FOCI: 1. Build the angle between two objects, defined in the loaded Almanac.
//       2. Rebuild the angular momentum vector to demonstrate the cross product.

use serde_derive::{Deserialize, Serialize};
use serde_dhall::{SimpleType, StaticType};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, StaticType)]
pub struct MetaFile {
    /// URI of this meta file
    pub uri: String,
    /// Optionally specify the CRC32 of this file, which will be checked prior to loading.
    pub crc32: Option<u32>,
}

// Temp: add static type to the current frameuid
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, StaticType)]
pub struct FrameUid {
    pub ephemeris_id: i32,
    pub orientation_id: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Vector {
    Fixed {
        x: f64,
        y: f64,
        z: f64,
    }, // Unitless vector, for arbitrary computations
    Position {
        from_frame: FrameUid,
        to_frame: FrameUid,
    },
    Velocity {
        from_frame: FrameUid,
        to_frame: FrameUid,
    },
    CrossProduct {
        a: Box<Vector>,
        b: Box<Vector>,
    },
}

use std::collections::HashMap;
// Manual implementation of StaticType for the recursive Vector enum.
impl StaticType for Vector {
    // This function defines the Dhall type that corresponds to our Rust type.
    fn static_type() -> SimpleType {
        let mut fields = HashMap::new();
        fields.insert(
            "Fixed".to_string(),
            // The type for the `Fixed` variant is a record.
            Some(SimpleType::Record(
                [
                    ("x".to_string(), SimpleType::Natural), // Using Natural as a stand-in for f64
                    ("y".to_string(), SimpleType::Natural),
                    ("z".to_string(), SimpleType::Natural),
                ]
                .iter()
                .cloned()
                .collect(),
            )),
        );
        fields.insert(
            "Position".to_string(),
            Some(SimpleType::Record(
                [
                    ("from_frame".to_string(), FrameUid::static_type()),
                    ("to_frame".to_string(), FrameUid::static_type()),
                ]
                .iter()
                .cloned()
                .collect(),
            )),
        );
        fields.insert(
            "Velocity".to_string(),
            Some(SimpleType::Record(
                [
                    ("from_frame".to_string(), FrameUid::static_type()),
                    ("to_frame".to_string(), FrameUid::static_type()),
                ]
                .iter()
                .cloned()
                .collect(),
            )),
        );
        fields.insert(
            "CrossProduct".to_string(),
            Some(SimpleType::Record(
                [
                    // HERE is the magic: instead of calling Vector::static_type()
                    // and causing infinite recursion, we use the placeholder.
                    ("a".to_string(), recursive_type.clone()),
                    ("b".to_string(), recursive_type.clone()),
                ]
                .iter()
                .cloned()
                .collect(),
            )),
        );
        SimpleType::Union(fields)
    }
}
