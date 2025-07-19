# ANISE (Attitude, Navigation, Instrument, Spacecraft, Ephemeris)

ANISE is a rewrite of the core functionalities of the NAIF SPICE toolkit with enhanced performance, and ease of use, while leveraging Rust's safety and speed.

ANISE was used throughout the operations of the Firefly Blue Ghost lunar lander, from launch until successful landing. As such, the library is now considered NASA Technology Readiness Level (TRL) 9.

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
+ **Auto-downloading capability**: ANISE simplifies your workflow by automatically downloading the latest Earth orientation parameters, or any other SPICE or ANISE file from a remote location, seamlessly integrating them into the `Almanac` for immediate use.

## Usage

Start using it by adding to your Rust project:

```sh
cargo add anise
```

### Frame Transformation of an Orbit

ANISE provides the ability to create Cartesian states (also simply called `Orbit`s), calculate orbital elements from them in an error free way (computations that may fail return a `Result` type), and transform these states into other frames via the loaded context, called `Almanac`, which stores all of the SPICE and ANISE files you need.

```rust
use anise::prelude::*;
// ANISE provides pre-built frames, but examples below show how to build them from their NAIF IDs.
use anise::constants::frames::{EARTH_ITRF93, EARTH_J2000};

// Initialize an empty Almanac
let ctx = Almanac::default();

// Load a SPK/BSP file
let spk = SPK::load("../data/de440.bsp").unwrap();
// Load the high precision ITRF93 kernel
let bpc = BPC::load("../data/earth_latest_high_prec.bpc").unwrap();
// Build the planetary constants file, which includes the gravitational parameters and the IAU low fidelity rotations
use anise::naif::kpl::parser::convert_tpc;
// Note that the PCK variable can also be serialized to disk to avoid having to rebuild it next time.
let pck = convert_tpc("../data/pck00008.tpc", "../data/gm_de431.tpc").unwrap();

// And add all of these to the Almanac context
let almanac = ctx
    .with_spk(spk)
    .unwrap()
    .with_bpc(bpc)
    .unwrap()
    .with_planetary_data(pck);

// Let's build an orbit
// Start by grabbing a copy of the frame.
let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();

// Define an epoch, in TDB, but you may specify UTC, TT, TAI, GPST, and more.
let epoch = Epoch::from_str("2021-10-29 12:34:56 TDB").unwrap();

// Define the orbit from its Keplerian orbital elements.
// Note that we must specify the frame of this orbit: ANISE checks all frames are valid before any translation or rotation, even internally.
let orig_state = Orbit::keplerian(
    8_191.93, 1e-6, 12.85, 306.614, 314.19, 99.887_7, epoch, eme2k,
);

// Transform that orbit into another frame.
let state_itrf93 = almanac
    .transform_to(orig_state, EARTH_ITRF93, None)
    .unwrap();

// The `:x` prints this orbit's Keplerian elements
println!("{orig_state:x}");
// The `:X` prints the prints the range, altitude, latitude, and longitude with respect to the planetocentric frame in floating point with units if frame is celestial,
println!("{state_itrf93:X}");

// Convert back
let from_state_itrf93_to_eme2k = almanac
    .transform_to(state_itrf93, EARTH_J2000, Aberration::NONE)
    .unwrap();

println!("{from_state_itrf93_to_eme2k}");

// Check that our return data matches the original one exactly
assert_eq!(orig_state, from_state_itrf93_to_eme2k);
```

### Load and query a PCK/BPC file (high fidelity rotation)

```rust
use anise::prelude::*;
use anise::constants::frames::{EARTH_ITRF93, EME2000};

let pck = "../data/earth_latest_high_prec.bpc";

let bpc = BPC::load(pck).unwrap();
let almanac = Almanac::from_bpc(bpc).unwrap();

// Load the useful frame constants
use anise::constants::frames::*;

// Define an Epoch in the dynamical barycentric time scale
let epoch = Epoch::from_str("2020-11-15 12:34:56.789 TDB").unwrap();

// Query for the DCM
let dcm = almanac.rotate(EARTH_ITRF93, EME2000, epoch).unwrap();

println!("{dcm}");
```

### Load and query a text PCK/KPL file (low fidelity rotation)

