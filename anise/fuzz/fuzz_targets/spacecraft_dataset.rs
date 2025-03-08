#![no_main]
use anise::structure::SpacecraftDataSet;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = SpacecraftDataSet::try_from_bytes(data);
});
