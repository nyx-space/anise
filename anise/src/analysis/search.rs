/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{
    almanac::Almanac,
    analysis::{
        event::EventEdge,
        event_ops::find_arc_intersections,
        utils::{adaptive_step_scanner, brent_solver},
        AnalysisResult,
    },
};
use hifitime::Epoch;
use rayon::prelude::*;

use super::{AnalysisError, StateSpec};
use crate::analysis::event::{Condition, Event, EventArc, EventDetails};

impl Almanac {
    pub fn report_events(
        &self,
        state_spec: &StateSpec,
        event: &Event,
        start_epoch: Epoch,
        end_epoch: Epoch,
    ) -> Result<Vec<EventDetails>, AnalysisError> {
        if matches!(
            event.condition,
            Condition::Between(..) | Condition::LessThan(..) | Condition::GreaterThan(..)
        ) {
            return Err(AnalysisError::InvalidEventEval {
                err: format!(
                    "cannot report an individual event on an event like {:?}, use report_event_arcs",
                    event.condition
                ),
            });
        }

        match event.condition {
            Condition::Equals(_val) => {
                let f_eval = |epoch: Epoch| -> Result<f64, AnalysisError> {
                    let state = state_spec.evaluate(epoch, self)?;
                    event.eval(state, self)
                };

                // Find the zero crossings of the event itself
                let zero_crossings = adaptive_step_scanner(f_eval, event, start_epoch, end_epoch)?;
                // Find the exact events
                let mut events = zero_crossings
                    .par_iter()
                    .map(|(start_epoch, end_epoch)| -> AnalysisResult<Epoch> {
                        brent_solver(f_eval, event, *start_epoch, *end_epoch)
                    })
                    .filter_map(|brent_rslt| brent_rslt.ok())
                    .map(|epoch: Epoch| -> AnalysisResult<EventDetails> {
                        // Note that we don't call f_eval because it recomputes the state
                        // but it does not return it, and we need the state anyway.
                        let state = state_spec.evaluate(epoch, self)?;
                        let this_eval = event.eval(state, self)?;
                        let prev_state = state_spec
                            .evaluate(epoch - event.epoch_precision, self)
                            .ok();
                        let next_state = state_spec
                            .evaluate(epoch + event.epoch_precision, self)
                            .ok();

                        EventDetails::new(state, this_eval, event, prev_state, next_state, self)
                    })
                    .filter_map(|details_rslt| match details_rslt {
                        Ok(deets) => Some(deets),
                        Err(e) => {
                            eprintln!("{e} when building event details -- please file a bug");
                            None
                        }
                    })
                    .collect::<Vec<EventDetails>>();

                // Sort them using a _stable_ sort, which is faster than the unstable sort when trying are partially sorted (it's the case here).
                events.sort_by(|event_detail1, event_detail2| {
                    event_detail1.orbit.epoch.cmp(&event_detail2.orbit.epoch)
                });

                Ok(events)
            }
            Condition::Minimum() | Condition::Maximum() => {
                // Rebuild the event as an Equals, and therefore rebuild the closure.
                let boundary = Event {
                    scalar: event.scalar.clone(),
                    condition: Condition::Equals(0.0),
                    epoch_precision: event.epoch_precision,
                    ab_corr: event.ab_corr,
                };
                let f_eval = |epoch: Epoch| -> Result<f64, AnalysisError> {
                    let state = state_spec.evaluate(epoch, self)?;
                    boundary.eval(state, self)
                };
                let h_tiny = event.epoch_precision * 10.0;
                let f_deriv = |epoch: Epoch| -> Result<f64, AnalysisError> {
                    // Use central difference, handling boundary errors
                    let (y_next, y_prev) = match (f_eval(epoch + h_tiny), f_eval(epoch - h_tiny)) {
                        (Ok(next), Ok(prev)) => (next, prev),
                        // If one fails (e.g., at boundary), use forward/backward diff
                        (Ok(next), Err(_)) => (next, f_eval(epoch)?),
                        (Err(_), Ok(prev)) => (f_eval(epoch)?, prev),
                        (Err(_), Err(_)) => (0.0, 0.0), // Can't evaluate, assume no change
                    };
                    let h_sec = h_tiny.to_seconds() * 2.0;
                    if h_sec.abs() < 1e-12 {
                        return Ok(0.0); // Avoid div by zero
                    }
                    Ok((y_next - y_prev) / h_sec)
                };

                // Find the extremas, i.e. when the derivative is a zero crossing.
                let extremas = adaptive_step_scanner(f_deriv, &boundary, start_epoch, end_epoch)?;

                // Find the exact events by running the Brent solver on the derivative.
                let mut events = extremas
                    .par_iter()
                    .map(|(start_epoch, end_epoch)| -> AnalysisResult<Epoch> {
                        brent_solver(f_deriv, &boundary, *start_epoch, *end_epoch)
                    })
                    .filter_map(|brent_rslt| brent_rslt.ok())
                    .map(|epoch: Epoch| -> AnalysisResult<EventDetails> {
                        // Find the actual event extrema at this time by evaluating the event
                        // (not its derivative like we have been).
                        let state = state_spec.evaluate(epoch, self)?;
                        let this_eval = boundary.eval(state, self)?;
                        let prev_state = state_spec
                            .evaluate(epoch - event.epoch_precision, self)
                            .ok();
                        let next_state = state_spec
                            .evaluate(epoch + event.epoch_precision, self)
                            .ok();

                        EventDetails::new(state, this_eval, &boundary, prev_state, next_state, self)
                    })
                    .filter_map(|details_rslt| match details_rslt {
                        Ok(deets) => Some(deets),
                        Err(e) => {
                            eprintln!("{e} when building event details -- please file a bug");
                            None
                        }
                    })
                    .filter(|details| match event.condition {
                        // An extremum at the boundary of the search interval might be classified as
                        // Rising/Falling instead of LocalMin/LocalMax by `EventDetails::new` because
                        // one of the neighbors is missing. We include them here to catch those cases.
                        Condition::Minimum() => {
                            matches!(details.edge, EventEdge::LocalMin | EventEdge::Rising)
                        }
                        Condition::Maximum() => {
                            matches!(details.edge, EventEdge::LocalMax | EventEdge::Falling)
                        }
                        _ => unreachable!(),
                    })
                    .collect::<Vec<EventDetails>>();

                // Sort them using a _stable_ sort, which is faster than the unstable sort when trying are partially sorted (it's the case here).
                events.sort_by(|event_detail1, event_detail2| {
                    event_detail1.orbit.epoch.cmp(&event_detail2.orbit.epoch)
                });

                Ok(events)
            }
            Condition::Between(..) | Condition::LessThan(..) | Condition::GreaterThan(..) => {
                unreachable!()
            }
        }
    }

