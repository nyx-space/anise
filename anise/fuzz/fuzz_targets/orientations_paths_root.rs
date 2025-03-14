#![no_main]
use anise::almanac::Almanac;
use anise::naif::BPC;

use libfuzzer_sys::fuzz_target;

use anise_fuzz::ArbitraryBPC;

fuzz_target!(|data: ArbitraryBPC| {
    let bpc: BPC = data.into();
    let almanac = Almanac::from_bpc(bpc).unwrap();
    let _ = almanac.try_find_orientation_root();
});
