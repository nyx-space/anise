#![no_main]
use anise::almanac::Almanac;
use anise::frames::Frame;
use hifitime::Epoch;

use libfuzzer_sys::fuzz_target;

use anise_fuzz::{ArbitraryEpoch, ArbitraryFrame};

fuzz_target!(|data: (ArbitraryFrame, ArbitraryFrame, ArbitraryEpoch)| {
    let from_frame = Frame::from(data.0);
    let to_frame = Frame::from(data.1);
    let epoch = Epoch::from(data.2);

    let almanac = Almanac::default();

    let _ = almanac.rotate(from_frame, to_frame, epoch);
});
