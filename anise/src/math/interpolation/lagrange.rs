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
   1. This code is manually transliterated from CSPICE's `lgrint.c`.
   2. The relevant comments (including authors) from lgrint are kept.
   3. The tests are not part of the original SPICE code.
   4. The transliteration in itself justifies the change of license from unrestricted to MPL.
*/
/* lgrint.f -- translated by f2c (version 19980913).
   You must link the resulting object file with the libraries:
    -lf2c -lm   (in that order)
*/

/* $ Restrictions */

/*     None. */

/* $ Literature_References */

/*     [1]  "Numerical Recipes---The Art of Scientific Computing" by */
/*           William H. Press, Brian P. Flannery, Saul A. Teukolsky, */
/*           William T. Vetterling (see sections 3.0 and 3.1). */

/* $ Author_and_Institution */

/*     N.J. Bachman   (JPL) */

/* $ Version */

/* -    SPICELIB Version 1.0.0, 16-AUG-1993 (NJB) */

use crate::errors::MathError;

use super::InterpolationError;

pub fn lagrange_eval(xs: &[f64], ys: &[f64], x_eval: f64) -> Result<f64, InterpolationError> {
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

    /*     We're going to compute the value of our interpolating polynomial */
    /*     at X by taking advantage of a recursion relation between */
    /*     Lagrange polynomials of order n+1 and order n.  The method works */
    /*     as follows: */

    /*        Define */

    /*           P               (x) */
    /*            i(i+1)...(i+j) */

    /*        to be the unique Lagrange polynomial that interpolates our */
    /*        input function at the abscissa values */

    /*           x ,  x   , ... x   . */
    /*            i    i+1       i+j */

    /*        Then we have the recursion relation */

    /*           P              (x)  = */
    /*            i(i+1)...(i+j) */

    /*                                  x - x */
    /*                                   i */
    /*                                 -----------  *  P                (x) */
    /*                                  x - x           (i+1)...(i+j) */
    /*                                   i   i+j */

    /*                                  x  -  x */
    /*                                         i+j */
    /*                               + -----------  *  P                (x) */
    /*                                  x  -  x         i(i+1)...(i+j-1) */
    /*                                   i     i+j */

    /*        Repeated application of this relation allows us to build */
    /*        successive columns, in left-to-right order, of the */
    /*        triangular table */

    /*           P (x) */
    /*            1 */
    /*                    P  (x) */
    /*                     12 */
    /*           P (x)             P   (x) */
    /*            2                 123 */
    /*                    P  (x) */
    /*                     23               . */
    /*                             P   (x) */
    /*           .                  234            . */
    /*           . */
    /*           .        .                               . */
    /*                    . */
    /*                    .        .                           P      (x) */
    /*                             .                      .     12...N */
    /*                             . */
    /*                                             . */

    /*                                      . */

    /*                             P           (x) */
    /*                              (N-2)(N-1)N */
    /*                    P     (x) */
    /*                     (N-1)N */
    /*           P (x) */
    /*            N */

    /*        and after N-1 steps arrive at our desired result, */

    /*           P       (x). */
    /*            12...N */

    /*     The computation is easier to do than to describe. */

    /*     We'll use the scratch array WORK to contain the current column of */
    /*     our interpolation table.  To start out with, WORK(I) will contain */

    /*        P (x). */
    /*         I */

    let mut work = ys.to_vec();

    let n = xs.len();

    for j in 1..n {
        for i in 0..(n - j) {
            let denom = xs[i] - xs[i + j];
            if denom.abs() < f64::EPSILON {
                return Err(InterpolationError::InterpMath {
                    source: MathError::DivisionByZero {
                        action: "lagrange data contains duplicate states",
                    },
                });
            }

            work[i] = ((x_eval - xs[i + j]) * work[i] + (xs[i] - x_eval) * work[i + 1])
                / (xs[i] - xs[i + j]);
        }
    }
    Ok(work[0])
}

#[test]
fn lagrange_spice_docs_example() {
    let ts = [-1.0, 0.0, 3.0, 5.0];
    let yvals = [-2.0, -7.0, -8.0, 26.0];

    // Check that we can interpolate the values exactly.
    for (i, t) in ts.iter().enumerate() {
        let eval = lagrange_eval(&ts, &yvals, *t).unwrap();
        let eval_err = (eval - yvals[i]).abs();
        assert!(eval_err < f64::EPSILON, "f(x) error is {eval_err:e}");
    }

    // Check the interpolation from the SPICE documentation
    let x = lagrange_eval(&ts, &yvals, 2.0).unwrap();

    // WARNING: The documentation data is wrong! Evaluation at 2.0 returns -12.2999... in spiceypy.
    let expected_x = -12.299999999999999;
    assert!((x - expected_x).abs() < f64::EPSILON, "X error");
}
