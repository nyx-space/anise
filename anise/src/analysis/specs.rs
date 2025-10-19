/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::Epoch;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::fmt;

use crate::{
    almanac::Almanac,
    analysis::{AlmanacStateSpecSnafu, AnalysisError},
    astro::Aberration,
    math::{cartesian::CartesianState, rotation::DCM, Matrix3},
    prelude::Frame,
};

#[cfg(feature = "python")]
use pyo3::prelude::*;

use super::VectorExpr;

/// FrameSpec allows defining a frame that can be computed from another set of loaded frames, which include a center.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum FrameSpec {
    Loaded(Frame),
    Manual {
        name: String,
        defn: Box<OrthogonalFrame>,
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

// Defines how to build an orthogonal frame from custom vector expressions
//
// WARNING: Building such a frame does NOT normalize the vectors, you must use the Unit vector expression
// to build an orthonormal frame.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum OrthogonalFrame {
    XY { x: VectorExpr, y: VectorExpr },
    XZ { x: VectorExpr, z: VectorExpr },
    YZ { y: VectorExpr, z: VectorExpr },
}

impl fmt::Display for OrthogonalFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::XY { x, y } => write!(f, "frame XY {x} x {y}"),
            Self::XZ { x, z } => write!(f, "frame XZ {x} x {z}"),
            Self::YZ { y, z } => write!(f, "frame YZ {y} x {z}"),
        }
    }
}

impl OrthogonalFrame {
    /// Orthogonal frames do not set the time derivative of the DCM
    pub fn evaluate(&self, epoch: Epoch, almanac: &Almanac) -> Result<DCM, AnalysisError> {
        let (x, y, z) = match self {
            Self::XY { x, y } => {
                let x_vec = x.evaluate(epoch, almanac)?;
                let y_vec = y.evaluate(epoch, almanac)?;

                let z_vec = x_vec.cross(&y_vec);

                (x_vec, y_vec, z_vec)
            }
            Self::XZ { x, z } => {
                let x_vec = x.evaluate(epoch, almanac)?;
                let z_vec = z.evaluate(epoch, almanac)?;

                let y_vec = x_vec.cross(&z_vec);

                (x_vec, y_vec, z_vec)
            }
            Self::YZ { y, z } => {
                let y_vec = y.evaluate(epoch, almanac)?;
                let z_vec = z.evaluate(epoch, almanac)?;

                let x_vec = y_vec.cross(&z_vec);

                (x_vec, y_vec, z_vec)
            }
        };

        let rot_mat = Matrix3::new(x[0], x[1], x[2], y[0], y[1], y[2], z[0], z[1], z[2]);

        Ok(DCM {
            rot_mat,
            rot_mat_dt: None,
            from: -1,
            to: -2,
        })
    }
}

/// Plane selector, sets the missing component to zero.
/// For example, Plane::YZ will multiply the DCM by [[1, 0. 0], [0, 1, 0], [0, 0, 0]]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.analysis"))]
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Plane {
    XY,
    XZ,
    YZ,
}

impl Plane {
    pub fn mask(self) -> Matrix3 {
        match self {
            Self::XY => Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0),
            Self::XZ => Matrix3::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0),
            Self::YZ => Matrix3::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
        }
    }
}

/// StateSpec allows defining a state from the target to the observer
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct StateSpec {
    pub target_frame: FrameSpec,
    pub observer_frame: FrameSpec,
    pub ab_corr: Option<Aberration>,
}

impl fmt::Display for StateSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.target_frame, self.observer_frame)
    }
}

impl StateSpec {
    /// Export this State Specification to S-Expression / LISP syntax
    pub fn to_s_expr(&self) -> Result<String, serde_lexpr::Error> {
        Ok(serde_lexpr::to_value(self)?.to_string())
    }

    /// Load this State Specification from an S-Expression / LISP syntax
    pub fn from_s_expr(expr: &str) -> Result<Self, serde_lexpr::Error> {
        serde_lexpr::from_str(expr)
    }

    /// Evaluates this state specification at the provided epoch.
    pub fn evaluate(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<CartesianState, AnalysisError> {
        if let FrameSpec::Loaded(target_frame) = self.target_frame {
            if let FrameSpec::Loaded(observer_frame) = self.observer_frame {
                almanac
                    .transform(target_frame, observer_frame, epoch, self.ab_corr)
                    .context(AlmanacStateSpecSnafu {
                        spec: Box::new(self.clone()),
                        epoch,
                    })
            } else {
                unimplemented!("custom frames in not yet supported")
            }
        } else {
            unimplemented!("custom frames in not yet supported")
        }
    }
}
