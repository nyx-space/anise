# ANISE: Attitude, Navigation, Instrument, Spacecraft, Ephemeris

## A modern, high-performance toolkit for space mission design and operations.

Tired of the usual complexities in spacecraft navigation? ANISE is a fresh, Rust-powered alternative to the NAIF SPICE toolkit, engineered for performance, safety, and ease of use.

![ANISE LOGO](./ANISE-logo.png)

**NASA TRL 9**: ANISE was used throughout the operations of the Firefly Blue Ghost lunar lander, from launch until successful landing.

[**Please fill out our user survey**](https://7ug5imdtt8v.typeform.com/to/qYDB14Hj)

## Why ANISE?

Space missions demand precision. ANISE delivers. It handles the critical calculations of spacecraft position, orientation, and time with validated, high-precision accuracy. Whether you're plotting a trajectory to Mars or orienting a satellite, ANISE provides the tools you need to get it right.

Here's what you can do with ANISE:

* **Load and process essential space data files**: SPK, BPC, PCK, FK, and TPC.
* **Perform high-precision calculations**: Translations, rotations, and rigid body transformations.
* **Seamlessly convert between time systems**: TT, TAI, ET, TDB, UTC, GPS, and more, powered by the `hifitime` library.

Validated against the traditional SPICE toolkit, ANISE achieves machine precision (2e-16) for translations and minimal error for rotations (less than two arcseconds in pointing, one arcsecond in angle).

## Key Features

* **High Precision**: Get the accuracy you need, matching SPICE to machine precision in translations and minimal errors in rotations.
* **Time System Mastery**: Extensive support for all the time systems crucial in astrodynamics.
* **Rust Performance**: Harness the speed and safety of Rust for your space computations.
* **Built for Concurrency**: Say goodbye to the mutexes and race conditions of older toolkits. ANISE guarantees thread safety.
* **Frame Safety**: ANISE ensures that all frame translations and rotations are physically valid before any computation, preventing costly errors.
* **Automatic Data Acquisition**: Simplify your workflow. ANISE can automatically download the latest Earth orientation parameters or any other SPICE or ANISE file from a remote source and integrate it for immediate use.

## Get Started in Your Language of Choice

### Rust

ANISE is built in Rust, giving you direct access to its full range of features, including memory safety, efficient concurrency, a powerful testing framework, and robust error handling. If you're a Rust developer, you're getting the best of ANISE, first.

Dive into the [Rust README](./anise/README.md) for more, or check out the [API documentation](https://docs.rs/anise/latest/anise/).

### Python

We get it. Python is everywhere in the space community. That's why ANISE has first-class support for Python, so you can leverage its power without leaving your favorite environment. If you find a feature missing, let us know by opening a GitHub issue.

For tutorials and resources, head over to the [Python README](./anise-py/README.md) and our Jupyter notebooks.

### GUI

Need to inspect your data files? ANISE provides a graphical interface for SPK, BPC, and PCA (Planetary Constant ANISE) files. You can quickly check segment start and end times in any time scale, including UNIX UTC seconds.

Find out more in the [GUI README](./anise-gui/README.md).

### C++

Coming soon! C++ bindings are in progress.

## Validated and Reliable

[![ANISE Validation](https://github.com/nyx-space/anise/actions/workflows/rust.yml/badge.svg)](https://github.com/nyx-space/anise/actions/workflows/rust.yml)

We rigorously validate ANISE against SPICE. Our validation workflow runs over 100,000 queries on the DE440.bsp file, 7,305 queries for each frame in the PCK08 file (covering 20 years of data), and thousands of rotations from Earth's high-precision BPC file.

**A Note on Precision**: The PCK data from the IAU is based on angle, rate, and acceleration data expressed in centuries past J2000. SPICE uses floating-point values for these calculations, which can introduce rounding errors. ANISE, using `hifitime`, relies on integer arithmetic for all time computations, eliminating this risk. You might see discrepancies of up to 1 millidegree in rotation angles, but this is a sign of ANISE's higher precision.

## Resources and Assets

Nyx Space provides several important SPICE files for your convenience:

* **[de440s.bsp](http://public-data.nyxspace.com/anise/de440s.bsp)**: JPL's latest ephemeris dataset (1900-20250).
* **[de440.bsp](http://public-data.nyxspace.com/anise/de440.bsp)**: JPL's long-term ephemeris dataset.
* **[pck08.pca](http://public-data.nyxspace.com/anise/v0.5/pck08.pca)**: Planetary constants kernel, built from JPL's gravitational data and planetary constants file.
* **[pck11.pca](http://public-data.nyxspace.com/anise/v0.5/pck11.pca)**: An alternative planetary constants kernel.
* **[moon_fk_de440.epa](http://public-data.nyxspace.com/anise/v0.5/moon_fk_de440.epa)**: A Moon frame kernel built from JPL data.

almanac = MetaAlmanac("ci_config.dhall").process(True)

### Understanding Moon Frames

Astrodynamicists use three main body-fixed frames for the Moon:

* **IAU Moon**: A low-fidelity frame.
* **Moon Principal Axes (PA)**: Used for representing mass concentrations and gravity fields.
* **Moon Mean Earth (ME)**: The cartographic frame, used for images of the lunar surface.

For accurate work, use the provided `moon_fk_de440.epa` file with `moon_pa_de440_200625.bpc` and `de440.bsp` (or `de440s.bsp`), as recommended in the [`moon_de440_220930.txt`](./data/moon_de440_220930.txt) documentation.

## Contributing

ANISE is an open-source project, and we welcome contributions! Whether you want to request a feature, report a bug, contribute code, or improve the documentation, we appreciate your help.

## License

ANISE is distributed under the Mozilla Public License 2.0 (MPL-2.0). This license allows you to use ANISE in both open and proprietary software, with the requirement that any modifications to the ANISE source code are also released under the MPL-2.0.

For more details, see the [full license text](./LICENSE) or a [summary on GitHub](https://choosealicense.com/licenses/mpl-2.0/).

## Acknowledgements

ANISE is heavily inspired by the NAIF SPICE toolkit and its excellent documentation.

## Contact

Have questions or feedback? [Open an issue on GitHub](https://github.com/nyx-space/anise/issues) or email the maintainer at christopher.rabotin@gmail.com.