    /// Find all event arcs, i.e. the start and stop time of when a given event occurs. This function
    /// calls the memory and computationally intensive [report_events_slow] function.
    pub fn report_event_arcs(
        &self,
        state_spec: &StateSpec,
        event: &Event,
        start_epoch: Epoch,
        end_epoch: Epoch,
    ) -> Result<Vec<EventArc>, AnalysisError> {
        if matches!(
            event.condition,
            Condition::Equals(..) | Condition::Minimum() | Condition::Maximum()
        ) {
            return Err(AnalysisError::InvalidEventEval {
                err: format!(
                    "cannot report event arcs on an individual event like {:?}, use report_events",
                    event.condition
                ),
            });
        }

        match event.condition {
            Condition::Between(min_val, max_val) => {
                let lt_event = Event {
                    scalar: event.scalar.clone(),
                    condition: Condition::LessThan(max_val),
                    epoch_precision: event.epoch_precision,
                    ab_corr: event.ab_corr,
                };

                let gt_event = Event {
                    scalar: event.scalar.clone(),
                    condition: Condition::GreaterThan(min_val),
                    epoch_precision: event.epoch_precision,
                    ab_corr: event.ab_corr,
                };

                let min_boundary_event = Event {
                    scalar: event.scalar.clone(),
                    condition: Condition::Equals(min_val),
                    epoch_precision: event.epoch_precision,
                    ab_corr: event.ab_corr,
                };

                let max_boundary_event = Event {
                    scalar: event.scalar.clone(),
                    condition: Condition::Equals(max_val),
                    epoch_precision: event.epoch_precision,
                    ab_corr: event.ab_corr,
                };

                // We could probably run these in parallel but the overhead to spin up a thread
                // is likely greater than just running both calls sequentially.
                let lt_events =
                    self.report_event_arcs(state_spec, &lt_event, start_epoch, end_epoch)?;

                let gt_events =
                    self.report_event_arcs(state_spec, &gt_event, start_epoch, end_epoch)?;

                // Compute the start and stop times when both conditions are true, i.e. the event is greater
                // than the min value and less than the max value
                let intersection = find_arc_intersections(vec![lt_events, gt_events]);
                let mut arcs = Vec::with_capacity(intersection.len());
                // Rebuild the EventDetails and EventArcs for each.
                for (intersect_start, intersect_end) in intersection {
                    let start_orbit = state_spec.evaluate(intersect_start, self)?;
                    let end_orbit = state_spec.evaluate(intersect_end, self)?;

                    let start_eval = min_boundary_event.eval(start_orbit, self)?;
                    let end_eval = max_boundary_event.eval(start_orbit, self)?;

                    // We don't need the prev/next evaluations because we know that the event is rising and falling
                    // via the intersection call.
                    arcs.push(EventArc {
                        rise: EventDetails {
                            orbit: start_orbit,
                            edge: EventEdge::Rising,
                            value: start_eval,
                            prev_value: None,
                            next_value: None,
                            pm_duration: event.epoch_precision,
                            repr: min_boundary_event.eval_string(start_orbit, self)?,
                        },
                        fall: EventDetails {
                            orbit: end_orbit,
                            edge: EventEdge::Falling,
                            value: end_eval,
                            prev_value: None,
                            next_value: None,
                            pm_duration: event.epoch_precision,
                            repr: max_boundary_event.eval_string(end_orbit, self)?,
                        },
                    })
                }

                Ok(arcs)
            }
            Condition::LessThan(val) | Condition::GreaterThan(val) => {
                let boundary_event = Event {
                    scalar: event.scalar.clone(),
                    condition: Condition::Equals(val),
                    epoch_precision: event.epoch_precision,
                    ab_corr: event.ab_corr,
                };

                let crossings =
                    self.report_events(state_spec, &boundary_event, start_epoch, end_epoch)?;

                if crossings.is_empty() {
                    // We never cross the boundary, so check if we're in the boundary at the start or not.
                    let start_orbit = state_spec.evaluate(start_epoch, self)?;
                    let start_eval = boundary_event.eval(start_orbit, self)?;
                    let end_orbit = state_spec.evaluate(end_epoch, self)?;
                    let end_eval = boundary_event.eval(end_orbit, self)?;
                    let start_inside = start_eval >= 0.0;
                    // In the case of both angles and other scalars, the evaluation will be negative if the current
                    // value is less the desired value; positive otherwise.
                    // If the user is seeking when the event is LessThan X, and start_eval >= 0.0, it means that there
                    // are NO event windows that match the desired value.
                    let no_events = (matches!(event.condition, Condition::LessThan(..))
                        && start_inside)
                        || (matches!(event.condition, Condition::GreaterThan(..)) && !start_inside);

                    if no_events {
                        return Ok(Vec::new());
                    } else {
                        // We're less than the desired value the whole time.
                        let rise = EventDetails::new(
                            start_orbit,
                            start_eval,
                            event,
                            None,
                            Some(end_orbit),
                            self,
                        )?;
                        let fall = EventDetails::new(
                            end_orbit,
                            end_eval,
                            event,
                            Some(start_orbit),
                            None,
                            self,
                        )?;
                        return Ok(vec![EventArc { rise, fall }]);
                    }
                }

                // We have at least one crossing at this point.
                let start_orbit = state_spec.evaluate(start_epoch, self)?;
                let start_eval = boundary_event.eval(start_orbit, self)?;

                // So we can employ the same logic, we're using signum checks directly.
                let desired_sign = if matches!(event.condition, Condition::LessThan(..)) {
                    -1.0
                } else {
                    1.0
                };

                let mut is_inside_arc = start_eval.signum() == desired_sign;

                let mut arcs = Vec::new();

                let mut rise: Option<EventDetails> = None;

                // If we start *inside* the arc, create a "rise" event for the start.
                if is_inside_arc {
                    let start_orbit = state_spec.evaluate(start_epoch, self)?;
                    let start_eval = boundary_event.eval(start_orbit, self)?;
                    let next_orbit = state_spec.evaluate(start_epoch + event.epoch_precision, self);
                    let start_details = EventDetails::new(
                        start_orbit,
                        start_eval,
                        &boundary_event,
                        None,
                        next_orbit.ok(),
                        self,
                    )?;
                    rise = Some(start_details);
                }

                // Loop over *all* crossings. Each crossing is a state flip.
                for crossing in crossings {
                    if is_inside_arc {
                        // We were IN an arc, this crossing is the FALL.
                        // Close the arc.
                        arcs.push(EventArc {
                            rise: rise.take().unwrap(), // We must have had a rise
                            fall: crossing,
                        });
                        is_inside_arc = false;
                    } else {
                        // We were OUT of an arc, this crossing is the RISE.
                        // Start a new arc.
                        rise = Some(crossing);
                        is_inside_arc = true;
                    }
                }
                // After the loop, if we are *still* in an arc, it must continue until `end_epoch`.
                if is_inside_arc {
                    if let Some(rise) = rise.take() {
                        println!("will eval {end_epoch}");
                        let end_orbit = state_spec.evaluate(end_epoch, self)?;
                        dbg!(end_orbit);
                        let end_eval = boundary_event.eval(end_orbit, self)?;
                        dbg!(end_eval);
                        let prev_orbit =
                            state_spec.evaluate(end_epoch - event.epoch_precision, self);
                        dbg!(&prev_orbit);

                        let fall_details = EventDetails::new(
                            end_orbit,
                            end_eval,
                            &boundary_event,
                            prev_orbit.ok(),
                            None,
                            self,
                        )?;

                        arcs.push(EventArc {
                            rise,
                            fall: fall_details,
                        });
                    }
                }
                Ok(arcs)
            }
            Condition::Equals(..) | Condition::Minimum() | Condition::Maximum() => unreachable!(),
        }
    }
}
