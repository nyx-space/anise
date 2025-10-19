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
use hifitime::{Duration, Epoch, TimeSeries, Unit};
use log::{debug, error, warn};
use rayon::prelude::*;
use std::sync::mpsc::channel;

use super::{AnalysisError, Event, StateSpec};
use crate::analysis::event::{EventArc, EventDetails};

impl Almanac {
    /// Find the exact state where the request event happens. The event function is expected to be monotone in the provided interval because we find the event using a Brent solver.
    /// This will only return _one_ event within the provided bracket.
    #[allow(clippy::identity_op)]
    pub fn report_event_once(
        &self,
        state_spec: &StateSpec,
        event: &Event,
        start_epoch: Epoch,
        end_epoch: Epoch,
    ) -> Result<EventDetails, AnalysisError> {
        let max_iter = 50;

        let has_converged =
            |xa: f64, xb: f64| (xa - xb).abs() <= event.epoch_precision.to_seconds();

        let xa_e = start_epoch;
        let xb_e = end_epoch;

        // Search in seconds (convert to epoch just in time)
        let mut xa = 0.0;
        let mut xb = (xb_e - xa_e).to_seconds();
        // Evaluate the event at both bounds

        let ya_state = state_spec.evaluate(xa_e, self)?;
        let yb_state = state_spec.evaluate(xb_e, self)?;
        let mut ya = event.eval(ya_state, self)?;
        let mut yb = event.eval(yb_state, self)?;

        // Check if we're already at the root
        if ya.abs() <= event.value_precision.abs() {
            debug!(
                "{event} -- found with |{ya}| < {} @ {xa_e}",
                event.value_precision.abs()
            );
            let prev_state = state_spec.evaluate(xa_e - event.epoch_precision, self).ok();
            let next_state = state_spec.evaluate(xa_e + event.epoch_precision, self).ok();

            return EventDetails::new(ya_state, ya, event, prev_state, next_state, self);
        } else if yb.abs() <= event.value_precision.abs() {
            debug!(
                "{event} -- found with |{yb}| < {} @ {xb_e}",
                event.value_precision.abs()
            );
            let prev_state = state_spec.evaluate(xb_e - event.epoch_precision, self).ok();
            let next_state = state_spec.evaluate(xb_e + event.epoch_precision, self).ok();

            return EventDetails::new(ya_state, ya, event, prev_state, next_state, self);
        }

        // The Brent solver, from the roots crate (sadly could not directly integrate it here)
        // Source: https://docs.rs/roots/0.0.5/src/roots/numerical/brent.rs.html#57-131

        let (mut xc, mut yc, mut xd) = (xa, ya, xa);
        let mut flag = true;

        for _ in 0..max_iter {
            if ya.abs() <= event.value_precision.abs() {
                // Can't fail, we got it earlier
                let epoch = xa_e + xa * Unit::Second;
                let state = state_spec.evaluate(epoch, self)?;
                debug!(
                    "{event} -- found with |{ya}| < {} @ {}",
                    event.value_precision.abs(),
                    state.epoch,
                );
                let prev_state = state_spec
                    .evaluate(epoch - event.epoch_precision, self)
                    .ok();
                let next_state = state_spec
                    .evaluate(epoch + event.epoch_precision, self)
                    .ok();

                return EventDetails::new(state, ya, event, prev_state, next_state, self);
            }
            if yb.abs() <= event.value_precision.abs() {
                // Can't fail, we got it earlier
                let epoch = xa_e + xb * Unit::Second;
                let state = state_spec.evaluate(epoch, self)?;
                debug!(
                    "{event} -- found with |{yb}| < {} @ {}",
                    event.value_precision.abs(),
                    state.epoch
                );
                let prev_state = state_spec
                    .evaluate(epoch - event.epoch_precision, self)
                    .ok();
                let next_state = state_spec
                    .evaluate(epoch + event.epoch_precision, self)
                    .ok();

                return EventDetails::new(state, ya, event, prev_state, next_state, self);
            }
            if has_converged(xa, xb) {
                // The event isn't in the bracket
                return Err(AnalysisError::EventNotFound {
                    start: start_epoch,
                    end: end_epoch,
                    event: Box::new(event.clone()),
                });
            }
            let mut s = if (ya - yc).abs() > f64::EPSILON && (yb - yc).abs() > f64::EPSILON {
                xa * yb * yc / ((ya - yb) * (ya - yc))
                    + xb * ya * yc / ((yb - ya) * (yb - yc))
                    + xc * ya * yb / ((yc - ya) * (yc - yb))
            } else {
                xb - yb * (xb - xa) / (yb - ya)
            };

            let cond1 = (s - xb) * (s - (3.0 * xa + xb) / 4.0) > 0.0;
            let cond2 = flag && (s - xb).abs() >= (xb - xc).abs() / 2.0;
            let cond3 = !flag && (s - xb).abs() >= (xc - xd).abs() / 2.0;
            let cond4 = flag && has_converged(xb, xc);
            let cond5 = !flag && has_converged(xc, xd);
            if cond1 || cond2 || cond3 || cond4 || cond5 {
                s = (xa + xb) / 2.0;
                flag = true;
            } else {
                flag = false;
            }

            let next_try = state_spec.clone().evaluate(xa_e + s * Unit::Second, self)?;
            let ys = event.eval(next_try, self)?;

            xd = xc;
            xc = xb;
            yc = yb;

            if ya * ys < 0.0 {
                // Root is bracketed between xa and s
                xb = s;
                yb = ys;
            } else {
                // Root is bracketed between s and xb
                xa = s;
                ya = ys;
            }

            // The `arrange` part from your code is to ensure that `b` is always the best guess.
            // This is a common practice in Brent solvers.
            if ya.abs() < yb.abs() {
                // Swap a and b
                std::mem::swap(&mut xa, &mut xb);
                std::mem::swap(&mut ya, &mut yb);
            }
        }
        error!("Brent solver failed after {max_iter} iterations");
        Err(AnalysisError::EventNotFound {
            start: start_epoch,
            end: end_epoch,
            event: Box::new(event.clone()),
        })
    }

