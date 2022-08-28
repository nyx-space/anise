/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

// TODO: Consider making a trait for these

use hifitime::Epoch;
use log::trace;

use crate::{
    asn1::{spline::Splines, splinecoeffs::Coefficient, splinekind::SplineKind},
    prelude::AniseError,
};

/// Attempts to evaluate a Chebyshev polynominal given the coefficients, returning the value and its derivative
///
/// # Notes
/// 1. At this point, the splines are expected to be in Chebyshev format and no verification is done.
pub(crate) fn cheby_eval(
    eval_epoch: Epoch,  // Must be in the same time system at this point
    start_epoch: Epoch, // Must be in the same time system at this point
    splines: &Splines,
    coeff: Coefficient,
) -> Result<(f64, f64), AniseError> {
    // TODO: Figure how what I do with the radius and midpoint data
    // assert_eq!(meta.interval_length as f64, 2. * seg_coeff.rcrd_radius_s); ==> always true
    // Figure out whether the first parameter should be the ephem instead of the spline ... and if so, maybe put it in interp_ephem directly? There are only five cases there when not accounting for covariance and pos only vs pos+vel
    match splines.kind {
        SplineKind::FixedWindow { window_duration_s } => {
            let radius_s = dbg!(window_duration_s) / 2.0;
            trace!("delta = {}", eval_epoch - start_epoch);
            let delta_s = (eval_epoch - start_epoch).in_seconds();
            if delta_s < 0.0 {
                return Err(AniseError::MissingInterpolationData(eval_epoch));
            }
            // Convert to seconds
            let eval_epoch_s = eval_epoch.as_tdb_seconds();
            let start_epoch_s = start_epoch.as_tdb_seconds();
            let spline_idx_f = (dbg!(delta_s) / window_duration_s).floor();
            let midpoint = start_epoch_s + spline_idx_f * window_duration_s + radius_s;
            let normalized_t = (eval_epoch_s - midpoint) / radius_s;
            let normalized_t2 = 2.0 * normalized_t;
            // Workspace arrays
            let mut w = [0.0_f64; 3];
            let mut dw = [0.0_f64; 3];
            dbg!(splines.config.degree, coeff, spline_idx_f);
            for j in (2..=splines.config.degree.into()).rev() {
                w[2] = w[1];
                w[1] = w[0];
                w[0] = dbg!(splines.fetch(spline_idx_f as usize, j - 1, coeff)?)
                    + (normalized_t2 * w[1] - w[2]);

                dw[2] = dw[1];
                dw[1] = dw[0];
                dw[0] = w[1] * 2. + dw[1] * normalized_t2 - dw[2];
            }

            let val = dbg!(splines.fetch(spline_idx_f as usize, 0, coeff)?)
                + (normalized_t * w[0] - w[1]);
            let deriv = w[0] + normalized_t * dw[0] - dw[1];
            Ok((val, deriv))
        }
        SplineKind::SlidingWindow { indexes: _indexes } => todo!(),
    }
}
