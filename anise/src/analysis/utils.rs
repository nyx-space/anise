/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{AnalysisError, Event};
use hifitime::{Epoch, Unit};

/// A Brent method's root finder to find where, within the provided start/stop epoch brackets, the evaluator function evaluates to zero.
/// The event's epoch precision is used for convergence.
/// This method finds at most one zero crossing, so ensure that only one such event happens in the provided bracket.
/// This method will error immediately if the evaluator evaluates to the same sign on both sides of the bracket.
pub fn brent_solver<F>(
    evaluator: F,
    event: &Event,
    start_epoch: Epoch,
    end_epoch: Epoch,
) -> Result<Epoch, AnalysisError>
where
    F: Fn(Epoch) -> Result<f64, AnalysisError>,
{
    let max_iter = 50;

    // Convergence criteria is strictly on the epoch bracketing.
    let has_converged = |xa: f64, xb: f64| (xa - xb).abs() <= event.epoch_precision.to_seconds();

    let xa_e = start_epoch;
    let xb_e = end_epoch;

    // Search in seconds (convert to epoch just in time)
    let mut xa = 0.0;
    let mut xb = (xb_e - xa_e).to_seconds();

    // Evaluate the event at both bounds
    let mut ya = evaluator(xa_e)?;
    if ya.abs() <= f64::EPSILON {
        return Ok(xa_e);
    }
    let mut yb = evaluator(xb_e)?;
    if yb.abs() <= f64::EPSILON {
        return Ok(xb_e);
    }

    // If the root is not bracketed, there is no point in iterating, return an error.
    if ya * yb >= 0.0 {
        // The event isn't in the bracket
        return Err(AnalysisError::EventNotFound {
            start: start_epoch,
            end: end_epoch,
            event: Box::new(event.clone()),
        });
    }

    let (mut xc, mut yc, mut xd) = (xa, ya, xa);
    let mut flag = true;

    for _ in 0..max_iter {
        if has_converged(xa, xb) {
            return Ok(xa_e + xb * Unit::Second);
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

        let ys = evaluator(xa_e + s * Unit::Second)?;
        if ys.abs() <= f64::EPSILON {
            return Ok(xa_e + s * Unit::Second);
        }

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
    Err(AnalysisError::EventNotFound {
        start: start_epoch,
        end: end_epoch,
        event: Box::new(event.clone()),
    })
}

/// Scans across an evaluator function using an adaptive step-size approach imspired from the error control steps of Runge Kutta integrator methods.
/// Returns the brackets where there is a sign change in the evaluator.
pub fn adaptive_step_scanner<F>(
    evaluator: F,
    event: &Event,
    start_epoch: Epoch,
    end_epoch: Epoch,
) -> Result<Vec<(Epoch, Epoch)>, AnalysisError>
where
    F: Fn(Epoch) -> Result<f64, AnalysisError>,
{
    let min_step = event.epoch_precision;
    // Max step is for example ~16 minutes for an epoch precision of 0.1 seconds
    let max_step = min_step * 10_000;
    let mut brackets = Vec::new();

    let mut y_prev = evaluator(start_epoch)?;
    let mut t = start_epoch;
    // Always start with the max step, and the adaptive step will reject and take a smaller step if needed.
    let mut step = max_step;

    while t < end_epoch {
        let remaining = end_epoch - t;
        // Ensure that we don't evaluate outside of the desired bound
        step = step.min(remaining);
        let y_next = match evaluator(t + step) {
            Ok(val) => val,
            Err(_) => {
                // Stop searching but don't throw away our work.
                break;
            }
        };

        // Ensure that we're scanning linearly.
        let delta = (y_next - y_prev).abs();
        let delta_ratio = delta / step.to_seconds();

        // For angles, we smooth the evaluation with an atan2 function of the difference
        // between the desired and the event's evaluation angle.
        // This means we'll see two sign crossings: one for the event, and one for the
        // discontinuity. For the latter, the absolute difference (delta) will be nearly
        // 360.0 degrees. Let's be conservative: if we see a delta of half of that, consider
        // this normal, and accept the step, but don't consider it a valid event bracket.

        if event.scalar.is_angle() {
            // Atan2 is a triangular signal so a bracket exists only if y_prev is negative and y_next is positive.
            // Anything else is a fluke, and we can quickly speed through the whole trajectory.
            if y_prev.signum() != y_next.signum() && delta < 180.0 {
                brackets.push((t, t + step));
            }
        } else {
            // Update the step to try to achieve linearity.
            let next_step = (step.to_seconds() / delta_ratio) * Unit::Second;
            if delta_ratio > 1.1 && step >= min_step {
                // More than 10% faster than linear scan, reject advancement, use updated step.
                step = next_step;
                continue;
            }

            // Previous step accepted, check if there was a zero crossing.
            if y_prev * y_next < 0.0 {
                brackets.push((t, t + step));
            }
            step = next_step.clamp(min_step, max_step);
        }
        y_prev = y_next;
        t += step;
    }

    Ok(brackets)
}
