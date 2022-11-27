#include <stdio.h>
#include <math.h>

typedef int integer;
typedef double doublereal;

int hrmint_(integer* n, doublereal* xvals, doublereal* yvals,
    doublereal* x, doublereal* work, doublereal* f,
    doublereal* df);

int main()
{
    // double xvals[] = {
    //     // 773063753.0320327,
    //     773063842.6860328,
    //     773063932.1790327,
    //     773064021.5950327,
    //     773064111.0160326,
    //     773064200.4970326,
    //     773064290.0490326,
    //     773064379.5660326,
    //     // 773064467.8020325,
    // };
    // double yvals[] = {
    //     // 1264.0276092333008,
    //     // -1.0119972729331588,
    //     1169.380111723055,
    //     -1.0982621220038147,
    //     1067.501355281949,
    //     -1.1773202325269372,
    //     958.9770086109238,
    //     -1.248793644639029,
    //     844.4072328473662,
    //     -1.3123304769876323,
    //     724.4430188794065,
    //     -1.3675873394086253,
    //     599.8186349004518,
    //     -1.414230273831576,
    //     471.46623936222625,
    //     -1.4519274117465721,
    //     // 342.04349989730264,
    //     // -1.4801351852184736,
    // };
    // double x = 773064069.1841084;
    double xvals[] = { -1.0, 0.0, 3.0, 5.0 };
    double yvals[] = { 6.0, 3.0, 5.0, 0.0, 2210.0, 5115.0, 78180.0, 109395.0 };
    double x = 2.0;
    double f,
        df;
    int n = 7;

    double work[256] = { 0 };
    double want_x = 8.9871033515359500e+02;
    double want_vx = -1.2836208430532707e+00;

    int rslt = hrmint_(&n, xvals, yvals, &x, work, &f, &df);
    printf("rslt = %d\n", rslt);
    printf("f = %f\tdf= %f\n", f, df);
    printf("Δf = %e\tΔdf= %e\n", fabs(f - want_x), fabs(df - want_vx));

    return 0;
}