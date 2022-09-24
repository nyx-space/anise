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

use hifitime::{Epoch, TimeUnits, Unit};
use log::trace;

use crate::{
    asn1::{ephemeris, spline::Splines, splinecoeffs::Coefficient, splinekind::SplineSpacing},
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
        SplineSpacing::Even { window_duration_s } => {
            let radius_s = window_duration_s / 2.0;
            let ephem_start_delta = eval_epoch - start_epoch;
            println!(
                "window_duration_s = {}\tdelta = {ephem_start_delta}={} days ==> {}",
                window_duration_s.seconds(),
                ephem_start_delta.in_unit(Unit::Day),
                start_epoch
            );
            let ephem_start_delta_s = ephem_start_delta.in_seconds();

            if ephem_start_delta_s < 0.0 {
                return Err(AniseError::MissingInterpolationData(eval_epoch));
            }

            // Convert to seconds
            // let eval_epoch_jde_d = dbg!(eval_epoch.as_jde_et_days());
            // let start_epoch_jde_d = dbg!(start_epoch.as_jde_et_days());
            // // Without the +1 , then I fetch the spline that one index too early. Why?
            // let spline_idx_f = (dbg!(delta_s) / window_duration_s).floor() + 1.0;
            // let midpoint = start_epoch_jde_d
            //     + spline_idx_f * (window_duration_s / 86400.0)
            //     + radius_s / 86400.;
            // let normalized_t = (eval_epoch_jde_d - midpoint) / (radius_s / 86400.0);

            // In seconds
            let eval_epoch_et_s = eval_epoch.as_et_seconds();
            let start_epoch_et_s = start_epoch.as_et_seconds();
            let spline_idx_f = (ephem_start_delta_s / window_duration_s).round(); // Round seems to work

            let midpoint = start_epoch_et_s
                + window_duration_s * (ephem_start_delta_s / window_duration_s).floor()
                + radius_s;
            let normalized_t = (eval_epoch_et_s - midpoint) / radius_s;

            let s = (eval_epoch.as_jde_et_days()
                - (start_epoch.as_jde_et_days()
                    + spline_idx_f * (window_duration_s / 86400.0)
                    + (radius_s / 86400.0)))
                / (radius_s / 86400.0);

            // let sp = (eval_epoch.as_tdb_seconds()
            //     - (start_epoch.as_tdb_seconds() + spline_idx_f * window_duration_s + radius_s))
            //     / radius_s;

            let normalized_t_e = 9.286013664677739e-5;
            dbg!(normalized_t, normalized_t_e, s);

            let normalized_t2 = 2.0 * normalized_t;
            // Workspace arrays
            let mut w = [0.0_f64; 3];
            let mut dw = [0.0_f64; 3];

            // let coeffs = [
            //     94037325.71993284,
            //     11411525.13992438,
            //     -1166083.2349016676,
            //     -23695.241672735978,
            //     1162.9591476829928,
            //     14.443131926291102,
            //     -0.39727576281757704,
            //     -0.0038013779050788425,
            //     1.6737073018552924e-5,
            //     2.681835938104285e-7,
            // ];

            dbg!(splines.config.degree, coeff, spline_idx_f);
            for j in (2..=splines.config.degree.into()).rev() {
                w[2] = w[1];
                w[1] = w[0];
                // dbg!(splines.fetch(spline_idx_f as usize, j - 1, coeff)?);
                w[0] = dbg!(splines.fetch(spline_idx_f as usize, j - 1, coeff)?)
                    + (normalized_t2 * w[1] - w[2]);
                // w[0] = coeffs[j - 1] + (normalized_t2 * w[1] - w[2]);

                dw[2] = dw[1];
                dw[1] = dw[0];
                dw[0] = w[1] * 2. + dw[1] * normalized_t2 - dw[2];
            }

            let val = dbg!(splines.fetch(spline_idx_f as usize, 0, coeff)?)
                + (normalized_t * w[0] - w[1]);
            // let val = coeffs[0] + (normalized_t * w[0] - w[1]);

            let deriv = w[0] + normalized_t * dw[0] - dw[1];
            Ok((val, deriv))
        }
        SplineSpacing::Uneven { indexes: _indexes } => todo!(),
    }
}
