# ANISE

**This is a proof of concept implementation of the ANISE toolkit specification.**

Please refer to https://github.com/anise-toolkit/specs for the specifications.

## Development
### Requirements
1. `rustc` version `1.56` or higher (required for the 2021 edition): https://rust-lang.org/
1. `git`

### Generating the Rust files
1. Update the submodule with the specs: `git submodule update --init --recursive`
1. Then generate the files in the `generated` folder: `flatc --gen-all --rust -o generated ../specs/*.fbs`

_Note:_ Because this code will (eventually) have CI/CD, it's easier for now to check-in the generated files instead of creating them at compilation time. It's also much easier for development because rust-analyzer will do all the autocompletion!