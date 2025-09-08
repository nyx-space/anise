/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::{Epoch, TimeSeries};
use std::{collections::HashMap, fmt};

use crate::{almanac::Almanac, astro::Aberration, errors::AlmanacError, prelude::Frame};

use super::elements::OrbitalElement;
use super::specs::StateSpec;

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
