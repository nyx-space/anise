use anise::constants::celestial_objects::EARTH;
use anise::constants::frames::{EARTH_ITRF93, EME2000};
use anise::constants::orientations::{ITRF93, J2000};
use anise::math::rotation::DCM;
use anise::math::Matrix3;
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
    use core::str::FromStr;
    let bpc = BPC::load("data/earth_latest_high_prec.bpc").unwrap();
    let almanac = Almanac::from_bpc(bpc).unwrap();

    let epoch = Epoch::from_str("2019-03-01T04:02:51.0 ET").unwrap();

    let dcm = almanac
        .rotation_to_parent(Frame::from_ephem_orient(EARTH, ITRF93), epoch)
        .unwrap();

    println!("{dcm}\n{}", dcm.rot_mat_dt.unwrap());
}

#[test]
fn test_bpc_to_j2k() {
    use core::str::FromStr;
    let bpc = BPC::load("data/earth_latest_high_prec.bpc").unwrap();
    let almanac = Almanac::from_bpc(bpc).unwrap();

    let epoch = Epoch::from_str("2019-03-01T04:02:51.0 ET").unwrap();

    let dcm = almanac
        .rotate_from_to(EARTH_ITRF93, EME2000, epoch)
        .unwrap();

    assert_eq!(dcm.from, ITRF93);
    assert_eq!(dcm.to, J2000);

    // Ensure transposed works too.
    let dcm_t = almanac
        .rotate_from_to(EME2000, EARTH_ITRF93, epoch)
        .unwrap();

    assert_eq!(dcm_t.from, J2000);
    assert_eq!(dcm_t.to, ITRF93);
    assert_eq!(
        dcm,
        dcm_t.transpose(),
        "dcm = {dcm} dcm_t = {dcm_t} whose transpose is {}",
        dcm_t.transpose()
    );

    /*
            ANISE

    Derivative:
      ┌                                                                                           ┐
      │        0.00004574960310176632       -0.00005213242562060531       0.000022509515550217996 │
      │       0.000056784242735185715        0.00004193347476832602        -0.0000182928331920711 │
      │  0.00000000008998156328991614  0.00000000007924039408010754 -0.00000000003453279422058282 │
      └

      SPICE: Rotation ITRF93 -> J2000 (transport theorem = true)
      ┌                                                                            ┐
      │      -0.7787074378266214       0.6273845404742724    0.0018342975179237739 │
      │      -0.6273856264104672      -0.7787087230243394 -0.000021432407757815408 │
      │    0.0014149371165367297   -0.0011675014726372779       0.9999983174452183 │
      └                                                                            ┘

    Derivative:
      ┌                                                                                                    ┐
      │          0.000045749603091397784           0.00005678424274353827     0.00000000008998156330541006 │
      │         -0.000056784336444384685           0.00004574968205088016     0.00000000008643799681544929 │
      │        -0.0000000850112519852614       -0.00000010316798647710046 -0.00000000000016320065843054112 │
      └                                                                                                    ┘                                                                                     ┘
           */

    // From the validation test case
    /*
      SPICE: Rotation ITRF93 -> J2000 (transport theorem = true)
    ┌                                                                            ┐
    │      -0.7787074378266214       0.6273845404742724    0.0018342975179237739 │
    │      -0.6273856264104672      -0.7787087230243394 -0.000021432407757815408 │
    │    0.0014149371165367297   -0.0011675014726372779       0.9999983174452183 │
    └                                                                            ┘


    ┌                                                                                                    ┐
    │          0.000045749603091397784           0.00005678424274353827     0.00000000008998156330541006 │
    │         -0.000056784336444384685           0.00004574968205088016     0.00000000008643799681544929 │
    │        -0.0000000850112519852614       -0.00000010316798647710046 -0.00000000000016320065843054112 │
    └                                                                                                    ┘
       */
    let spice_dcm = DCM {
        from: ITRF93,
        to: J2000,
        rot_mat: Matrix3::new(
            -0.7787074378266214,
            0.6273845404742724,
            0.0018342975179237739,
            -0.6273856264104672,
            -0.7787087230243394,
            -0.000021432407757815408,
            0.0014149371165367297,
            -0.0011675014726372779,
            0.9999983174452183,
        ),
        rot_mat_dt: Some(Matrix3::new(
            0.000045749603091397784,
            0.00005678424274353827,
            0.00000000008998156330541006,
            -0.000056784336444384685,
            0.00004574968205088016,
            0.00000000008643799681544929,
            -0.0000000850112519852614,
            -0.00000010316798647710046,
            -0.00000000000016320065843054112,
        )),
    };

    println!("ANISE: {dcm}");

    println!("SPICE: {spice_dcm}");

    assert!(
        (dcm.rot_mat - spice_dcm.rot_mat).norm() < 1e-9,
        "dcm error! got: {}want:{}err = {:.3e}: {:.3e}",
        dcm.rot_mat,
        spice_dcm.rot_mat,
        (dcm.rot_mat - spice_dcm.rot_mat).norm(),
        dcm.rot_mat - spice_dcm.rot_mat
    );

    // Check the derivative
    assert!(
        (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm() < 1e-13,
        "derivative error! got: {}want:{}err = {:.3e}: {:.3e}",
        dcm.rot_mat_dt.unwrap(),
        spice_dcm.rot_mat_dt.unwrap(),
        (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm(),
        dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
    );
}
