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

use crate::{almanac::Almanac, prelude::FrameUid};
// TODO: Once https://github.com/Nadrieril/dhall-rust/issues/242 is closed, enable Dhall serialization.
// Will be implemented in https://github.com/nyx-space/anise/issues/466
// use serde_derive::{Deserialize, Serialize};
// use serde_dhall::StaticType;

#[derive(Clone, Debug, PartialEq)]
pub enum FrameDef {
    Uid(FrameUid),
    Manual(Box<ReferenceFrame>),
}

/// StateDef allows defining a state from one frame (`from_frame`) to another (`to_frame`)
#[derive(Clone, Debug, PartialEq)]
pub struct StateDef {
    from_frame: FrameDef,
    to_frame: FrameDef,
}

/// VectorExpr defines a vector expression, which can either be computed from a state, or from a fixed definition.
#[derive(Clone, Debug, PartialEq)]
pub enum VectorExpr {
    Fixed { x: f64, y: f64, z: f64 }, // Unitless vector, for arbitrary computations
    Position(StateDef),
    Velocity(StateDef),
    OrbitalMomentum(StateDef),
    EccentricityVector(StateDef),
    CrossProduct { a: Box<Self>, b: Box<Self> },
}

/// VectorScalar defines a scalar computation from a (set of) vector expression(s).
#[derive(Clone, Debug, PartialEq)]
pub enum ScalarCalc {
    Norm(VectorExpr),
    NormSquared(VectorExpr),
    DotProduct { a: VectorExpr, b: VectorExpr },
    AngleBetween { a: VectorExpr, b: VectorExpr },
    OrbitalElement(OrbitalElement),
}

/// Orbital element defines all of the supported orbital elements in ANISE, which are all built from a State.
#[derive(Clone, Debug, PartialEq)]
pub enum OrbitalElement {
    SemiMajorAxis(StateDef),
    RAAN(StateDef),
    Eccentricity(StateDef),
}

// Defines how to build an orthogonal frame from custom vector expressions
#[derive(Clone, Debug, PartialEq)]
pub enum OrthogonalFrame {
    CrossProductXY { x: VectorExpr, y: VectorExpr },
    CrossProductXZ { x: VectorExpr, z: VectorExpr },
    CrossProductYZ { y: VectorExpr, z: VectorExpr },
}

/// Defines a runtime reference frame from an orthogonal frame, allowing it to be normalized or some vectors negated.
/// Note that if trying to negate a vector that isn't used in the definition of the orthogonal frame, an error will be raised.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceFrame {
    pub frame: OrthogonalFrame,
    pub negate_x: bool,
    pub negate_y: bool,
    pub negate_z: bool,
}

impl Almanac {}

#[cfg(test)]
mod ut_analysis {

    use crate::analysis::{FrameUid, OrbitalElement, ScalarCalc, StateDef, VectorExpr};
    use crate::constants::celestial_objects::EARTH;
    use crate::constants::frames::{EME2000, SUN_J2000};
    use crate::constants::orientations::J2000;
    use crate::prelude::Almanac;
    use arrow::array::Scalar;
    use rstest::*;
    use serde_dhall::StaticType;

    use super::FrameDef;

    #[fixture]
    fn almanac() -> Almanac {
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

    #[rstest]
    fn test_analysis_orbital_element(almanac: Almanac) {
        // Try to compute the SMA of the Earth with respect to the Sun.

        let from_frame = FrameDef::Uid(EME2000.into());
        let to_frame = FrameDef::Uid(SUN_J2000.into());

        let state = StateDef {
            from_frame,
            to_frame,
        };

        let calculations = vec![
            ScalarCalc::OrbitalElement(OrbitalElement::SemiMajorAxis(state)),
            ScalarCalc::OrbitalElement(OrbitalElement::Eccentricity(state)),
        ];

        // almanac.calculate(calculations).unwrap();
    }
}
