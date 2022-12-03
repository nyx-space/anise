#include <stdio.h>
#include <math.h>

typedef int integer;
typedef double doublereal;

int hrmint_ (integer *n, doublereal *xvals, doublereal *yvals, doublereal *x,
             doublereal *work, doublereal *f, doublereal *df);

int
main ()
{

  double xvals[] = { -1.0, 0.0, 3.0, 5.0 };
  double yvals[] = { 6.0, 3.0, 5.0, 0.0, 2210.0, 5115.0, 78180.0, 109395.0 };
  double x = 2.0;
  double f, df;
  int n = 7;

  double work[256] = { 0 };
  double want_x = 8.9871033515359500e+02;
  double want_vx = -1.2836208430532707e+00;

  int rslt = hrmint_ (&n, xvals, yvals, &x, work, &f, &df);
  printf ("rslt = %d\n", rslt);
  printf ("f = %f\tdf= %f\n", f, df);
  printf ("Δf = %e\tΔdf= %e\n", fabs (f - want_x), fabs (df - want_vx));

  return 0;
}