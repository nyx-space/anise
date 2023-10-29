// Start by creating the ANISE planetary data
use anise::{
    astro::orbit::Orbit,
    constants::frames::{EARTH_ITRF93, EARTH_J2000},
    naif::kpl::parser::convert_tpc,
    prelude::{Aberration, Almanac, BPC, SPK},
};
use core::str::FromStr;
use hifitime::Epoch;

#[test]
fn test_load_ctx() {
    dbg!(core::mem::size_of::<Almanac>());

    let dataset = convert_tpc("data/pck00008.tpc", "data/gm_de431.tpc").unwrap();

    // Load BSP and BPC
    let ctx = Almanac::default();

    let spk = SPK::load("data/de440.bsp").unwrap();
    let bpc = BPC::load("data/earth_latest_high_prec.bpc").unwrap();

    let mut loaded_ctx = ctx.with_spk(spk).unwrap().with_bpc(bpc).unwrap();

    loaded_ctx.planetary_data = dataset;

    println!("{loaded_ctx}");

    dbg!(core::mem::size_of::<Almanac>());
}

#[test]
fn test_state_translation() {
    // Load BSP and BPC
    let ctx = Almanac::default();

    let spk = SPK::load("data/de440.bsp").unwrap();
    let bpc = BPC::load("data/earth_latest_high_prec.bpc").unwrap();
    let pck = convert_tpc("data/pck00008.tpc", "data/gm_de431.tpc").unwrap();

    let almanac = ctx
        .with_spk(spk)
        .unwrap()
        .with_bpc(bpc)
        .unwrap()
        .with_planetary_data(pck);

    // Let's build an orbit
    // Start by grabbing a copy of the frame.
    let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();
    // Define an epoch
    let epoch = Epoch::from_str("2021-10-29 12:34:56 TDB").unwrap();

    let orig_state = Orbit::keplerian(
        8_191.93, 1e-6, 12.85, 306.614, 314.19, 99.887_7, epoch, eme2k,
    );

    // Transform that into another frame.
    let transformed_state = almanac
        .transform_to(orig_state, EARTH_ITRF93, Aberration::None)
        .unwrap();

    // BUG: ITRF93 is NOT considered geodetic with my new change, ugh.

    // This will print the orbital elements
    println!("{orig_state:x}");
    // This will print the geodetic data (because frame is geodetic)
    println!("{transformed_state:x}");
}
