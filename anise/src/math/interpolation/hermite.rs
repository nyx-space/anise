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
use snafu::ensure;

use super::{InterpolationError, StridedDataAccess, MAX_SAMPLES};
use crate::math::interpolation::TooManySamplesSnafu; // For MAX_SAMPLES check
use crate::math::interpolation::NotEnoughSamplesSnafu; // For empty/too few check
use crate::math::interpolation::InconsistentDataSnafu; // For length mismatch
use crate::math::interpolation::MathSnafu; // For division by zero


/// From the abscissas (xs), the ordinates (ys), and the first derivatives (yps), build the Hermite interpolation of the function and evaluate it at the requested abscissa (x_eval).
///
/// This function uses the Neville's algorithm variant for Hermite interpolation.
///
/// # Type Parameters
///
/// * `Y`: Type for `ys` that implements `StridedDataAccess`.
/// * `YP`: Type for `yps` that implements `StridedDataAccess`.
///
/// # Arguments
///
/// * `xs`: A slice of `f64` representing the abscissa values (x-coordinates). Must be ordered.
/// * `ys`: A reference to a data structure implementing `StridedDataAccess` for the ordinate values (y-coordinates).
/// * `yps`: A reference to a data structure implementing `StridedDataAccess` for the first derivatives of y with respect to x (y'-coordinates).
/// * `x_eval`: The `f64` abscissa value at which to evaluate the interpolated function and its derivative.
///
/// # Returns
///
/// A `Result` containing a tuple `(f64, f64)` representing the interpolated y-value and y'-value at `x_eval`,
/// or an `InterpolationError` if an issue occurs (e.g., inconsistent data, too few/many samples, division by zero).
///
/// # Runtime verifications
/// 1. Ensures that `xs`, `ys`, and `yps` have the same length.
/// 2. Ensures that there are at least 2 samples and no more than `MAX_SAMPLES` (currently 32).
/// 3. Ensures no division by zero during calculations (values close to zero are checked against `f64::EPSILON`).
pub fn hermite_eval<Y: StridedDataAccess, YP: StridedDataAccess>(
    xs: &[f64],
    ys: &Y,
    yps: &YP,
    x_eval: f64,
) -> Result<(f64, f64), InterpolationError> {
    let n = xs.len();

    ensure!(n > 0, NotEnoughSamplesSnafu { got: n }); // Handles empty xs
    ensure!(
        n == ys.len() && n == yps.len(),
        InconsistentDataSnafu {
            reason: "lengths of xs, ys, and yps must be equal and non-zero"
        }
    );
    ensure!(n >= 2, NotEnoughSamplesSnafu { got: n }); // Hermite needs at least 2 points for the typical divided difference table logic used here with derivatives
    ensure!(
        n <= MAX_SAMPLES,
        TooManySamplesSnafu {
            max_samples: MAX_SAMPLES,
            got: n // Since all lengths must be equal, n is sufficient here.
        }
    );
    // The original code's MAX_SAMPLES check used xs.len().max(ys.len()).max(yps.len())
    // but since we ensure n == ys.len() == yps.len(), just using n is fine.
    // If we wanted to be super pedantic before ensuring lengths are equal:
    // ensure!(
    //     xs.len() <= MAX_SAMPLES && ys.len() <= MAX_SAMPLES && yps.len() <= MAX_SAMPLES,
    //     TooManySamplesSnafu { max_samples: MAX_SAMPLES, got: xs.len().max(ys.len()).max(yps.len()) }
    // );


    // At this point, we know that n (which is xs.len()) is between 2 and MAX_SAMPLES inclusive.
    // And ys.len() and yps.len() are equal to n.

    let work: &mut [f64] = &mut [0.0; 8 * MAX_SAMPLES]; // work array size is based on compile-time MAX_SAMPLES

    /*  Copy the input array into WORK.  After this, the first column */
    /*  of WORK represents the first column of our triangular */
    /*  interpolation table. */

    for i in 0..n {
        work[2 * i] = ys.get(i);
        work[2 * i + 1] = yps.get(i);
    }

    /*  Compute the second column of the interpolation table: this */
    /*  consists of the N-1 values obtained by evaluating the */
    /*  first-degree interpolants at x_eval. We'll also evaluate the */
    /*  derivatives of these interpolants at x_eval and save the results in */
    /*  the second column of WORK. Because the derivative computations */
    /*  depend on the function computations from the previous column in */
    /*  the interpolation table, and because the function interpolation */
    /*  overwrites the previous column of interpolated function values, */
    /*  we must evaluate the derivatives first. */

    for i in 1..=n - 1 {
        let c1 = xs[i] - x_eval;
        let c2 = x_eval - xs[i - 1];
        let denom = xs[i] - xs[i - 1];
        ensure!(
            denom.abs() >= f64::EPSILON, // Using >= to avoid issues with exactly EPSILON if that's problematic
            MathSnafu {
                source: MathError::DivisionByZero {
                    action: "hermite data contains likely duplicate abscissa (xs[i] - xs[i-1] is too small), remove duplicate states or check input precision",
                }
            }
        );
        

        /*  The second column of WORK contains interpolated derivative */
        /*  values. */

        /*  The odd-indexed interpolated derivatives are simply the input */
        /*  derivatives. */

    let prev_work_idx = 2 * i - 1; // Base index for previous pair in work array
    let curr_work_idx = 2 * i;     // Base index for current value in work array
    work[prev_work_idx + 2 * n - 1] = work[prev_work_idx]; // work[2i-1 + 2n-1] = work[2i-1] (derivative part)

        /*  The even-indexed interpolated derivatives are the slopes of */
        /*  the linear interpolating polynomials for adjacent input */
        /*  abscissa/ordinate pairs. */
    // work[ (2i-1)+1 + 2n -1 ] = (work[2i] - work[2i-2]) / denom
    work[prev_work_idx + 1 + 2 * n - 1] = (work[curr_work_idx] - work[prev_work_idx - 1]) / denom;


        /*  The first column of WORK contains interpolated function values. */
        /*  The odd-indexed entries are the linear Taylor polynomials, */
    /*  for each input abscissa value, evaluated at x_eval. */

    // temp = y'_i-1 * (x_eval - x_i-1) + y_i-1
    let temp = work[prev_work_idx] * (x_eval - xs[i - 1]) + work[prev_work_idx - 1];
    // work[2i-1] = ( (xs_i - x_eval)*y_i-1 + (x_eval - xs_i-1)*y_i ) / denom
    work[prev_work_idx] = (c1 * work[prev_work_idx - 1] + c2 * work[curr_work_idx]) / denom;
    work[prev_work_idx - 1] = temp;
    }

    /*  The last column entries were not computed by the preceding loop; */
    /*  compute them now. */
    // work[2n-1 + 2n-1] = work[2n-1] (derivative part)
    // work[4n-2] = work[2n-1]
    work[2 * (2 * n - 1)] = work[2 * n - 1]; // Last entry in first derivative column of work table
    // work[2(n-1)] = y_n-1 + y'_n-1 * (x_eval - x_n-1)
    work[2 * (n - 1)] += work[2 * n - 1] * (x_eval - xs[n - 1]);


    /*  Compute columns 3 through 2*N of the table. */
    // j is current column being computed in the divided difference table (conceptual)
    // i is the row within that conceptual column
    for j in 2..=(2 * n) - 1 {
        for i in 1..=(2 * n) - j {
            /*  In the theoretical construction of the interpolation table, */
            /*  there are 2*N abscissa values, since each input abcissa */
            /*  value occurs with multiplicity two. In this theoretical */
            /*  construction, the Jth column of the interpolation table */
            /*  contains results of evaluating interpolants that span J+1 */
            /*  consecutive abscissa values.  The indices `xi_idx` and `xij_idx` below */
            /*  are used to pick the correct abscissa values out of the */
            /*  physical xs array, in which the abscissa values are not */
            /*  repeated. These are 0-indexed for xs slice. */

            let xi_idx = (i).div_ceil(2) -1; // (i/2)th distinct x value, 0-indexed
            let xij_idx = (i + j).div_ceil(2) -1; // ((i+j)/2)th distinct x value, 0-indexed

            let c1 = xs[xij_idx] - x_eval;
            let c2 = x_eval - xs[xi_idx];
            let denom = xs[xij_idx] - xs[xi_idx];
            ensure!(
                denom.abs() >= f64::EPSILON,
                MathSnafu {
                    source: MathError::DivisionByZero {
                        action: "hermite data contains duplicate states (xs[xij_idx] - xs[xi_idx] is too small), check input precision",
                    }
                }
            );
            

            /*  Compute the interpolated derivative at x_eval for the Ith */
            /*  interpolant. This is the derivative with respect to x_eval of */
            /*  the expression for the interpolated function value, which */
            /*  is the second expression below. This derivative computation */
            /*  is done first because it relies on the interpolated */
            /*  function values from the previous column of the */
            /*  interpolation table. */

            /*  The derivative expression here corresponds to equation */
            /*  2.35 on page 64 in reference [2]. */
            // work_deriv[i-1] = (c1 * work_deriv[i-1] + c2 * work_deriv[i] + (work_func[i] - work_func[i-1])) / denom;
            // work indices: work_func is 0-indexed up to 2n-1. work_deriv is also 0-indexed up to 2n-1, effectively starting at work[2n].
            // So, work[ (i-1) + 2n ] for work_deriv[i-1]
            // work[ i + 2n ] for work_deriv[i]
            // work[i] for work_func[i]
            // work[i-1] for work_func[i-1]
            
            let func_val_i = work[i];
            let func_val_i_minus_1 = work[i-1];
            let deriv_val_i_minus_1 = work[(i-1) + 2*n]; // work[i-1] in derivative side of table
            let deriv_val_i = work[i + 2*n];         // work[i] in derivative side of table

            work[(i-1) + 2*n] = (c1 * deriv_val_i_minus_1 + c2 * deriv_val_i + (func_val_i - func_val_i_minus_1)) / denom;

            /*  Compute the interpolated function value at x_eval for the Ith */
            /*  interpolant. */
            // work_func[i-1] = (c1 * work_func[i-1] + c2 * work_func[i]) / denom
            work[i-1] = (c1 * func_val_i_minus_1 + c2 * func_val_i) / denom;
        }
    }

    /*  Our interpolated function value is sitting in WORK(1,1) at this */
    /*  point. The interpolated derivative is located in WORK(1,2). */
    // These are 0-indexed in `work` array. work[0] for function, work[2*n] for derivative.
    let f = work[0];
    let df = work[2 * n];
    Ok((f, df))
}

