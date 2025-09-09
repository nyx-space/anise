/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::fmt;

use crate::prelude::Orbit;

use super::elements::OrbitalElement;
use super::specs::StateSpec;
use super::AnalysisError;

/// VectorExpr defines a vector expression, which can either be computed from a state, or from a fixed definition.
///
/// # API note
/// This VectorExpr is different from the Python class because of a limitation in the support for recursive types in PyO3.
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
            Self::CrossProduct { a, b } => write!(f, "{a} ⨯ {b}"),
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

impl ScalarExpr {
    /// Computes this scalar expression for the provided orbit.
    pub fn evaluate(&self, orbit: Orbit) -> Result<f64, AnalysisError> {
        todo!()
    }
}

impl fmt::Display for ScalarExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Norm(e) => write!(f, "|{e}|"),
            Self::NormSquared(e) => write!(f, "|{e}|^2"),
            Self::DotProduct { a, b } => write!(f, "{a} · {b}"),
            Self::AngleBetween { a, b } => write!(f, "∠ {a}, {b}"),
            Self::VectorX(e) => write!(f, "{e}_x"),
            Self::VectorY(e) => write!(f, "{e}_y"),
            Self::VectorZ(e) => write!(f, "{e}_z"),
            Self::Element(e) => write!(f, "{e:?}"),
        }
    }
}
