#![no_main]
use anise::naif::kpl::{KPLItem, tpc::TPCItem};

use libfuzzer_sys::fuzz_target;

use anise_fuzz::ArbitraryAssignment;

fuzz_target!(|data: ArbitraryAssignment| {
    let assignment = data.into();
    let _ = TPCItem::extract_key(&assignment);
});

