#![no_main]
use anise::naif::kpl::parser::convert_tpc_items;
use std::collections::HashMap;

use libfuzzer_sys::fuzz_target;

use anise_fuzz::ArbitraryTPCItem;

fuzz_target!(|data: (HashMap<i32, ArbitraryTPCItem>, HashMap<i32, ArbitraryTPCItem>)| {
    let (planetary_data, gravity_data) = data;
    let planetary_data = planetary_data
        .into_iter()
        .map(|(idx, item)| (idx, item.into()))
        .collect();
    let gravity_data = gravity_data
        .into_iter()
        .map(|(idx, item)| (idx, item.into()))
        .collect();
    let _ = convert_tpc_items(planetary_data, gravity_data);
});

