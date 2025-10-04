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
    astro::Aberration,
    prelude::{Almanac, Orbit},
};
use hifitime::{Duration, Epoch, Unit};
use log::warn;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Defines a state parameter event finder
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Event {
    /// The state parameter
    pub scalar: ScalarExpr,
    /// The desired self.desired_value, must be in the same units as the state parameter
    pub desired_value: f64,
    /// The duration precision after which the solver will report that it cannot find any more precise
    pub epoch_precision: Duration,
    /// The precision on the desired value. Avoid setting it too low (e.g. 1e-3 degrees) because it may
    /// cause events to be skipped if the value is not found within the epoch precision.
    pub value_precision: f64,
    pub ab_corr: Option<Aberration>,
}

impl Event {
    /// Apoapsis event finder
    pub fn apoapsis() -> Self {
        Event {
            scalar: ScalarExpr::Element(OrbitalElement::TrueAnomaly),
            desired_value: 180.0,
            epoch_precision: Unit::Second * 0.1,
            value_precision: 1e-2,
            ab_corr: None,
        }
    }

    /// Periapsis event finder
    pub fn periapsis() -> Self {
        Event {
            scalar: ScalarExpr::Element(OrbitalElement::TrueAnomaly),
            desired_value: 0.0,
            epoch_precision: Unit::Second * 0.1,
            value_precision: 1e-2,
            ab_corr: None,
        }
    }

    pub fn eval(&self, orbit: Orbit, almanac: &Almanac) -> Result<f64, AnalysisError> {
        let current_val = self.scalar.evaluate(orbit, self.ab_corr, almanac)?;

        // Check if the scalar is an angle that needs special handling
        let is_angle = match self.scalar {
            ScalarExpr::Element(oe) => oe.is_angle(),
            ScalarExpr::AngleBetween { a: _, b: _ }
            | ScalarExpr::BetaAngle
            | ScalarExpr::SunAngle { observer_id: _ }
            | ScalarExpr::AzimuthFromLocation {
                location_id: _,
                obstructing_body: _,
            }
            | ScalarExpr::ElevationFromLocation {
                location_id: _,
                obstructing_body: _,
            } => true,
            _ => false,
        };

        if is_angle {
            // Use the arctan function because it's smooth around zero, but convert back to degrees
            // for the comparison.

            let current_rad = current_val.to_radians();
            let desired_rad = self.desired_value.to_radians();

            // Convert the angles to points on a unit circle
            let (cur_sin, cur_cos) = current_rad.sin_cos();
            let (des_sin, des_cos) = desired_rad.sin_cos();

            // Calculate the difference vector and find its angle with atan2.
            // This will be zero only when the angles are identical.
            let y = cur_sin * des_cos - cur_cos * des_sin; // sin(current - desired)
            let x = cur_cos * des_cos + cur_sin * des_sin; // cos(current - desired)

            Ok(y.atan2(x).to_degrees())
        } else {
            // For all non-angular scalars, use the original logic
            Ok(current_val - self.desired_value)
        }
    }

    // Evaluation of event crossing, must return whether the condition happened between between both states.
    pub fn eval_crossing(
        &self,
        prev_state: Orbit,
        next_state: Orbit,
        almanac: &Almanac,
    ) -> Result<bool, AnalysisError> {
        let prev = self.eval(prev_state, almanac)?;
        let next = self.eval(next_state, almanac)?;

        Ok(prev * next < 0.0)
    }

    pub fn eval_string(&self, orbit: Orbit, almanac: &Almanac) -> Result<String, AnalysisError> {
        let val = self.eval(orbit, almanac)?;

        if self.desired_value.abs() > 1e3 {
            Ok(format!(
                "|{} - {:e}| = {val:e} on {}",
                self.scalar, self.desired_value, orbit.epoch
            ))
        } else if self.desired_value > self.value_precision {
            Ok(format!(
                "|{} - {:.3}| = {val:.3} on {}",
                self.scalar, self.desired_value, orbit.epoch
            ))
        } else {
            Ok(format!("|{}| = {val:.3} on {}", self.scalar, orbit.epoch))
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.scalar)?;
        if self.desired_value.abs() > 1e3 {
            write!(
                f,
                " = {:e} (± {:e})",
                self.desired_value, self.value_precision,
            )
        } else {
            write!(f, " = {} (± {})", self.desired_value, self.value_precision)
        }
    }
}
/// Enumerates the possible edges of an event in a trajectory.
///
/// `EventEdge` is used to describe the nature of a trajectory event, particularly in terms of its temporal dynamics relative to a specified condition or threshold. This enum helps in distinguishing whether the event is occurring at a rising edge, a falling edge, or if the edge is unclear due to insufficient data or ambiguous conditions.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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

/// Represents the details of an event occurring along a trajectory.
///
/// `EventDetails` encapsulates the state at which a particular event occurs in a trajectory, along with additional information about the nature of the event. This struct is particularly useful for understanding the dynamics of the event, such as whether it represents a rising or falling edge, or if the edge is unclear.
///
/// # Generics
/// S: Interpolatable - A type that represents the state of the trajectory. This type must implement the `Interpolatable` trait, ensuring that it can be interpolated and manipulated according to the trajectory's requirements.
#[derive(Clone, PartialEq)]
pub struct EventDetails {
    /// The state of the trajectory at the found event.
    pub orbit: Orbit,
    /// Indicates whether the event is a rising edge, falling edge, or unclear. This helps in understanding the direction of change at the event point.
    pub edge: EventEdge,
    /// Numerical evaluation of the event condition, e.g. if seeking the apoapsis, this returns the near zero
    pub value: f64,
    /// Numertical evaluation of the event condition one epoch step before the found event (used to compute the rising/falling edge).
    pub prev_value: Option<f64>,
    /// Numertical evaluation of the event condition one epoch step after the found event (used to compute the rising/falling edge).
    pub next_value: Option<f64>,
    /// Precision of the epoch for this value
    pub pm_duration: Duration,
    // Store the representation of this event as a string because we can't move or clone the event reference
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

#[derive(Clone, PartialEq)]
pub struct EventArc {
    pub rise: EventDetails,
    pub fall: EventDetails,
}

impl EventArc {
    pub fn duration(&self) -> Duration {
        self.end_epoch() - self.start_epoch()
    }

    pub fn start_epoch(&self) -> Epoch {
        self.rise.orbit.epoch
    }

    pub fn end_epoch(&self) -> Epoch {
        self.fall.orbit.epoch
    }
}

impl fmt::Display for EventArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} until {} (lasts {})",
            self.rise.orbit.epoch,
            self.fall.orbit.epoch,
            self.fall.orbit.epoch - self.rise.orbit.epoch
        )
    }
}
impl fmt::Debug for EventArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} until {}", self.rise, self.fall)
    }
}
