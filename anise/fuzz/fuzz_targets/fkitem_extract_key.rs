#![no_main]
use anise::naif::kpl::{fk::FKItem, KPLItem};

use libfuzzer_sys::fuzz_target;

use anise_fuzz::ArbitraryAssignment;

fuzz_target!(|data: ArbitraryAssignment| {
    let assignment = data.into();
    let _ = FKItem::extract_key(&assignment);
});
