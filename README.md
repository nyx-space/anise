# ANISE (Attitude, Navigation, Instrument, Spacecraft, Ephemeris)

ANISE is a rewrite of the core functionalities of the NAIF SPICE toolkit with enhanced performance, and ease of use, while leveraging Rust's safety and speed.

[**Please fill out our user survey**](https://7ug5imdtt8v.typeform.com/to/qYDB14Hj)

![ANISE LOGO](./ANISE-logo.png)

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
+ **Auto-downloading capability**: ANISE simplifies your workflow by automatically downloading the latest Earth orientation parameters, or any other SPICE or ANISE file from a remote location, seamlessly integrating them into the `Almanac` for immediate use.

## Interfaces

### Rust

ANISE is developed in Rust, leveraging Rust's robust features such as memory safety, efficient concurrency handling, a superb test framework, and excellent error management. These capabilities ensure that all ANISE features are highly reliable and secure from the outset. Being native to Rust, these features are available first within the Rust ecosystem, offering cutting-edge functionality to Rust developers. They are then integrated into other interfaces. If there is a feature in Rust that has yet to be ported into the language of your choice, please open a Github issue.

Refer to the [Rust README](./anise/README.md) for further details. The Rust API documentation is available on <https://docs.rs/anise/latest/anise/>.

### Python

Although ANISE is primarily developed in Rust, it offers first-class support for Python, recognizing that many users will interact with ANISE through its Python interface. This integration ensures that Python users can leverage most of ANISE's capabilities without compromise. If you encounter any limitations or missing features in the Python support, we encourage you to open a GitHub issue to help us improve the interface.

For Python-specific tutorials and resources, please refer to the [Python README](./anise-py/README.md), which includes Jupyter notebook tutorials tailored for Python users.

### GUI

ANISE provides a graphical interface to inspect SPK, BPC, and PCA (Planetary Constant ANISE) files. Allows you to check the start/end times of the segments (shown in whichever time scale you want, including UNIX UTC seconds).

Refer to the [GUI](./anise-gui/README.md) README for details.

## Validation

[![ANISE Validation](https://github.com/nyx-space/anise/actions/workflows/rust.yml/badge.svg)](https://github.com/nyx-space/anise/actions/workflows/rust.yml)

ANISE is validated by running the same queries in ANISE and in SPICE (single threaded) in the _Validation_ step linked above. This workflow validates 101,000 BSP queries in the DE440.BSP file, and 7305 queries each frame in the PCK08 file (every day for 20 years), along with thousands of rotations from Earth high precision BPC file.

**Note:** The PCK data comes from the IAU Reports, which publishes angle, angle rate, and angle acceleration data, expressed in centuries past the J2000 reference epoch.
ANISE uses Hifitime for time conversions. Hifitime's reliance solely on integers for all time computations eliminates the risk of rounding errors. In contrast, SPICE utilizes floating-point values, which introduces rounding errors in calculations like centuries past J2000. Consequently, you might observe a discrepancy of up to 1 millidegree in rotation angles between SPICE and ANISE. However, this difference is a testament to ANISE's superior precision.

## Resources / Assets

For convenience, Nyx Space provides a few important SPICE files on a public bucket:

+ [de440s.bsp](http://public-data.nyxspace.com/anise/de440s.bsp): JPL's latest ephemeris dataset from 1900 until 20250
+ [de440.bsp](http://public-data.nyxspace.com/anise/de440.bsp): JPL's latest long-term ephemeris dataset
+ [pck08.pca](http://public-data.nyxspace.com/anise/v0.5/pck08.pca): planetary constants ANISE (`pca`) kernel, built from the JPL gravitational data [gm_de431.tpc](http://public-data.nyxspace.com/anise/gm_de431.tpc) and JPL's plantary constants file [pck00008.tpc](http://public-data.nyxspace.com/anise/pck00008.tpc)
+ [pck11.pca](http://public-data.nyxspace.com/anise/v0.5/pck11.pca): planetary constants ANISE (`pca`) kernel, built from the JPL gravitational data [gm_de431.tpc](http://public-data.nyxspace.com/anise/gm_de431.tpc) and JPL's plantary constants file [pck00011.tpc](http://public-data.nyxspace.com/anise/pck00011.tpc)
+ [moon_fk_de440.epa](http://public-data.nyxspace.com/anise/v0.5/moon_fk_de440.epa): Euler Parameter ANISE (`epa`) kernel, built from the JPL Moon Frame Kernel `moon_080317.txt`

You may load any of these using the `load()` shortcut that will determine the file type upon loading, e.g. `let almanac = Almanac::new("pck08.pca").unwrap();` or in Python `almanac = Almanac("pck08.pca")`. To automatically download remote assets, from the Nyx Cloud or elsewhere, use the MetaAlmanac: `almanac = MetaAlmanac("ci_config.dhall").process()` in Python.

### Moon frames

Astrodynamicists use three main body fixed frames at the Moon, all suitable for computing latitude and longitude that represent fixed points on the surface of the Moon. The IAU Moon frame is a low-fidelity body-fixed frame. The Moon Principal Axes frames, Moon PA, is used to represent the mass concentrations of the Moon, and therefore is the frame to use for gravity fields defined as spherical harmonics at the Moon. Finally, the Moon Mean Earth frame, Moon ME, is the cartographic frame: images of the Moon centered on a latitude and longitude are almost always provided in the Moon ME frame.

As per the [`moon_de440_220930.txt`](./data/moon_de440_220930.txt) documentation, you should use the provided `moon_fk_de440.epa` file with the `moon_pa_de440_200625.bpc` and `de440.bsp` (or `de440s.bsp`).

## Contributing

Contributions to ANISE are welcome! Whether it's in the form of feature requests, bug reports, code contributions, or documentation improvements, every bit of help is greatly appreciated.

## License

ANISE is distributed under the Mozilla Public License 2.0 (MPL-2.0), offering a balanced approach to open-source by allowing the use of source code within both open and proprietary software. MPL-2.0 requires that modifications to the covered code be released under the same license, thus ensuring improvements remain open-source. However, it allows the combining of the covered software with proprietary parts, providing flexibility for both academic and commercial integrations.

For more details, please see the [full text of the license](./LICENSE) or read [a summary by Github](https://choosealicense.com/licenses/mpl-2.0/).

## Acknowledgements

ANISE is heavily inspired by the NAIF SPICE toolkit and its excellent documentation.


## Contact

For any inquiries, feedback, or discussions, please [open an issue here](https://github.com/nyx-space/anise/issues) or contact the maintainer at christopher.rabotin@gmail.com.
