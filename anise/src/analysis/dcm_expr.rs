/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::almanac::Almanac;
use crate::analysis::prelude::VectorExpr;
use crate::analysis::specs::{StateSpec, StateSpecTrait};
use crate::analysis::PhysicsDcmExprSnafu;
use crate::math::rotation::{EulerParameter, DCM};
use crate::prelude::Epoch;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::fmt;

use super::AnalysisError;

/// DCM Expr defines an expression built from a DCM. At the moment, this does not allow for combining rotations.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum DcmExpr {
    Identity {
        from: i32,
        to: i32,
    },
    R1 {
        angle_rad: f64,
        from: i32,
        to: i32,
    },
    R2 {
        angle_rad: f64,
        from: i32,
        to: i32,
    },
    R3 {
        angle_rad: f64,
        from: i32,
        to: i32,
    },
    Triad {
        primary_axis: VectorExpr,
        primary_vec: VectorExpr,
        secondary_axis: VectorExpr,
        secondary_vec: VectorExpr,
        from: i32,
        to: i32,
    },
    Quaternion {
        x: f64,
        y: f64,
        z: f64,
        w: f64,
        from: i32,
        to: i32,
    },
    RIC {
        state: Box<StateSpec>,
        from: i32,
    },
    VNC {
        state: Box<StateSpec>,
        from: i32,
    },
    RCN {
        state: Box<StateSpec>,
        from: i32,
    },
    SEZ {
        state: Box<StateSpec>,
        from: i32,
    },
}

impl fmt::Display for DcmExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identity { from, to } => write!(f, "I[{from} -> {to}]"),
            Self::R1 {
                angle_rad,
                from,
                to,
            } => write!(f, "R1({angle_rad})[{from} -> {to}]"),
            Self::R2 {
                angle_rad,
                from,
                to,
            } => write!(f, "R2({angle_rad})[{from} -> {to}]"),
            Self::R3 {
                angle_rad,
                from,
                to,
            } => write!(f, "R3({angle_rad})[{from} -> {to}]"),
            Self::Triad {
                primary_axis,
                primary_vec,
                secondary_axis,
                secondary_vec,
                from,
                to,
            } => {
                write!(
                    f,
                    "TRIAD [{from} -> {to}] primary_axis: {primary_axis} primary_vec: {primary_vec}"
                )?;
                write!(
                    f,
                    "secondary_axis: {secondary_axis} secondary_vec: {secondary_vec}"
                )
            }
            Self::Quaternion {
                x,
                y,
                z,
                w,
                from,
                to,
            } => {
                let q = EulerParameter {
                    x: *x,
                    y: *y,
                    z: *z,
                    w: *w,
                    from: *from,
                    to: *to,
                };
                write!(f, "{q}")
            }
            Self::RIC { state, from: _ } => write!(f, "RIC of {state}"),
            Self::RCN { state, from: _ } => write!(f, "RCN of {state}"),
            Self::VNC { state, from: _ } => write!(f, "VNC of {state}"),
            Self::SEZ { state, from: _ } => write!(f, "SEZ of {state}"),
        }
    }
}

impl DcmExpr {
    /// Evaluates this vector expression, returning a Vector
    pub fn evaluate(&self, epoch: Epoch, almanac: &Almanac) -> Result<DCM, AnalysisError> {
        match self {
            Self::Quaternion {
                x,
                y,
                z,
                w,
                from,
                to,
            } => {
                let q = EulerParameter {
                    x: *x,
                    y: *y,
                    z: *z,
                    w: *w,
                    from: *from,
                    to: *to,
                };
                Ok(q.into())
            }

            Self::Identity { from, to } => Ok(DCM::identity(*from, *to)),

            Self::RIC { state, from } => {
                let state = state.evaluate(epoch, almanac)?;

                let mut dcm = state
                    .dcm_from_ric_to_inertial()
                    .context(PhysicsDcmExprSnafu {
                        expr: Box::new(self.clone()),
                        epoch,
                    })?;
                dcm.from = *from;

                Ok(dcm)
            }

            Self::RCN { state, from } => {
                let state = state.evaluate(epoch, almanac)?;

                let mut dcm = state
                    .dcm_from_rcn_to_inertial()
                    .context(PhysicsDcmExprSnafu {
                        expr: Box::new(self.clone()),
                        epoch,
                    })?;
                dcm.from = *from;

                Ok(dcm)
            }

            Self::VNC { state, from } => {
                let state = state.evaluate(epoch, almanac)?;

                let mut dcm = state
                    .dcm_from_vnc_to_inertial()
                    .context(PhysicsDcmExprSnafu {
                        expr: Box::new(self.clone()),
                        epoch,
                    })?;
                dcm.from = *from;

                Ok(dcm)
            }

            Self::SEZ { state, from } => {
                let state = state.evaluate(epoch, almanac)?;

                let mut dcm =
                    state
                        .dcm_from_topocentric_to_body_fixed()
                        .context(PhysicsDcmExprSnafu {
                            expr: Box::new(self.clone()),
                            epoch,
                        })?;
                dcm.from = *from;

                Ok(dcm)
            }

            Self::Triad {
                primary_axis,
                primary_vec,
                secondary_axis,
                secondary_vec,
                from,
                to,
            } => {
                let primary_body_axis = primary_axis.evaluate(epoch, almanac)?;
                let primary_vec = primary_vec.evaluate(epoch, almanac)?;
                let secondary_body_axis = secondary_axis.evaluate(epoch, almanac)?;
                let secondary_vec = secondary_vec.evaluate(epoch, almanac)?;

                DCM::align_and_clock(
                    primary_body_axis,
                    primary_vec,
                    secondary_body_axis,
                    secondary_vec,
                    *from,
                    *to,
                )
                .context(PhysicsDcmExprSnafu {
                    expr: Box::new(self.clone()),
                    epoch,
                })
            }

            Self::R1 {
                angle_rad,
                from,
                to,
            } => Ok(DCM::r1(*angle_rad, *from, *to)),
            Self::R2 {
                angle_rad,
                from,
                to,
            } => Ok(DCM::r2(*angle_rad, *from, *to)),
            Self::R3 {
                angle_rad,
                from,
                to,
            } => Ok(DCM::r3(*angle_rad, *from, *to)),
        }
    }
}
