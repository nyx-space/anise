use anise::{constants::frames::EARTH_J2000, file_mmap, prelude::*};

use iai::black_box;
use spice;

fn benchmark_spice_single_hop_type13_hermite() {
    let epoch = Epoch::from_gregorian_hms(2000, 1, 1, 14, 0, 0, TimeScale::UTC);

    // SPICE load
    spice::furnsh("data/gmat-hermite.bsp");

    black_box(spice::spkezr(
        "-10000001",
        epoch.to_et_seconds(),
        "J2000",
        "NONE",
        "EARTH",
    ));

    spice::unload("data/gmat-hermite.bsp");
}

fn benchmark_anise_single_hop_type13_hermite() {
    let epoch = Epoch::from_gregorian_hms(2000, 1, 1, 14, 0, 0, TimeScale::UTC);

    let path = "./data/de438s.bsp";
    let buf = file_mmap!(path).unwrap();
    let spk = SPK::parse(&buf).unwrap();

    let buf = file_mmap!("data/gmat-hermite.bsp").unwrap();
    let spacecraft = SPK::parse(&buf).unwrap();

    let ctx = Context::from_spk(&spk)
        .unwrap()
        .load_spk(&spacecraft)
        .unwrap();

    let my_sc_j2k = Frame::from_ephem_j2000(-10000001);

    black_box(
        ctx.translate_from_to_km_s_geometric(my_sc_j2k, EARTH_J2000, epoch)
            .unwrap(),
    );
}

iai::main!(
    benchmark_spice_single_hop_type13_hermite,
    benchmark_anise_single_hop_type13_hermite
);
