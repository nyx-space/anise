/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{OrbitalElement, ScalarExpr};
use crate::{
    analysis::AnalysisError,
    astro::{Aberration, AzElRange, Location},
    prelude::{Almanac, Frame, Orbit},
};
use hifitime::{Duration, Epoch, Unit};
use log::warn;
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use super::python::PyScalarExpr;
#[cfg(feature = "python")]
use pyo3::exceptions::PyException;
#[cfg(feature = "python")]
use pyo3::types::PyType;

/// Defines an event condition
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.analysis"))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Condition {
    Equals(f64),
    Between(f64, f64),
    LessThan(f64),
    GreaterThan(f64),
    Minimum(),
    Maximum(),
}

/// Defines a state parameter event finder from the desired value of the scalar expression to compute, precision on timing and value, and the aberration.
///
/// :type scalar: ScalarExpr
/// :type condition: Condition
/// :type epoch_precision: Duration
/// :type ab_corr: Aberration, optional
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.analysis"))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    /// Scalar expression to evaluate
    pub scalar: ScalarExpr,
    /// Condition that defines the bounded values of the event.
    pub condition: Condition,
    /// The duration precision used in the adaptive step scanner, and to consider an Event to have converged. Typically use 100 ms.
    pub epoch_precision: Duration,
    pub ab_corr: Option<Aberration>,
}

impl Event {
    /// Builds a new event where the epoch precision is set to its default of 10 milliseconds
    #[must_use]
    pub fn new(scalar: ScalarExpr, condition: Condition) -> Self {
        Self {
            scalar,
            condition,
            epoch_precision: Unit::Millisecond * 10,
            ab_corr: None,
        }
    }

    /// Apoapsis event finder
    pub fn apoapsis() -> Self {
        Event {
            scalar: ScalarExpr::Element(OrbitalElement::TrueAnomaly),
            condition: Condition::Equals(180.0),
            epoch_precision: Unit::Second * 0.1,
            ab_corr: None,
        }
    }

    /// Periapsis event finder
    pub fn periapsis() -> Self {
        Event {
            scalar: ScalarExpr::Element(OrbitalElement::TrueAnomaly),
            condition: Condition::Equals(0.0),
            epoch_precision: Unit::Millisecond * 10,
            ab_corr: None,
        }
    }

    /// Total eclipse event finder: returns events where the eclipsing percentage is greater than 99%.
    pub fn total_eclipse(eclipsing_frame: Frame) -> Self {
        Event {
            scalar: ScalarExpr::SolarEclipsePercentage { eclipsing_frame },
            condition: Condition::GreaterThan(99.0),
            epoch_precision: Unit::Millisecond * 10,
            ab_corr: None,
        }
    }

    /// Eclipse event finder, including penumbras: returns events where the eclipsing percentage is greater than 1%.
    pub fn eclipse(eclipsing_frame: Frame) -> Self {
        Event {
            scalar: ScalarExpr::SolarEclipsePercentage { eclipsing_frame },
            condition: Condition::GreaterThan(1.0),
            epoch_precision: Unit::Millisecond * 10,
            ab_corr: None,
        }
    }

    /// Penumbral eclipse event finder: returns events where the eclipsing percentage is greater than 1% and less than 99%.
    pub fn penumbra(eclipsing_frame: Frame) -> Self {
        Event {
            scalar: ScalarExpr::SolarEclipsePercentage { eclipsing_frame },
            condition: Condition::Between(1.0, 99.0),
            epoch_precision: Unit::Millisecond * 10,
            ab_corr: None,
        }
    }

    /// Report events where the object is above the terrain (or horizon if terrain is not set) when seen from the provided location ID.
    pub fn visible_from_location_id(location_id: i32, obstructing_body: Option<Frame>) -> Self {
        Event {
            scalar: ScalarExpr::ElevationFromLocation {
                location_id,
                obstructing_body,
            },
            condition: Condition::GreaterThan(0.0),
            epoch_precision: Unit::Millisecond * 10,
            ab_corr: None,
        }
    }

