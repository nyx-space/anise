use anise::constants::orientations::J2000;
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
