/* hrmint.f -- translated by f2c (version 19980913).
   You must link the resulting object file with the libraries:
        -lf2c -lm   (in that order)
*/

// #include "f2c.h"
#include <stdio.h>
typedef int integer;
typedef double doublereal;

/* $Procedure HRMINT ( Hermite polynomial interpolation  ) */
/* Subroutine */ int hrmint_(integer* n, doublereal* xvals, doublereal* yvals,
    doublereal* x, doublereal* work, doublereal* f,
    doublereal* df)
{
    /* System generated locals */
    // integer xvals_dim1, yvals_dim1, work_dim1, work_offset, i__1, i__2, i__3,
    //     i__4, i__5, i__6, i__7;
    integer work_dim1, work_offset;

    /* Builtin functions */
    integer s_rnge(char*, integer, char*, integer);

    /* Local variables */
    doublereal temp;
    integer this__, prev, next, i__, j;
    // extern /* Subroutine */ int chkin_(char*, ftnlen);
    doublereal denom;
    // extern /* Subroutine */ int errdp_(char*, doublereal*, ftnlen);
    doublereal c1, c2;
    integer xi;
    // extern /* Subroutine */ int sigerr_(char*, ftnlen), chkout_(char*, ftnlen),
    //     setmsg_(char*, ftnlen), errint_(char*, integer*, ftnlen);
    // extern logical return_(void);
    integer xij;

    /* $ Abstract */

    /*     Evaluate a Hermite interpolating polynomial at a specified */
    /*     abscissa value. */

    /* $ Disclaimer */

    /*     THIS SOFTWARE AND ANY RELATED MATERIALS WERE CREATED BY THE */
    /*     CALIFORNIA INSTITUTE OF TECHNOLOGY (CALTECH) UNDER A U.S. */
    /*     GOVERNMENT CONTRACT WITH THE NATIONAL AERONAUTICS AND SPACE */
    /*     ADMINISTRATION (NASA). THE SOFTWARE IS TECHNOLOGY AND SOFTWARE */
    /*     PUBLICLY AVAILABLE UNDER U.S. EXPORT LAWS AND IS PROVIDED "AS-IS" */
    /*     TO THE RECIPIENT WITHOUT WARRANTY OF ANY KIND, INCLUDING ANY */
    /*     WARRANTIES OF PERFORMANCE OR MERCHANTABILITY OR FITNESS FOR A */
    /*     PARTICULAR USE OR PURPOSE (AS SET FORTH IN UNITED STATES UCC */
    /*     SECTIONS 2312-2313) OR FOR ANY PURPOSE WHATSOEVER, FOR THE */
    /*     SOFTWARE AND RELATED MATERIALS, HOWEVER USED. */

    /*     IN NO EVENT SHALL CALTECH, ITS JET PROPULSION LABORATORY, OR NASA */
    /*     BE LIABLE FOR ANY DAMAGES AND/OR COSTS, INCLUDING, BUT NOT */
    /*     LIMITED TO, INCIDENTAL OR CONSEQUENTIAL DAMAGES OF ANY KIND, */
    /*     INCLUDING ECONOMIC DAMAGE OR INJURY TO PROPERTY AND LOST PROFITS, */
    /*     REGARDLESS OF WHETHER CALTECH, JPL, OR NASA BE ADVISED, HAVE */
    /*     REASON TO KNOW, OR, IN FACT, SHALL KNOW OF THE POSSIBILITY. */

    /*     RECIPIENT BEARS ALL RISK RELATING TO QUALITY AND PERFORMANCE OF */
    /*     THE SOFTWARE AND ANY RELATED MATERIALS, AND AGREES TO INDEMNIFY */
    /*     CALTECH AND NASA FOR ALL THIRD-PARTY CLAIMS RESULTING FROM THE */
    /*     ACTIONS OF RECIPIENT IN THE USE OF THE SOFTWARE. */

    /* $ Required_Reading */

    /*     None. */

    /* $ Keywords */

    /*     INTERPOLATION */
    /*     POLYNOMIAL */

    /* $ Declarations */
    /* $ Brief_I/O */

    /*     Variable  I/O  Description */
    /*     --------  ---  -------------------------------------------------- */
    /*     N          I   Number of points defining the polynomial. */
    /*     XVALS      I   Abscissa values. */
    /*     YVALS      I   Ordinate and derivative values. */
    /*     X          I   Point at which to interpolate the polynomial. */
    /*     WORK      I-O  Work space array. */
    /*     F          O   Interpolated function value at X. */
    /*     DF         O   Interpolated function's derivative at X. */

    /* $ Detailed_Input */

    /*     N              is the number of points defining the polynomial. */
    /*                    The arrays XVALS and YVALS contain N and 2*N */
    /*                    elements respectively. */

    /*     XVALS          is an array of length N containing abscissa values. */

    /*     YVALS          is an array of length 2*N containing ordinate and */
    /*                    derivative values for each point in the domain */
    /*                    defined by FIRST, STEP,  and N.  The elements */

    /*                       YVALS( 2*I - 1 ) */
    /*                       YVALS( 2*I     ) */

    /*                    give the value and first derivative of the output */
    /*                    polynomial at the abscissa value */

    /*                       XVALS(I) */

    /*                    where I ranges from 1 to N. */

    /*     WORK           is a work space array.  It is used by this routine */
    /*                    as a scratch area to hold intermediate results. */

    /*     X              is the abscissa value at which the interpolating */
    /*                    polynomial and its derivative are to be evaluated. */

    /* $ Detailed_Output */

    /*     F, */
    /*     DF             are the value and derivative at X of the unique */
    /*                    polynomial of degree 2N-1 that fits the points and */
    /*                    derivatives defined by XVALS and YVALS. */

    /* $ Parameters */

    /*     None. */

    /* $ Exceptions */

    /*     1)  If two input abscissas are equal, the error */
    /*         SPICE(DIVIDEBYZERO) will be signaled. */

    /*     2)  If N is less than 1, the error SPICE(INVALIDSIZE) is */
    /*         signaled. */

    /*     3)  This routine does not attempt to ward off or diagnose */
    /*         arithmetic overflows. */

    /* $ Files */

    /*     None. */

    /* $ Particulars */

    /*     Users of this routine must choose the number of points to use */
    /*     in their interpolation method.  The authors of Reference [1] have */
    /*     this to say on the topic: */

    /*        Unless there is solid evidence that the interpolating function */
    /*        is close in form to the true function f, it is a good idea to */
    /*        be cautious about high-order interpolation.  We */
    /*        enthusiastically endorse interpolations with 3 or 4 points, we */
    /*        are perhaps tolerant of 5 or 6; but we rarely go higher than */
    /*        that unless there is quite rigorous monitoring of estimated */
    /*        errors. */

    /*     The same authors offer this warning on the use of the */
    /*     interpolating function for extrapolation: */

    /*        ...the dangers of extrapolation cannot be overemphasized: */
    /*        An interpolating function, which is perforce an extrapolating */
    /*        function, will typically go berserk when the argument x is */
    /*        outside the range of tabulated values by more than the typical */
    /*        spacing of tabulated points. */

    /* $ Examples */

    /*     1)  Fit a 7th degree polynomial through the points ( x, y, y' ) */

    /*             ( -1,      6,       3 ) */
    /*             (  0,      5,       0 ) */
    /*             (  3,   2210,    5115 ) */
    /*             (  5,  78180,  109395 ) */

    /*         and evaluate this polynomial at x = 2. */

    /*            PROGRAM TEST_HRMINT */

    /*            DOUBLE PRECISION      ANSWER */
    /*            DOUBLE PRECISION      DERIV */
    /*            DOUBLE PRECISION      XVALS (4) */
    /*            DOUBLE PRECISION      YVALS (8) */
    /*            DOUBLE PRECISION      WORK  (8,2) */
    /*            INTEGER               N */

    /*            N         =   4 */

    /*            XVALS(1)  =      -1.D0 */
    /*            XVALS(2)  =       0.D0 */
    /*            XVALS(3)  =       3.D0 */
    /*            XVALS(4)  =       5.D0 */

    /*            YVALS(1)  =       6.D0 */
    /*            YVALS(2)  =       3.D0 */
    /*            YVALS(3)  =       5.D0 */
    /*            YVALS(4)  =       0.D0 */
    /*            YVALS(5)  =    2210.D0 */
    /*            YVALS(6)  =    5115.D0 */
    /*            YVALS(7)  =   78180.D0 */
    /*            YVALS(8)  =  109395.D0 */

    /*            CALL HRMINT ( N, XVALS, YVALS, 2.D0, WORK, ANSWER, DERIV ) */

    /*            WRITE (*,*) 'ANSWER = ', ANSWER */
    /*            WRITE (*,*) 'DERIV  = ', DERIV */
    /*            END */

    /*        The returned value of ANSWER should be 141.D0, and the returned */
    /*        derivative value should be 456.D0, since the unique 7th degree */
    /*        polynomial that fits these constraints is */

    /*                     7       2 */
    /*           f(x)  =  x   +  2x  + 5 */

    /* $ Restrictions */

    /*     None. */

    /* $ Literature_References */

    /*     [1]  "Numerical Recipes---The Art of Scientific Computing" by */
    /*           William H. Press, Brian P. Flannery, Saul A. Teukolsky, */
    /*           William T. Vetterling (see sections 3.0 and 3.1). */

    /*     [2]  "Elementary Numerical Analysis---An Algorithmic Approach" */
    /*           by S. D. Conte and Carl de Boor.  See p. 64. */

    /* $ Author_and_Institution */

    /*     N.J. Bachman   (JPL) */

    /* $ Version */

    /* -    SPICELIB Version 1.2.1, 28-JAN-2014 (NJB) */

    /*        Fixed a few comment typos. */

    /* -    SPICELIB Version 1.2.0, 01-FEB-2002 (NJB) (EDW) */

    /*        Bug fix:  declarations of local variables XI and XIJ */
    /*        were changed from DOUBLE PRECISION to INTEGER. */
    /*        Note:  bug had no effect on behavior of this routine. */

    /* -    SPICELIB Version 1.1.0, 28-DEC-2001 (NJB) */

    /*        Blanks following final newline were truncated to */
    /*        suppress compilation warnings on the SGI-N32 platform. */

    /* -    SPICELIB Version 1.0.0, 01-MAR-2000 (NJB) */

    /* -& */
    /* $ Index_Entries */

    /*     interpolate function using Hermite polynomial */
    /*     Hermite interpolation */

    /* -& */

    /*     SPICELIB functions */

    /*     Local variables */

    /*     Check in only if an error is detected. */

    /* Parameter adjustments */
    work_dim1 = *n * 2;
    work_offset = work_dim1 + 1;
    // yvals_dim1 = *n * 2;
    // xvals_dim1 = *n;

    /* Function Body */

    /*     No data, no interpolation. */

    if (*n < 1) {
        printf("Array size must be positive; was #%d", *n);
        return 0;
    }

    /*     Copy the input array into WORK.  After this, the first column */
    /*     of WORK represents the first column of our triangular */
    /*     interpolation table. */

    // i__1 = *n * 2;
    for (i__ = 1; i__ <= *n * 2; ++i__) {
        // work[(i__2 = i__ + work_dim1 - work_offset)] = yvals[(i__3 = i__ - 1)];
        work[(i__ + work_dim1 - work_offset)] = yvals[(i__ - 1)];
    }

    printf("[0] { ");
    size_t fj;
    for (size_t j = 0; j < 256; j++) {
        double val = work[j];
        if (val == 0) {
            break;
        }
        printf("%f ", val);
        fj = j;
    }
    printf("} (items = %ld)\n", fj);

    /*     Compute the second column of the interpolation table: this */
    /*     consists of the N-1 values obtained by evaluating the */
    /*     first-degree interpolants at X. We'll also evaluate the */
    /*     derivatives of these interpolants at X and save the results in */
    /*     the second column of WORK. Because the derivative computations */
    /*     depend on the function computations from the previous column in */
    /*     the interpolation table, and because the function interpolation */
    /*     overwrites the previous column of interpolated function values, */
    /*     we must evaluate the derivatives first. */

    // i__1 = *n - 1;
    double ts[256] = { 0 };
    for (i__ = 1; i__ <= *n - 1; ++i__) {
        c1 = xvals[i__] - *x;
        c2 = *x - xvals[i__ - 1];
        denom = xvals[i__] - xvals[i__ - 1];

        /*        The second column of WORK contains interpolated derivative */
        /*        values. */

        /*        The odd-indexed interpolated derivatives are simply the input */
        /*        derivatives. */

        prev = (i__ * 2) - 1;
        this__ = prev + 1;
        next = this__ + 1;
        work[prev + (work_dim1 * 2) - work_offset] = work[this__ + work_dim1 - work_offset];
        printf("set work[%d] = work[%d]\n", prev + (work_dim1 * 2) - work_offset, (this__ + work_dim1 - work_offset));

        /*        The even-indexed interpolated derivatives are the slopes of */
        /*        the linear interpolating polynomials for adjacent input */
        /*        abscissa/ordinate pairs. */

        ts[i__ - 1] = (work[(next + work_dim1 - work_offset)] - work[(prev + work_dim1 - work_offset)]) / denom;
        work[(this__ + (work_dim1 * 2) - work_offset)] = (work[(next + work_dim1 - work_offset)] - work[(prev + work_dim1 - work_offset)]) / denom;
        // Calculate the difference between that time derivative and the input
        double err = work[(this__ + (work_dim1 * 2) - work_offset)] - yvals[this__ - 1];
        printf("set work[%d] = (work[%d] - work[%d])/%f => %f\n", this__ + (work_dim1 * 2) - work_offset, prev + work_dim1 - work_offset, denom, err);

        /*        The first column of WORK contains interpolated function values. */
        /*        The odd-indexed entries are the linear Taylor polynomials, */
        /*        for each input abscissa value, evaluated at X. */

        temp = work[(this__ + work_dim1 - work_offset)] * (*x - xvals[(i__ - 1)]) + work[(prev + work_dim1 - work_offset)];
        work[(this__ + work_dim1 - work_offset)] = (c1 * work[(prev + work_dim1 - work_offset)] + c2 * work[(next + work_dim1 - work_offset)]) / denom;
        work[(prev + work_dim1 - work_offset)] = temp;
    }

    printf("[1] TS { ");
    for (size_t j = 0; j < 256; j++) {
        double val = ts[j];
        if (val == 0) {
            break;
        }
        printf("%f ", val);
        fj = j;
    }
    printf("} (items = %ld)\n", fj);

    printf("[1] { ");
    for (size_t j = 0; j < 256; j++) {
        double val = work[j];
        if (val == 0) {
            break;
        }
        printf("%f ", val);
        fj = j;
    }
    printf("} (items = %ld)\n", fj);

    /*     The last column entries were not computed by the preceding loop; */
    /*     compute them now. */

    work[((*n * 2) - 1 + (work_dim1 * 2) - work_offset)] = work[((*n * 2) + work_dim1 - work_offset)];
    work[((*n * 2) - 1 + work_dim1 - work_offset)] = work[((*n * 2) + work_dim1 - work_offset)] * (*x - xvals[(*n - 1)]) + work[((*n * 2) - 1 + work_dim1 - work_offset)];
    printf("[2] { ");
    for (size_t j = 0; j < 256; j++) {
        double val = work[j];
        if (val == 0) {
            break;
        }
        printf("%f ", val);
        fj = j;
    }
    printf("} (items = %ld)\n", fj);

    /*     Compute columns 3 through 2*N of the table. */

    for (j = 2; j <= (*n * 2) - 1; ++j) {
        for (i__ = 1; i__ <= (*n * 2) - j; ++i__) {

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

            xi = (i__ + 1) / 2;
            xij = (i__ + j + 1) / 2;
            c1 = xvals[(xij - 1)] - *x;
            c2 = *x - xvals[(xi - 1)];
            denom = xvals[(xij - 1)] - xvals[(xi - 1)];

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

            work[(i__ + (work_dim1 * 2) - work_offset)] = (c1 * work[(i__ + (work_dim1 * 2) - work_offset)] + c2 * work[(i__ + 1 + (work_dim1 * 2) - work_offset)] + (work[(i__ + 1 + work_dim1 - work_offset)] - work[(i__ + work_dim1 - work_offset)])) / denom;

            /*           Compute the interpolated function value at X for the Ith */
            /*           interpolant. */

            work[(i__ + work_dim1 - work_offset)] = (c1 * work[(i__ + work_dim1 - work_offset)] + c2 * work[(i__ + 1 + work_dim1 - work_offset)]) / denom;
        }
    }
    printf("[3] { ");
    for (size_t j = 0; j < 256; j++) {
        double val = work[j];
        if (val == 0) {
            break;
        }
        printf("%f ", val);
        fj = j;
    }
    printf("} (items = %ld)\n", fj);

    /*     Our interpolated function value is sitting in WORK(1,1) at this */
    /*     point.  The interpolated derivative is located in WORK(1,2). */

    *f = work[(work_dim1 + 1 - work_offset)];
    *df = work[((work_dim1 * 2) + 1 - work_offset)];
    return 0;
} /* hrmint_ */
