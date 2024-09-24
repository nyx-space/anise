// Start by creating the ANISE planetary data
use anise::{
    constants::frames::{EARTH_ITRF93, EARTH_J2000, SUN_J2000},
    naif::kpl::parser::convert_tpc,
    prelude::{Aberration, Almanac, Orbit, BPC, SPK},
};
use core::str::FromStr;
use hifitime::Epoch;

#[test]
fn test_load_ctx() {
    dbg!(core::mem::size_of::<Almanac>());

    let dataset = convert_tpc("../data/pck00008.tpc", "../data/gm_de431.tpc").unwrap();

    // Load BSP and BPC
    let ctx = Almanac::new("../data/de440.bsp").unwrap();

    let mut loaded_ctx = ctx.load("../data/earth_latest_high_prec.bpc").unwrap();

    loaded_ctx.planetary_data = dataset;

    println!("{loaded_ctx}");

    dbg!(core::mem::size_of::<Almanac>());
}

#[test]
fn test_state_transformation() {
    // Load BSP and BPC
    let ctx = Almanac::default();

    let spk = SPK::load("../data/de440.bsp").unwrap();
    let bpc = BPC::load("../data/earth_latest_high_prec.bpc").unwrap();

    let almanac = ctx
        .with_spk(spk)
        .unwrap()
        .with_bpc(bpc)
        .unwrap()
        .load("../data/pck08.pca")
        .unwrap();

    // Let's build an orbit
    // Start by grabbing a copy of the frame.
    let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();
    // Define an epoch
    let epoch = Epoch::from_str("2021-10-29 12:34:56 TDB").unwrap();

    let orig_state = Orbit::keplerian(
        8_191.93, 1e-6, 12.85, 306.614, 314.19, 99.887_7, epoch, eme2k,
    );

    // Transform that into another frame.
    let state_itrf93 = almanac
        .transform_to(orig_state, EARTH_ITRF93, Aberration::NONE)
        .unwrap();

    // Check that the frame is correctly set.
    assert_eq!(state_itrf93.frame.ephemeris_id, EARTH_ITRF93.ephemeris_id);
    assert_eq!(
        state_itrf93.frame.orientation_id,
        EARTH_ITRF93.orientation_id
    );

    println!("{orig_state:x}");
    println!("{state_itrf93:X}");

    // Convert back.
    // Note that the Aberration correction constants are actually options!
    let from_state_itrf93_to_eme2k = almanac
        .transform_to(state_itrf93, EARTH_J2000, None)
        .unwrap();

    println!("{from_state_itrf93_to_eme2k}");
    println!("{from_state_itrf93_to_eme2k:x}");

    assert_eq!(orig_state, from_state_itrf93_to_eme2k);
}

#[test]
fn test_type3_state_transformation() {
    // Load BSP and BPC
    let ctx = Almanac::default();

    let spk = SPK::load("../data/de440_type3.bsp").unwrap();

    let almanac = ctx
        .with_spk(spk)
        .unwrap()
        .load("../data/pck08.pca")
        .unwrap();

    let epoch = Epoch::from_str("2021-10-29 12:34:56 TDB").unwrap();

    let to_parent = almanac.translate_to_parent(EARTH_J2000, epoch).unwrap();

    println!("{to_parent}");

    // Ensure that we can query the type 3 chebyshev DE440 file

    let state = almanac
        .translate(EARTH_J2000, SUN_J2000, epoch, None)
        .expect("type 3 chebyshev could not be queried");

    println!("{state:x}");
}
