/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::Epoch;

use crate::prelude::AniseError;

/// Attempts to evaluate a Chebyshev polynomial given the coefficients, returning the value and its derivative
///
/// # Notes
/// 1. At this point, the splines are expected to be in Chebyshev format and no verification is done.
pub(crate) fn cheby_eval(
    relative_normalized_time_s: f64,
    spline_data: &[f64],
    spline_radius_s: f64,
    eval_epoch: Epoch,
    degree: usize,
) -> Result<(f64, f64), AniseError> {
    // Workspace arrays
    let mut w = [0.0_f64; 3];
    let mut dw = [0.0_f64; 3];

    for j in (2..=degree).rev() {
        w[2] = w[1];
        w[1] = w[0];
        w[0] = (spline_data
            .get(j - 1)
            .ok_or_else(|| AniseError::MissingInterpolationData(eval_epoch))?)
            + (2.0 * relative_normalized_time_s * w[1] - w[2]);

        dw[2] = dw[1];
        dw[1] = dw[0];
        dw[0] = w[1] * 2. + dw[1] * 2.0 * relative_normalized_time_s - dw[2];
    }

    let val = (spline_data
        .get(0)
        .ok_or_else(|| AniseError::MissingInterpolationData(eval_epoch))?)
        + (relative_normalized_time_s * w[0] - w[1]);

    let deriv = (w[0] + relative_normalized_time_s * dw[0] - dw[1]) / spline_radius_s;
    Ok((val, deriv))
}
