# ANISE

This is the main implementation of the ANISE toolkit specifications in Rust.

# Features

- Thread safe computations from the SPICE toolkit
- Convert NAIF SPK files into ANISE files with `anise convert path-to-spk`, yields a 5% space reduction (only Chebyshev Type 2 currently supported)
- Inspect an ANISE file with `anise inspect path-to-file`
- Perform frame translations (no rotations yet) between whichever ephemeris is in the context, or from a provided Cartesian state into another frame

Please refer to https://github.com/anise-toolkit/specs for the specifications.

# Design
TODO
## Implementation choices
As with any specification, some implementation choices, or limitations, must be made. In particular, ANISE.rs does not use any memory allocation, therefore everything is statically allocated and lives on the program stack. This is important for performance for programs on soft real-time embedded devices.

### Depth of translations and rotations
In this implementation, a translation or a rotation may not be more than 8 nodes from the root of the ANISE context.

**Behavior:** this library can still read an ANISE file which stores data deeper than 8 nodes, however, it will not be able to perform any translations or rotations which involve it, and instead return a `MaxTreeDepth` error.

**Practical example:**
The following ephemeris is valid, can be stored, and computations made with this ephemeris (from central node of the context to the further away):

```
Solar System barycenter
╰─> Earth Moon Barycenter
    ╰─> Earth
        ╰─> ISS
            ╰─> Columbus
                ╰─> Hub window #1
                    ╰─> Camera mount
                        ╰─> Camera lense /!\ MAX DEPTH REACHED (cannot add a deeper ephemeris) /!\
```

# Development
## Requirements
1. `rustc` version `1.64` or higher (required for the 2021 edition): https://rust-lang.org/ (TODO: Set a minimum compatible rust version)
2. `git`
1. `rust-spice` is used for exhaustive testing of the SPICE interoperability. It requires the cspice library.