#![no_main]
use anise::math::rotation::Quaternion;
use anise::math::Vector3;

use libfuzzer_sys::fuzz_target;

use anise_fuzz::{ArbitraryQuaternion, ArbitraryVector3};

fuzz_target!(|data: (ArbitraryQuaternion, ArbitraryVector3)| {
    let quaternion = Quaternion::from(data.0);
    let vector = Vector3::from(data.1);
    let _ = quaternion * vector;
});
