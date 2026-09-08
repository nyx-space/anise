use anise::{
    constants::frames::{EARTH_ICRS, MOON_ICRS},
    file2heap,
    prelude::*,
};
use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

#[library_benchmark]
fn benchmark_spice_single_hop_type2_cheby() {
    let epoch = Epoch::from_gregorian_at_noon(2025, 5, 25, TimeScale::ET);

    // SPICE load
    spice::furnsh("../data/de440s.bsp");

    black_box(spice::spkezr(
        "EARTH",
        epoch.to_et_seconds(),
        "J2000",
        "NONE",
        "MOON",
    ));

    spice::unload("../data/de440s.bsp");
}

#[library_benchmark]
fn benchmark_anise_single_hop_type2_cheby() {
    let epoch = Epoch::from_gregorian_at_noon(2025, 5, 25, TimeScale::ET);

    let path = "../data/de440s.bsp";
    let buf = file2heap!(path).unwrap();
    let spk = SPK::parse(buf).unwrap();
    let ctx = Almanac::from_spk(spk);

    black_box(
        ctx.translate_geometric(EARTH_ICRS, MOON_ICRS, epoch)
            .unwrap(),
    );
}

library_benchmark_group!(name = bench_jpl_ephem; benchmarks = benchmark_anise_single_hop_type2_cheby, benchmark_spice_single_hop_type2_cheby);
main!(library_benchmark_groups = bench_jpl_ephem);
