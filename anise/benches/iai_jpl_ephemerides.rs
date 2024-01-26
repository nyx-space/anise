use anise::{
    constants::frames::{EARTH_J2000, MOON_J2000},
    file2heap,
    prelude::*,
};

use iai::black_box;

const NUM_QUERIES_PER_PAIR: f64 = 100.0;

fn benchmark_spice_single_hop_type2_cheby() {
    let start_epoch = Epoch::from_gregorian_at_noon(1900, 1, 1, TimeScale::ET);
    let end_epoch = Epoch::from_gregorian_at_noon(2099, 1, 1, TimeScale::ET);
    let time_step = ((end_epoch - start_epoch).to_seconds() / NUM_QUERIES_PER_PAIR).seconds();
    let time_it = TimeSeries::exclusive(start_epoch, end_epoch - time_step, time_step);

    // SPICE load
    spice::furnsh("../data/de440s.bsp");

    for epoch in time_it {
        black_box(spice::spkezr(
            "EARTH",
            epoch.to_et_seconds(),
            "J2000",
            "NONE",
            "MOON",
        ));
    }

    spice::unload("../data/de440s.bsp");
}

fn benchmark_anise_single_hop_type2_cheby() {
    let start_epoch = Epoch::from_gregorian_at_noon(1900, 1, 1, TimeScale::ET);
    let end_epoch = Epoch::from_gregorian_at_noon(2099, 1, 1, TimeScale::ET);
    let time_step = ((end_epoch - start_epoch).to_seconds() / NUM_QUERIES_PER_PAIR).seconds();
    let time_it = TimeSeries::exclusive(start_epoch, end_epoch - time_step, time_step);

    let path = "../data/de440s.bsp";
    let buf = file2heap!(path).unwrap();
    let spk = SPK::parse(buf).unwrap();
    let ctx = Almanac::from_spk(spk).unwrap();

    for epoch in time_it {
        black_box(
            ctx.translate_geometric(EARTH_J2000, MOON_J2000, epoch)
                .unwrap(),
        );
    }
}

iai::main!(
    benchmark_spice_single_hop_type2_cheby,
    benchmark_anise_single_hop_type2_cheby
);
