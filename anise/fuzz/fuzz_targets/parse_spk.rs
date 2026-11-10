#![no_main]
use anise::naif::SPK;

use bytes::Bytes;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = SPK::parse(Bytes::copy_from_slice(data));
});
