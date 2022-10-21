/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::{Epoch, Unit as DurationUnit};

use crate::{
    asn1::{
        spline::Evenness,
        spline::{Field, Splines},
    },
    prelude::AniseError,
};

/// Attempts to evaluate a Chebyshev polynomial given the coefficients, returning the value and its derivative
///
/// # Notes
/// 1. At this point, the splines are expected to be in Chebyshev format and no verification is done.
pub(crate) fn cheby_eval(
    eval_epoch: Epoch,
    start_epoch: Epoch,
    splines: &Splines,
    field: Field,
) -> Result<(f64, f64), AniseError> {
    match splines.metadata.evenness {
        Evenness::Even { duration_ns } => {
            let window_duration_s: f64 =
                ((duration_ns as f64) * DurationUnit::Nanosecond).to_seconds();

            let radius_s = window_duration_s / 2.0;
            let ephem_start_delta = eval_epoch - start_epoch;
            let ephem_start_delta_s = ephem_start_delta.to_seconds();

            if ephem_start_delta_s < 0.0 {
                return Err(AniseError::MissingInterpolationData(eval_epoch));
            }

            // In seconds
            let eval_epoch_et_s = eval_epoch.to_et_seconds();
            let spline_idx_f = (ephem_start_delta_s / window_duration_s).round();

            let midpoint = splines.fetch(spline_idx_f as usize, 0, Field::MidPoint)?;

            let normalized_t = (eval_epoch_et_s - midpoint) / radius_s;

            // Workspace arrays
            let mut w = [0.0_f64; 3];
            let mut dw = [0.0_f64; 3];

            for j in (2..=splines.metadata.state_kind.degree().into()).rev() {
                w[2] = w[1];
                w[1] = w[0];
                w[0] = (splines.fetch(spline_idx_f as usize, j - 1, field)?)
                    + (2.0 * normalized_t * w[1] - w[2]);

                dw[2] = dw[1];
                dw[1] = dw[0];
                dw[0] = w[1] * 2. + dw[1] * 2.0 * normalized_t - dw[2];
            }

            let val =
                (splines.fetch(spline_idx_f as usize, 0, field)?) + (normalized_t * w[0] - w[1]);

            let deriv = (w[0] + normalized_t * dw[0] - dw[1]) / radius_s;
            Ok((val, deriv))
        }
        Evenness::Uneven { indexes: _indexes } => todo!(),
    }
}
