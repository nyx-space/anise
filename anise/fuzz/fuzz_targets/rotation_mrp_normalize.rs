#![no_main]
use anise::math::rotation::MRP;

use libfuzzer_sys::fuzz_target;

use anise_fuzz::ArbitraryMRP;

fuzz_target!(|data: ArbitraryMRP| {
    let mrp = MRP::from(data);
    let _ = mrp.normalize();
});
