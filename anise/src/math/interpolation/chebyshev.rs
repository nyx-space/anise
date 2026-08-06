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
use nalgebra::{Vector3, Vector4};

use super::InterpolationError;

/// Attempts to evaluate a Chebyshev polynomial given the coefficients, returning the value and its derivative
///
/// # Notes
/// 1. At this point, the splines are expected to be in Chebyshev format and no verification is done.
pub fn chebyshev_eval(
    normalized_time: f64,
    spline_coeffs_x: &[f64],
    spline_coeffs_y: &[f64],
    spline_coeffs_z: &[f64],
    spline_radius_s: f64,
    eval_epoch_et_s: f64,
    degree: usize,
) -> Result<(Vector3<f64>, Vector3<f64>), InterpolationError> {
    if spline_radius_s.abs() < f64::EPSILON {
        return Err(InterpolationError::InterpMath {
            source: MathError::DivisionByZero {
                action: "spline radius in Chebyshev eval is zero",
            },
        });
    }

    // Workspace arrays, all in Vector4 to force SIMD where possible.
    let mut w = [Vector4::zeros(); 3];
    let mut dw = [Vector4::zeros(); 3];

    for j in (2..=degree + 1).rev() {
        w[2] = w[1];
        w[1] = w[0];

        let c = Vector4::new(
            *spline_coeffs_x
                .get(j - 1)
                .ok_or(InterpolationError::MissingInterpolationData {
                    epoch: Epoch::from_et_seconds(eval_epoch_et_s),
                })?,
            *spline_coeffs_y
                .get(j - 1)
                .ok_or(InterpolationError::MissingInterpolationData {
                    epoch: Epoch::from_et_seconds(eval_epoch_et_s),
                })?,
            *spline_coeffs_z
                .get(j - 1)
                .ok_or(InterpolationError::MissingInterpolationData {
                    epoch: Epoch::from_et_seconds(eval_epoch_et_s),
                })?,
            0.0,
        );

        w[0] = c + (2.0 * normalized_time * w[1] - w[2]);

        dw[2] = dw[1];
        dw[1] = dw[0];
        dw[0] = w[1] * 2. + dw[1] * 2.0 * normalized_time - dw[2];
    }

    let c0 = Vector4::new(
        *spline_coeffs_x
            .first()
            .ok_or(InterpolationError::MissingInterpolationData {
                epoch: Epoch::from_et_seconds(eval_epoch_et_s),
            })?,
        *spline_coeffs_y
            .first()
            .ok_or(InterpolationError::MissingInterpolationData {
                epoch: Epoch::from_et_seconds(eval_epoch_et_s),
            })?,
        *spline_coeffs_z
            .first()
            .ok_or(InterpolationError::MissingInterpolationData {
                epoch: Epoch::from_et_seconds(eval_epoch_et_s),
            })?,
        0.0,
    );

    let val = (c0 + (normalized_time * w[0] - w[1]))
        .fixed_rows::<3>(0)
        .into_owned();

    let deriv = ((w[0] + normalized_time * dw[0] - dw[1]) / spline_radius_s)
        .fixed_rows::<3>(0)
        .into_owned();
    Ok((val, deriv))
}

/// Attempts to evaluate a Chebyshev polynomial given the coefficients, returning only the value
///
/// # Notes
/// 1. At this point, the splines are expected to be in Chebyshev format and no verification is done.
pub fn chebyshev_eval_poly(
    normalized_time: f64,
    spline_coeffs: &[f64],
    eval_epoch_et_s: f64,
    degree: usize,
) -> Result<f64, InterpolationError> {
    // Workspace array
    let mut w = [0.0_f64; 3];

    for j in (2..=degree + 1).rev() {
        w[2] = w[1];
        w[1] = w[0];
        w[0] = (spline_coeffs
            .get(j - 1)
            .ok_or(InterpolationError::MissingInterpolationData {
                epoch: Epoch::from_et_seconds(eval_epoch_et_s),
            })?)
            + (2.0 * normalized_time * w[1] - w[2]);
    }

    // Code from chbval.c:
    // *p = s * w[0] - w[1] + cp[0];
    // For us, s is normalized_time, cp are the spline coeffs, and w is also the workspace.

    let val = (normalized_time * w[0]) - w[1]
        + (spline_coeffs
            .first()
            .ok_or(InterpolationError::MissingInterpolationData {
                epoch: Epoch::from_et_seconds(eval_epoch_et_s),
            })?);

    Ok(val)
}
