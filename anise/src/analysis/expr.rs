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

use snafu::ResultExt;

use crate::almanac::Almanac;
use crate::analysis::PhysicsVecExprSnafu;
use crate::math::Vector3;
use crate::prelude::{Epoch, Orbit};

use super::elements::OrbitalElement;
use super::specs::StateSpec;
use super::AnalysisError;

/// VectorExpr defines a vector expression, which can either be computed from a state, or from a fixed definition.
///
/// # API note
/// This VectorExpr is different from the Python class because of a limitation in the support for recursive types in PyO3.
#[derive(Clone, Debug, PartialEq)]
pub enum VectorExpr {
    // Vector with unspecified units, for arbitrary computations
    Fixed {
        x: f64,
        y: f64,
        z: f64,
    },
    /// Radius/position vector of this state specification
    Radius(StateSpec),
    /// Velocity vector of this state specification
    Velocity(StateSpec),
    /// Orbital moment (H) vector of this state specification
    OrbitalMomentum(StateSpec),
    /// Eccentricity vector of this state specification
    EccentricityVector(StateSpec),
    /// Cross product between two vector expression
    CrossProduct {
        a: Box<Self>,
        b: Box<Self>,
    },
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

impl VectorExpr {
    /// Evaluates this vector expression, returning a Vector
    pub fn evaluate(&self, epoch: Epoch, almanac: &Almanac) -> Result<Vector3, AnalysisError> {
        match self {
            Self::Fixed { x, y, z } => Ok(Vector3::new(*x, *y, *z)),
            Self::Radius(spec) => Ok(spec.evaluate(epoch, almanac)?.radius_km),
            Self::Velocity(spec) => Ok(spec.evaluate(epoch, almanac)?.velocity_km_s),
            Self::OrbitalMomentum(spec) => {
                spec.evaluate(epoch, almanac)?
                    .hvec()
                    .context(PhysicsVecExprSnafu {
                        expr: Box::new(self.clone()),
                        epoch,
                    })
            }
            Self::EccentricityVector(spec) => {
                spec.evaluate(epoch, almanac)?
                    .evec()
                    .context(PhysicsVecExprSnafu {
                        expr: Box::new(self.clone()),
                        epoch,
                    })
            }
            Self::CrossProduct { a, b } => {
                let vec_a = a.evaluate(epoch, almanac)?;
                let vec_b = b.evaluate(epoch, almanac)?;
                Ok(vec_a.cross(&vec_b))
            }
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
    pub fn evaluate(&self, orbit: Orbit, almanac: &Almanac) -> Result<f64, AnalysisError> {
        match self {
            Self::Element(oe) => oe.evaluate(orbit),
            Self::Norm(vexpr) => Ok(vexpr.evaluate(orbit.epoch, almanac)?.norm()),
            Self::NormSquared(vexpr) => Ok(vexpr.evaluate(orbit.epoch, almanac)?.norm_squared()),
            Self::VectorX(vexpr) => Ok(vexpr.evaluate(orbit.epoch, almanac)?.x),
            Self::VectorY(vexpr) => Ok(vexpr.evaluate(orbit.epoch, almanac)?.y),
            Self::VectorZ(vexpr) => Ok(vexpr.evaluate(orbit.epoch, almanac)?.z),
            Self::DotProduct { a, b } => {
                let vec_a = a.evaluate(orbit.epoch, almanac)?;
                let vec_b = b.evaluate(orbit.epoch, almanac)?;
                Ok(vec_a.dot(&vec_b))
            }
            Self::AngleBetween { a, b } => {
                let vec_a = a.evaluate(orbit.epoch, almanac)?;
                let vec_b = b.evaluate(orbit.epoch, almanac)?;
                Ok(vec_a.angle(&vec_b))
            }
        }
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