    /// Export this Event to S-Expression / LISP syntax
    pub fn to_s_expr(&self) -> Result<String, serde_lexpr::Error> {
        Ok(serde_lexpr::to_value(self)?.to_string())
    }

    /// Load this Event from an S-Expression / LISP syntax
    pub fn from_s_expr(expr: &str) -> Result<Self, serde_lexpr::Error> {
        serde_lexpr::from_str(expr)
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Event {
    /// Compute the event finding function of this event provided an Orbit and Almanac.
    /// If we're "in the event", the evaluation will be greater or equal to zero.
    ///
    /// :type orbit: Orbit
    /// :type almanac: Almanac
    /// :rtype: float
    pub fn eval(&self, orbit: Orbit, almanac: &Almanac) -> Result<f64, AnalysisError> {
        let mut current_val = self.scalar.evaluate(orbit, self.ab_corr, almanac)?;

        // Special handling for angular values when we need to find a root.
        if let Condition::Equals(mut desired_val) = self.condition {
            let use_trig = self.scalar.is_angle()
                || self.scalar.is_local_time()
                || matches!(self.scalar, ScalarExpr::Modulo { .. });

            if use_trig {
                // Scale to be akin to a full circle.
                if self.scalar.is_local_time() {
                    current_val *= 360.0 / 24.0;
                    desired_val *= 360.0 / 24.0;
                } else if let ScalarExpr::Modulo { v: _, ref m } = self.scalar {
                    let modmax = m.evaluate(orbit, self.ab_corr, almanac)?;
                    if modmax >= 1e-12 {
                        current_val *= 360.0 / modmax;
                        desired_val *= 360.0 / modmax;
                    }
                }

                // Use the arctan function because it's smooth around zero, but convert back to degrees for the comparison.
                let current_rad = current_val.to_radians();
                let desired_rad = desired_val.to_radians();

                // Convert the angles to points on a unit circle
                let (cur_sin, cur_cos) = current_rad.sin_cos();
                let (des_sin, des_cos) = desired_rad.sin_cos();

                // Calculate the difference vector and find its angle with atan2.
                // This will be zero only when the angles are identical.
                let y = cur_sin * des_cos - cur_cos * des_sin; // sin(current - desired)
                let x = cur_cos * des_cos + cur_sin * des_sin; // cos(current - desired)

                return Ok(y.atan2(x).to_degrees());
            }
        }

        // For all non-angular scalars, or for conditions other than Equals.
        match self.condition {
            Condition::Equals(val) => Ok(current_val - val),
            Condition::Between(min_val, max_val) => {
                // Return positive if inside, negative if outside.
                // Smallest of the two distances to the boundaries.
                let dist_to_min = current_val - min_val;
                let dist_to_max = max_val - current_val;
                Ok(dist_to_min.min(dist_to_max))
            }
            Condition::LessThan(val) => Ok(val - current_val), // Positive if current_val < val
            Condition::GreaterThan(val) => Ok(current_val - val), // Positive if current_val > val
            Condition::Minimum() | Condition::Maximum() => Err(AnalysisError::InvalidEventEval {
                err: format!(
                    "cannot call Eval on {:?}, it must be handled by finding the derivative of the scalar",
                    self.condition
                ),
            }),
        }
    }

    /// Pretty print the evaluation of this event for the provided Orbit and Almanac
    ///
    /// :type orbit: Orbit
    /// :type almanac: Almanac
    /// :rtype: str
    pub fn eval_string(&self, orbit: Orbit, almanac: &Almanac) -> Result<String, AnalysisError> {
        if let Condition::Equals(desired_val) = self.condition {
            let val = self.eval(orbit, almanac)?;
            if desired_val.abs() > 1e3 {
                Ok(format!(
                    "|{} - {desired_val:e}| = {val:e} on {}",
                    self.scalar, orbit.epoch
                ))
            } else if desired_val.abs() > 1e-2 {
                Ok(format!(
                    "|{} - {desired_val:.3}| = {val:.3} on {}",
                    self.scalar, orbit.epoch
                ))
            } else {
                Ok(format!("|{}| = {val:.3} on {}", self.scalar, orbit.epoch))
            }
        } else {
            let current_val = self.scalar.evaluate(orbit, self.ab_corr, almanac)?;
            // For other conditions, just show the current value of the scalar.
            if current_val.abs() > 1e3 || (current_val.abs() < 1e-2 && current_val != 0.0) {
                Ok(format!(
                    "{} = {current_val:e} on {}",
                    self.scalar, orbit.epoch
                ))
            } else {
                Ok(format!(
                    "{} = {current_val:.3} on {}",
                    self.scalar, orbit.epoch
                ))
            }
        }
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl Event {
    /// Convert the S-Expression to a Event
    /// :type expr: str
    /// :rtype: Event
    #[classmethod]
    #[pyo3(name = "from_s_expr")]
    fn py_from_s_expr(_cls: Bound<'_, PyType>, expr: &str) -> Result<Self, PyErr> {
        Self::from_s_expr(expr).map_err(|e| PyException::new_err(e.to_string()))
    }

    /// Converts this Event to its S-Expression
    /// :rtype: str
    #[pyo3(name = "to_s_expr")]
    fn py_to_s_expr(&self) -> Result<String, PyErr> {
        self.to_s_expr()
            .map_err(|e| PyException::new_err(e.to_string()))
    }

    #[classmethod]
    #[pyo3(name = "apoapsis")]
    /// Apoapsis event finder, with an epoch precision of 0.1 seconds
    /// :rtype: Event
    fn py_apoapsis(_cls: Bound<'_, PyType>) -> Self {
        Event::apoapsis()
    }

    /// Periapsis event finder, with an epoch precision of 0.1 seconds
    /// :rtype: Event
    #[classmethod]
    #[pyo3(name = "periapsis")]
    fn py_periapsis(_cls: Bound<'_, PyType>) -> Self {
        Event::periapsis()
    }

    /// Total eclipse event finder: returns events where the eclipsing percentage is greater than 98.9%.
    ///
    /// :type eclipsing_frame: Frame
    /// :rtype: Event
    #[classmethod]
    #[pyo3(name = "total_eclipse")]
    fn py_total_eclipse(_cls: Bound<'_, PyType>, eclipsing_frame: Frame) -> Self {
        Event::total_eclipse(eclipsing_frame)
    }

    /// Eclipse event finder, including penumbras: returns events where the eclipsing percentage is greater than 1%.
    ///
    /// :type eclipsing_frame: Frame
    /// :rtype: Event
    #[classmethod]
    #[pyo3(name = "eclipse")]
    fn py_eclipse(_cls: Bound<'_, PyType>, eclipsing_frame: Frame) -> Self {
        Event::eclipse(eclipsing_frame)
    }

    /// Penumbral eclipse event finder: returns events where the eclipsing percentage is greater than 1% and less than 99%.
    ///
    /// :type eclipsing_frame: Frame
    /// :rtype: Event
    #[classmethod]
    #[pyo3(name = "penumbra")]
    fn py_penumbra(_cls: Bound<'_, PyType>, eclipsing_frame: Frame) -> Self {
        Event::penumbra(eclipsing_frame)
    }

    /// Report events where the object is above the terrain (or horizon if terrain is not set) when seen from the provided location ID.
    ///
    /// :type location_id: int
    /// :type obstructing_body: Frame, optional
    /// :rtype: Event
    #[classmethod]
    #[pyo3(name = "visible_from_location_id", signature=(location_id, obstructing_body=None))]
    fn py_visible_from_location_id(
        _cls: Bound<'_, PyType>,
        location_id: i32,
        obstructing_body: Option<Frame>,
    ) -> Self {
        Event::visible_from_location_id(location_id, obstructing_body)
    }

    #[new]
    #[pyo3(signature=(scalar, condition, epoch_precision, ab_corr=None))]
    fn py_new(
        scalar: PyScalarExpr,
        condition: Condition,
        epoch_precision: Duration,
        ab_corr: Option<Aberration>,
    ) -> Self {
        let scalar = ScalarExpr::from(scalar);

        Self {
            scalar,
            condition,
            epoch_precision,
            ab_corr,
        }
    }

    /// The scalar expression to compute
    /// :rtype: ScalarExpr
    #[getter]
    fn scalar(&self) -> Result<PyScalarExpr, PyErr> {
        PyScalarExpr::try_from(self.scalar.clone())
    }

    /// The desired self.desired_value, must be in the same units as the state parameter
    /// :rtype: Condition
    #[getter]
    fn condition(&self) -> Condition {
        self.condition
    }
    /// The duration precision after which the solver will report that it cannot find any more precise
    /// :rtype: Duration
    #[getter]
    fn epoch_precision(&self) -> Duration {
        self.epoch_precision
    }
    /// :rtype: Aberration
    #[getter]
    fn ab_corr(&self) -> Option<Aberration> {
        self.ab_corr
    }

    /// :type scalar: ScalarExpr
    #[setter]
    fn set_scalar(&mut self, scalar: PyScalarExpr) {
        self.scalar = scalar.into();
    }

    /// :type desired_value: float
    #[setter]
    fn set_condition(&mut self, condition: Condition) {
        self.condition = condition;
    }

    /// :type epoch_precision: Duration
    #[setter]
    fn set_epoch_precision(&mut self, epoch_precision: Duration) {
        self.epoch_precision = epoch_precision;
    }

    /// type ab_corr: Aberration, optional
    #[setter]
    fn set_ab_corr(&mut self, ab_corr: Option<Aberration>) {
        self.ab_corr = ab_corr;
    }

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self}@{self:p}")
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
    fn __ne__(&self, other: &Self) -> bool {
        self != other
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.scalar)?;
        match self.condition {
            Condition::Equals(val) => {
                if val.abs() > 1e3 {
                    write!(f, " = {val:e} (± {})", self.epoch_precision)
                } else {
                    write!(f, " = {val} (± {})", self.epoch_precision)
                }
            }
            Condition::Between(a, b) => {
                write!(f, " in [{a}, {b}] (± {})", self.epoch_precision)
            }
            Condition::LessThan(val) => {
                if val.abs() > 1e3 {
                    write!(f, " <= {val:e} (± {})", self.epoch_precision)
                } else {
                    write!(f, " <= {val} (± {})", self.epoch_precision)
                }
            }
            Condition::GreaterThan(val) => {
                if val.abs() > 1e3 {
                    write!(f, " >= {val:e} (± {})", self.epoch_precision)
                } else {
                    write!(f, " >= {val} (± {})", self.epoch_precision)
                }
            }
            Condition::Minimum() => write!(f, " minimum value (± {})", self.epoch_precision),
            Condition::Maximum() => write!(f, " maximum value (± {})", self.epoch_precision),
        }
    }
}
/// Enumerates the possible edges of an event in a trajectory.
///
/// `EventEdge` is used to describe the nature of a trajectory event, particularly in terms of its temporal dynamics relative to a specified condition or threshold. This enum helps in distinguishing whether the event is occurring at a rising edge, a falling edge, or if the edge is unclear due to insufficient data or ambiguous conditions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.analysis"))]
pub enum EventEdge {
    /// Represents a rising edge of the event. This indicates that the event is transitioning from a lower to a higher evaluation of the event. For example, in the context of elevation, a rising edge would indicate an increase in elevation from a lower angle.
    Rising,
    /// Represents a falling edge of the event. This is the opposite of the rising edge, indicating a transition from a higher to a lower value of the event evaluator. For example, if tracking the elevation of an object, a falling edge would signify a
    Falling,
    /// Represents a local minimum of the event. This indicates that the previous and next values are both greater than the current value.
    LocalMin,
    /// Represents a local maximum of the event. This indicates that the previous and next values are both lower than the current value.
    LocalMax,
    /// If the edge cannot be clearly defined, it will be marked as unclear. This happens if the event is at a saddle point and the epoch precision is too large to find the exact slope.
    Unclear,
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl EventEdge {
    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
    fn __ne__(&self, other: &Self) -> bool {
        self != other
    }
}

/// Represents the details of an event occurring along a trajectory.
///
/// `EventDetails` encapsulates the state at which a particular event occurs in a trajectory, along with additional information about the nature of the event. This struct is particularly useful for understanding the dynamics of the event, such as whether it represents a rising or falling edge, or if the edge is unclear.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.analysis", get_all))]
pub struct EventDetails {
    /// The state of the trajectory at the found event.
    /// :rtype: Orbit
    pub orbit: Orbit,
    /// Indicates whether the event is a rising edge, falling edge, or unclear. This helps in understanding the direction of change at the event point.
    /// :rtype: EventEdge
    pub edge: EventEdge,
    /// Numerical evaluation of the event condition, e.g. if seeking the apoapsis, this returns the near zero
    /// :rtype: float
    pub value: f64,
    /// Numertical evaluation of the event condition one epoch step before the found event (used to compute the rising/falling edge).
    /// :rtype: float
    pub prev_value: Option<f64>,
    /// Numertical evaluation of the event condition one epoch step after the found event (used to compute the rising/falling edge).
    /// :rtype: float
    pub next_value: Option<f64>,
    /// Precision of the epoch for this value
    /// :rtype: Duration
    pub pm_duration: Duration,
    /// Store the representation of this event as a string because we can't move or clone the event reference
    /// :rtype: str
    pub repr: String,
}

impl EventDetails {
    /// Generates detailed information about an event at a specific epoch in a trajectory.
    ///
    /// This takes an `Epoch` as an input and returns a `Result<Self, EventError>`.
    /// It is designed to determine the state of a trajectory at a given epoch, evaluate the specific event at that state, and ascertain the nature of the event (rising, falling, or unclear).
    /// The initialization intelligently determines the edge type of the event by comparing the event's value at the current, previous, and next epochs.
    /// It ensures robust event characterization in trajectories.
    ///
    /// # Returns
    /// - `Ok(EventDetails<S>)` if the state at the given epoch can be determined and the event details are successfully evaluated.
    /// - `Err(EventError)` if there is an error in retrieving the state at the specified epoch.
    ///
    pub fn new(
        state: Orbit,
        value: f64,
        event: &Event,
        prev_state: Option<Orbit>,
        next_state: Option<Orbit>,
        almanac: &Almanac,
    ) -> Result<Self, AnalysisError> {
        let prev_value = if let Some(state) = prev_state {
            Some(event.eval(state, almanac)?)
        } else {
            None
        };

        let next_value = if let Some(state) = next_state {
            Some(event.eval(state, almanac)?)
        } else {
            None
        };

        let edge = if let Some(prev_value) = prev_value {
            if let Some(next_value) = next_value {
                if prev_value > value {
                    if value > next_value {
                        EventEdge::Falling
                    } else {
                        EventEdge::LocalMin
                    }
                } else if prev_value < value {
                    if value < next_value {
                        EventEdge::Rising
                    } else {
                        EventEdge::LocalMax
                    }
                } else {
                    EventEdge::Unclear
                }
            } else if prev_value > value {
                EventEdge::Falling
            } else {
                EventEdge::Rising
            }
        } else if let Some(next_value) = next_value {
            if next_value > value {
                EventEdge::Rising
            } else {
                EventEdge::Falling
            }
        } else {
            warn!(
                "could not determine edge of {event} because state could be queried around {}",
                state.epoch
            );
            EventEdge::Unclear
        };

        Ok(EventDetails {
            edge,
            orbit: state,
            value,
            prev_value,
            next_value,
            pm_duration: event.epoch_precision,
            repr: event.eval_string(state, almanac)?,
        })
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl EventDetails {
    /// :rtype: str
    fn describe(&self) -> String {
        format!("{self:?}")
    }
    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self}@{self:p}")
    }
    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
    fn __ne__(&self, other: &Self) -> bool {
        self != other
    }
}

impl fmt::Display for EventDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({:?})", self.repr, self.edge)
    }
}

impl fmt::Debug for EventDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prev_fmt = match self.prev_value {
            Some(value) => format!("{value:.6}"),
            None => "".to_string(),
        };

