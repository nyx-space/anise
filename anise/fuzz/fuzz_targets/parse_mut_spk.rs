#![no_main]
use anise::naif::MutSPK;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = MutSPK::parse(data);
});
