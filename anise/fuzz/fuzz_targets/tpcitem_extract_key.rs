#![no_main]
use anise::naif::kpl::{KPLItem, tpc::TPCItem, parser::Assignment};

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary;

#[derive(arbitrary::Arbitrary, Debug)]
struct ArbitraryAssignment {
    pub keyword: String,
    pub value: String,
}

fuzz_target!(|data: ArbitraryAssignment| {
    let assignment = Assignment {
        keyword: data.keyword,
        value: data.value,
    };

    let _ = TPCItem::extract_key(&assignment);
});