        let next_fmt = match self.next_value {
            Some(value) => format!("{value:.6}"),
            None => "".to_string(),
        };

        write!(
            f,
            "{} and is {:?} (roots with {} intervals: {}, {:.6}, {})",
            self.repr, self.edge, self.pm_duration, prev_fmt, self.value, next_fmt
        )
    }
}

#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.analysis", get_all))]
#[derive(Clone, PartialEq)]
pub struct EventArc {
    /// rise event of this arc
    /// :rtype: EventDetails
    pub rise: EventDetails,
    /// fall event of this arc
    /// :rtype: EventDetails
    pub fall: EventDetails,
}

#[cfg_attr(feature = "python", pymethods)]
impl EventArc {
    /// :rtype: Duration
    pub fn duration(&self) -> Duration {
        self.end_epoch() - self.start_epoch()
    }

    /// :rtype: Epoch
    pub fn start_epoch(&self) -> Epoch {
        self.rise.orbit.epoch
    }

    /// :rtype: Epoch
    pub fn end_epoch(&self) -> Epoch {
        self.fall.orbit.epoch
    }

    #[cfg(feature = "python")]
    fn __str__(&self) -> String {
        format!("{self}")
    }
    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        format!("{self}@{self:p}")
    }
}

impl fmt::Display for EventArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} until {} (lasts {})",
            self.start_epoch(),
            self.end_epoch(),
            self.duration()
        )
    }
}

