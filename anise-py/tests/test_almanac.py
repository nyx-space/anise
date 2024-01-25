from pathlib import Path
import pickle

from anise import Almanac, MetaAlmanac
from anise.astro import *
from anise.astro.constants import Frames
from anise.time import Epoch

from os import environ


def test_state_transformation():
    """
    This is the Python equivalent to anise/tests/almanac/mod.rs
    but the data is loaded from the remote servers
    """

    if environ.get("CI", False):
        # Load from meta kernel to not use Git LFS quota
        data_path = Path(__file__).parent.joinpath("..", "..", "data", "ci_config.dhall")
        meta = MetaAlmanac(str(data_path))
        print(meta)
        # Process the files to be loaded
        try:
            ctx = meta.process()
        except Exception as e:
            if "lfs" in str(e):
                # Must be some LFS error in the CI again
                return
            raise # Otherwise, raise the error!
    else:
        data_path = Path(__file__).parent.joinpath("..", "..", "data")
        # Must ensure that the path is a string
        ctx = Almanac(str(data_path.joinpath("de440s.bsp")))
        # Let's add another file here -- note that the Almanac will load into a NEW variable, so we must overwrite it!
        # This prevents memory leaks (yes, I promise)
        ctx = ctx.load(str(data_path.joinpath("pck08.pca"))).load(
            str(data_path.joinpath("earth_latest_high_prec.bpc"))
        )

    eme2k = ctx.frame_info(Frames.EME2000)
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

    # In Python, we can set the aberration to None
    aberration = None

    state_itrf93 = ctx.transform_to(orig_state, Frames.EARTH_ITRF93, aberration)

    print(orig_state)
    print(state_itrf93)

    assert abs(state_itrf93.latitude_deg() - 10.549246868302738) < 1e-10
    assert abs(state_itrf93.longitude_deg() - 133.76889100913047) < 1e-10
    assert abs(state_itrf93.height_km() - 1814.503598063825) < 1e-10

    # Convert back
    from_state_itrf93_to_eme2k = ctx.transform_to(
        state_itrf93, Frames.EARTH_J2000, aberration
    )

    print(from_state_itrf93_to_eme2k)

    assert orig_state == from_state_itrf93_to_eme2k

    # Demo creation of a ground station
    mean_earth_angular_velocity_deg_s = 0.004178079012116429
    # Grab the loaded frame info
    itrf93 = ctx.frame_info(Frames.EARTH_ITRF93)
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
    # Cannot yet pickle Epoch, so we can't pickle an Orbit yet
    # cf. https://github.com/nyx-space/hifitime/issues/270


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
    test_meta_load()
    test_exports()
    test_frame_defs()
    test_state_transformation()
