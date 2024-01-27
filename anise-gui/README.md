# ANISE GUI

ANISE provides a graphical interface to inspect SPK, BPC, and PCA (Planetary Constant ANISE) files. Allows you to check the start/end times of the segments (shown in whichever time scale you want, including UNIX UTC seconds).

**Latest binaries:** <https://github.com/nyx-space/anise/discussions/138>

When updates are published, they'll be announced in the [Discussions](https://github.com/nyx-space/anise/discussions).

## Demos

Inspect an SPK file ([video link](http://public-data.nyxspace.com/anise/demo/ANISE-SPK.webm)):

![Inspect an SPK file](http://public-data.nyxspace.com/anise/demo/ANISE-SPK.gif)

Inspect an Binary PCK file (BPC) ([video link](http://public-data.nyxspace.com/anise/demo/ANISE-BPC.webm)):

![Inspect an SPK file](http://public-data.nyxspace.com/anise/demo/ANISE-BPC.gif)

## Building

### Native

To build the GUI natively to your platform, just run `cargo build --bin anise-gui` from anywhere in the repo. You'll need to have Rust installed.

To run it, use `cargo run` instead of `cargo build`. Keep in mind that if building from a [distrobox](https://github.com/89luca89/distrobox), you will need to exit the distrobox to execute the program. Rust does not require dynamic loading of libraries, so it should work on any platform without issue.

### Web Assembly (wasm)

To build the GUI for web assembly, you will need to install [Trunk](https://trunkrs.dev/).

1. Install the required target with `rustup target add wasm32-unknown-unknown`.
1. Install Trunk with `cargo install --locked trunk`.
1. Run `trunk serve anise-gui/index.html` to build and serve on http://127.0.0.1:8080. Trunk will rebuild automatically if you edit the project.

Note that running `trunk serve` without any arguments will cause an error because the index page is not in the root of the project.
