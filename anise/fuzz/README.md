# Fuzz Testing for Anise

This directory contains fuzz tests for the Anise project, using [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) and [`libFuzzer`](https://llvm.org/docs/LibFuzzer.html). Fuzzing helps uncover bugs and security issues by running randomized inputs through your code.

---

## Prerequisites

- **Rust nightly toolchain** (required by `cargo-fuzz`)
- **cargo-fuzz** (install with `cargo install cargo-fuzz`)
- (Optional) **LLVM tools** for advanced debugging

---

## Setup

1. **Install Rust Nightly:**
   ```sh
   rustup toolchain install nightly
   rustup override set nightly
   ```

2. **Install cargo-fuzz:**
   ```sh
   cargo install cargo-fuzz
   ```

---

## Building Fuzz Targets

Fuzz targets must be built from within the `anise/anise` folder consisting of the Rust code. To build all fuzz targets:
```sh
cargo fuzz build
```
Or build a specific target:
```sh
cargo fuzz build <target-name>
```

---

## Running Fuzz Tests

Fuzz targets must be run from within the `anise/anise` folder consisting of the Rust code. To run a fuzz target:
```sh
cargo fuzz run <target-name>
```
You can limit runtime (in seconds) with:
```sh
cargo fuzz run <target-name> -- -max_total_time=60
```

---

## Debugging Fuzz Failures

When a crash or bug is found, a minimized test case will be saved in the `artifacts/` directory. To debug:
1. Run the target with the crashing input:
   ```sh
   cargo fuzz run <target-name> artifacts/<target-name>/<crash-file>
   ```
2. Use `RUST_BACKTRACE=1` for stack traces:
   ```sh
   RUST_BACKTRACE=1 cargo fuzz run <target-name> artifacts/<target-name>/<crash-file>
   ```

---

## Adding New Fuzz Targets

1. Create a new file in [`fuzz_targets/`](fuzz_targets/) (e.g., `my_target.rs`).
2. Implement a `fuzz_target!` macro as in other targets.
3. Register the new target in [`Cargo.toml`](Cargo.toml) under `[[bin]]`:
   ```toml
   [[bin]]
   name = "my_target"
   path = "fuzz_targets/my_target.rs"
   test = false
   doc = false
   bench = false
   ```
4. (Optional) If using a custom structure, update [`src/lib.rs`](src/lib.rs) to include arbitrary structure.
5. (Optional) Add a seed corpus in the `corpus/` directory.

---

## OSS-Fuzz Integration

This fuzz suite is integrated with [OSS-Fuzz](https://github.com/google/oss-fuzz) for continuous fuzzing on Google's infrastructure.

- **Build system:** OSS-Fuzz uses the same `cargo-fuzz` targets defined here.
- **Adding/Removing Targets:** Update both this repo and the [OSS-Fuzz project YAML](https://github.com/google/oss-fuzz/tree/master/projects/anise) as needed.
- **Corpus/Artifacts:** OSS-Fuzz manages its own corpus and crash artifacts, but you can sync with local corpora for better coverage.
- **Updating Dependencies:** Keep dependencies in sync with upstream to avoid build failures in OSS-Fuzz.
- **Contact:** If you update the fuzz targets or dependencies, notify the OSS-Fuzz maintainers via a pull request or issue.

For more details, see the [OSS-Fuzz documentation](https://google.github.io/oss-fuzz/).

---

## Directory Structure

- [`fuzz_targets/`](fuzz_targets/): Individual fuzz target entrypoints.
- [`src/lib.rs`](src/lib.rs): Shared fuzzing utilities and types.
- [`corpus/`](corpus/): Optional seed corpora for each target.
- [`artifacts/`](artifacts/): Crash/minimized test cases.
- [`Cargo.toml`](Cargo.toml): Fuzz target registration.

---

## References

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Structure-Aware fuzzing documentation](https://rust-fuzz.github.io/book/cargo-fuzz/structure-aware-fuzzing.html)
- [OSS-Fuzz documentation](https://google.github.io/oss-fuzz/)

---

Feel free to open issues or pull requests to improve the fuzzing setup!
