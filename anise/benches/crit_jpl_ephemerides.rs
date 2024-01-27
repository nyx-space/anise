use anise::{
    constants::frames::{EARTH_J2000, MOON_J2000},
    file2heap,
    prelude::*,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const NUM_QUERIES_PER_PAIR: f64 = 100.0;

fn benchmark_spice_single_hop_type2_cheby(time_it: TimeSeries) {
    for epoch in time_it {
        black_box(spice::spkezr(
            "EARTH",
            epoch.to_et_seconds(),
            "J2000",
            "NONE",
            "MOON",
        ));
    }
}

fn benchmark_anise_single_hop_type2_cheby(ctx: &Almanac, time_it: TimeSeries) {
    for epoch in time_it {
        black_box(
            ctx.translate_geometric(EARTH_J2000, MOON_J2000, epoch)
                .unwrap(),
        );
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let start_epoch = Epoch::from_gregorian_at_noon(1900, 1, 1, TimeScale::ET);
    let end_epoch = Epoch::from_gregorian_at_noon(2099, 1, 1, TimeScale::ET);
    let time_step = ((end_epoch - start_epoch).to_seconds() / NUM_QUERIES_PER_PAIR).seconds();
    let time_it = TimeSeries::exclusive(start_epoch, end_epoch - time_step, time_step);

    // Load ANISE data
    let path = "../data/de440s.bsp";
    let buf = file2heap!(path).unwrap();
    let spk = SPK::parse(buf).unwrap();
    let ctx = Almanac::from_spk(spk).unwrap();

    // Load SPICE data
    spice::furnsh("../data/de440s.bsp");

    c.bench_function("ANISE ephemerides single hop", |b| {
        b.iter(|| benchmark_anise_single_hop_type2_cheby(&ctx, time_it.clone()))
    });

    c.bench_function("SPICE ephemerides single hop", |b| {
        b.iter(|| benchmark_spice_single_hop_type2_cheby(time_it.clone()))
    });
}

criterion_group!(de440s, criterion_benchmark);
criterion_main!(de440s);