// Test functions and mock structures are moved into the hermite_ut module below.

#[cfg(test)]
mod hermite_ut {
    use super::*; // To access hermite_eval and f64::EPSILON
    // StridedDataAccess is already in scope via `use super::StridedDataAccess;` at the top of the parent module.
    // If MockStridedData's impl of StridedDataAccess needs the trait name explicitly,
    // it would be `use crate::math::interpolation::StridedDataAccess;` or `use super::super::StridedDataAccess;`
    // but `super::StridedDataAccess` should work because the trait is pub in `super`.
    // Let's be explicit for the trait for MockStridedData:
    use crate::math::interpolation::StridedDataAccess;


    const EPSILON: f64 = 1e-9; // Define EPSILON locally for tests if not already available.
                               // It is available via f64::EPSILON, but tests often use a custom one.
                               // The existing tests use f64::EPSILON, so this line is not strictly needed
                               // if we qualify it as f64::EPSILON. Let's keep it for now if tests used a local const.
                               // The existing tests use f64::EPSILON directly, so this const can be removed.

    #[test]
    fn hermite_spice_docs_example() {
        let ts = [-1.0, 0.0, 3.0, 5.0];
    let yvals = [6.0, 5.0, 2210.0, 78180.0];
    let yps_vals = [3.0, 0.0, 5115.0, 109395.0];

    // Check that we can interpolate the values exactly.
    for (i, t) in ts.iter().enumerate() {
        let (eval, deriv) = hermite_eval(&ts, &yvals, &yps_vals, *t).unwrap();
        let eval_err = (eval - yvals[i]).abs();
        assert!(eval_err < f64::EPSILON, "f(x) error is {eval_err:e}");

        let deriv_err = (deriv - yps_vals[i]).abs();
        assert!(deriv_err < f64::EPSILON, "f'(x) error is {deriv_err:e}");
    }

    // Check the interpolation from the SPICE documentation
    let (x, vx) = hermite_eval(&ts, &yvals, &yps_vals, 2.0).unwrap();

        assert!((x - 141.0).abs() < f64::EPSILON, "X error");
        assert!((vx - 456.0).abs() < f64::EPSILON, "VX error");
    }

