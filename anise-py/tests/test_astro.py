from anise.astro import *
from anise.astro.constants import Frames


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
    test_exports()
    test_frame_defs()
