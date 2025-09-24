# ANISE Architecture

## Data Storage

The `Almanac` struct is the central component for storing all data within ANISE. It holds both raw NAIF (Navigation and Ancillary Information Facility) data and ANISE's own binary data formats. This design allows for a unified interface to access different types of ephemeris and orientation data.

### NAIF Data

NAIF data, such as SPK (Spacecraft and Planet Kernel) and BPC (Binary PCK), are loaded and stored in their original format. The `Almanac` struct contains arrays to hold multiple `SPK` and `BPC` objects:

```rust
pub struct Almanac {
    pub spk_data: [Option<SPK>; MAX_LOADED_SPKS],
    pub bpc_data: [Option<BPC>; MAX_LOADED_BPCS],
    // ...
}
```

This approach allows for efficient access to the raw data and leverages the existing, well-tested NAIF data structures.

### ANISE Data

ANISE uses a custom binary format for its own data types, such as planetary data, spacecraft data, and Euler parameters. These are stored in `DataSet` objects within the `Almanac`:

```rust
pub struct Almanac {
    // ...
    pub planetary_data: PlanetaryDataSet,
    pub spacecraft_data: SpacecraftDataSet,
    pub euler_param_data: EulerParameterDataSet,
    pub location_data: LocationDataSet,
}
```

Each `DataSet` is a generic structure that contains the data itself, along with metadata and a lookup table for efficient access. This allows for a consistent way to handle different types of ANISE data.

## Memory Allocation

ANISE is designed to be efficient in its memory usage. When a file is loaded, it is first read into a `Bytes` object. The `file2heap!` macro is used for this purpose, which reads the entire file into memory.

```rust
let bytes = file2heap!(path).context(LoadingSnafu {
    path: path.to_string(),
})?;
```

The `Bytes` object is a smart pointer that provides a view into a contiguous region of memory. This allows for zero-copy parsing of the data, as the data can be read directly from the `Bytes` object without needing to be copied into other data structures. This is particularly important for large NAIF files, where copying the data would be a significant performance bottleneck.

When parsing ANISE data, the `from_der` function is used to decode the data directly from the `Bytes` object. This function is implemented for each `DataSet` type and uses the `der` crate to perform the decoding. This process is also designed to be zero-copy, further reducing memory usage and improving performance.

## Querying

ANISE provides a flexible querying system that allows for efficient access to the stored data. This is achieved through a set of traits that define the interface for interacting with the different data types.

### The `DataSetT` Trait

The `DataSetT` trait is the foundation of the ANISE data storage system. It defines the basic properties of a dataset, such as its name, and provides the necessary functionality for encoding and decoding the data.

```rust
pub trait DataSetT: Clone + Default + Encode + for<'a> Decode<'a> {
    const NAME: &'static str;
}
```

This trait is implemented by all ANISE data types, ensuring that they can be handled in a consistent manner. The `DataSet` struct is a generic container that uses the `DataSetT` trait to manage the data.

### The `DAF` Trait

The `DAF` (Double Precision Array File) trait provides an interface for interacting with NAIF data. It defines a set of methods for accessing the data in a DAF file, such as searching for summaries and reading data records.

```rust
pub trait DAF<T: NAIFSummaryRecord>: Send + Sync {
    // ...
}
```

The `DAF` trait is implemented by the `SPK` and `BPC` structs, allowing them to be used in a generic way. The trait provides methods for searching for segments, which are used to find the data for a specific object at a specific time.

## Error Handling

ANISE uses the `snafu` crate for its error handling. This provides a structured and consistent way to define and manage errors throughout the application.

### Error Types

Errors are defined as enums, with each variant representing a specific error condition. For example, the `AlmanacError` enum defines the possible errors that can occur when working with an `Almanac` object:

```rust
#[derive(Debug, PartialEq, Snafu)]
#[snafu(visibility(pub))]
pub enum AlmanacError {
    #[snafu(display("{action} encountered an error with ephemeris computation {source}"))]
    Ephemeris {
        action: &'static str,
        #[snafu(source(from(EphemerisError, Box::new)))]
        source: Box<EphemerisError>,
    },
    // ...
}
```

