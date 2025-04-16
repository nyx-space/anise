#![no_main]
use anise::math::rotation::DCM;

use libfuzzer_sys::fuzz_target;

use anise_fuzz::ArbitraryDCM;

fuzz_target!(|data: (ArbitraryDCM, ArbitraryDCM)| {
    let dcm_0 = DCM::from(data.0);
    let dcm_1 = DCM::from(data.1);
    let _ = dcm_0 == dcm_1;
});
