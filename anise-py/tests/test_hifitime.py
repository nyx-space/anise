from anise.time import *

"""
The time tests only make sure that we can call all of the functions that are re-exported.
For comprehensive tests of the time, refer to the hifitime test suite
"""


def test_exports():
    for cls in [
        Epoch,
        TimeSeries,
        Duration,
        Unit,
        Ut1Provider,
        LatestLeapSeconds,
        LeapSecondsFile,
    ]:
        print(f"{cls} OK")


if __name__ == "__main__":
    test_exports()