    /// Report all of the states and event details where the provided event occurs.
    ///
    /// # Limitations
    /// This method uses a Brent solver, provides a superlinearity convergence (Golden ratio rate).
    /// If the function that defines the event is not unimodal, the event finder may not converge correctly.
    /// After the Brent solver is used, this function will check the median gap between events. Assuming most events are periodic,
    /// any gap whose median repetition is greater than 125% will be slow searches. This _typically_ finds all of the events ... but it
    /// may also add duplicates! To prevent reporting duplicate events, the found events are deduplicated if the same event is found
    /// within 3 times the epoch precision. For example, if the epoch precision is 100 ms, if three events are "found" within 300 ms of each other
    /// then only one of these three is preserved.
    ///
    /// # Heuristic detail
    /// The initial search step is 1% of the duration requested, if the heuristic is set to None.
    /// For example, if the trajectory is 100 days long, then we split the trajectory into 100 chunks of 1 day and see whether
    /// the event is in there. If the event happens twice or more times within 1% of the trajectory duration, only the _one_ of
    /// such events will be found.
    #[allow(clippy::identity_op)]
    pub fn report_events(
        &self,
        state_spec: &StateSpec,
        event: &Event,
        start_epoch: Epoch,
        end_epoch: Epoch,
        heuristic: Option<Duration>,
    ) -> Result<Vec<EventDetails>, AnalysisError> {
        if start_epoch == end_epoch {
            return Err(AnalysisError::EventNotFound {
                start: start_epoch,
                end: end_epoch,
                event: Box::new(event.clone()),
            });
        }
        let heuristic = heuristic.unwrap_or((end_epoch - start_epoch) / 100);
        debug!("searching for {event} with initial heuristic of {heuristic}");

        let (sender, receiver) = channel();

        let epochs: Vec<Epoch> = TimeSeries::inclusive(start_epoch, end_epoch, heuristic).collect();
        epochs.into_par_iter().for_each_with(sender, |s, epoch| {
            if let Ok(event_state) =
                self.report_event_once(state_spec, event, epoch, epoch + heuristic)
            {
                s.send(event_state).unwrap()
            };
        });
        let mut events: Vec<_> = receiver.iter().collect();

        if events.is_empty() {
            warn!("Heuristic failed to find any {event} event, using slower approach");
            // Crap, we didn't find the event.
            // Let's find the min and max of this event throughout the trajectory, and search around there.
            return self.report_events_slow(state_spec, event, start_epoch, end_epoch);
        }

        // Remove duplicates and reorder
        events.sort_by(|s1, s2| s1.orbit.epoch.partial_cmp(&s2.orbit.epoch).unwrap());
        events.dedup_by(|e1, e2| {
            (e1.orbit.epoch - e2.orbit.epoch).abs() <= event.epoch_precision * 3.0
        });

        let possible_gap_times = if events.len() > 1 {
            // We found some states, let's roughly check if we could have missed some events by searching for periodicity.
            let mut dt_bw_events = events
                .iter()
                .take(events.len() - 1)
                .zip(events.iter().skip(1))
                .map(|(first, second)| {
                    (
                        first.orbit.epoch,
                        second.orbit.epoch,
                        second.orbit.epoch - first.orbit.epoch,
                    )
                })
                .collect::<Vec<(Epoch, Epoch, Duration)>>();

            dt_bw_events.sort_by(|dt1, dt2| dt1.2.cmp(&dt2.2));

            let median_duration = dt_bw_events[dt_bw_events.len() / 2].2;

            dt_bw_events
                .iter()
                .copied()
                .filter(|(_start, _end, dt)| *dt > median_duration * 1.25)
                .collect::<Vec<(Epoch, Epoch, Duration)>>()
        } else {
            vec![(start_epoch, end_epoch, end_epoch - start_epoch)]
        };

        let prev_len = events.len();
        // Search specifically these gaps.
        for (gap_start, gap_end, _) in possible_gap_times {
            if let Ok(mut gapped_events) = self.report_events_slow(
                state_spec,
                event,
                gap_start + event.epoch_precision,
                gap_end - event.epoch_precision,
            ) {
                events.append(&mut gapped_events);
            }
        }

        if events.len() != prev_len {
            // Remove duplicates and reorder once more
            events.sort_by(|s1, s2| s1.orbit.epoch.partial_cmp(&s2.orbit.epoch).unwrap());
            // Dedupliate by the same event with three times the epoch precision,
            // because that would be one edge and then the other edge of the precision plus one event in the middle
            // from the slow search.
            events.dedup_by(|e1, e2| {
                (e1.orbit.epoch - e2.orbit.epoch).abs() <= event.epoch_precision * 3.0
            });
        }

        match events.len() {
            0 => debug!("event {event} not found"),
            1 => debug!("event {event} found once on {}", events[0].orbit.epoch),
            _ => {
                debug!(
                    "event {event} found {} times from {} until {}",
                    events.len(),
                    events.first().unwrap().orbit.epoch,
                    events.last().unwrap().orbit.epoch
                )
            }
        };

        Ok(events)
    }

