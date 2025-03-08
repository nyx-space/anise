#![no_main]
use anise::naif::kpl::parser::convert_fk_items;
use std::collections::HashMap;

use libfuzzer_sys::fuzz_target;

use anise_fuzz::ArbitraryFKItem;

fuzz_target!(|data: HashMap<i32, ArbitraryFKItem>| {
    let assignments = data
        .into_iter()
        .map(|(idx, item)| (idx, item.into()))
        .collect();
    let _ = convert_fk_items(assignments);
});
