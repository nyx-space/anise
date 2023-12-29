from anise import Almanac
from anise.astro.constants import Frames

from pathlib import Path

def test_state_transformation():
    """
    This is the Python equivalent to anise/tests/almanac/mod.rs
    """
    path = Path(__file__).parent.joinpath("..", "..", "data", "de440s.bsp")
    # Must ensure that the path is a string
    ctx = Almanac(str(path))
    # Let's add another file here -- note that the Almanac will load into a NEW variable, so we must overwrite it!
    # This prevents memory leaks (yes, I promise)
    ctx = ctx.load(str(path.parent.joinpath("pck08.pca")))
    eme2k = ctx.frame_info(Frames.EME2000)
    assert eme2k.mu_km3_s2() == 398600.435436096
    assert eme2k.shape.polar_radius_km == 6356.75
    assert abs(eme2k.shape.flattening() - 0.0033536422844278) < 2e-16
    breakpoint()

if __name__ == "__main__":
    test_state_transformation()