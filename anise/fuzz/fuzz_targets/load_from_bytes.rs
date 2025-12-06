#![no_main]
use anise::almanac::Almanac;
use bytes::BytesMut;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // create default almanac to serve as test env
    let almanac = Almanac::default();
    // convert fuzzed data into Bytes object, matching _load_from_bytes function
    let _ = almanac.load_from_bytes(BytesMut::from(data));
});
