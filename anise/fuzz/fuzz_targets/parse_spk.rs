#![no_main]
use anise::naif::SPK;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = SPK::parse(data);
});
