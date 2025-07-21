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
use serde_dhall::StaticType;

// Temp: add static type to the current frameuid
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, StaticType)]
pub struct FrameUid {
    pub ephemeris_id: i32,
    pub orientation_id: i32,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, StaticType)]
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
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, StaticType)]
pub enum VectorMath {
    Identity { v: VectorExpr },
    CrossProduct { a: VectorExpr, b: VectorExpr },
    DotProduct { a: VectorExpr, b: VectorExpr },
    AngleBetween { a: VectorExpr, b: VectorExpr },
}
// Consider having only CrossProduct as a VectorMath and moving the other variants to Calculations... really this ought to be recursive but it isn't supported by the dhall crate yet.
// The calculations would also include the state parameters, and would necessarily return a f64.
// I could, yet again, try to serialize in another format. Maybe toml?

/*
So, if you look at `Topo` and `Mu` (aka `Rec`), and apply each of them to a functor (say `VectorExpr`), you get `List (VectorExpr Natural)` and `âˆ€(a : Type) â†’ (VectorExpr a â†’ a) â†’ a`, respectively. `Topo` holds a list of nodes that each refer to other indices in the list for their children â€“ it can represent finite directed graphs (including with cycles). `Mu` suspends the value as a function that says â€œif you can tell me how to turn one node into the result type, I can turn the whole tree into that result typeâ€ (i.e., induction) â€“ `Mu` can only represent finite trees (not graphs, and of course no cycles). A similar but less abstract type than `Mu` is `Fix`, but it canâ€™t be defined in Dhall. `Fix VectorExpr` would partially normalize to `VectorExpr (Fix VectorExpr)`, and if you try to fully normalize it, you get in trouble, with `VectorExpr (VectorExpr (VectorExpr (VectorExpr â€¦)))`. But, itâ€™s basically the same as writing a directly-recursive `enum VectorExpr { ..., CrossProduct { a: Box<VectorExpr>, b: Box<VectorExpr>, }`.
*/

#[cfg(test)]
mod ut_vector_dhall {

    use crate::analysis::{FrameUid, VectorExpr, VectorMath};
    use crate::prelude::Almanac;
    use rstest::*;
    use serde_dhall::SimpleType;

    #[test]
    fn test_vector_expr_fixed() {
        let v = VectorExpr::Fixed {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let m = VectorMath::Identity { v };
        let v_str = serde_dhall::serialize(&m)
            .static_type_annotation()
            .to_string()
            .unwrap();
        println!("{v_str:?}");
        let v_deser: VectorMath = serde_dhall::from_str(&v_str).parse().unwrap();
        assert_eq!(v_deser, m);
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

        let pos_str = serde_dhall::serialize(&pos)
            .static_type_annotation()
            .to_string()
            .unwrap();
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

        let h_vec = VectorMath::CrossProduct { a: pos, b: vel };

        let h_vec_str = serde_dhall::serialize(&h_vec)
            .static_type_annotation()
            .to_string()
            .unwrap();
        println!("{h_vec_str:?}");
        let v_deser: VectorMath = serde_dhall::from_str(&h_vec_str).parse().unwrap();
        assert_eq!(v_deser, h_vec);
    }

    #[fixture]
    pub fn almanac() -> Almanac {
        use std::path::PathBuf;

        let manifest_dir =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string()));

        Almanac::new(
            &manifest_dir
                .clone()
                .join("../data/de440s.bsp")
                .to_string_lossy(),
        )
        .unwrap()
        .load(
            &manifest_dir
                .clone()
                .join("../data/pck08.pca")
                .to_string_lossy(),
        )
        .unwrap()
    }
}
