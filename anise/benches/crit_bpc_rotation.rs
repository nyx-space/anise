use anise::{constants::orientations::ITRF93, prelude::*};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const NUM_QUERIES_PER_PAIR: f64 = 100.0;

fn benchmark_spice_single_hop_type2_cheby(time_it: TimeSeries) {
    for epoch in time_it {
        black_box(spice::pxform(
            "ECLIPJ2000",
            "ITRF93",
            epoch.to_tdb_seconds(),
        ));
    }
}

fn benchmark_anise_single_hop_type2_cheby(ctx: &Almanac, time_it: TimeSeries) {
    for epoch in time_it {
        black_box(
            ctx.rotation_to_parent(Frame::from_orient_ssb(ITRF93), epoch)
                .unwrap(),
        );
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let start_epoch = Epoch::from_gregorian_at_noon(2012, 1, 1, TimeScale::ET);
    let end_epoch = Epoch::from_gregorian_at_noon(2021, 1, 1, TimeScale::ET);
    let time_step = ((end_epoch - start_epoch).to_seconds() / NUM_QUERIES_PER_PAIR).seconds();
    let time_it = TimeSeries::exclusive(start_epoch, end_epoch - time_step, time_step);

    let pck = "../data/earth_latest_high_prec.bpc";
    spice::furnsh(pck);
    let bpc = BPC::load(pck).unwrap();
    let almanac = Almanac::from_bpc(bpc).unwrap();

    c.bench_function("ANISE DAF/BPC single hop to parent", |b| {
        b.iter(|| benchmark_anise_single_hop_type2_cheby(&almanac, time_it.clone()))
    });

    c.bench_function("SPICE DAF/BPC single hop to parent", |b| {
        b.iter(|| benchmark_spice_single_hop_type2_cheby(time_it.clone()))
    });
}

criterion_group!(bpc, criterion_benchmark);
criterion_main!(bpc);
