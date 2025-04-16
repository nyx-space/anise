#![no_main]
use anise::naif::kpl::{tpc::TPCItem, KPLItem};

use libfuzzer_sys::fuzz_target;

use anise_fuzz::ArbitraryAssignment;

fuzz_target!(|data: ArbitraryAssignment| {
    let assignment = data.into();
    let _ = TPCItem::extract_key(&assignment);
});
