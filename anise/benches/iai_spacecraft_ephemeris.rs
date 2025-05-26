use anise::{constants::frames::EARTH_J2000, file2heap, prelude::*};
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

#[library_benchmark]
fn benchmark_spice_single_hop_type13_hermite() {
    let epoch = Epoch::from_gregorian_hms(2023, 12, 15, 14, 0, 0, TimeScale::UTC);

    // SPICE load
    spice::furnsh("../data/de440s.bsp");
    spice::furnsh("../data/lro.bsp");

    black_box(spice::spkezr(
        "-85",
        epoch.to_et_seconds(),
        "J2000",
        "NONE",
        "EARTH",
    ));

    spice::unload("../data/lro.bsp");
    spice::unload("../data/de440s.bsp");
}

#[library_benchmark]
fn benchmark_anise_single_hop_type13_hermite() {
    let epoch = Epoch::from_gregorian_hms(2023, 12, 15, 14, 0, 0, TimeScale::UTC);

    let path = "../data/de440s.bsp";
    let buf = file2heap!(path).unwrap();
    let spk = SPK::parse(buf).unwrap();

    let buf = file2heap!("../data/lro.bsp").unwrap();
    let spacecraft = SPK::parse(buf).unwrap();

    let ctx = Almanac::from_spk(spk)
        .unwrap()
        .with_spk(spacecraft)
        .unwrap();

    let my_sc_j2k = Frame::from_ephem_j2000(-85);

    black_box(
        ctx.translate_geometric(my_sc_j2k, EARTH_J2000, epoch)
            .unwrap(),
    );
}

library_benchmark_group!(name = bench_spacecraft_ephem; benchmarks = benchmark_anise_single_hop_type13_hermite, benchmark_spice_single_hop_type13_hermite);
main!(library_benchmark_groups = bench_spacecraft_ephem);
