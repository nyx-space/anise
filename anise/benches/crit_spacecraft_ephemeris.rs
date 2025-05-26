use anise::{constants::frames::EARTH_J2000, file2heap, prelude::*};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use std::hint::black_box;

const NUM_QUERIES: f64 = 1000.0;
const RNG_SEED: u64 = 1234567890;

fn benchmark_spice_single_hop_type13_hermite(time_vec: &[Epoch]) {
    // SPICE load
    spice::furnsh("../data/de440s.bsp");
    spice::furnsh("../data/lro.bsp");

    for epoch in time_vec {
        black_box(spice::spkezr(
            "-85",
            epoch.to_et_seconds(),
            "J2000",
            "NONE",
            "EARTH",
        ));
    }

    spice::unload("../data/lro.bsp");
    spice::unload("../data/de440s.bsp");
}

fn benchmark_anise_single_hop_type13_hermite(ctx: &Almanac, time_vec: &[Epoch]) {
    let my_sc_j2k = Frame::from_ephem_j2000(-85);
    for epoch in time_vec.iter().copied() {
        black_box(
            ctx.translate_geometric(my_sc_j2k, EARTH_J2000, epoch)
                .unwrap(),
        );
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let start_epoch = Epoch::from_gregorian_at_noon(2023, 12, 15, TimeScale::UTC);
    let end_epoch = Epoch::from_gregorian_at_midnight(2024, 1, 9, TimeScale::UTC);
    let time_step = ((end_epoch - start_epoch).to_seconds() / NUM_QUERIES).seconds();
    let time_it = TimeSeries::exclusive(start_epoch, end_epoch - time_step, time_step);
    // Shuffle the time iterator
    let mut rng = Pcg64::seed_from_u64(RNG_SEED);
    let mut time_vec: Vec<Epoch> = time_it.collect();
    time_vec.shuffle(&mut rng);

    let path = "../data/de440s.bsp";
    let buf = file2heap!(path).unwrap();
    let spk = SPK::parse(buf).unwrap();

    let buf = file2heap!("../data/lro.bsp").unwrap();
    let spacecraft = SPK::parse(buf).unwrap();

    let ctx = Almanac::from_spk(spk)
        .unwrap()
        .with_spk(spacecraft)
        .unwrap();

    c.bench_function("ANISE hermite", |b| {
        b.iter(|| benchmark_anise_single_hop_type13_hermite(&ctx, &time_vec))
    });

    c.bench_function("SPICE hermite", |b| {
        b.iter(|| benchmark_spice_single_hop_type13_hermite(&time_vec))
    });
}

criterion_group!(hermite, criterion_benchmark);
criterion_main!(hermite);
