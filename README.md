# ANISE (Attitude, Navigation, Instrument, Spacecraft, Ephemeris)

ANISE, inspired by the iconic Dune universe, is a modern Rust-based library aimed at revolutionizing space navigation and ephemeris calculations. It reimagines the functionalities of the NAIF SPICE toolkit with enhanced performance, precision, and ease of use, leveraging Rust's safety and speed.

[Please fill out our user survey](https://7ug5imdtt8v.typeform.com/to/qYDB14Hj)

## Introduction

In the realm of space exploration, navigation, and astrophysics, precise and efficient computation of spacecraft position, orientation, and time is critical. ANISE, standing for "Attitude, Navigation, Instrument, Spacecraft, Ephemeris," offers a Rust-native approach to these challenges. This toolkit provides a suite of functionalities including but not limited to:

+ Loading SPK, BPC, PCK, FK, and TPC files.
+ High-precision translations, rotations, and their combination (rigid body transformations).
+ Comprehensive time system conversions using the hifitime library (including TT, TAI, ET, TDB, UTC, GPS time, and more).

ANISE stands validated against the traditional SPICE toolkit, ensuring accuracy and reliability, with translations achieving machine precision (2e-16) and rotations presenting minimal error (less than two arcseconds in the pointing of the rotation axis and less than one arcsecond in the angle about this rotation axis).

## Features

+ **High Precision**: Achieves near machine precision in translations and minimal errors in rotations.
+ **Time System Conversions**: Extensive support for various time systems crucial in astrodynamics.
+ **Rust Efficiency**: Harnesses the speed and safety of Rust for space computations.

## Getting Started

## Installation

```sh
cargo add anise
```

## Usage

Here's a simple example to get started with ANISE:

```rust

// Example code demonstrating a basic operation with ANISE
```

Please refer to the [test suite](./tests/) for comprehensive examples until I write better documentation.

## Contributing

Contributions to ANISE are welcome! Whether it's in the form of feature requests, bug reports, code contributions, or documentation improvements, every bit of help is greatly appreciated.

## License

ANISE is distributed under the Mozilla Public License 2.0 (MPL-2.0), offering a balanced approach to open-source by allowing the use of source code within both open and proprietary software. MPL-2.0 requires that modifications to the covered code be released under the same license, thus ensuring improvements remain open-source. However, it allows the combining of the covered software with proprietary parts, providing flexibility for both academic and commercial integrations.

For more details, please see the [full text of the license](./LICENSE) or read [a summary by Github](https://choosealicense.com/licenses/mpl-2.0/).

## Acknowledgements

ANISE is heavily inspired by the NAIF SPICE toolkit and its excellent documentation


## Contact

For any inquiries, feedback, or discussions, please [open an issue here](https://github.com/nyx-space/anise/issues) or contact the maintainer at christopher.rabotin@gmail.com.