```rust
use anise::prelude::*;
// Load the TPC converter, which will create the ANISE representation too, in ASN1 format, that you may reuse.
use anise::naif::kpl::parser::convert_tpc;

// Note that the ASN1 ANISE format for planetary data also stores the gravity parameters, so we must convert both at once into a single ANISE file.
let planetary_data = convert_tpc("../data/pck00008.tpc", "../data/gm_de431.tpc").unwrap();

let almanac = Almanac {
    planetary_data,
    ..Default::default()
};

// Load the useful frame constants
use anise::constants::frames::*;

// Define an Epoch in the dynamical barycentric time scale
let epoch = Epoch::from_str("2020-11-15 12:34:56.789 TDB").unwrap();

// Query for the DCM to the immediate parent
let dcm = almanac.rotation_to_parent(IAU_VENUS_FRAME, epoch).unwrap();

println!("{dcm}");
```

### Load and query an SPK/BSP file (ephemeris)

```rust
use anise::prelude::*;
use anise::constants::frames::*;

let spk = SPK::load("../data/de440s.bsp").unwrap();
let ctx = Almanac::from_spk(spk).unwrap();

// Define an Epoch in the dynamical barycentric time scale
let epoch = Epoch::from_str("2020-11-15 12:34:56.789 TDB").unwrap();

let state = ctx
    .translate(
        VENUS_J2000, // Target
        EARTH_MOON_BARYCENTER_J2000, // Observer
        epoch,
        None,
    )
    .unwrap();

println!("{state}");
```

## Validation

