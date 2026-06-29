from pathlib import Path

import numpy as np

from anise.astro import LocalFrame, Maneuver, Opm
from anise.time import Duration, Epoch

# OPM needs no SPICE kernels, so these tests run on a clean checkout.
OPM_DIR = Path(__file__).parent.joinpath("..", "..", "data", "tests", "ccsds", "opm")


def test_parse_real_opm():
    """A real operational OPM (EUTELSAT W4) with unit annotations, GPS time, and maneuvers."""
    opm = Opm.from_ccsds_opm_file(str(OPM_DIR.joinpath("eutelsat_w4.opm")))

    assert opm.object_name == "EUTELSAT W4"
    assert opm.object_id == "2000-028A"

    # State vector parsed despite trailing `[km]` / `[km/s]` unit annotations.
    np.testing.assert_allclose(
        opm.orbit.radius_km(), [6655.9942, -40218.5751, -82.9177]
    )

    # Spacecraft parameters.
    assert opm.mass.dry_mass_kg == 1913.0
    assert opm.srp_data.area_m2 == 10.0
    assert opm.drag_data.coeff_drag == 2.3

    # Three maneuvers, mixing inertial (EME2000) and RIC (RTN) frames.
    assert len(opm.maneuvers) == 3
    assert opm.maneuvers[0].ref_frame == LocalFrame.Inertial
    assert opm.maneuvers[0].delta_mass_kg == -18.418
    assert opm.maneuvers[1].ref_frame == LocalFrame.RIC


def test_parse_opm_covariance():
    """The synthetic fixture exercises the named-keyword covariance block."""
    opm = Opm.from_ccsds_opm_file(str(OPM_DIR.joinpath("sample.opm")))

    cov = opm.covariance.matrix
    assert cov.shape == (6, 6)
    assert cov[0, 0] == 3.3313494e-04
    assert cov[0, 1] == cov[1, 0]  # symmetric


def test_opm_roundtrip(tmp_path):
    opm = Opm.from_ccsds_opm_file(str(OPM_DIR.joinpath("eutelsat_w4.opm")))

    out = tmp_path.joinpath("eutelsat_out.opm")
    opm.write_ccsds_opm(str(out), originator="ANISE Test")

    again = Opm.from_ccsds_opm_file(str(out))
    assert again.object_id == opm.object_id
    assert len(again.maneuvers) == len(opm.maneuvers)
    np.testing.assert_allclose(again.orbit.radius_km(), opm.orbit.radius_km())


def test_build_opm_and_add_maneuver():
    """Construct an OPM from a real file's orbit, attach a maneuver, and write it out."""
    base = Opm.from_ccsds_opm_file(str(OPM_DIR.joinpath("eutelsat_w4.opm")))

    opm = Opm(base.orbit)
    opm.object_id = "2026-001A"
    opm.add_maneuver(
        Maneuver(
            Epoch("2026-01-01T00:00:00 UTC"),
            Duration("0 s"),
            -1.5,
            LocalFrame.RIC,
            np.array([0.0, 0.1, 0.0]),
        )
    )

    assert len(opm.maneuvers) == 1
    np.testing.assert_allclose(opm.maneuvers[0].delta_v_km_s, [0.0, 0.1, 0.0])
