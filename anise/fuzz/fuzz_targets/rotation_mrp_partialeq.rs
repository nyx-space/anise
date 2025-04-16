#![no_main]
use anise::math::rotation::MRP;

use libfuzzer_sys::fuzz_target;

use anise_fuzz::ArbitraryMRP;

fuzz_target!(|data: (ArbitraryMRP, ArbitraryMRP)| {
    let mrp_0 = MRP::from(data.0);
    let mrp_1 = MRP::from(data.1);
    let _ = mrp_0 == mrp_1;
});