The `#[snafu]` attribute is used to automatically generate the necessary boilerplate for the error type, including implementations of the `Error` and `Display` traits.

### Error Propagation

Errors are propagated up the call stack using the `?` operator. The `context` method from `snafu` is used to add context to an error, providing more information about where the error occurred and what caused it.

For example, when loading a file, the `LoadingSnafu` context is used to add the file path to the error:

```rust
let bytes = file2heap!(path).context(LoadingSnafu {
    path: path.to_string(),
})?;
```

This ensures that when an error is reported, it contains a clear and detailed description of the problem, making it easier to diagnose and fix.

## Querying Examples

To illustrate how the different components of ANISE work together, this section details the call stacks for some common operations.

### `Almanac::transform`

The `Almanac::transform` function is used to calculate the state of a target frame with respect to an observer frame. This involves both a translation and a rotation.

1.  **`Almanac::transform(target_frame, observer_frame, epoch, ab_corr)`**
    *   Calls `Almanac::translate` to compute the translational component of the transformation.
        *   `Almanac::translate` calls `Almanac::common_ephemeris_path` to find the common ancestor of the two frames in the ephemeris tree.
        *   It then calls `Almanac::translation_parts_to_parent` repeatedly to compute the state of each frame relative to the common ancestor.
        *   Finally, it combines the states to get the relative state of the target with respect to the observer.
    *   Calls `Almanac::rotate` to compute the rotational component of the transformation.
        *   `Almanac::rotate` calls `Almanac::common_orientation_path` to find the common ancestor of the two frames in the orientation tree.
        *   It then calls `Almanac::rotation_to_parent` repeatedly to compute the rotation of each frame relative to the common ancestor.
        *   Finally, it combines the rotations to get the relative rotation of the target with respect to the observer.
    *   The resulting state is then rotated by the computed DCM to get the final transformed state.

### Frame Data Retrieval

Frame data, such as mass and shape, is retrieved using the `frame_from_uid` function.

1.  **`Almanac::frame_from_uid(uid)`**
    *   The `uid` is converted into a `FrameUid`.
    *   The `ephemeris_id` from the `FrameUid` is used to look up the `PlanetaryData` in the `planetary_data` dataset.
    *   The `get_by_id` method of the `DataSet` is called, which retrieves the data from the `data` vector using the index from the `lut`.
    *   The `to_frame` method is called on the retrieved `PlanetaryData` to construct the `Frame` object. The mass (`mu_km3_s2`) and shape information are part of the `PlanetaryData` struct.

### Azimuth/Elevation Calculation

Azimuth and elevation are calculated using the `azimuth_elevation_range_sez` family of functions.

1.  **`Almanac::azimuth_elevation_range_sez_from_location_name(rx, location_name, ...)`**
    *   Calls `location_data.get_by_name` to retrieve the `Location` object from the `location_data` dataset.
    *   Calls `Almanac::azimuth_elevation_range_sez_from_location`.
2.  **`Almanac::azimuth_elevation_range_sez_from_location(rx, location, ...)`**
    *   Calls `Almanac::frame_from_uid` to get the `Frame` for the location.
    *   Calls `Almanac::angular_velocity_wtr_j2000_rad_s` to get the angular velocity of the frame.
    *   Creates a `Orbit` object for the location using `Orbit::try_latlongalt_omega`.
    *   Calls `Almanac::azimuth_elevation_range_sez`.
3.  **`Almanac::azimuth_elevation_range_sez(rx, tx, ...)`**
    *   Calculates the SEZ (South-East-Zenith) frame for the transmitter (`tx`).
    *   Transforms the receiver (`rx`) into the transmitter's frame using `Almanac::transform_to`.
    *   Calculates the range, azimuth, and elevation from the relative positions in the SEZ frame.