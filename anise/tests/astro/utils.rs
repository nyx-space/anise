use anise::astro::utils::modified_equinoctial_to_mean_motion;
use anise::constants::frames::EARTH_J2000;
use anise::f64_eq_tol;
use anise::prelude::*;
use anise::time::Epoch;

use rstest::{fixture, rstest};

#[fixture]
fn almanac() -> Almanac {
    Almanac::new("../data/pck08.pca")
        .unwrap()
        .load("../data/de440s.bsp")
        .unwrap()
}

/// Confirms that `modified_equinoctial_to_mean_motion` agrees with the standard
/// Keplerian mean motion for a typical elliptical orbit.
#[rstest]
fn verif_modified_equinoctial_to_mean_motion(almanac: Almanac) {
    let eme2000 = almanac.frame_info(EARTH_J2000).unwrap();
    let epoch = Epoch::from_gregorian_utc_hms(2026, 8, 26, 12, 0, 0);

    let orbit = Orbit::try_keplerian(
        8_000.0,  // sma_km
        0.1,      // ecc
        45.0,     // inc_deg
        60.0,     // raan_deg
        30.0,     // aop_deg
        120.0,    // ta_deg
        epoch,
        eme2000,
    ).unwrap();

    let expected_mean_motion_rad_s = orbit.mean_motion_deg_s().unwrap().to_radians();

    let eoe = orbit.modified_equinoctial_elements().unwrap();
    let actual_mean_motion_rad_s = modified_equinoctial_to_mean_motion(
        eme2000.mu_km3_s2().unwrap(),
        eoe[0], // p_km
        eoe[1], // f
        eoe[2], // g
    )
    .unwrap();

    f64_eq_tol!(
        actual_mean_motion_rad_s,
        expected_mean_motion_rad_s,
        1e-12,
        "Mean motion from modified equinoctial elements did not match Keplerian mean motion"
    );
}
