#![no_main]
use anise::structure::SpacecraftDataSet;
use bytes::Bytes;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = SpacecraftDataSet::try_from_bytes(Bytes::copy_from_slice(data));
});
