from anise.astro import Ephemeris, Orbit, DataType, Frame
from anise.time import Epoch
import numpy as np

def test_setters():
    epoch = Epoch("2024-02-29T12:34:56")
    frame = Frame(399, 399).with_mu_km3_s2(398600.4415) # Earth
    orbit = Orbit.from_keplerian(7000, 0.01, 28.5, 0, 0, 0, epoch, frame)
    ephem = Ephemeris([orbit], "InitialID")

    print(f"Initial ID: {ephem.object_id}")
    print(f"Initial Interpolation: {ephem.interpolation}")
    print(f"Initial Degree: {ephem.degree}")

    print("\nAttempting to set values...")
    try:
        ephem.object_id = "NewID"
        print(f"Set object_id successfully to {ephem.object_id}")
    except AttributeError as e:
        print(f"Failed to set object_id: {e}")

    try:
        ephem.interpolation = "LAGRANGE"
        print(f"Set interpolation successfully to {ephem.interpolation}")
    except (AttributeError, TypeError) as e:
        print(f"Failed to set interpolation: {e}")

    try:
        ephem.degree = 11
        print(f"Set degree successfully to {ephem.degree}")
    except AttributeError as e:
        print(f"Failed to set degree: {e}")

if __name__ == "__main__":
    test_setters()
