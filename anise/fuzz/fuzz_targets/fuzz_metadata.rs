#![no_main]
use anise::structure::metadata::Metadata;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = Metadata::decode_header(data);
});
