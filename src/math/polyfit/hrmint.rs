/* hrmint.f -- translated by f2c (version 19980913).
   You must link the resulting object file with the libraries:
        -lf2c -lm   (in that order)
*/

/* $Procedure HRMINT ( Hermite polynomial interpolation  ) */
/* Subroutine */
pub fn hrmint_(xvals: &[f64], yvals: &[f64], x: f64) -> (f64, f64) {
    let work: &mut [f64] = &mut [0.0; 256];
    let n: usize = xvals.len();

    assert!(n > 1);

    /*     Copy the input array into WORK.  After this, the first column */
    /*     of WORK represents the first column of our triangular */
    /*     interpolation table. */

    for i in 0..n * 2 {
        work[i] = yvals[i];
    }

    /*     Compute the second column of the interpolation table: this */
    /*     consists of the N-1 values obtained by evaluating the */
    /*     first-degree interpolants at X. We'll also evaluate the */
    /*     derivatives of these interpolants at X and save the results in */
    /*     the second column of WORK. Because the derivative computations */
    /*     depend on the function computations from the previous column in */
    /*     the interpolation table, and because the function interpolation */
    /*     overwrites the previous column of interpolated function values, */
    /*     we must evaluate the derivatives first. */

    for i in 1..=n - 1 {
        let c1 = xvals[i] - x;
        let c2 = x - xvals[i - 1];
        let denom = xvals[i] - xvals[i - 1];

        /*        The second column of WORK contains interpolated derivative */
        /*        values. */

        /*        The odd-indexed interpolated derivatives are simply the input */
        /*        derivatives. */

        let prev = 2 * i - 1;
        let curr = 2 * i;
        let next = 2 * i + 1;
        work[prev + 2 * n - 1] = work[curr - 1];

        /*        The even-indexed interpolated derivatives are the slopes of */
        /*        the linear interpolating polynomials for adjacent input */
        /*        abscissa/ordinate pairs. */

        work[curr + 2 * n - 1] = (work[next - 1] - work[prev - 1]) / denom;

        /*        The first column of WORK contains interpolated function values. */
        /*        The odd-indexed entries are the linear Taylor polynomials, */
        /*        for each input abscissa value, evaluated at X. */

        let temp = work[curr - 1] * (x - xvals[i - 1]) + work[prev - 1];
        work[curr - 1] = (c1 * work[prev - 1] + c2 * work[next - 1]) / denom;
        work[prev - 1] = temp;
    }

    /*     The last column entries were not computed by the preceding loop; */
    /*     compute them now. */

    work[(n * 2) - 1 + 2 * n - 1] = work[(n * 2) - 1];
    work[(n * 2) - 1 - 1] = work[(n * 2) - 1] * (x - xvals[n - 1]) + work[(n * 2) - 1 - 1];

    /*     Compute columns 3 through 2*N of the table. */

    for j in 2..=(n * 2) - 1 {
        for i in 1..=(n * 2) - j {
            /*           In the theoretical construction of the interpolation table,
             */
            /*           there are 2*N abscissa values, since each input abcissa */
            /*           value occurs with multiplicity two. In this theoretical */
            /*           construction, the Jth column of the interpolation table */
            /*           contains results of evaluating interpolants that span J+1 */
            /*           consecutive abscissa values.  The indices XI and XIJ below */
            /*           are used to pick the correct abscissa values out of the */
            /*           physical XVALS array, in which the abscissa values are not */
            /*           repeated. */

            let xi = (i + 1) / 2;
            let xij = (i + j + 1) / 2;
            let c1 = xvals[xij - 1] - x;
            let c2 = x - xvals[xi - 1];
            let denom = xvals[xij - 1] - xvals[xi - 1];

            /*           Compute the interpolated derivative at X for the Ith */
            /*           interpolant. This is the derivative with respect to X of */
            /*           the expression for the interpolated function value, which */
            /*           is the second expression below. This derivative computation
             */
            /*           is done first because it relies on the interpolated */
            /*           function values from the previous column of the */
            /*           interpolation table. */

            /*           The derivative expression here corresponds to equation */
            /*           2.35 on page 64 in reference [2]. */

            work[i + 2 * n - 1] = (c1 * work[i + 2 * n - 1]
                + c2 * work[i + 1 + 2 * n - 1]
                + (work[i + 1 - 1] - work[i - 1]))
                / denom;

            /*           Compute the interpolated function value at X for the Ith */
            /*           interpolant. */

            work[i - 1] = (c1 * work[i - 1] + c2 * work[i + 1 - 1]) / denom;
        }
    }

    /*     Our interpolated function value is sitting in WORK(1,1) at this */
    /*     point.  The interpolated derivative is located in WORK(1,2). */

    let f = work[0];
    let df = work[2 * n];
    (f, df)
} /* hrmint_ */
