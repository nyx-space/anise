#![no_main]
use anise::structure::PlanetaryDataSet;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = PlanetaryDataSet::try_from_bytes(data);
});
