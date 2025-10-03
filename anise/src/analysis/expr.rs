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
use crate::analysis::AlmanacExprSnafu;
use crate::astro::Aberration;
use crate::errors::EphemerisSnafu;
use crate::frames::Frame;
use crate::prelude::Orbit;
use crate::NaifId;

use super::elements::OrbitalElement;
use super::{AnalysisError, VectorExpr};

/// ScalarExpr defines a scalar computation from a (set of) vector expression(s).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum ScalarExpr {
    Constant(f64),
    /// Mean radius of the provided body, must be loaded in the almanac
    MeanEquatorialRadius {
        celestial_object: i32,
    },
    SemiMajorEquatorialRadius {
        celestial_object: i32,
    },
    SemiMinorEquatorialRadius {
        celestial_object: i32,
    },
    PolarRadius {
        celestial_object: i32,
    },
    Flattening {
        celestial_object: i32,
    },
    GravParam {
        celestial_object: i32,
    },
    Add {
        a: Box<Self>,
        b: Box<Self>,
    },
    Mul {
        a: Box<Self>,
        b: Box<Self>,
    },
    Negate(Box<Self>),
    Invert(Box<Self>),
    Sqrt(Box<Self>),
    Powi {
        scalar: Box<Self>,
        n: i32,
    },
    Powf {
        scalar: Box<Self>,
        n: f64,
    },
    Norm(VectorExpr),
    NormSquared(VectorExpr),
    DotProduct {
        a: VectorExpr,
        b: VectorExpr,
    },
    /// Angle between two vectors, in degrees
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
    /// Computes the beta angle, in degrees. Aberration correction is that of the state spec.
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
            Self::Constant(v) => Ok(*v),
            Self::MeanEquatorialRadius { celestial_object } => almanac
                .planetary_data
                .get_by_id(*celestial_object)
                .or(Err(AnalysisError::AlmanacMissingDataExpr {
                    expr: Box::new(self.clone()),
                }))?
                .shape
                .map_or(
                    Err(AnalysisError::AlmanacMissingDataExpr {
                        expr: Box::new(self.clone()),
                    }),
                    |shape| Ok(shape.mean_equatorial_radius_km()),
                ),
            Self::SemiMajorEquatorialRadius { celestial_object } => almanac
                .planetary_data
                .get_by_id(*celestial_object)
                .or(Err(AnalysisError::AlmanacMissingDataExpr {
                    expr: Box::new(self.clone()),
                }))?
                .shape
                .map_or(
                    Err(AnalysisError::AlmanacMissingDataExpr {
                        expr: Box::new(self.clone()),
                    }),
                    |shape| Ok(shape.semi_major_equatorial_radius_km),
                ),
            Self::SemiMinorEquatorialRadius { celestial_object } => almanac
                .planetary_data
                .get_by_id(*celestial_object)
                .or(Err(AnalysisError::AlmanacMissingDataExpr {
                    expr: Box::new(self.clone()),
                }))?
                .shape
                .map_or(
                    Err(AnalysisError::AlmanacMissingDataExpr {
                        expr: Box::new(self.clone()),
                    }),
                    |shape| Ok(shape.semi_minor_equatorial_radius_km),
                ),
            Self::PolarRadius { celestial_object } => almanac
                .planetary_data
                .get_by_id(*celestial_object)
                .or(Err(AnalysisError::AlmanacMissingDataExpr {
                    expr: Box::new(self.clone()),
                }))?
                .shape
                .map_or(
                    Err(AnalysisError::AlmanacMissingDataExpr {
                        expr: Box::new(self.clone()),
                    }),
                    |shape| Ok(shape.polar_radius_km),
                ),
            Self::Flattening { celestial_object } => almanac
                .planetary_data
                .get_by_id(*celestial_object)
                .or(Err(AnalysisError::AlmanacMissingDataExpr {
                    expr: Box::new(self.clone()),
                }))?
                .shape
                .map_or(
                    Err(AnalysisError::AlmanacMissingDataExpr {
                        expr: Box::new(self.clone()),
                    }),
                    |shape| Ok(shape.flattening()),
                ),

            Self::GravParam { celestial_object } => Ok(almanac
                .planetary_data
                .get_by_id(*celestial_object)
                .or(Err(AnalysisError::AlmanacMissingDataExpr {
                    expr: Box::new(self.clone()),
                }))?
                .mu_km3_s2),

            Self::Add { a, b } => {
                Ok(a.evaluate(orbit, ab_corr, almanac)? + b.evaluate(orbit, ab_corr, almanac)?)
            }

            Self::Mul { a, b } => {
                Ok(a.evaluate(orbit, ab_corr, almanac)? * b.evaluate(orbit, ab_corr, almanac)?)
            }

            Self::Negate(v) => Ok(-v.evaluate(orbit, ab_corr, almanac)?),

            Self::Invert(v) => {
                let v = v.evaluate(orbit, ab_corr, almanac)?;

                if v.abs() > f64::MIN {
                    Ok(1.0 / v)
                } else {
                    Err(AnalysisError::MathExpr {
                        expr: Box::new(self.clone()),
                        source: Box::new(crate::errors::MathError::DivisionByZero {
                            action: "computing expression",
                        }),
                    })
                }
            }

            Self::Sqrt(v) => Ok(v.evaluate(orbit, ab_corr, almanac)?.sqrt()),
            Self::Powi { scalar, n } => Ok(scalar.evaluate(orbit, ab_corr, almanac)?.powi(*n)),
            Self::Powf { scalar, n } => Ok(scalar.evaluate(orbit, ab_corr, almanac)?.powf(*n)),

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

    pub fn default_event_precision(&self) -> f64 {
        match self {
            Self::Norm(_) | Self::NormSquared(_) | Self::DotProduct { a: _, b: _ } => 0.1,
            Self::AngleBetween { a: _, b: _ }
            | Self::BetaAngle
            | Self::SunAngle { observer_id: _ }
            | Self::AzimuthFromLocation {
                location_id: _,
                obstructing_body: _,
            }
            | Self::ElevationFromLocation {
                location_id: _,
                obstructing_body: _,
            } => 1e-2,
            Self::VectorX(_)
            | Self::VectorY(_)
            | Self::VectorZ(_)
            | Self::RangeFromLocation {
                location_id: _,
                obstructing_body: _,
            }
            | Self::RangeRateFromLocation {
                location_id: _,
                obstructing_body: _,
            }
            | Self::MeanEquatorialRadius {
                celestial_object: _,
            }
            | Self::SemiMajorEquatorialRadius {
                celestial_object: _,
            }
            | Self::SemiMinorEquatorialRadius {
                celestial_object: _,
            }
            | Self::PolarRadius {
                celestial_object: _,
            } => 1e-2,
            Self::Element(e) => e.default_event_precision(),
            Self::SolarEclipsePercentage { eclipsing_frame: _ }
            | Self::OccultationPercentage {
                front_frame: _,
                back_frame: _,
            } => 1e-1,
            Self::Constant(_)
            | Self::Add { a: _, b: _ }
            | Self::Mul { a: _, b: _ }
            | Self::Invert(_)
            | Self::Negate(_)
            | Self::Powi { scalar: _, n: _ }
            | Self::Powf { scalar: _, n: _ }
            | Self::GravParam {
                celestial_object: _,
            }
            | Self::Flattening {
                celestial_object: _,
            }
            | Self::Sqrt(_) => f64::EPSILON,
        }
    }

    /// Export this Scalar Expression to S-Expression / LISP syntax
    pub fn to_s_expr(&self) -> String {
        serde_lexpr::to_value(&self).unwrap().to_string()
    }

    /// Load this Scalar Expression from an S-Expression / LISP syntax
    pub fn from_s_expr(expr: &str) -> Result<Self, serde_lexpr::Error> {
        serde_lexpr::from_str(expr)
    }
}

