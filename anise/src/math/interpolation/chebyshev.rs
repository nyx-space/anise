/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::errors::MathError;

use hifitime::Epoch;

use super::InterpolationError;

/// Attempts to evaluate a Chebyshev polynomial given the coefficients, returning the value and its derivative
///
/// # Notes
/// 1. At this point, the splines are expected to be in Chebyshev format and no verification is done.
pub fn chebyshev_eval(
    normalized_time: f64,
    spline_coeffs: &[f64],
    spline_radius_s: f64,
    eval_epoch: Epoch,
    degree: usize,
) -> Result<(f64, f64), InterpolationError> {
    if spline_radius_s.abs() < f64::EPSILON {
        return Err(InterpolationError::InterpMath {
            source: MathError::DivisionByZero {
                action: "spline radius in Chebyshev eval is zero",
            },
        });
    }
    // Workspace arrays
    let mut w = [0.0_f64; 3];
    let mut dw = [0.0_f64; 3];

    for j in (2..=degree + 1).rev() {
        w[2] = w[1];
        w[1] = w[0];
        w[0] = (spline_coeffs
            .get(j - 1)
            .ok_or(InterpolationError::MissingInterpolationData { epoch: eval_epoch })?)
            + (2.0 * normalized_time * w[1] - w[2]);

        dw[2] = dw[1];
        dw[1] = dw[0];
        dw[0] = w[1] * 2. + dw[1] * 2.0 * normalized_time - dw[2];
    }

    let val = (spline_coeffs
        .first()
        .ok_or(InterpolationError::MissingInterpolationData { epoch: eval_epoch })?)
        + (normalized_time * w[0] - w[1]);

    let deriv = (w[0] + normalized_time * dw[0] - dw[1]) / spline_radius_s;
    Ok((val, deriv))
}

/// Attempts to evaluate a Chebyshev polynomial given the coefficients, returning only the value
///
/// # Notes
/// 1. At this point, the splines are expected to be in Chebyshev format and no verification is done.
pub fn chebyshev_eval_poly(
    normalized_time: f64,
    spline_coeffs: &[f64],
    eval_epoch: Epoch,
    degree: usize,
) -> Result<f64, InterpolationError> {
    // Workspace array
    let mut w = [0.0_f64; 3];

    for j in (2..=degree + 1).rev() {
        w[2] = w[1];
        w[1] = w[0];
        w[0] = (spline_coeffs
            .get(j - 1)
            .ok_or(InterpolationError::MissingInterpolationData { epoch: eval_epoch })?)
            + (2.0 * normalized_time * w[1] - w[2]);
    }

    // Code from chbval.c:
    // *p = s * w[0] - w[1] + cp[0];
    // For us, s is normalized_time, cp are the spline coeffs, and w is also the workspace.

    let val = (normalized_time * w[0]) - w[1]
        + (spline_coeffs
            .first()
            .ok_or(InterpolationError::MissingInterpolationData { epoch: eval_epoch })?);

    Ok(val)
}

/// Evaluate a Chebyshev polynomial and its first derivative (SPICE-style)
///
/// * `tau` - Normalized time ∈ [-1, 1]
/// * `coeffs` - Chebyshev coefficients, ordered as [c0, c1, ..., cn]
/// * `spline_radius_s` - Half the time span of the segment, in seconds
pub fn chebyshev_eval_spice_style(
    tau: f64,
    coeffs: &[f64],
    spline_radius_s: f64,
) -> Result<(f64, f64), InterpolationError> {
    let n = coeffs.len();

    if spline_radius_s.abs() < f64::EPSILON {
        return Err(InterpolationError::InterpMath {
            source: MathError::DivisionByZero {
                action: "spline radius in Chebyshev eval is zero",
            },
        });
    }

    if n == 0 {
        // Or handle as per existing chebyshev_eval if it has specific error for empty coeffs
        return Err(InterpolationError::MissingInterpolationData { epoch: hifitime::Epoch::from_gregorian_tai(1970, 1, 1, 0, 0, 0, 0) }); // Placeholder epoch
    }


    let mut b0 = 0.0;
    let mut b1 = 0.0;
    let mut b2 = 0.0;
    let mut db0 = 0.0;
    let mut db1 = 0.0;
    let mut db2 = 0.0;

    // Iterate from the second-to-last coefficient down to the second coefficient (index 1)
    // The loop should go from index n-1 down to 1 if coeffs are [c0, ..., c_{n-1}]
    // Original C code: j from degp+1 down to 2. cp[j-1] -> cp[degp] ... cp[1]
    // degp = n-1. So j from n down to 2. coeffs[j-1] -> coeffs[n-1] ... coeffs[1]
    for i in (1..n).rev() { // Processes coeffs[n-1], ..., coeffs[1]
        b2 = b1;
        b1 = b0;
        b0 = 2.0 * tau * b1 - b2 + coeffs[i];

        db2 = db1;
        db1 = db0;
        db0 = 2.0 * tau * db1 - db2 + 2.0 * b1;
    }

    // C SPICE algorithm for chebvl_ is (note that cp[0] is the first coefficient, cp[1] is the second, etc.):
    // val = tau * b0 - b1 + cp[0]
    // deriv = (tau * db0 - db1 + b0) / xradius
    // However, the provided code from the issue for the derivative is:
    // let velocity = (2.0 * db0) / spline_radius_s; 
    // This matches the logic in CSPICE's chbder_ which is used for Type 2 SPK records for velocity.
    // chbint_ (for position) uses: val = tau * b0 - b1 + cp[0].
    // chbder_ (for velocity) uses: deriv = (tau * db0 - db1 + b0) / xradius.
    // The provided code for `db0` calculation seems to align with `chbder_`'s calculation for `term_0` in its derivative sum.
    // If `db0` implicitly contains the `b0` term, then `2.0 * db0` might be correct for some formulations.
    // Given the prompt to "implement the proposed function", I will stick to the derivative calculation `(2.0 * db0) / spline_radius_s`.
    // But it's worth noting this differs from a direct translation of chebvl_ derivative part.
    // The SPICE routine SPKEZ2 uses chbval_ (which computes value) and chbder_ (which computes derivative).
    // chbval_ has: b0 = (2.0 * x * b1) - b2 + cp[i]
    //              val = (x * b0) - b1 + cp[0]
    // chbder_ has: d0 = (2.0 * x * d1) - d2 + (2.0 * b1)
    //              deriv = ( (x*d0) - d1 + b0 ) / xradius
    // The loop for b0 in the provided code is: b0 = 2.0 * tau * b1 - b2 + coeffs[i]; (Matches chbval/chbder for b terms)
    // The loop for db0 in the provided code is: db0 = 2.0 * tau * db1 - db2 + 2.0 * b1; (Matches chbder for d terms)
    // Position in provided code: position = tau * b0 - b1 + coeffs[0]; (Matches chbval/chbder position part)
    // Velocity in provided code: velocity = (2.0 * db0) / spline_radius_s;
    // CSPICE chbder.f: DERIV = ( X*D(1) - D(2) + B(1) ) / XRATE
    // If D(1) is db0, D(2) is db1, B(1) is b0, X is tau, XRATE is spline_radius_s
    // Then CSPICE chbder.f is: (tau * db0 - db1 + b0) / spline_radius_s
    // The prompt's code for velocity is (2.0 * db0) / spline_radius_s. This is different.
    // I will use the formula from the prompt: (2.0 * db0) / spline_radius_s.

    let position = tau * b0 - b1 + coeffs[0];
    let velocity = (2.0 * db0) / spline_radius_s; 
    Ok((position, velocity))
}
