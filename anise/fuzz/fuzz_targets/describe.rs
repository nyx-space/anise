#![no_main]

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary::{self, Arbitrary};
use anise::time::TimeScale;
use anise::almanac::Almanac;

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    spk: Option<bool>,
    bpc: Option<bool>,
    planetary: Option<bool>,
    eulerparams: Option<bool>,
    time_scale: Option<u8>,
    round_time: Option<bool>,
}

fuzz_target!(|input: FuzzInput| {
    let time_scale = match input.time_scale {
        Some(0) => Some(TimeScale::TDB),
        Some(1) => Some(TimeScale::TAI),
        Some(2) => Some(TimeScale::UTC),
        Some(3) => Some(TimeScale::GPST),
        Some(4) => Some(TimeScale::GST),
        Some(5) => Some(TimeScale::TT),
        Some(6) => Some(TimeScale::ET),
        _ => None,
    };

    // Initialize Almanac (consider preloading test data)
    let almanac = Almanac::default();
    
    // Execute the target function
    almanac.describe(
        input.spk,
        input.bpc,
        input.planetary,
        input.eulerparams,
        time_scale,
        input.round_time,
    );
});
