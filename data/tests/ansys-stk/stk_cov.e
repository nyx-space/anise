stk.v.12.0

# WrittenBy    STK_v12.10.0

BEGIN Ephemeris

    NumberOfEphemerisPoints		 3

    ScenarioEpoch		 1 Jun 2020 12:00:00.000000

    InterpolationMethod		 Lagrange

    InterpolationSamplesM1		 1
    InterpolationOrder      1

    CentralBody		 Earth

    CoordinateSystem		 ICRF

    DistanceUnit Kilometers

    CovarianceFormat LowerTriangular

    EphemerisTimePosVel

        0.0    2.865691508757101e+02   -2.139941760551576e+04    1.634195486175098e+04    2.767385843060133e+00    1.626117031305496e+00    2.072579173220514e+00
       60.0    4.525991399948996e+02   -2.130106534115885e+04    1.646570790840699e+04    2.766933135431601e+00    1.652268219511481e+00    2.052482913456063e+00
      120.0    6.185958681347711e+02   -2.120114761297122e+04    1.658825062632493e+04    2.766277061803237e+00    1.678298862536459e+00    2.032235099098199e+00
    END EphemerisTimePosVel

    BEGIN CovarianceTimePosVel
        0.0 1.0
        2.0 3.0
        4.0 5.0 6.0
        7.0 8.0 9.0 10.0
        11.0 12.0 13.0 14.0 15.0
        16.0 17.0 18.0 19.0 20.0 21.0

       60.0 1.0
        2.0 3.0
        4.0 5.0 6.0
        7.0 8.0 9.0 10.0
        11.0 12.0 13.0 14.0 15.0
        16.0 17.0 18.0 19.0 20.0 21.0

      120.0 1.0
        2.0 3.0
        4.0 5.0 6.0
        7.0 8.0 9.0 10.0
        11.0 12.0 13.0 14.0 15.0
        16.0 17.0 18.0 19.0 20.0 21.0
    END CovarianceTimePosVel

END Ephemeris
