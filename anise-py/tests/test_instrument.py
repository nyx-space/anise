import os
from pathlib import Path
import pickle
from math import radians

from anise import (
    Almanac,
    MetaAlmanac,
    MetaFile,
    LocationDataSet,
    LocationDhallSet,
    LocationDhallSetEntry,
)
from anise.analysis import OrbitalElement
from anise.astro import *
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

    state_vector = np.array([8_191.93, 0.0, 0.0, 0.0, 7.6, 0.0])

    orbit = Orbit(state_vector, epoch, eme2k)

    # Build the DCM from TRIAD using an align and clock vector
    # Repeat of the test_align_and_clock_ric Rust test
    r_inertial = np.array([0, 1, 0], dtype=np.float64)
    v_inertial = np.array([-1, 0, 0], dtype=np.float64)

    c_hat = np.cross(r_inertial, v_inertial)
    i_hat = np.cross(c_hat, r_inertial)

    # Align the body X toward the radial vector in the RIC frame
    primary_body_axis = np.array([1, 0, 0], dtype=np.float64)
    primary_inertial_vec = r_inertial
    # Secondary constraint locks Z with the angular momentum
    # This resolves the rotation about the X axis (the clock angle).
    # It implicitly forces the remaining axis (Y) to align with the In-Track vector.
    secondary_body_axis = np.array([0, 0, 1], dtype=np.float64)
    secondary_inertial_vec = c_hat

    dcm = DCM.from_align_and_clock(
        primary_body_axis,
        primary_inertial_vec,
        secondary_body_axis,
        secondary_inertial_vec,
        10,
        20,
    )
    print(dcm.rot_mat)
    quat = dcm.to_quaternion()
    print(quat)
    assert abs(quat.z - -np.sqrt(2) / 2) < 2e-16
    assert abs(quat.x) < 2e-16
    assert abs(quat.y) < 2e-16
    assert abs(quat.w - np.sqrt(2) / 2) < 2e-16


def test_instrument():
    pass


if __name__ == "__main__":
    test_instrument()
