# ANISE (Attitude, Navigation, Instrument, Spacecraft, Ephemeris)

ANISE is a rewrite of the core functionalities of the NAIF SPICE toolkit with enhanced performance, and ease of use, while leveraging Rust's safety and speed.

[**Please fill out our user survey**](https://7ug5imdtt8v.typeform.com/to/qYDB14Hj)

## Introduction

In the realm of space exploration, navigation, and astrophysics, precise and efficient computation of spacecraft position, orientation, and time is critical. ANISE, standing for "Attitude, Navigation, Instrument, Spacecraft, Ephemeris," offers a Rust-native approach to these challenges. This toolkit provides a suite of functionalities including but not limited to:

+ Loading SPK, BPC, PCK, FK, and TPC files.
+ High-precision translations, rotations, and their combination (rigid body transformations).
+ Comprehensive time system conversions using the hifitime library (including TT, TAI, ET, TDB, UTC, GPS time, and more).

ANISE stands validated against the traditional SPICE toolkit, ensuring accuracy and reliability, with translations achieving machine precision (2e-16) and rotations presenting minimal error (less than two arcseconds in the pointing of the rotation axis and less than one arcsecond in the angle about this rotation axis).

## Features

+ **High Precision**: Matches SPICE to machine precision in translations and minimal errors in rotations.
+ **Time System Conversions**: Extensive support for various time systems crucial in astrodynamics.
+ **Rust Efficiency**: Harnesses the speed and safety of Rust for space computations.
+ **Multi-threaded:** Yup! Forget about mutexes and race conditions you're used to in SPICE, ANISE _guarantees_ that you won't have any race conditions.
+ **Frame safety**: ANISE checks all frames translations or rotations are physically valid before performing any computation, even internally.

## Tutorials

- [01 - Querying SPK files](./tutorials/Tutorial%2001%20-%20Querying%20SPK%20files.ipynb)
- [02 - Loading remote and local files (MetaAlmanac)](./tutorials/Tutorial%2002%20-%20Loading%20remote%20SPICE%20and%20ANISE%20files%20(meta%20almanac).ipynb)
- [03 - Defining and working with the orbit structure](./tutorials/Tutorial%2003%20-%20Defining%20and%20working%20with%20the%20Orbit%20structure.ipynb)
- [04 - Computing azimuth, elevation, and range data (AER)](./tutorials/Tutorial%2004%20-%20Computing%20Azimuth%20Elevation%20and%20Range%20data.ipynb)

Note: The tutorials can be viewed in read-only form on [the Github repo](https://github.com/nyx-space/anise/tree/master/anise-py/tutorials).

## Usage

In Python, start by adding anise to your project: `pip install anise`.

```python
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
        orig_state, Frames.EARTH_ITRF93, None
    )

    print(orig_state)
    print(state_itrf93)

    assert state_itrf93.latitude_deg() == 10.549246868302738
    assert state_itrf93.longitude_deg() == 133.76889100913047
    assert state_itrf93.height_km() == 1814.503598063825

    # Convert back
    from_state_itrf93_to_eme2k = ctx.transform_to(
        state_itrf93, Frames.EARTH_J2000, None
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


if __name__ == "__main__":
    test_state_transformation()

```

## Development
 
1. Install `maturin`, e.g. via `pipx` as `pipx install maturin`
1. Create a virtual environment: `cd anise/anise-py && python3 -m venv .venv`
1. Jump into the virtual environment and install `patchelf` for faster builds: `pip install patchelf`, and `pytest` for the test suite: `pip install pytest`
1. Run `maturin develop` to build the development package and install it in the virtual environment
1. Finally, run the tests `python -m pytest`

To run the development version of ANISE in a Jupyter Notebook, install ipykernels in your virtual environment.

1. `pip install ipykernel`
1. Now, build the local kernel: `python -m ipykernel install --user --name=.venv`
1. Then, start jupyter notebook: `jupyter notebook`
1. Open the notebook, click on the top right and make sure to choose the environment you created just a few steps above.

### Generating the pyi type hints

Type hints are extremely useful for Python users. Building them is a bit of manual work.

1. `maturin develop` to build the latest library
1. `python generate_stubs.py anise anise.pyi` builds the top level type hints
1. Repeat for all submodules: `utils`, `time`, `astro`, `astro.constants` writing to a new file each time:
    1. `python generate_stubs.py anise.astro anise.astro.pyi`
    1. `python generate_stubs.py anise.time anise.time.pyi`
    1. `python generate_stubs.py anise.astro.constants anise.astro.constants.pyi`
    1. `python generate_stubs.py anise.utils anise.utils.pyi`
1. Final, concat all of these new files back to `anise.pyi` since that's the only one used by `maturin`.