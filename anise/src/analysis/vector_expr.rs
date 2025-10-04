/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::fmt;

use crate::almanac::Almanac;
use crate::analysis::PhysicsVecExprSnafu;
use crate::math::Vector3;
use crate::prelude::Epoch;

use super::specs::{OrthogonalFrame, Plane, StateSpec};
use super::AnalysisError;

/// VectorExpr defines a vector expression, which can either be computed from a state, or from a fixed definition.
///
/// # API note
/// This VectorExpr is different from the Python class because of a limitation in the support for recursive types in PyO3.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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
    /// Unit vector of this vector expression, returns zero vector if norm less than 1e-12
    Unit(Box<Self>),
    /// Negate a vector
    /// /// Negate a vector.
    Negate(Box<Self>),
    /// Vector projection of a onto b
    VecProjection {
        a: Box<Self>,
        b: Box<Self>,
    },
    // This should be as simple as multiplying the input VectorExpr by the DCM.
    // I think it makes sense to have trivial rotations like VNC, RIC, RCN available in the frame spec.
    // The test should consist in checking that we can rebuild the VNC frame and project the Sun Earth vector onto
    // the VNC frame of that same Sun Earth orbit, returning the X, Y, or Z component.
    // Projection should allow XY, XZ, YZ which determines the components to account for.
    /// Multiplies the DCM of thr frame with this vector, thereby rotating it into the provided orthogonal frame, optionally projecting onto the plan, optionally projecting onto the plane
    Project {
        v: Box<Self>,
        frame: Box<OrthogonalFrame>,
        plane: Option<Plane>,
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
            Self::Unit(v) => write!(f, "unit({v})"),
            Self::Negate(v) => write!(f, "-{v}"),
            Self::VecProjection { a, b } => write!(f, "proj {a} onto {b}"),
            Self::Project { v, frame, plane } => {
                if let Some(plane) = plane {
                    write!(f, "proj {v} onto {plane:?} of {frame}")
                } else {
                    write!(f, "{v} dot {frame}")
                }
            }
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
            Self::Unit(v) => Ok(v
                .evaluate(epoch, almanac)?
                .try_normalize(1e-12)
                .unwrap_or(Vector3::zeros())),
            Self::Negate(v) => Ok(-v.evaluate(epoch, almanac)?),
            Self::VecProjection { a, b } => {
                let vec_a = a.evaluate(epoch, almanac)?;
                let vec_b = b.evaluate(epoch, almanac)?;

                Ok(vec_a.dot(&vec_b) * vec_b)
            }
            Self::Project { v, frame, plane } => {
                let dcm = frame.evaluate(epoch, almanac)?;

                let vector = v.evaluate(epoch, almanac)?;

                if let Some(plane) = plane {
                    Ok(dcm * plane.mask() * vector)
                } else {
                    Ok(dcm * vector)
                }
            }
        }
    }
}
