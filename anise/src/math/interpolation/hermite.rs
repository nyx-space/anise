/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

/*
   NOTES:
   1. This code is manually transliterated from CSPICE's `hrmint_.c`.
   The main difference is that this function takes in the derivatives as a separate input and interlaces
   the `work` array as is expected to the done manually in SPICE. This is purely an API design choice for clarity.
   2. The relevant comments (including authors) from hrmint are kept.
   3. The tests are not part of the original SPICE code.
   4. The transliteration in itself justifies the change of license from unrestricted to MPL.
*/
/* hrmint.f -- translated by f2c (version 19980913).
   You must link the resulting object file with the libraries:
        -lf2c -lm   (in that order)
*/

/* $ Restrictions */

/*  None. */

/* $ Literature_References */

/*  [1]  "Numerical Recipes---The Art of Scientific Computing" by */
/*  William H. Press, Brian P. Flannery, Saul A. Teukolsky, */
/*  William T. Vetterling (see sections 3.0 and 3.1). */

/*  [2]  "Elementary Numerical Analysis---An Algorithmic Approach" */
/*  by S. D. Conte and Carl de Boor.  See p. 64. */

/* $ Author_and_Institution */

/*  N.J. Bachman   (JPL) */

/* $ Version */

/* -    SPICELIB Version 1.2.1, 28-JAN-2014 (NJB) */

/*  Fixed a few comment typos. */

/* -    SPICELIB Version 1.2.0, 01-FEB-2002 (NJB) (EDW) */

/*  Bug fix:  declarations of local variables XI and XIJ */
/*  were changed from DOUBLE PRECISION to INTEGER. */
/*  Note:  bug had no effect on behavior of this routine. */

/* -    SPICELIB Version 1.1.0, 28-DEC-2001 (NJB) */

/*  Blanks following final newline were truncated to */
/*  suppress compilation warnings on the SGI-N32 platform. */

/* -    SPICELIB Version 1.0.0, 01-MAR-2000 (NJB) */

use crate::errors::MathError;
use log::error;

use super::{InterpolationError, MAX_SAMPLES};

