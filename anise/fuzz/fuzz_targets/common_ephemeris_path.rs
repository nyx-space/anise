#![no_main]
use anise::frames::Frame;
use anise::almanac::Almanac;
use bytes::Bytes;
use hifitime::Epoch;

use libfuzzer_sys::fuzz_target;

use anise_fuzz::{ArbitraryFrame, ArbitraryEpoch};

fuzz_target!(|data: (&[u8], ArbitraryFrame, ArbitraryFrame, ArbitraryEpoch)| {
    let from_frame = Frame::from(data.1);
    let to_frame = Frame::from(data.2);
    let epoch = Epoch::from(data.3);
    let almanac = Almanac::default();
    let data = Bytes::copy_from_slice(data.0);

    if let Ok(almanac) = almanac.load_from_bytes(data) {
        let _ = almanac.common_ephemeris_path(from_frame, to_frame, epoch);
    }
});
