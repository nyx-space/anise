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

use crate::{
    asn1::{spline::Splines, splinecoeffs::Coefficient},
    prelude::AniseError,
};

/// Attempts to evaluate a Chebyshev polynominal given the coefficients, returning the value and its derivative
///
/// # Notes
/// 1. At this point, the splines are expected to be in Chebyshev format and no verification is done.
pub(crate) fn cheby_eval(
    spline: &Splines,
    spline_idx: usize,
    coeff: Coefficient,
) -> Result<(f64, f64), AniseError> {
    // TODO: Figure how what I do with the radius and midpoint data
    // assert_eq!(meta.interval_length as f64, 2. * seg_coeff.rcrd_radius_s); ==> always true
    // Figure out whether the first parameter should be the ephem instead of the spline ... and if so, maybe put it in interp_ephem directly? There are only five cases there when not accounting for covariance and pos only vs pos+vel
    todo!()
}
