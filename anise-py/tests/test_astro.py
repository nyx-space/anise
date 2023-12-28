from anise.astro import *

def test_exports():
    for cls in [Frame, Ellipsoid, Orbit]:
        print(f"{cls} OK")

if __name__ == "__main__":
    test_exports()