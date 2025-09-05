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

use hifitime::{Epoch, TimeSeries};
use std::{collections::HashMap, fmt};

use crate::{almanac::Almanac, errors::AlmanacError, prelude::Frame};
// TODO: Once https://github.com/Nadrieril/dhall-rust/issues/242 is closed, enable Dhall serialization.
// Will be implemented in https://github.com/nyx-space/anise/issues/466
// use serde_derive::{Deserialize, Serialize};
// use serde_dhall::StaticType;

#[derive(Clone, Debug, PartialEq)]
pub enum FrameSpec {
    Loaded(Frame),
    Manual {
        name: String,
        defn: Box<CustomFrameDef>,
    },
}

impl fmt::Display for FrameSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Loaded(frame) => write!(f, "{frame:x}"),
            Self::Manual { name, defn: _ } => write!(f, "{name}"),
        }
    }
}

/// StateDef allows defining a state from one frame (`from_frame`) to another (`to_frame`)
#[derive(Clone, Debug, PartialEq)]
pub struct StateSpec {
    from_frame: FrameSpec,
    to_frame: FrameSpec,
}

impl fmt::Display for StateSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.from_frame, self.to_frame)
    }
}

/// VectorExpr defines a vector expression, which can either be computed from a state, or from a fixed definition.
#[derive(Clone, Debug, PartialEq)]
pub enum VectorExpr {
    Fixed { x: f64, y: f64, z: f64 }, // Unitless vector, for arbitrary computations
    Radius(StateSpec),
    Velocity(StateSpec),
    OrbitalMomentum(StateSpec),
    EccentricityVector(StateSpec),
    CrossProduct { a: Box<Self>, b: Box<Self> },
}

impl fmt::Display for VectorExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed { x, y, z } => write!(f, "[{x}, {y}, {z}]"),
            Self::Radius(state) => write!(f, "Radius({state})"),
            Self::Velocity(state) => write!(f, "Velocity({state})"),
            Self::OrbitalMomentum(state) => write!(f, "OrbitalMomentum({state})"),
            Self::EccentricityVector(state) => write!(f, "EccentricityVector({state})"),
            Self::CrossProduct { a, b } => write!(f, "{a} x {b}"),
        }
    }
}

/// VectorScalar defines a scalar computation from a (set of) vector expression(s).
#[derive(Clone, Debug, PartialEq)]
pub enum ScalarExpr {
    Norm(VectorExpr),
    NormSquared(VectorExpr),
    DotProduct { a: VectorExpr, b: VectorExpr },
    AngleBetween { a: VectorExpr, b: VectorExpr },
    VectorX(VectorExpr),
    VectorY(VectorExpr),
    VectorZ(VectorExpr),
    Element(OrbitalElement),
}

/// Orbital element defines all of the supported orbital elements in ANISE, which are all built from a State.
#[derive(Clone, Debug, PartialEq)]
pub enum OrbitalElement {
    SemiMajorAxis,
    RAAN,
    Eccentricity,
}

// Defines how to build an orthogonal frame from custom vector expressions
#[derive(Clone, Debug, PartialEq)]
pub enum OrthonormalFrame {
    CrossProductXY { x: VectorExpr, y: VectorExpr },
    CrossProductXZ { x: VectorExpr, z: VectorExpr },
    CrossProductYZ { y: VectorExpr, z: VectorExpr },
}

/// Defines a runtime reference frame from an orthogonal frame, allowing it to be normalized or some vectors negated.
/// Note that if trying to negate a vector that isn't used in the definition of the orthogonal frame, an error will be raised.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomFrameDef {
    pub frame: OrthonormalFrame,
    pub negate_x: bool,
    pub negate_y: bool,
    pub negate_z: bool,
}

impl Almanac {
    pub fn generate_report(
        &self,
        scalars: &[ScalarExpr],
        state_def: StateSpec,
        timeseries: TimeSeries,
    ) -> Result<HashMap<Epoch, HashMap<String, f64>>, AlmanacError> {
        todo!()
    }
}

#[cfg(test)]
mod ut_analysis {

    use crate::analysis::{Frame, OrbitalElement, ScalarExpr, StateSpec, VectorExpr};
    use crate::constants::frames::{EME2000, SUN_J2000};
    use crate::prelude::Almanac;
    use hifitime::{Epoch, TimeSeries, Unit};
    use rstest::*;

    use super::FrameSpec;

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

    #[test]
    fn test_displays() {
        let from_frame = FrameSpec::Loaded(EME2000);
        let to_frame = FrameSpec::Loaded(SUN_J2000);

        let state = StateSpec {
            from_frame,
            to_frame,
        };

        assert_eq!(format!("{state}"), "Earth J2000 -> Sun J2000");

        let r = VectorExpr::Radius(state.clone());
        let v = VectorExpr::Velocity(state.clone());
        let h = VectorExpr::CrossProduct {
            a: Box::new(r.clone()),
            b: Box::new(v.clone()),
        };
        println!("{r}\n{v}\n{h}");
    }

    #[rstest]
    fn test_analysis_orbital_element(almanac: Almanac) {
        // Try to compute the SMA of the Earth with respect to the Sun.

        let from_frame = FrameSpec::Loaded(EME2000);
        let to_frame = FrameSpec::Loaded(SUN_J2000);

        let state = StateSpec {
            from_frame,
            to_frame,
        };

        let scalars = [
            ScalarExpr::Element(OrbitalElement::SemiMajorAxis),
            ScalarExpr::Element(OrbitalElement::Eccentricity),
        ];

        almanac
            .generate_report(
                &scalars,
                state,
                TimeSeries::inclusive(
                    Epoch::from_gregorian_tai_at_midnight(2025, 1, 1),
                    Epoch::from_gregorian_tai_at_noon(2026, 1, 1),
                    Unit::Minute * 1,
                ),
            )
            .unwrap();
    }
}
