#![no_main]
use anise::naif::MutBPC;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = MutBPC::parse(data);
});
