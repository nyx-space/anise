from anise.astro import Ephemeris
from anise import Almanac
import os

def test_ephemeris_degree():
    # Load from OEM file
    # We rely on existing data in the repo
    oem_path = "data/tests/ccsds/oem/LRO_Nyx.oem"
    if not os.path.exists(oem_path):
        # Fallback path relative to anise-py/tests if running from there,
        # but pytest usually runs from root or anise-py root.
        oem_path = "../data/tests/ccsds/oem/LRO_Nyx.oem"

    if os.path.exists(oem_path):
        ephem = Ephemeris.from_ccsds_oem_file(oem_path)
        print(f"Ephemeris degree: {ephem.degree}")
        assert ephem.degree == 7, f"Expected degree 7, got {ephem.degree}"
    else:
        print("Skipping test_ephemeris_degree because data file not found")

if __name__ == "__main__":
    test_ephemeris_degree()
