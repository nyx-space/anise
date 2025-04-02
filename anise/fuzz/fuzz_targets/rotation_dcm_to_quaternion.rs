#![no_main]
use anise::math::rotation::{Quaternion, DCM};

use libfuzzer_sys::fuzz_target;

use anise_fuzz::ArbitraryDCM;

fuzz_target!(|data: ArbitraryDCM| {
    let dcm = DCM::from(data);
    let _ = Quaternion::from(dcm);
});