impl fmt::Display for ScalarExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constant(v) => write!(f, "{v}"),
            Self::Add { a, b } => write!(f, "{a} + {b}"),
            Self::Mul { a, b } => write!(f, "{a} * {b}"),
            Self::Invert(v) => write!(f, "1.0/{v}"),
            Self::Powi { scalar, n } => write!(f, "{scalar}^{n}"),
            Self::Powf { scalar, n } => write!(f, "{scalar}^{n}"),
            Self::Negate(v) => write!(f, "-{v}"),
            Self::Sqrt(v) => write!(f, "sqrt({v})"),
            Self::MeanEquatorialRadius { celestial_object } => {
                write!(f, "mean eq. radius of {celestial_object}")
            }
            Self::SemiMajorEquatorialRadius { celestial_object } => {
                write!(f, "semi-major eq. radius of {celestial_object}")
            }
            Self::SemiMinorEquatorialRadius { celestial_object } => {
                write!(f, "semi-minor eq. radius of {celestial_object}")
            }
            Self::PolarRadius { celestial_object } => {
                write!(f, "polar radius of {celestial_object}")
            }
            Self::Flattening { celestial_object } => write!(f, "flattening of {celestial_object}"),
            Self::GravParam { celestial_object } => write!(f, "grav. param of {celestial_object}"),
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
