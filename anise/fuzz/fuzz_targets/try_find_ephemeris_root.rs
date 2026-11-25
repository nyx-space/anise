#![no_main]
use anise::almanac::Almanac;
use bytes::Bytes;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let almanac = Almanac::default();
    let data = Bytes::copy_from_slice(data);
    if let Ok(almanac) = almanac.load_from_bytes(data) {
        let _ = almanac.try_find_ephemeris_root();
    }
});
