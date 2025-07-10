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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum VectorExpr {
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
        a: Box<VectorExpr>,
        b: Box<VectorExpr>,
    },
}

use std::collections::HashMap;
// Manual implementation of StaticType for the recursive Vector enum.
#[allow(unconditional_recursion)]
impl StaticType for VectorExpr {
    // This function defines the Dhall type that corresponds to our Rust type
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
                    ("a".to_string(), Self::static_type()),
                    ("b".to_string(), Self::static_type()),
                ]
                .iter()
                .cloned()
                .collect(),
            )),
        );
        SimpleType::Union(fields)
    }
}

#[cfg(test)]
mod ut_vector_dhall {

    use crate::{
        analysis::{FrameUid, VectorExpr},
        errors::VelocitySnafu,
    };

    #[test]
    fn test_vector_expr_fixed() {
        let v = VectorExpr::Fixed {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let v_str = serde_dhall::serialize(&v).to_string().unwrap();
        println!("{v_str:?}");
        let v_deser: VectorExpr = serde_dhall::from_str(&v_str).parse().unwrap();
        assert_eq!(v_deser, v); // This fails because I am not serializing it correctly.
    }

    #[test]
    fn test_vector_expr_state() {
        let pos = VectorExpr::Position {
            from_frame: FrameUid {
                ephemeris_id: 399,
                orientation_id: 0,
            },
            to_frame: FrameUid {
                ephemeris_id: 301,
                orientation_id: 0,
            },
        };

        let pos_str = serde_dhall::serialize(&pos).to_string().unwrap();
        println!("{pos_str:?}");
        let v_deser: VectorExpr = serde_dhall::from_str(&pos_str).parse().unwrap();
        assert_eq!(v_deser, pos);
    }

    #[test]
    fn test_vector_expr_cross() {
        let pos = VectorExpr::Position {
            from_frame: FrameUid {
                ephemeris_id: 399,
                orientation_id: 0,
            },
            to_frame: FrameUid {
                ephemeris_id: 301,
                orientation_id: 0,
            },
        };

        let vel = VectorExpr::Velocity {
            from_frame: FrameUid {
                ephemeris_id: 399,
                orientation_id: 0,
            },
            to_frame: FrameUid {
                ephemeris_id: 301,
                orientation_id: 0,
            },
        };

        let h_vec = VectorExpr::CrossProduct {
            a: Box::new(pos),
            b: Box::new(vel),
        };

        let h_vec_str = serde_dhall::serialize(&h_vec).to_string().unwrap();
        println!("{h_vec_str:?}");
        let v_deser: VectorExpr = serde_dhall::from_str(&h_vec_str).parse().unwrap();
        assert_eq!(v_deser, h_vec);
    }
}
