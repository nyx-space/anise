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

use crate::prelude::FrameUid;
use serde_derive::{Deserialize, Serialize};
use serde_dhall::StaticType;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, StaticType)]
pub struct State {
    from_frame: FrameUid,
    to_frame: FrameUid,
}

/// VectorExpr defines a vector expression, which can either be computed from a state, or from a fixed definition.
/// It will eventually support building new reference frames at runtime using a CrossProduct operation.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, StaticType)]
pub enum VectorExpr {
    Fixed { x: f64, y: f64, z: f64 }, // Unitless vector, for arbitrary computations
    Position(State),
    Velocity(State),
    OrbitalMomentum(State),
    EccentricityVector(State), /* TODO: Once https://github.com/Nadrieril/dhall-rust/issues/242 is closed
                               CrossProduct {
                                  a: Box<Self>,
                                  b: Box<Self>,
                               },*/
}

/// VectorScalar defines a scalar computation from a (set of) vector expression(s).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, StaticType)]
pub enum ScalarCalc {
    Norm(VectorExpr),
    NormSquared(VectorExpr),
    DotProduct { a: VectorExpr, b: VectorExpr },
    AngleBetween { a: VectorExpr, b: VectorExpr },
    OrbitalElement(OrbitalElement),
}

/// Orbital element defines all of the supported orbital elements in ANISE, which are all built from a State.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, StaticType)]
pub enum OrbitalElement {
    SemiMajorAxis(State),
    RAAN(State),
    Eccentricity(State),
}

/*  OrthogonalFrame also requires recursive types.
// Defines how to build an orthogonal frame from custom vector expressions
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, StaticType)]
pub enum OrthogonalFrame {
    CrossProductXY { x: VectorExpr, y: VectorExpr },
    CrossProductXZ { x: VectorExpr, z: VectorExpr },
    CrossProductYZ { y: VectorExpr, z: VectorExpr },
}

/// Defines a runtime reference frame from an orthogonal frame, allowing it to be normalized or some vectors negated.
/// Note that if trying to negate a vector that isn't used in the definition of the orthogonal frame, an error will be raised.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, StaticType)]
pub struct ReferenceFrame {
    pub frame: OrthogonalFrame,
    pub negate_x: bool,
    pub negate_y: bool,
    pub negate_z: bool,
}
*/

#[cfg(test)]
mod ut_analysis_dhall {

    use crate::analysis::{FrameUid, ScalarCalc, State, VectorExpr};
    use crate::prelude::Almanac;
    use rstest::*;
    use serde_dhall::StaticType;

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

    #[test]
    fn test_vector_expr_fixed() {
        let v = VectorExpr::Fixed {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let v_str = serde_dhall::serialize(&v)
            .static_type_annotation()
            .to_string()
            .unwrap();
        println!("{v_str:?}");
        let v_deser: VectorExpr = serde_dhall::from_str(&v_str).parse().unwrap();
        assert_eq!(v_deser, v);
    }

    #[test]
    fn test_vector_expr_state() {
        let state = State {
            from_frame: FrameUid {
                ephemeris_id: 399,
                orientation_id: 0,
            },
            to_frame: FrameUid {
                ephemeris_id: 301,
                orientation_id: 0,
            },
        };
        let pos = VectorExpr::Position(state);

        let pos_str = serde_dhall::serialize(&pos)
            .static_type_annotation()
            .to_string()
            .unwrap();
        println!("{pos_str:?}");
        let v_deser: VectorExpr = serde_dhall::from_str(&pos_str).parse().unwrap();
        assert_eq!(v_deser, pos);
    }

    #[test]
    fn test_orbit_calculation() {
        let state_m2e = State {
            from_frame: FrameUid {
                ephemeris_id: 301,
                orientation_id: 0,
            },
            to_frame: FrameUid {
                ephemeris_id: 399,
                orientation_id: 0,
            },
        };

        let pos_m2e = VectorExpr::Position(state_m2e);

        let expr_type = VectorExpr::static_type().to_string();
        println!("{expr_type}");

        let scalars = vec![
            ScalarCalc::OrbitalElement(crate::analysis::OrbitalElement::SemiMajorAxis(state_m2e)),
            ScalarCalc::Norm(pos_m2e),
        ];

        let scalars_str = serde_dhall::serialize(&scalars)
            .static_type_annotation()
            .to_string()
            .unwrap();
        println!("{scalars_str:?}");
        let scalars_deser: Vec<ScalarCalc> = serde_dhall::from_str(&scalars_str).parse().unwrap();

        assert_eq!(scalars_deser, scalars);
    }
}
