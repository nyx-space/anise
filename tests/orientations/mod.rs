use anise::constants::celestial_objects::EARTH;
use anise::constants::orientations::{ITRF93, J2000};
use anise::naif::kpl::parser::convert_tpc;

use anise::prelude::*;

mod validation;

#[test]
fn test_find_root() {
    // try_find_orientation_root
    let almanac = Almanac {
        planetary_data: convert_tpc("data/pck00008.tpc", "data/gm_de431.tpc").unwrap(),
        ..Default::default()
    };

    assert_eq!(almanac.try_find_orientation_root(), Ok(J2000));
}

#[test]
fn test_single_bpc() {
    // TOOO: Add benchmark
    use core::str::FromStr;
    let bpc = BPC::load("data/earth_latest_high_prec.bpc").unwrap();
    let almanac = Almanac::from_bpc(bpc).unwrap();

    let epoch = Epoch::from_str("2019-03-01T04:02:51.0 ET").unwrap();

    let dcm = almanac
        .rotation_to_parent(Frame::from_ephem_orient(EARTH, ITRF93), epoch)
        .unwrap();

    println!("{dcm}\n{}", dcm.rot_mat_dt.unwrap());
}
