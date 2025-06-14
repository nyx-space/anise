import os
from pathlib import Path
import pickle

from anise import Almanac, MetaAlmanac, MetaFile
from anise.astro import *
from anise.astro.constants import Frames
from anise.rotation import DCM
from anise.time import Duration, Epoch, TimeSeries, Unit
from anise.utils import convert_tpc

from os import environ


def test_state_transformation():
    """
    This is the Python equivalent to anise/tests/almanac/mod.rs
    but the data is loaded from the remote servers
    """

    if environ.get("CI", False):
        # Load from meta kernel to not use Git LFS quota
        data_path = Path(__file__).parent.joinpath(
            "..", "..", "data", "ci_config.dhall"
        )
        meta = MetaAlmanac(str(data_path))
        print(meta)
        # Process the files to be loaded
        try:
            almanac = meta.process()
        except Exception as e:
            if "lfs" in str(e):
                # Must be some LFS error in the CI again
                return
            raise  # Otherwise, raise the error!
    else:
        data_path = Path(__file__).parent.joinpath("..", "..", "data")
        # Must ensure that the path is a string
        almanac = Almanac(str(data_path.joinpath("de440s.bsp")))
        # Let's add another file here -- note that the Almanac will load into a NEW variable, so we must overwrite it!
        # This prevents memory leaks (yes, I promise)
        almanac = almanac.load(str(data_path.joinpath("pck08.pca"))).load(
            str(data_path.joinpath("earth_latest_high_prec.bpc"))
        )

    eme2k = almanac.frame_info(Frames.EME2000)
    assert eme2k.mu_km3_s2() == 398600.435436096
    assert eme2k.shape.polar_radius_km == 6356.75
    assert abs(eme2k.shape.flattening() - 0.0033536422844278) < 2e-16

    epoch = Epoch("2021-10-29 12:34:56 TDB")

    orig_state = Orbit.from_keplerian(
        8_191.93,
        1e-6,
        12.85,
        306.614,
        314.19,
        99.887_7,
        epoch,
        eme2k,
    )

    assert abs(orig_state.sma_km() - 8191.93) < 1e-10
    assert abs(orig_state.ecc() - 1.000000000361619e-06) < 1e-10
    assert abs(orig_state.inc_deg() - 12.849999999999987) < 1e-10
    assert abs(orig_state.raan_deg() - 306.614) < 1e-10
    assert abs(orig_state.tlong_deg() - 0.6916999999999689) < 1e-10

    assert orig_state.cartesian_pos_vel().shape == (6,)

    # Ensure we can call all of the DCM functions
    for func in [
        "dcm_from_ric_to_inertial",
        "dcm_from_rcn_to_inertial",
        "dcm_from_vnc_to_inertial",
    ]:
        dcm = getattr(orig_state, func)()
        assert dcm.get_state_dcm().shape == (6, 6)
        assert dcm.rot_mat.shape == (3, 3)
        assert dcm.rot_mat_dt.shape == (3, 3)
        print(f"== {func} ==\n{dcm}")
        # Test rebuilding the DCM from its parts
        dcm_rebuilt = DCM(dcm.rot_mat, dcm.from_id, dcm.to_id, dcm.rot_mat_dt)
        assert dcm_rebuilt == dcm

    topo_dcm = orig_state.dcm_from_topocentric_to_body_fixed(123)
    assert topo_dcm.get_state_dcm().shape == (6, 6)
    assert topo_dcm.rot_mat.shape == (3, 3)
    assert (
        topo_dcm.rot_mat_dt is not None and topo_dcm.rot_mat_dt.shape == (3, 3)
    ) or topo_dcm.rot_mat_dt is None

    # In Python, we can set the aberration to None
    aberration = None

    state_itrf93 = almanac.transform_to(orig_state, Frames.EARTH_ITRF93, aberration)

    print(orig_state)
    print(state_itrf93)

    assert abs(state_itrf93.latitude_deg() - 10.549246868302738) < 1e-10
    assert abs(state_itrf93.longitude_deg() - 133.76889100913047) < 1e-10
    assert abs(state_itrf93.height_km() - 1814.503598063825) < 1e-10

    # Convert back
    from_state_itrf93_to_eme2k = almanac.transform_to(state_itrf93, Frames.EARTH_J2000)

    print(from_state_itrf93_to_eme2k)

    assert orig_state == from_state_itrf93_to_eme2k

    # Demo creation of a ground station
    mean_earth_angular_velocity_deg_s = 0.004178079012116429
    # Grab the loaded frame info
    itrf93 = almanac.frame_info(Frames.EARTH_ITRF93)
    paris = Orbit.from_latlongalt(
        48.8566,
        2.3522,
        0.4,
        mean_earth_angular_velocity_deg_s,
        epoch,
        itrf93,
    )

    assert abs(paris.latitude_deg() - 48.8566) < 1e-3
    assert abs(paris.longitude_deg() - 2.3522) < 1e-3
    assert abs(paris.height_km() - 0.4) < 1e-3

    # Pickling test
    pickle.loads(pickle.dumps(eme2k)) == eme2k
    pickle.loads(pickle.dumps(eme2k.shape)) == eme2k.shape
    # Cannot pickle across module boundaries =(
    # pickle.loads(pickle.dumps(paris)) == paris

    # Function export test
    for fname in [
        "transform",
        "transform_to",
        "translate",
        "translate_to",
        "translate_geometric",
        "spk_ezr",
        "state_of",
        "solar_eclipsing",
        "occultation",
        "line_of_sight_obstructed",
        "azimuth_elevation_range_sez",
    ]:
        assert hasattr(almanac, fname)

    # Test the parallel function calls
    start = Epoch("2021-10-29 12:34:56 TDB")
    stop = Epoch("2022-10-29 12:34:56 TDB")
    time_series = TimeSeries(
        start,
        stop,
        Duration("1 min"),
        False,
    )

    tick = Epoch.system_now()

    states = almanac.transform_many(
        Frames.EARTH_J2000,
        Frames.SUN_J2000,
        time_series,
        None,
    )

    clock_time = Epoch.system_now().timedelta(tick)
    print(f"Queried {len(states)} states in {clock_time}")
    assert len(states) == int(stop.timedelta(start).to_unit(Unit.Minute))


