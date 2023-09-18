#[test]
fn test_load_ctx() {
    // Start by creating the ANISE planetary data
    use anise::{
        naif::kpl::parser::convert_tpc,
        prelude::{Almanac, BPC, SPK},
    };

    let dataset = convert_tpc("data/pck00008.tpc", "data/gm_de431.tpc").unwrap();

    // Load BSP and BPC
    let ctx = Almanac::default();

    let spk = SPK::load("data/de440.bsp").unwrap();
    let bpc = BPC::load("data/earth_latest_high_prec.bpc").unwrap();

    let mut loaded_ctx = ctx.load_spk(&spk).unwrap().load_bpc(&bpc).unwrap();

    loaded_ctx.planetary_data = dataset;

    println!("{loaded_ctx}");
}
