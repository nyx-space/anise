# ANISE Python

The Python interface to ANISE, a modern rewrite of NAIF SPICE.

## Getting started as a developer
 
1. Install `maturin`, e.g. via `pipx` as `pipx install maturin`
1. Create a virtual environment: `cd anise/anise-py && python3 -m venv .venv`
1. Jump into the vitual environment and install `patchelf` for faster builds: `pip install patchelf`, and `pytest` for the test suite: `pip install pytest`
1. Run `maturin develop` to build the development package and install it in the virtual environment
1. Finally, run the tests `python -m pytest`