    // Define MockStridedData and its StridedDataAccess impl for testing hermite_eval
    #[derive(Debug)]
    struct MockStridedData {
        data: Vec<f64>, // Using Vec for owned data in tests
        offset: usize,
        stride: usize,
        logical_len: usize,
    }

    impl MockStridedData {
        fn new(full_data: Vec<f64>, offset: usize, stride: usize, logical_len: usize) -> Self {
            Self { data: full_data, offset, stride, logical_len }
        }
    }

    impl StridedDataAccess for MockStridedData {
        fn get(&self, index: usize) -> f64 {
            if index >= self.logical_len {
                panic!("MockStridedData: Access out of bounds. Index: {}, Logical Length: {}", index, self.logical_len);
            }
            let actual_idx = self.offset + index * self.stride;
            if actual_idx >= self.data.len() {
                panic!("MockStridedData: Actual index out of bounds for underlying data. Actual Idx: {}, Data Length: {}", actual_idx, self.data.len());
            }
            self.data[actual_idx]
        }

        fn len(&self) -> usize {
            self.logical_len
        }
    }

    #[test]
    fn test_hermite_eval_with_mock_strided_data_spice_example() {
        // Use the same data as hermite_spice_docs_example
        let ts = [-1.0, 0.0, 3.0, 5.0]; // xs
    let yvals_vec = vec![6.0, 5.0, 2210.0, 78180.0];
    let yps_vals_vec = vec![3.0, 0.0, 5115.0, 109395.0];
    
    // Test with non-strided MockStridedData (stride=1, offset=0)
    let mock_ys_simple = MockStridedData::new(yvals_vec.clone(), 0, 1, 4);
    let mock_yps_simple = MockStridedData::new(yps_vals_vec.clone(), 0, 1, 4);

    let x_eval = 2.0;
    let (y_eval_simple, yp_eval_simple) = hermite_eval(&ts, &mock_ys_simple, &mock_yps_simple, x_eval).unwrap();

    assert!((y_eval_simple - 141.0).abs() < f64::EPSILON, "Simple Mock Y error. Expected 141.0, got {}", y_eval_simple);
    assert!((yp_eval_simple - 456.0).abs() < f64::EPSILON, "Simple Mock YP error. Expected 456.0, got {}", yp_eval_simple);

    // Test with interleaved data and strided MockStridedData
    let interleaved_data = vec![
        6.0, 3.0, // y0, yp0
        5.0, 0.0, // y1, yp1
        2210.0, 5115.0, // y2, yp2
        78180.0, 109395.0, // y3, yp3
    ];

    let mock_ys_strided = MockStridedData::new(interleaved_data.clone(), 0, 2, 4); // offset 0, stride 2 for ys
    let mock_yps_strided = MockStridedData::new(interleaved_data.clone(), 1, 2, 4); // offset 1, stride 2 for yps

    let (y_eval_strided, yp_eval_strided) = hermite_eval(&ts, &mock_ys_strided, &mock_yps_strided, x_eval).unwrap();
    
        assert!((y_eval_strided - 141.0).abs() < f64::EPSILON, "Strided Mock Y error. Expected 141.0, got {}", y_eval_strided);
        assert!((yp_eval_strided - 456.0).abs() < f64::EPSILON, "Strided Mock YP error. Expected 456.0, got {}", yp_eval_strided);
    }

