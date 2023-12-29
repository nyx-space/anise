from anise import Almanac, Aberration
from anise.astro.constants import Frames
from anise.astro import Orbit
from anise.time import Epoch

from pathlib import Path


def test_state_transformation():
    """
    This is the Python equivalent to anise/tests/almanac/mod.rs
    """
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

    assert orig_state.sma_km() == 8191.93
    assert orig_state.ecc() == 1.000000000361619e-06
    assert orig_state.inc_deg() == 12.849999999999987
    assert orig_state.raan_deg() == 306.614
    assert orig_state.tlong_deg() == 0.6916999999999689

    state_itrf93 = ctx.transform_to(
        orig_state, Frames.EARTH_ITRF93, Aberration.NotSet
    )

    print(orig_state)
    print(state_itrf93)

    assert state_itrf93.geodetic_latitude_deg() == 10.549246868302738
    assert state_itrf93.geodetic_longitude_deg() == 133.76889100913047
    assert state_itrf93.geodetic_height_km() == 1814.503598063825

    # Convert back
    from_state_itrf93_to_eme2k = ctx.transform_to(
        state_itrf93, Frames.EARTH_J2000, Aberration.NotSet
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

    assert abs(paris.geodetic_latitude_deg() - 48.8566) < 1e-3
    assert abs(paris.geodetic_longitude_deg() - 2.3522) < 1e-3
    assert abs(paris.geodetic_height_km() - 0.4) < 1e-3


if __name__ == "__main__":
    test_state_transformation()