/// From the abscissas (xs), the ordinates (ys), and the first derivatives (ydots), build the Hermite interpolation of the function and evaluate it at the requested abscissa (x).
///
/// # Runtime verifications
/// 1. Ensure that all provided arrays are of the same size.
/// 2. Ensure that there are no more than 32 items to interpolate.
/// 3. Ensure no division by zero errors (zero is set to core::f64::EPSILON, which is about 2e-16).
pub fn hermite_eval(
    xs: &[f64],
    ys: &[f64],
    ydots: &[f64],
    x_eval: f64,
) -> Result<(f64, f64), InterpolationError> {
    if xs.len() != ys.len() || xs.len() != ydots.len() {
        return Err(InterpolationError::CorruptedData {
            what: "lengths of abscissas (xs), ordinates (ys), and first derivatives (ydots) differ",
        });
    } else if xs.is_empty() {
        return Err(InterpolationError::CorruptedData {
            what: "list of abscissas (xs) is empty",
        });
    } else if xs.len() > MAX_SAMPLES {
        error!("More than {MAX_SAMPLES} samples provided, which is the maximum number of items allowed for a Hermite interpolation");
        return Err(InterpolationError::CorruptedData {
            what: "list of abscissas (xs) contains more items than MAX_SAMPLES (32)",
        });
    }

    // At this point, we know that the lengths of items is correct, so we can directly address them without worry for overflowing the array.

    let work: &mut [f64] = &mut [0.0; 8 * MAX_SAMPLES];
    let n: usize = xs.len();

    /*  Copy the input array into WORK.  After this, the first column */
    /*  of WORK represents the first column of our triangular */
    /*  interpolation table. */

    for i in 0..n {
        work[2 * i] = ys[i];
        work[2 * i + 1] = ydots[i];
    }

    /*  Compute the second column of the interpolation table: this */
    /*  consists of the N-1 values obtained by evaluating the */
    /*  first-degree interpolants at X. We'll also evaluate the */
    /*  derivatives of these interpolants at X and save the results in */
    /*  the second column of WORK. Because the derivative computations */
    /*  depend on the function computations from the previous column in */
    /*  the interpolation table, and because the function interpolation */
    /*  overwrites the previous column of interpolated function values, */
    /*  we must evaluate the derivatives first. */

    for i in 1..=n - 1 {
        let c1 = xs[i] - x_eval;
        let c2 = x_eval - xs[i - 1];
        let denom = xs[i] - xs[i - 1];
        if denom.abs() < f64::EPSILON {
            return Err(InterpolationError::InterpMath {
                source: MathError::DivisionByZero {
                    action:
                        "hermite data contains likely duplicate abcissa, remove duplicate states",
                },
            });
        }

        /*  The second column of WORK contains interpolated derivative */
        /*  values. */

        /*  The odd-indexed interpolated derivatives are simply the input */
        /*  derivatives. */

        let prev = 2 * i - 1;
        let curr = 2 * i;
        work[prev + 2 * n - 1] = work[prev];

        /*  The even-indexed interpolated derivatives are the slopes of */
        /*  the linear interpolating polynomials for adjacent input */
        /*  abscissa/ordinate pairs. */

        work[prev + 2 * n] = (work[curr] - work[prev - 1]) / denom;

        /*  The first column of WORK contains interpolated function values. */
        /*  The odd-indexed entries are the linear Taylor polynomials, */
        /*  for each input abscissa value, evaluated at X. */

        let temp = work[prev] * (x_eval - xs[i - 1]) + work[prev - 1];
        work[prev] = (c1 * work[prev - 1] + c2 * work[curr]) / denom;
        work[prev - 1] = temp;
    }

    /*  The last column entries were not computed by the preceding loop; */
    /*  compute them now. */

    work[4 * n - 2] = work[(2 * n) - 1];
    work[2 * (n - 1)] += work[(2 * n) - 1] * (x_eval - xs[n - 1]);

    /*  Compute columns 3 through 2*N of the table. */

    for j in 2..=(2 * n) - 1 {
        for i in 1..=(2 * n) - j {
            /*  In the theoretical construction of the interpolation table, */
            /*  there are 2*N abscissa values, since each input abcissa */
            /*  value occurs with multiplicity two. In this theoretical */
            /*  construction, the Jth column of the interpolation table */
            /*  contains results of evaluating interpolants that span J+1 */
            /*  consecutive abscissa values.  The indices XI and XIJ below */
            /*  are used to pick the correct abscissa values out of the */
            /*  physical XVALS array, in which the abscissa values are not */
            /*  repeated. */

            let xi = (i + 1) / 2;
            let xij = (i + j + 1) / 2;
            let c1 = xs[xij - 1] - x_eval;
            let c2 = x_eval - xs[xi - 1];
            let denom = xs[xij - 1] - xs[xi - 1];
            if denom.abs() < f64::EPSILON {
                return Err(InterpolationError::InterpMath {
                    source: MathError::DivisionByZero {
                        action: "hermite data contains duplicate states",
                    },
                });
            }

            /*  Compute the interpolated derivative at X for the Ith */
            /*  interpolant. This is the derivative with respect to X of */
            /*  the expression for the interpolated function value, which */
            /*  is the second expression below. This derivative computation */
            /*  is done first because it relies on the interpolated */
            /*  function values from the previous column of the */
            /*  interpolation table. */

            /*  The derivative expression here corresponds to equation */
            /*  2.35 on page 64 in reference [2]. */

            work[i + 2 * n - 1] =
                (c1 * work[i + 2 * n - 1] + c2 * work[i + 2 * n] + (work[i] - work[i - 1])) / denom;

            /*  Compute the interpolated function value at X for the Ith */
            /*  interpolant. */

            work[i - 1] = (c1 * work[i - 1] + c2 * work[i]) / denom;
        }
    }

    /*  Our interpolated function value is sitting in WORK(1,1) at this */
    /*  point. The interpolated derivative is located in WORK(1,2). */

    let f = work[0];
    let df = work[2 * n];
    Ok((f, df))
}

#[test]
fn hermite_spice_docs_example() {
    let ts = [-1.0, 0.0, 3.0, 5.0];
    let yvals = [6.0, 5.0, 2210.0, 78180.0];
    let ydotvals = [3.0, 0.0, 5115.0, 109395.0];

    // Check that we can interpolate the values exactly.
    for (i, t) in ts.iter().enumerate() {
        let (eval, deriv) = hermite_eval(&ts, &yvals, &ydotvals, *t).unwrap();
        let eval_err = (eval - yvals[i]).abs();
        assert!(eval_err < f64::EPSILON, "f(x) error is {eval_err:e}");

        let deriv_err = (deriv - ydotvals[i]).abs();
        assert!(deriv_err < f64::EPSILON, "f'(x) error is {deriv_err:e}");
    }

    // Check the interpolation from the SPICE documentation
    let (x, vx) = hermite_eval(&ts, &yvals, &ydotvals, 2.0).unwrap();

    assert!((x - 141.0).abs() < f64::EPSILON, "X error");
    assert!((vx - 456.0).abs() < f64::EPSILON, "VX error");
}
