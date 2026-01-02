from pathlib import Path

from anise import (
    Almanac,
    MetaAlmanac,
    MetaFile,
    LocationDataSet,
    LocationDhallSet,
    LocationDhallSetEntry,
)
from anise.analysis import OrbitalElement
from anise.astro import Orbit
from anise.constants import Frames
from anise.rotation import DCM, Quaternion
from anise.time import Duration, Epoch, TimeSeries, Unit
from anise.utils import convert_tpc
from anise.instrument import Instrument, FovShape, Ellipsoid

import numpy as np
from os import environ


def almanac_fixture():
    if environ.get("CI", False):
        # Load from meta kernel to not use Git LFS quota
        data_path = Path(__file__).parent.joinpath(
            "..", "..", "data", "ci_config.dhall"
        )
        meta = MetaAlmanac(str(data_path))
        return meta.process()
    else:
        data_path = Path(__file__).parent.joinpath("..", "..", "data")
        return Almanac(str(data_path.joinpath("pck08.pca")))


def test_triad_align_clock():
    almanac = almanac_fixture()
    epoch = Epoch("2021-10-29 12:34:56 TDB")
    eme2k = almanac.frame_info(Frames.EME2000)
    # 1. Define Orbit (X-axis position, Y-axis velocity)
    state_vector = np.array([8_191.93, 0.0, 0.0, 0.0, 7.6, 0.0])
    orbit = Orbit(state_vector, epoch, eme2k)

    # 2. Get the "Ground Truth" RIC matrix from the Orbit object
    # This matrix rotates a vector FROM the RIC frame TO the Inertial frame
    ric_dcm = orbit.dcm3x3_from_ric_to_inertial()

    # 3. Construct the same rotation using TRIAD / Align-and-Clock
    # Use the Orbit's actual inertial vectors
    r_inertial = orbit.radius_km()
    v_inertial = orbit.velocity_km_s()
    h_inertial = np.cross(r_inertial, v_inertial)

    # Constraint 1: Align Body X (Radial) with Inertial Position
    primary_body = np.array([1, 0, 0], dtype=np.float64)
    primary_inertial = r_inertial

    # Constraint 2: Align Body Z (Cross-Track) with Inertial Momentum
    secondary_body = np.array([0, 0, 1], dtype=np.float64)
    secondary_inertial = h_inertial

    triad_dcm = DCM.from_align_and_clock(
        primary_body,
        primary_inertial,
        secondary_body,
        secondary_inertial,
        ric_dcm.from_id,  # Use same IDs for correctness
        ric_dcm.to_id,
    )

    # 4. Assert they are identical
    print(f"Orbit RIC:\n{ric_dcm.rot_mat}")
    print(f"TRIAD:\n{triad_dcm.rot_mat}")

    # Check similarity (tolerance for float math)
    assert np.allclose(triad_dcm.rot_mat, ric_dcm.rot_mat, atol=1e-15)

    quat = ric_dcm.to_quaternion()
    print(quat)
    # We're aligned entirely along R given the state_vector
    # so the quaternion is a zero rotation.
    assert abs(quat.z) < 2e-16
    assert abs(quat.x) < 2e-16
    assert abs(quat.y) < 2e-16
    assert abs(quat.w - 1.0) < 2e-16


def test_instrument():
    pass


if __name__ == "__main__":
    test_instrument()