impl fmt::Debug for EventArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} until {}", self.rise, self.fall)
    }
}

#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.analysis", get_all))]
#[derive(Clone, Debug, PartialEq)]
pub struct VisibilityArc {
    /// rise event of this arc
    /// :rtype: EventDetails
    pub rise: EventDetails,
    /// fall event of this arc
    /// :rtype: EventDetails
    pub fall: EventDetails,
    /// :rtype: str
    pub location_ref: String,
    /// :rtype: Location
    pub location: Location,
    /// Azimuth, Elevation, Range, Range-rate
    /// :rtype: list
    pub aer_data: Vec<AzElRange>,
    /// :rtype: Duration
    pub sample_rate: Duration,
}

#[cfg_attr(feature = "python", pymethods)]
impl VisibilityArc {
    /// :rtype: Duration
    pub fn duration(&self) -> Duration {
        self.end_epoch() - self.start_epoch()
    }

    /// :rtype: Epoch
    pub fn start_epoch(&self) -> Epoch {
        self.rise.orbit.epoch
    }

    /// :rtype: Epoch
    pub fn end_epoch(&self) -> Epoch {
        self.fall.orbit.epoch
    }

    #[cfg(feature = "python")]
    fn __str__(&self) -> String {
        format!("{self}")
    }
    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        format!("{self}@{self:p}")
    }
}

impl fmt::Display for VisibilityArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) visible from {} until {} ({}) ({} AER data)",
            self.location_ref,
            self.location,
            self.start_epoch(),
            self.end_epoch(),
            self.duration(),
            self.aer_data.len()
        )
    }
}
