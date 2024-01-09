use anise::{constants::frames::EARTH_J2000, file2heap, prelude::*};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

const NUM_QUERIES: f64 = 100.0;

fn benchmark_spice_single_hop_type13_hermite(time_it: TimeSeries) {
    // SPICE load
    spice::furnsh("../data/gmat-hermite.bsp");

    for epoch in time_it {
        black_box(spice::spkezr(
            "-10000001",
            epoch.to_et_seconds(),
            "J2000",
            "NONE",
            "EARTH",
        ));
    }

    spice::unload("../data/gmat-hermite.bsp");
}

fn benchmark_anise_single_hop_type13_hermite(ctx: &Almanac, time_it: TimeSeries) {
    let my_sc_j2k = Frame::from_ephem_j2000(-10000001);
    for epoch in time_it {
        black_box(
            ctx.translate_geometric(my_sc_j2k, EARTH_J2000, epoch)
                .unwrap(),
        );
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let start_epoch = Epoch::from_gregorian_at_noon(2000, 1, 1, TimeScale::UTC);
    let end_epoch = Epoch::from_gregorian_hms(2000, 1, 1, 15, 0, 0, TimeScale::UTC);
    let time_step = ((end_epoch - start_epoch).to_seconds() / NUM_QUERIES).seconds();
    let time_it = TimeSeries::exclusive(start_epoch, end_epoch - time_step, time_step);

    let path = "../data/de440s.bsp";
    let buf = file2heap!(path).unwrap();
    let spk = SPK::parse(buf).unwrap();

    let buf = file2heap!("../data/gmat-hermite.bsp").unwrap();
    let spacecraft = SPK::parse(buf).unwrap();

    let ctx = Almanac::from_spk(spk)
        .unwrap()
        .with_spk(spacecraft)
        .unwrap();

    // Load SPICE data
    spice::furnsh("../data/de440s.bsp");

    c.bench_function("ANISE hermite", |b| {
        b.iter(|| benchmark_anise_single_hop_type13_hermite(&ctx, time_it.clone()))
    });

    c.bench_function("SPICE hermite", |b| {
        b.iter(|| benchmark_spice_single_hop_type13_hermite(time_it.clone()))
    });
}

criterion_group!(hermite, criterion_benchmark);
criterion_main!(hermite);
