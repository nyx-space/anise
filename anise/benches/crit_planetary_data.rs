use std::path::PathBuf;

use anise::{constants::frames::EARTH_ITRF93, naif::kpl::parser::convert_tpc, prelude::*};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let pca = PathBuf::from_str("pck11.pca").unwrap();
    let planetary_data = convert_tpc("../data/pck00011.tpc", "../data/gm_de431.tpc").unwrap();
    planetary_data.save_as(&pca, true).unwrap();

    let almanac = Almanac::new("pck11.pca").unwrap();

    c.bench_function("Frame fetch from planetary dataset", |b| {
        b.iter(|| black_box(almanac.clone().frame_from_uid(EARTH_ITRF93).unwrap()))
    });
}

criterion_group!(pca, criterion_benchmark);
criterion_main!(pca);
