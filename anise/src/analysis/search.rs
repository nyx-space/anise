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
use std::fs::File; // Added for logging
use std::io::Write; // Added for logging
use std::sync::mpsc::channel;

use super::{AnalysisError, Event, StateSpec};
use crate::analysis::event::{EventArc, EventDetails};

impl Almanac {
    /// Core Brent solver, refactored to operate on a generic closure `eval_fn`.
    /// This allows us to find the root of *any* function f(t), not just an `Event`.
    fn report_value_once<F>(
        &self,
        start_epoch: Epoch,
        end_epoch: Epoch,
        epoch_precision: Duration,
        value_precision: f64,
        eval_fn: &F,
        event: &Event,
    ) -> Result<Epoch, AnalysisError>
    where
        F: Fn(Epoch) -> Result<f64, AnalysisError> + Sync,
    {
        let max_iter = 50;
        let value_precision_abs = value_precision.abs();
        let epoch_precision_sec = epoch_precision.to_seconds();

        let has_converged = |xa: f64, xb: f64| (xa - xb).abs() <= epoch_precision_sec;

        let xa_e = start_epoch;
        let xb_e = end_epoch;

        // Search in seconds (convert to epoch just in time)
        let mut xa = 0.0;
        let mut xb = (xb_e - xa_e).to_seconds();

        // Evaluate the event at both bounds
        let mut ya = eval_fn(xa_e)?;
        let mut yb = eval_fn(xb_e)?;

        // Check if we're already at the root
        if ya.abs() <= value_precision_abs {
            debug!("found with |{ya}| < {value_precision_abs} @ {xa_e}");
            return Ok(xa_e);
        } else if yb.abs() <= value_precision_abs {
            debug!("found with |{yb}| < {value_precision_abs} @ {xb_e}");
            return Ok(xb_e);
        }

        let (mut xc, mut yc, mut xd) = (xa, ya, xa);
        let mut flag = true;

        for _ in 0..max_iter {
            if ya.abs() <= value_precision_abs {
                let epoch = xa_e + xa * Unit::Second;
                debug!("found with |{ya}| < {value_precision_abs} @ {epoch}");
                return Ok(epoch);
            }
            if yb.abs() <= value_precision_abs {
                let epoch = xa_e + xb * Unit::Second;
                debug!("found with |{yb}| < {value_precision_abs} @ {epoch}");
                return Ok(epoch);
            }
            if has_converged(xa, xb) {
                return Err(AnalysisError::EventNotFound {
                    start: start_epoch,
                    end: end_epoch,
                    event: Box::new(event.clone()),
                });
            }

            let mut s_newton: Option<f64> = None;
            let h = epoch_precision_sec;
            if h > f64::EPSILON {
                let xb_epoch = xa_e + xb * Unit::Second;
                if let (Ok(y_prev), Ok(y_next)) = (
                    eval_fn(xb_epoch - epoch_precision),
                    eval_fn(xb_epoch + epoch_precision),
                ) {
                    let deriv = (y_next - y_prev) / (2.0 * h);
                    if deriv.abs() > 1e-10 {
                        s_newton = Some(xb - yb / deriv);
                    }
                }
            }

            let mut s = if (ya - yc).abs() > f64::EPSILON && (yb - yc).abs() > f64::EPSILON {
                xa * yb * yc / ((ya - yb) * (ya - yc))
                    + xb * ya * yc / ((yb - ya) * (yb - yc))
                    + xc * ya * yb / ((yc - ya) * (yc - yb))
            } else if let Some(newton_step) = s_newton {
                newton_step
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

            let ys = eval_fn(xa_e + s * Unit::Second)?;

            xd = xc;
            xc = xb;
            yc = yb;

            if ya * ys < 0.0 {
                xb = s;
                yb = ys;
            } else {
                xa = s;
                ya = ys;
            }

            if ya.abs() < yb.abs() {
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

    /// Core adaptive step scanner, refactored to operate on a generic closure `eval_fn`.
    /// Returns a list of "coarse brackets" where a sign change was detected.
    #[allow(clippy::identity_op)]
    fn adaptive_scan<F>(
        &self,
        start_epoch: Epoch,
        end_epoch: Epoch,
        min_step_sec: f64,
        tol: f64,
        eval_fn: &F,
        log_sender: Option<&std::sync::mpsc::Sender<String>>, // MODIFIED: Added log sender
    ) -> Result<Vec<(Epoch, Epoch)>, AnalysisError>
    where
        F: Fn(Epoch) -> Result<f64, AnalysisError> + Sync, // MODIFIED: Added Sync bound for derivative calc
    {
        let mut brackets = Vec::new();
        let total_duration_sec = (end_epoch - start_epoch).to_seconds();
        let max_step_sec = (total_duration_sec / 10.0).max(min_step_sec * 100.0);
        let mut h_sec = (min_step_sec * 1000.0).min(max_step_sec);
        let safety_factor = 0.9;
        let power = 0.25; // Use 4th-order-like power for stability

        let mut t_prev_epoch = start_epoch;
        let mut y_prev = eval_fn(t_prev_epoch)?;

        while t_prev_epoch < end_epoch {
            let time_remaining_sec = (end_epoch - t_prev_epoch).to_seconds();
            if time_remaining_sec <= min_step_sec {
                break;
            }

            let mut h_sec_clamped = h_sec.max(min_step_sec);

            // Ensure we don't overshoot the end
            if h_sec_clamped > time_remaining_sec {
                h_sec_clamped = time_remaining_sec;
            }

            let t_next_epoch = t_prev_epoch + h_sec_clamped * Unit::Second;

            // MODIFIED: Moved logging logic inside the Ok() arm
            let y_next = match eval_fn(t_next_epoch) {
                Ok(y) => {
                    // --- ADDED: Logging block ---
                    if let Some(sender) = log_sender {
                        let h_dur = min_step_sec * Unit::Second;
                        let deriv = if min_step_sec > f64::EPSILON {
                            // Central difference for derivative
                            let y_plus_h = eval_fn(t_next_epoch + h_dur);
                            let y_minus_h = eval_fn(t_next_epoch - h_dur);
                            match (y_plus_h, y_minus_h) {
                                (Ok(y_p), Ok(y_m)) => (y_p - y_m) / (2.0 * min_step_sec),
                                _ => f64::NAN, // Handle eval errors during derivative calc
                            }
                        } else {
                            f64::NAN // Step is too small
                        };

                        // Format as CSV: epoch (MJD TAI days), y_value, derivative
                        // Using MJD TAI days as it's a stable f64 representation
                        let epoch_mjd = t_next_epoch.to_mjd_tai_days();
                        let csv_line = format!("{:.10}, {:.10}, {:.10}", epoch_mjd, y, deriv);

                        // Send the data. We don't care if it fails (e.g., receiver dropped).
                        let _ = sender.send(csv_line);
                    }
                    // --- END: Logging block ---
                    y // Return y to be assigned to y_next
                }
                Err(e) => {
                    warn!("Eval function failed during adaptive scan at {t_next_epoch}: {e}");
                    break;
                }
            };

            let error = (y_next - y_prev).abs();
            // A large jump (e.g., 360-deg wrap) is not a bracket.
            // We set a threshold: if the jump is > 1000x the
            // scanner tolerance, assume discontinuity.
            let is_discontinuous = error > (tol * 1000.0);

            if y_prev * y_next < 0.0 && !is_discontinuous {
                brackets.push((t_prev_epoch, t_next_epoch));
            }

            // This was causing the exponential step growth in your logs.
            // The `else` block handles both growth and shrinkage correctly.
            let h_factor = (tol / error).powf(power);
            let h_factor_clamped = h_factor.clamp(0.1, 5.0); // Clamp factor

            let proposed_step_s = if h_factor_clamped < 1.0 {
                h_sec * h_factor_clamped * safety_factor
            } else {
                h_sec * h_factor_clamped
            };

            h_sec = proposed_step_s.max(min_step_sec).min(max_step_sec);
            t_prev_epoch = t_next_epoch;
            y_prev = y_next;
        }

        Ok(brackets)
    }

    /// Public wrapper. Finds the exact state of an event by wrapping `report_value_once`.
    #[allow(clippy::identity_op)]
    pub fn report_event_once(
        &self,
        state_spec: &StateSpec,
        event: &Event,
        start_epoch: Epoch,
        end_epoch: Epoch,
    ) -> Result<EventDetails, AnalysisError> {
        // Create a closure for the "real event" function f(t)
        let f_real = |epoch: Epoch| -> Result<f64, AnalysisError> {
            let state = state_spec.evaluate(epoch, self)?;
            event.eval(state, self)
        };

        // Call the generic solver
        let root_epoch = self.report_value_once(
            start_epoch,
            end_epoch,
            event.epoch_precision,
            event.value_precision,
            &f_real,
            event,
        )?;

        // Build the EventDetails at the precise root epoch
        let state = state_spec.evaluate(root_epoch, self)?;
        let value = event.eval(state, self)?;
        let prev_state = state_spec
            .evaluate(root_epoch - event.epoch_precision, self)
            .ok();
        let next_state = state_spec
            .evaluate(root_epoch + event.epoch_precision, self)
            .ok();

        EventDetails::new(state, value, event, prev_state, next_state, self)
    }

    /// Report all of the states and event details where the provided event occurs.
    ///
    /// This function implements a robust, multi-pass approach:
    /// 1. Scan for roots of the event's *derivative* to find all extrema (mins/maxs).
    /// 2. Refine these extrema using the Brent solver to get their precise times.
    /// 3. Use these times as "fence posts" to create windows of *guaranteed monotony*.
    /// 4. Check each monotone window for a sign change.
    /// 5. Find the precise root within any window that has a sign change.
    #[allow(clippy::identity_op)]
    pub fn report_events(
        &self,
        state_spec: &StateSpec,
        event: &Event,
        start_epoch: Epoch,
        end_epoch: Epoch,
    ) -> Result<Vec<EventDetails>, AnalysisError> {
        if start_epoch == end_epoch {
            return Err(AnalysisError::EventNotFound {
                start: start_epoch,
                end: end_epoch,
                event: Box::new(event.clone()),
            });
        }
        debug!("searching for {event} with adaptive window scanner...");

        let min_step_sec = event.epoch_precision.to_seconds();

        // --- Create Closures ---
        // f(t) -> real event value
        // This closure captures references, so it's `Sync` and can be shared across threads.
        let f_real = |epoch: Epoch| -> Result<f64, AnalysisError> {
            let state = state_spec.evaluate(epoch, self)?;
            event.eval(state, self)
        };

        // --- Pass 1: Parallel Adaptive Scan for Roots (Roots of f_real) ---
        debug!("coarse scan for roots (f = 0)");
        // Use a loose tolerance for the event scan.
        // This is the most important parameter to tune.
        // For TA (-180 to 180 range), 1.0 is good.
        // For Eclipse (0 to 1 range), 0.1 might be good.
        let scan_tol = 1e-4;

        // Split the total duration into a number of chunks based on available threads
        let n_threads = rayon::current_num_threads().max(1) * 4;
        let total_duration = (end_epoch - start_epoch).to_seconds();
        // Ensure chunk size is reasonable, not smaller than 10x min step
        let chunk_size_sec = (total_duration / n_threads as f64).max(min_step_sec * 10.0);
        let mut chunks = Vec::new();
        let mut t_chunk_start = start_epoch;
        while t_chunk_start < end_epoch {
            let t_chunk_end = (t_chunk_start + chunk_size_sec * Unit::Second).min(end_epoch);
            chunks.push((t_chunk_start, t_chunk_end));
            t_chunk_start = t_chunk_end;
        }

        let (sender, receiver) = channel();

        // --- ADDED: Create logging channel ---
        let (log_sender, log_receiver) = channel::<String>();
        let log_file_path = "adaptive_scan_log.csv";
        // --- END: Create logging channel ---

        // Run the adaptive scan on each chunk in parallel
        chunks
            .into_par_iter()
            // MODIFIED: Pass both bracket sender and log sender
            .for_each_with(
                (sender, log_sender.clone()),
                |(s, log_s), (chunk_start, chunk_end)| {
                    if let Ok(brackets) = self.adaptive_scan(
                        chunk_start,
                        chunk_end,
                        min_step_sec,
                        scan_tol,
                        &f_real,
                        Some(log_s),
                    ) {
                        s.send(brackets).unwrap();
                    }
                },
            );

        // --- ADDED: Drop original log sender to close channel ---
        drop(log_sender);

        // --- ADDED: Collect and write logs ---
        debug!("Writing adaptive_scan log to {log_file_path}...");
        let mut log_data: Vec<String> = log_receiver.iter().collect();

        // Sort the data by epoch (the first column).
        // This is CRITICAL as data arrives out of order from parallel threads.
        log_data.sort_by(|a, b| {
            let a_epoch = a
                .split(',')
                .next()
                .unwrap_or("0.0")
                .parse::<f64>()
                .unwrap_or(0.0);
            let b_epoch = b
                .split(',')
                .next()
                .unwrap_or("0.0")
                .parse::<f64>()
                .unwrap_or(0.0);
            a_epoch
                .partial_cmp(&b_epoch)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Write to file
        match File::create(log_file_path) {
            Ok(mut file) => {
                // Write header
                if writeln!(file, "epoch_mjd_tai,f_value,f_derivative").is_err() {
                    warn!("Failed to write header to {log_file_path}");
                }
                // Write data
                for line in log_data {
                    if writeln!(file, "{}", line).is_err() {
                        warn!("Failed to write line to {log_file_path}");
                    }
                }
                debug!("Log file written successfully.");
            }
            Err(e) => {
                warn!("Failed to create log file {log_file_path}: {e}");
            }
        }
        // --- END: Log collection ---

        // Collect the lists of brackets from all chunks and flatten
        let all_bracket_lists: Vec<Vec<(Epoch, Epoch)>> = receiver.iter().collect();
        let root_brackets: Vec<(Epoch, Epoch)> = all_bracket_lists.into_iter().flatten().collect();
        // --- End Parallel Scan ---

        if root_brackets.is_empty() {
            return Ok(Vec::new());
        }

        // --- Pass 2: Refine Roots (Find Precise Times) ---
        debug!("refining {} root brackets...", root_brackets.len());
        let (sender, receiver) = channel();

        root_brackets
            .into_par_iter()
            .for_each_with(sender, |s, (start, end)| {
                if let Ok(epoch) = self.report_value_once(
                    start,
                    end,
                    event.epoch_precision,
                    event.value_precision,
                    &f_real,
                    event,
                ) {
                    s.send(epoch).unwrap();
                }
            });
        let root_epochs: Vec<Epoch> = receiver.iter().collect();
        if root_epochs.is_empty() {
            debug!("Pass 2 found no roots.");
            return Ok(Vec::new());
        }

        // --- Pass 3: Build EventDetails from Root Epochs ---
        debug!("building EventDetails for {} roots", root_epochs.len());
        let (sender, receiver) = channel();
        root_epochs
            .into_par_iter()
            .for_each_with(sender, |s, epoch| {
                // We can't use report_event_once directly as it re-solves.
                // We just need to build the details.
                if let Ok(state) = state_spec.evaluate(epoch, self) {
                    if let Ok(value) = event.eval(state, self) {
                        let prev_state = state_spec
                            .evaluate(epoch - event.epoch_precision, self)
                            .ok();
                        let next_state = state_spec
                            .evaluate(epoch + event.epoch_precision, self)
                            .ok();
                        if let Ok(details) =
                            EventDetails::new(state, value, event, prev_state, next_state, self)
                        {
                            s.send(details).unwrap();
                        }
                    }
                }
            });

        let mut events: Vec<_> = receiver.iter().collect();

        // Final sort and dedupe
        events.sort_by(|s1, s2| s1.orbit.epoch.partial_cmp(&s2.orbit.epoch).unwrap());

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
        let crossings = match self.report_events(state_spec, event, start_epoch, end_epoch) {
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
                    // else we're still in an arc on the next step.
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
