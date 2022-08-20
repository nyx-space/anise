# ANISE

**This is a proof of concept implementation of the ANISE toolkit specification.**

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
1. `rustc` version `1.56` or higher (required for the 2021 edition): https://rust-lang.org/ (TODO: Set a minimum compatible rust version)
1. `git`

## Generating the Rust files
1. Update the submodule with the specs: `git submodule update --init --recursive`
1. Then generate the files in the `generated` folder: `flatc --gen-all --rust -o generated ../specs/*.fbs`

_Note:_ Because this code will (eventually) have CI/CD, it's easier for now to check-in the generated files instead of creating them at compilation time. It's also much easier for development because rust-analyzer will do all the autocompletion!