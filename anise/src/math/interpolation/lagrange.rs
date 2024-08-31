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

use super::{InterpolationError, MAX_SAMPLES};

pub fn lagrange_eval(
    xs: &[f64],
    ys: &[f64],
    x_eval: f64,
) -> Result<(f64, f64), InterpolationError> {
    if xs.len() != ys.len() {
        return Err(InterpolationError::CorruptedData {
            what: "lengths of abscissas (xs), ordinates (ys), and first derivatives (ydots) differ",
        });
    } else if xs.is_empty() {
        return Err(InterpolationError::CorruptedData {
            what: "list of abscissas (xs) is empty",
        });
    }

    // At this point, we know that the lengths of items is correct, so we can directly address them without worry for overflowing the array.

    let work: &mut [f64] = &mut [0.0; MAX_SAMPLES];
    let dwork: &mut [f64] = &mut [0.0; MAX_SAMPLES];
    for (ii, y) in ys.iter().enumerate() {
        work[ii] = *y;
    }

    let n = xs.len();

    for j in 1..n {
        for i in 0..(n - j) {
            let xi = xs[i];
            let xij = xs[i + j];

            let denom = xi - xij;
            if denom.abs() < f64::EPSILON {
                return Err(InterpolationError::InterpMath {
                    source: MathError::DivisionByZero {
                        action: "lagrange data contains duplicate states",
                    },
                });
            }

            let work_i = work[i];
            let work_ip1 = work[i + 1];

            work[i] = ((x_eval - xij) * work_i + (xi - x_eval) * work_ip1) / denom;

            let deriv = (work_i - work_ip1) / denom;
            dwork[i] = ((x_eval - xij) * dwork[i] + (xi - x_eval) * dwork[i + 1]) / denom + deriv;
        }
    }

    let f = work[0];
    let df = dwork[0];
    Ok((f, df))
}

#[test]
fn lagrange_spice_docs_example() {
    let ts = [-1.0, 0.0, 3.0, 5.0];
    let yvals = [-2.0, -7.0, -8.0, 26.0];
    let dyvals = [
        -4.633_333_333_333_334,
        -4.983_333_333_333_333,
        7.766_666_666_666_667,
        27.766_666_666_666_666,
    ];

    // Check that we can interpolate the values exactly.
    for (i, t) in ts.iter().enumerate() {
        let (eval, deval) = lagrange_eval(&ts, &yvals, *t).unwrap();
        let eval_err = (eval - yvals[i]).abs();
        assert!(eval_err < f64::EPSILON, "f(x) error is {eval_err:e}");

        let deval_err = (deval - dyvals[i]).abs();
        assert!(
            deval_err < f64::EPSILON,
            "#{i}: f'(x) error is {deval_err:e}"
        );
    }

    // Check the interpolation from the SPICE documentation
    let (x, dx) = lagrange_eval(&ts, &yvals, 2.0).unwrap();

    // WARNING: The documentation data is wrong! Evaluation at 2.0 returns -12.2999... in spiceypy.
    let expected_x = -12.299999999999999;
    let expected_dx = 1.2166666666666666;
    dbg!(x, dx);
    assert!((x - expected_x).abs() < f64::EPSILON, "X error");
    assert!((dx - expected_dx).abs() < f64::EPSILON, "dX error");
}