    /// Slow approach to finding **all** of the events between two epochs. This will evaluate ALL epochs in between the two bounds.
    /// This approach is more robust, but more computationally demanding since it's O(N).
    #[allow(clippy::identity_op)]
    pub fn report_events_slow(
        &self,
        state_spec: &StateSpec,
        event: &Event,
        start_epoch: Epoch,
        end_epoch: Epoch,
    ) -> Result<Vec<EventDetails>, AnalysisError> {
        let step = event.epoch_precision;

        let (sender, receiver) = channel();

        let epochs: Vec<Epoch> = TimeSeries::inclusive(start_epoch, end_epoch, step).collect();

        epochs.into_par_iter().for_each_with(sender, |s, epoch| {
            if let Ok(state) = state_spec.evaluate(epoch, self) {
                if let Ok(this_eval) = event.eval(state, self) {
                    if this_eval.abs() < event.value_precision.abs() {
                        // This is an event!
                        let prev_state = state_spec
                            .evaluate(epoch - event.epoch_precision, self)
                            .ok();
                        let next_state = state_spec
                            .evaluate(epoch + event.epoch_precision, self)
                            .ok();

                        if let Ok(details) =
                            EventDetails::new(state, this_eval, event, prev_state, next_state, self)
                        {
                            if s.send(details).is_err() {
                                eprintln!("receiver for event search dropped");
                            }
                        }
                    }
                }
            }
        });

        let mut events: Vec<_> = receiver.iter().collect();

        // If there still isn't any match, report that the event was not found
        if events.is_empty() {
            return Err(AnalysisError::EventNotFound {
                start: start_epoch,
                end: end_epoch,
                event: Box::new(event.clone()),
            });
        }

        // Remove duplicates and reorder once more
        events.sort_by(|s1, s2| s1.orbit.epoch.partial_cmp(&s2.orbit.epoch).unwrap());
        Ok(events)
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
        // Step 1: Get all zero-crossings. We will completely ignore their reported 'edge' status, already sorted by time.
        let crossings = match self.report_events_slow(state_spec, event, start_epoch, end_epoch) {
            Ok(events) => events,
            Err(_) => {
                // No crossings were found. The only possibility for an arc is if the entire
                // trajectory is within the event.
                let start_orbit = state_spec.evaluate(start_epoch, self)?;
                let start_eval = event.eval(start_orbit, self)?;
                if start_eval > 0.0 {
                    let end_orbit = state_spec.evaluate(end_epoch, self)?;
                    let end_eval = event.eval(end_orbit, self)?;
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
                return Ok(Vec::new()); // The event never happens.
            }
        };

        // We have at least one crossing at this point.
        let init_crossing = crossings.first().unwrap();
        let mut is_inside_arc = init_crossing.value >= 0.0;

        let mut arcs = Vec::new();
        let mut rise: Option<EventDetails> = None;

        if is_inside_arc {
            if let Some(next_value) = init_crossing.next_value {
                if next_value < 0.0 {
                    // We start at the _end_ of this event.
                    // Compute the event details for this next epoch.
                    let fall = self.report_event_once(
                        state_spec,
                        event,
                        init_crossing.orbit.epoch + event.epoch_precision,
                        init_crossing.orbit.epoch + event.epoch_precision * 2,
                    )?;
                    arcs.push(EventArc {
                        rise: init_crossing.clone(),
                        fall,
                    });
                }
            }
            rise = Some(init_crossing.clone());
        }

        for crossing in crossings.iter().skip(1) {
            let event_value = crossing.value;
            if event_value >= 0.0 {
                // We're in an arc.
                if let Some(next_value) = crossing.next_value {
                    if next_value < 0.0 && rise.is_some() {
                        // At the next immediate step, the event ends, so this marks the end of the arc.
                        arcs.push(EventArc {
                            rise: rise.clone().unwrap(),
                            fall: crossing.clone(),
                        });
                        is_inside_arc = false;
                        continue; // Move onto the next crossing.
                    }
                    // else we're still in the arc on the next step.
                }
                // If we weren't in an arc, store this as the new rise.
                if !is_inside_arc {
                    rise = Some(crossing.clone());
                    is_inside_arc = true;
                }
            } else if is_inside_arc {
                // We aren't in an arc but we were until this event crossing.
                // Close out this arc.
                if let Some(rise) = rise.take() {
                    arcs.push(EventArc {
                        rise,
                        fall: crossing.clone(),
                    });
                }
                is_inside_arc = false;
            }
        }

        if is_inside_arc {
            if let Some(rise) = rise {
                if let Some(fall) = crossings.last().cloned() {
                    arcs.push(EventArc { rise, fall });
                }
            }
        }

        Ok(arcs)
    }
}