def test_convert_tpc():
    """Attempt to reproduce GH issue #339"""
    try:
        os.remove("test_constants.tpc")
    except FileNotFoundError:
        pass

    # First call to convert_tpc works
    convert_tpc("data/pck00011.tpc", "data/gm_de440.tpc", "test_constants.tpc")

    # Second call, with overwrite enabled, also works
    convert_tpc(
        "data/pck00011.tpc", "data/gm_de440.tpc", "test_constants.tpc", overwrite=True
    )

    # Try to load the constants file
    constants_file = MetaFile("test_constants.tpc")
    new_meta = MetaAlmanac()
    new_meta.files = [
        constants_file,
    ]
    almanac = new_meta.process()

    earth_j2k = almanac.frame_info(Frames.EARTH_J2000)
    assert earth_j2k.mu_km3_s2 is not None
    almanac.describe()


def test_meta_load():
    data_path = Path(__file__).parent.joinpath("..", "..", "data", "local.dhall")
    meta = MetaAlmanac(str(data_path))
    print(meta)
    try:
        # Process the files to be loaded
        almanac = meta.process()
    except Exception as e:
        print("Not sure where the files are on Github CI")
        print(e)
    else:
        # And check that everything is loaded
        eme2k = almanac.frame_info(Frames.EME2000)
        assert eme2k.mu_km3_s2() == 398600.435436096
        assert eme2k.shape.polar_radius_km == 6356.75
        assert abs(eme2k.shape.flattening() - 0.0033536422844278) < 2e-16


def test_exports():
    for cls in [Frame, Ellipsoid, Orbit]:
        print(f"{cls} OK")


def test_frame_defs():
    print(f"{Frames.SSB_J2000}")
    print(dir(Frames))
    assert Frames.EME2000 == Frames.EME2000
    assert Frames.EME2000 == Frames.EARTH_J2000
    assert Frames.EME2000 != Frames.SSB_J2000


if __name__ == "__main__":
    # test_meta_load()
    # test_exports()
    # test_frame_defs()
    # test_convert_tpc()
    test_state_transformation()
