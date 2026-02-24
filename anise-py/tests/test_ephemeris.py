import pytest
from anise.astro import Ephemeris, Orbit, Frame
from anise.constants import Frames
from anise.time import Epoch, Unit

def test_ephemeris_iter():
    start = Epoch("2024-01-01 00:00:00 UTC")
    # EME2000 is J2000. We need to set GM for Keplerian conversion.
    frame = Frames.EME2000.with_mu_km3_s2(398600.4418)

    orbits = []
    count = 10
    for i in range(count):
        epoch = start + Unit.Hour * i
        # sma_km, ecc, inc_deg, raan_deg, aop_deg, ta_deg, epoch, frame
        orbit = Orbit.from_keplerian(
            7000.0, 0.001, 45.0, 10.0, 20.0, float(i) * 10.0, epoch, frame
        )
        orbits.append(orbit)

    ephem = Ephemeris(orbits, "test_obj")

    # Test forward iteration
    iterated_orbits = []
    for record in ephem:
        iterated_orbits.append(record.orbit)

    assert len(iterated_orbits) == count
    for i in range(count):
        assert iterated_orbits[i].epoch == orbits[i].epoch
        # Check some value
        assert abs(iterated_orbits[i].sma_km() - 7000.0) < 1e-9

    # Test reversed iteration
    reversed_orbits = []
    for record in reversed(ephem):
        reversed_orbits.append(record.orbit)

    assert len(reversed_orbits) == count
    for i in range(count):
        # Should match reversed original list
        assert reversed_orbits[i].epoch == orbits[count - 1 - i].epoch

if __name__ == "__main__":
    test_ephemeris_iter()
