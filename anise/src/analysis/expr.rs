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
use crate::analysis::{AlmanacExprSnafu, PhysicsVecExprSnafu};
use crate::astro::Aberration;
use crate::errors::EphemerisSnafu;
use crate::frames::Frame;
use crate::math::Vector3;
use crate::prelude::{Epoch, Orbit};
use crate::NaifId;

use super::elements::OrbitalElement;
use super::specs::{OrthogonalFrame, Plane, StateSpec};
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

/// ScalarExpr defines a scalar computation from a (set of) vector expression(s).
#[derive(Clone, Debug, PartialEq)]
pub enum ScalarExpr {
    Norm(VectorExpr),
    NormSquared(VectorExpr),
    DotProduct {
        a: VectorExpr,
        b: VectorExpr,
    },
    AngleBetween {
        a: VectorExpr,
        b: VectorExpr,
    },
    VectorX(VectorExpr),
    VectorY(VectorExpr),
    VectorZ(VectorExpr),
    Element(OrbitalElement),
    /// Computes the eclipsing percentage due to the eclipsing frame. Aberration correction is that of the state spec.
    SolarEclipsePercentage {
        eclipsing_frame: Frame,
    },
    /// Computes the occultation percentage of the back_frame due to the front_frame. Aberration correction is that of the state spec.
    OccultationPercentage {
        back_frame: Frame,
        front_frame: Frame,
    },
    /// Computes the beta angle. Aberration correction is that of the state spec.
    BetaAngle,
    /// Computes the Sun angle where observer_id is the ID of the spacecraft for example.
    /// If the frame of the state spec is in an Earth frame, then this computes the Sun Probe Earth angle.
    /// Refer to the sun_angle_deg function for detailed documentation.
    SunAngle {
        observer_id: NaifId,
    },
    AzimuthFromLocation {
        location_id: i32,
        obstructing_body: Option<Frame>,
    },
    ElevationFromLocation {
        location_id: i32,
        obstructing_body: Option<Frame>,
    },
    RangeFromLocation {
        location_id: i32,
        obstructing_body: Option<Frame>,
    },
    RangeRateFromLocation {
        location_id: i32,
        obstructing_body: Option<Frame>,
    },
}

impl ScalarExpr {
    /// Computes this scalar expression for the provided orbit.
    pub fn evaluate(
        &self,
        orbit: Orbit,
        ab_corr: Option<Aberration>,
        almanac: &Almanac,
    ) -> Result<f64, AnalysisError> {
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
                Ok(vec_a.angle(&vec_b).to_degrees())
            }
            Self::BetaAngle => almanac
                .beta_angle_deg(orbit, ab_corr)
                .context(AlmanacExprSnafu {
                    expr: Box::new(self.clone()),
                    state: orbit,
                }),
            Self::SolarEclipsePercentage { eclipsing_frame } => Ok(almanac
                .solar_eclipsing(*eclipsing_frame, orbit, ab_corr)
                .context(AlmanacExprSnafu {
                    expr: Box::new(self.clone()),
                    state: orbit,
                })?
                .percentage),
            Self::OccultationPercentage {
                back_frame,
                front_frame,
            } => Ok(almanac
                .occultation(*back_frame, *front_frame, orbit, ab_corr)
                .context(AlmanacExprSnafu {
                    expr: Box::new(self.clone()),
                    state: orbit,
                })?
                .percentage),
            Self::SunAngle { observer_id } => almanac
                .sun_angle_deg(orbit.frame.ephemeris_id, *observer_id, orbit.epoch, ab_corr)
                .context(EphemerisSnafu {
                    action: "computing sun angle in expression",
                })
                .context(AlmanacExprSnafu {
                    expr: Box::new(self.clone()),
                    state: orbit,
                }),
            Self::AzimuthFromLocation {
                location_id,
                obstructing_body,
            } => Ok(almanac
                .azimuth_elevation_range_sez_from_location_id(
                    orbit,
                    *location_id,
                    *obstructing_body,
                    ab_corr,
                )
                .context(AlmanacExprSnafu {
                    expr: Box::new(self.clone()),
                    state: orbit,
                })?
                .azimuth_deg),
            Self::ElevationFromLocation {
                location_id,
                obstructing_body,
            } => Ok(almanac
                .azimuth_elevation_range_sez_from_location_id(
                    orbit,
                    *location_id,
                    *obstructing_body,
                    ab_corr,
                )
                .context(AlmanacExprSnafu {
                    expr: Box::new(self.clone()),
                    state: orbit,
                })?
                .elevation_deg),
            Self::RangeFromLocation {
                location_id,
                obstructing_body,
            } => Ok(almanac
                .azimuth_elevation_range_sez_from_location_id(
                    orbit,
                    *location_id,
                    *obstructing_body,
                    ab_corr,
                )
                .context(AlmanacExprSnafu {
                    expr: Box::new(self.clone()),
                    state: orbit,
                })?
                .range_km),
            Self::RangeRateFromLocation {
                location_id,
                obstructing_body,
            } => Ok(almanac
                .azimuth_elevation_range_sez_from_location_id(
                    orbit,
                    *location_id,
                    *obstructing_body,
                    ab_corr,
                )
                .context(AlmanacExprSnafu {
                    expr: Box::new(self.clone()),
                    state: orbit,
                })?
                .range_rate_km_s),
        }
    }
}

impl fmt::Display for ScalarExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Norm(e) => write!(f, "|{e}|"),
            Self::NormSquared(e) => write!(f, "|{e}|^2"),
            Self::DotProduct { a, b } => write!(f, "{a} · {b}"),
            Self::AngleBetween { a, b } => write!(f, "∠ {a}, {b} (deg)"),
            Self::VectorX(e) => write!(f, "{e}_x"),
            Self::VectorY(e) => write!(f, "{e}_y"),
            Self::VectorZ(e) => write!(f, "{e}_z"),
            Self::Element(e) => write!(f, "{e:?} ({})", e.unit()),
            Self::SolarEclipsePercentage { eclipsing_frame } => {
                write!(f, "solar eclipse due to {eclipsing_frame:x} (%)")
            }
            Self::OccultationPercentage {
                front_frame,
                back_frame,
            } => write!(
                f,
                "occultation of {back_frame:x} due to {front_frame:x} (%)"
            ),
            Self::BetaAngle => write!(f, "beta angle (deg)"),
            Self::SunAngle { observer_id } => write!(f, "sun angle for obs={observer_id}"),
            Self::AzimuthFromLocation {
                location_id,
                obstructing_body: _,
            } => {
                write!(f, "azimuth from location #{location_id} (deg)")
            }
            Self::ElevationFromLocation {
                location_id,
                obstructing_body: _,
            } => {
                write!(f, "elevation from location #{location_id} (deg)")
            }
            Self::RangeFromLocation {
                location_id,
                obstructing_body: _,
            } => {
                write!(f, "range from location #{location_id} (km)")
            }
            Self::RangeRateFromLocation {
                location_id,
                obstructing_body: _,
            } => {
                write!(f, "range-rate from location #{location_id} (km/s)")
            }
        }
    }
}
