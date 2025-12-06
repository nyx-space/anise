#![no_main]
use anise::almanac::Almanac;
use anise::frames::Frame;
use bytes::BytesMut;
use hifitime::Epoch;

use libfuzzer_sys::fuzz_target;

use anise_fuzz::{ArbitraryEpoch, ArbitraryFrame};

fuzz_target!(
    |data: (&[u8], ArbitraryFrame, ArbitraryFrame, ArbitraryEpoch)| {
        let from_frame = Frame::from(data.1);
        let to_frame = Frame::from(data.2);
        let epoch = Epoch::from(data.3);
        let almanac = Almanac::default();

        let mut bytes = BytesMut::new();
        bytes.reserve(data.0.len());
        bytes.extend(data.0.iter());
        if let Ok(almanac) = almanac.load_from_bytes(bytes) {
            let _ = almanac.common_ephemeris_path(from_frame, to_frame, epoch);
        }
    }
);
