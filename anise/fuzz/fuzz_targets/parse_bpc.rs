#![no_main]
use anise::naif::BPC;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = BPC::parse(data);
});
