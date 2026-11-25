#![no_main]
use anise::naif::BPC;

use bytes::Bytes;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = BPC::parse(Bytes::copy_from_slice(data));
});
