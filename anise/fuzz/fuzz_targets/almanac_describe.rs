#![no_main]
use anise::almanac::Almanac;
use bytes::BytesMut;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let almanac = Almanac::default();
    let mut bytes = BytesMut::new();
    bytes.reserve(data.len());
    bytes.extend(data.iter());
    if let Ok(almanac) = almanac.load_from_bytes(bytes) {
        almanac.describe(
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            Some(true),
            None,
            None,
        );
    }
});
