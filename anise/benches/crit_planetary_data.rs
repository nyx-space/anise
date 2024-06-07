use std::path::PathBuf;

use anise::{constants::frames::EARTH_ITRF93, naif::kpl::parser::convert_tpc, prelude::*};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_fetch(almanac: &Almanac, frame: Frame) {
    black_box(almanac.frame_from_uid(frame).unwrap());
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let pca = PathBuf::from_str("pck11.pca").unwrap();
    let planetary_data = convert_tpc("../data/pck00011.tpc", "../data/gm_de431.tpc").unwrap();
    planetary_data.save_as(&pca, true).unwrap();

    let almanac = Almanac::new("pck11.pca").unwrap();

    c.bench_function("Frame fetch from planetary dataset", |b| {
        b.iter(|| benchmark_fetch(&almanac, EARTH_ITRF93))
    });
}

criterion_group!(pca, criterion_benchmark);
criterion_main!(pca);