    #[test]
    fn test_hermite_eval_with_mock_strided_data_simple_linear() {
        // Simpler case: linear interpolation
        // y(t) = y0 + v0*t if all derivatives are equal
    // Or, if derivatives are zero, pure linear interpolation of y values.
    let epochs = &[0.0, 10.0]; // xs

    // Data: y0=0, yp0=1;  y1=10, yp1=1
    // Interpolate at t=5.0
    // Using my earlier calculation:
    // y(0.5) = y0*h00 + y1*h01 + yp0*h10*dt + yp1*h11*dt
    // t=0.5, dt=10.0
    // h00(0.5)=0.5, h01(0.5)=0.5, h10(0.5)=0.125, h11(0.5)=-0.125
    // y(5.0) = 0*0.5 + 10*0.5 + 1*0.125*10 + 1*(-0.125)*10
    //        = 0 + 5 + 1.25 - 1.25 = 5.0
    // yp(0.5) = y0*h00_prime/dt + y1*h01_prime/dt + yp0*h10_prime + yp1*h11_prime
    // h00_prime(0.5)/dt = -1.5/10 = -0.15 (using h_prime values for t=0.5, not scaled by dt here)
    // h01_prime(0.5)/dt = 1.5/10 = 0.15
    // h10_prime(0.5) = -0.25
    // h11_prime(0.5) = -0.25
    // yp(5.0) = 0*(-0.15) + 10*(0.15) + 1*(-0.25) + 1*(-0.25)
    //         = 0 + 1.5 - 0.25 - 0.25 = 1.0
    let flat_data = vec![0.0, 1.0, 10.0, 1.0]; // y0, yp0, y1, yp1

    let mock_ys = MockStridedData::new(flat_data.clone(), 0, 2, 2); // len=2 for 2 points
    let mock_yps = MockStridedData::new(flat_data.clone(), 1, 2, 2);

    let x_eval = 5.0;
    let (y_eval, yp_eval) = hermite_eval(epochs, &mock_ys, &mock_yps, x_eval).unwrap();

    assert!((y_eval - 5.0).abs() < f64::EPSILON, "Expected y_eval = 5.0, got {}", y_eval);
    assert!((yp_eval - 1.0).abs() < f64::EPSILON, "Expected yp_eval = 1.0, got {}", yp_eval);

    // Another point, x_eval = 2.0 (t=0.2)
    // h00(0.2) = 2(0.008) - 3(0.04) + 1 = 0.016 - 0.12 + 1 = 0.896
    // h01(0.2) = -2(0.008) + 3(0.04) = -0.016 + 0.12 = 0.104
    // h10(0.2) = (0.008) - 2(0.04) + 0.2 = 0.008 - 0.08 + 0.2 = 0.128
    // h11(0.2) = (0.008) - (0.04) = -0.032
    // y(2.0) = 0*(0.896) + 10*(0.104) + 1*0.128*10 + 1*(-0.032)*10
    //        = 1.04 + 1.28 - 0.32 = 2.32 - 0.32 = 2.0
    //
    // h00_prime(t) = 6t^2 - 6t => h00_prime(0.2) = 6*0.04 - 6*0.2 = 0.24 - 1.2 = -0.96
    // h01_prime(t) = -6t^2 + 6t => h01_prime(0.2) = -0.24 + 1.2 = 0.96
    // h10_prime(t) = 3t^2 - 4t + 1 => h10_prime(0.2) = 3*0.04 - 4*0.2 + 1 = 0.12 - 0.8 + 1 = 0.32
    // h11_prime(t) = 3t^2 - 2t => h11_prime(0.2) = 3*0.04 - 2*0.2 = 0.12 - 0.4 = -0.28
    // yp(2.0) = 0*(-0.96/10) + 10*(0.96/10) + 1*(0.32) + 1*(-0.28)
    //         = 0.96 + 0.32 - 0.28 = 0.96 + 0.04 = 1.0

    let x_eval_2 = 2.0;
        let (y_eval_2, yp_eval_2) = hermite_eval(epochs, &mock_ys, &mock_yps, x_eval_2).unwrap();
        assert!((y_eval_2 - 2.0).abs() < f64::EPSILON, "Expected y_eval_2 = 2.0, got {}", y_eval_2);
        assert!((yp_eval_2 - 1.0).abs() < f64::EPSILON, "Expected yp_eval_2 = 1.0, got {}", yp_eval_2);
    }
}
