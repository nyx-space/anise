#![no_main]
use anise::structure::EulerParameterDataSet;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = EulerParameterDataSet::try_from_bytes(data);
});