[![ANISE Validation](https://github.com/nyx-space/anise/actions/workflows/rust.yml/badge.svg)](https://github.com/nyx-space/anise/actions/workflows/rust.yml)

ANISE is validated by running the same queries in ANISE and in SPICE (single threaded) in the _Validation_ step linked above. This workflow validates 101,000 BSP queries in the DE440.BSP file, and 7305 queries each frame in the PCK08 file (every day for 20 years), along with thousands of rotations from Earth high precision BPC file.

**Note:** The PCK data comes from the IAU Reports, which publishes angle, angle rate, and angle acceleration data, expressed in centuries past the J2000 reference epoch.
ANISE uses Hifitime for time conversions. Hifitime's reliance solely on integers for all time computations eliminates the risk of rounding errors. In contrast, SPICE utilizes floating-point values, which introduces rounding errors in calculations like centuries past J2000. Consequently, you might observe a discrepancy of up to 1 millidegree in rotation angles between SPICE and ANISE. However, this difference is a testament to ANISE's superior precision.

## Resources / Assets

For convenience, Nyx Space provides a few important SPICE files on a public bucket:

+ [de440s.bsp](http://public-data.nyxspace.com/anise/de440s.bsp): JPL's latest short-term ephemeris dataset from 1900 until 2050
+ [de440.bsp](http://public-data.nyxspace.com/anise/de440.bsp): JPL's latest long-term ephemeris dataset
+ [pck08.pca](http://public-data.nyxspace.com/anise/v0.5/pck08.pca): planetary constants ANISE (`pca`) kernel, built from the JPL gravitational data [gm_de431.tpc](http://public-data.nyxspace.com/anise/gm_de431.tpc) and JPL's plantary constants file [pck00008.tpc](http://public-data.nyxspace.com/anise/pck00008.tpc)
+ [pck11.pca](http://public-data.nyxspace.com/anise/v0.5/pck11.pca): planetary constants ANISE (`pca`) kernel, built from the JPL gravitational data [gm_de431.tpc](http://public-data.nyxspace.com/anise/gm_de431.tpc) and JPL's plantary constants file [pck00011.tpc](http://public-data.nyxspace.com/anise/pck00011.tpc)
+ [moon_fk_de440.epa](http://public-data.nyxspace.com/anise/v0.5/moon_fk_de440.epa): Euler Parameter ANISE (`epa`) kernel, built from the JPL Moon Frame Kernel `moon_080317.txt`

You may load any of these using the `load()` shortcut that will determine the file type upon loading, e.g. `let almanac = Almanac::new("pck08.pca").unwrap();` or in Python `almanac = Almanac("pck08.pca")`. To automatically download remote assets, from the Nyx Cloud or elsewhere, use the MetaAlmanac: `almanac = MetaAlmanac("ci_config.dhall").process(true)` in Python.

## How-To Guides

### Calculate Azimuth, Elevation, and Range

To calculate the azimuth, elevation, and range of a spacecraft from a ground station, you can use the `azimuth_elevation_range_sez` function. This function takes the spacecraft's orbit, the ground station's orbit, and the desired frames as input.

```rust
use anise::prelude::*;
use anise::constants::frames::{IAU_EARTH_FRAME, EME2000};
use anise::constants::usual_planetary_constants::MEAN_EARTH_ANGULAR_VELOCITY_DEG_S;
use hifitime::Epoch;
use core::str::FromStr;

// Define location of DSN DSS-65 in Madrid, Spain
const DSS65_LATITUDE_DEG: f64 = 40.427_222;
const DSS65_LONGITUDE_DEG: f64 = 4.250_556;
const DSS65_HEIGHT_KM: f64 = 0.834_939;

let almanac = MetaAlmanac::latest().unwrap();

let eme2k = almanac.frame_from_uid(EME2000).unwrap();

let rx = Orbit::new(
    58643.769540,
    -61696.435624,
    -36178.745722,
    2.148654,
    -1.202489,
    -0.714016,
    Epoch::from_str("2023-11-16T13:35:30.231999909 UTC").unwrap(),
    eme2k,
);

// Build the ground station state
let tx = Orbit::try_latlongalt(
    DSS65_LATITUDE_DEG,
    DSS65_LONGITUDE_DEG,
    DSS65_HEIGHT_KM,
    MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
    rx.epoch,
    almanac.frame_from_uid(IAU_EARTH_FRAME).unwrap(),
)
.unwrap();

let aer = almanac
    .azimuth_elevation_range_sez(rx, tx, None, None)
    .unwrap();

println!("{aer}");
```

### Determine Solar Eclipsing

You can determine if a spacecraft is in a solar eclipse using the `solar_eclipsing` function. This function returns an `Occultation` enum, which can be `Visible`, `Partial`, or `Obstructed`.

```rust
use anise::prelude::*;
use anise::constants::frames::EARTH_J2000;
use hifitime::Epoch;

let almanac = Almanac::new("../data/de440s.bsp").unwrap().load("../data/pck08.pca").unwrap();
let epoch = Epoch::from_gregorian_utc_at_midnight(2024, 1, 1);
let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();
let orbit = Orbit::try_keplerian_altitude(
    500.0, 0.01, 1.0, 72.0, 45.0, 270.0, epoch, eme2k,
).unwrap();

let occult = almanac
    .solar_eclipsing(
        EARTH_J2000,
        orbit.at_epoch(epoch).expect("two body prop failed"),
        None,
    )
    .unwrap();

println!("{occult}");
```

### Transform Between Frames

ANISE allows you to transform an orbit from one frame to another using the `transform_to` function.

```rust
use anise::prelude::*;
use anise::constants::frames::{EARTH_ITRF93, EARTH_J2000};
use hifitime::Epoch;
use core::str::FromStr;

let almanac = MetaAlmanac::latest().unwrap();

let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();
let epoch = Epoch::from_str("2021-10-29 12:34:56 TDB").unwrap();

let orig_state = Orbit::try_keplerian(
    8_191.93, 1e-6, 12.85, 306.614, 314.19, 99.887_7, epoch, eme2k,
).unwrap();

// Transform that into another frame.
let state_itrf93 = almanac
    .transform_to(orig_state, EARTH_ITRF93, Aberration::NONE)
    .unwrap();

println!("{state_itrf93}");
```

### Rotate Between Orientations

You can get the rotation matrix between two orientation frames using the `rotate` function.

```rust
use anise::prelude::*;
use anise::constants::frames::{EARTH_ITRF93, EME2000};
use hifitime::Epoch;
use core::str::FromStr;

let almanac = Almanac::from_bpc(BPC::load("../data/earth_latest_high_prec.bpc").unwrap()).unwrap();
let epoch = Epoch::from_str("2019-03-01T04:02:51.0 ET").unwrap();
let dcm = almanac.rotate(EARTH_ITRF93, EME2000, epoch).unwrap();

println!("{dcm}");
```

## Contributing

Contributions to ANISE are welcome! Whether it's in the form of feature requests, bug reports, code contributions, or documentation improvements, every bit of help is greatly appreciated.

## License

ANISE is distributed under the Mozilla Public License 2.0 (MPL-2.0), offering a balanced approach to open-source by allowing the use of source code within both open and proprietary software. MPL-2.0 requires that modifications to the covered code be released under the same license, thus ensuring improvements remain open-source. However, it allows the combining of the covered software with proprietary parts, providing flexibility for both academic and commercial integrations.

For more details, please see the [full text of the license](./LICENSE) or read [a summary by Github](https://choosealicense.com/licenses/mpl-2.0/).

## Acknowledgements

ANISE is heavily inspired by the NAIF SPICE toolkit and its excellent documentation.


## Contact

For any inquiries, feedback, or discussions, please [open an issue here](https://github.com/nyx-space/anise/issues) or contact the maintainer at christopher.rabotin@gmail.com.

## Versioning

ANISE abides by the Rust versioning rules for dependencies but not necessarily for dev-dependencies. Please open a ticket if this causes an issue.
