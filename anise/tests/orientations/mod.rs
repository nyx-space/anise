use std::path::PathBuf;

use anise::constants::frames::{
    EARTH_ITRF93, EME2000, IAU_JUPITER_FRAME, IAU_MOON_FRAME, JUPITER_BARYCENTER_J2000, MOON_J2000,
};
use anise::constants::orientations::{ECLIPJ2000, IAU_JUPITER, IAU_MOON, ITRF93, J2000};
use anise::math::rotation::DCM;
use anise::math::Matrix3;
use anise::naif::kpl::parser::convert_tpc;

use anise::prelude::*;

mod validation;

#[test]
fn test_find_root_from_pca() {
    let planetary_data = convert_tpc("../data/pck00011.tpc", "../data/gm_de431.tpc").unwrap();
    // Serialize to disk
    planetary_data
        .save_as(&PathBuf::from_str("../data/pck11.pca").unwrap(), true)
        .unwrap();

    println!("PCK11 CRC32: {}", planetary_data.crc32());

    let almanac = Almanac::default().load("../data/pck11.pca").unwrap();

    assert_eq!(almanac.try_find_orientation_root(), Ok(J2000));

    let planetary_data = convert_tpc("../data/pck00008.tpc", "../data/gm_de431.tpc").unwrap();
    // Serialize to disk
    planetary_data
        .save_as(&PathBuf::from_str("../data/pck08.pca").unwrap(), true)
        .unwrap();
    println!("PCK08 CRC32: {}", planetary_data.crc32());
    assert!(Almanac::default().load("../data/pck08.pca").is_ok());
}

#[test]
fn test_single_bpc() {
    use core::str::FromStr;
    let bpc = BPC::load("../data/earth_latest_high_prec.bpc").unwrap();
    let almanac = Almanac::from_bpc(bpc).unwrap();

    // Test the BPC domain since a BPC is loaded here.
    let (start, end) = almanac.bpc_domain(3000).unwrap();
    assert!(
        (start - Epoch::from_gregorian_utc_at_midnight(2000, 1, 1)).abs() < 1.0_f64.microseconds(),
        "wrong start epoch"
    );
    assert!(
        (end - Epoch::from_gregorian_utc_at_midnight(2023, 1, 11)).abs() < 50.0_f64.microseconds(),
        "wrong end epoch"
    );

    let epoch = Epoch::from_str("2019-03-01T04:02:51.0 ET").unwrap();

    let dcm = almanac.rotation_to_parent(EARTH_ITRF93, epoch).unwrap();

    assert_eq!(dcm.from, ECLIPJ2000);
    assert_eq!(dcm.to, ITRF93);

    let spice_dcm = DCM {
        from: ITRF93,
        to: ECLIPJ2000,
        rot_mat: Matrix3::new(
            -0.7787074378266214,
            -0.5750522285696024,
            0.25085784956949614,
            0.627_384_540_474_272_4,
            -0.7149156903669622,
            0.30868137948540997,
            0.0018342975179237739,
            0.3977568228003932,
            0.9174890436775538,
        ),
        rot_mat_dt: Some(Matrix3::new(
            0.000045749603091397784,
            -0.00005213242562826116,
            0.00002250951555355774,
            0.00005678424274353827,
            0.00004193347475880688,
            -0.000018292833187960967,
            0.00000000008998156330541006,
            0.00000000007924039406561106,
            -0.000000000034532794214329133,
        )),
    };

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
        "derivative error! got: {}want:{}derivative err = {:.3e}: {:.3e}",
        dcm.rot_mat_dt.unwrap(),
        spice_dcm.rot_mat_dt.unwrap(),
        (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm(),
        dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
    );
}

#[test]
fn test_itrf93_to_j2k() {
    use core::str::FromStr;
    let bpc = BPC::load("../data/earth_latest_high_prec.bpc").unwrap();
    let almanac = Almanac::from_bpc(bpc).unwrap();

    let epoch = Epoch::from_str("2019-03-01T04:02:51.0 ET").unwrap();

    let dcm = almanac.rotate(EARTH_ITRF93, EME2000, epoch).unwrap();

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

    assert_eq!(dcm.from, ITRF93);
    assert_eq!(dcm.to, J2000);

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
        "derivative error! got: {}want:{}derivative err = {:.3e}: {:.3e}",
        dcm.rot_mat_dt.unwrap(),
        spice_dcm.rot_mat_dt.unwrap(),
        (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm(),
        dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
    );
}

#[test]
fn test_j2k_to_itrf93() {
    use core::str::FromStr;
    let bpc = BPC::load("../data/earth_latest_high_prec.bpc").unwrap();
    let almanac = Almanac::from_bpc(bpc).unwrap();

    let epoch = Epoch::from_str("2019-03-01T04:02:51.0 ET").unwrap();

    let dcm = almanac.rotate(EME2000, EARTH_ITRF93, epoch).unwrap();

    let spice_dcm_t = DCM {
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

    let spice_dcm = spice_dcm_t.transpose();

    assert_eq!(dcm.to, ITRF93);
    assert_eq!(dcm.from, J2000);

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
        "derivative error! got: {}want:{}derivative err = {:.3e}: {:.3e}",
        dcm.rot_mat_dt.unwrap(),
        spice_dcm.rot_mat_dt.unwrap(),
        (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm(),
        dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
    );
}

/// The IAU_MOON frame rotation differs from SPICE by a norm of 4.1e-6.
/// Although this looks like a lot, the computation is identical to that of the other IAU frames.
/// For the IAU frames, the data in the PCK files comes from the IAU Report publications.
/// SPICE uses floating point values for time and a rounding error shows up when computing the angular rate per centuries.
/// ANISE uses Hifitime, which does not exhibit this rounding error.
/// As such, after discussion with Greg Henry's coworkers, I've decided that SPICE is in error here, and not ANISE.
#[test]
fn regression_test_issue_112_test_iau_moon() {
    use core::str::FromStr;

    let almanac = Almanac::new("../data/pck11.pca").unwrap();

    let epoch = Epoch::from_str("2030-01-01 00:00:00").unwrap();

    let dcm = almanac.rotate(MOON_J2000, IAU_MOON_FRAME, epoch).unwrap();

    let spice_dcm = DCM {
        from: J2000,
        to: IAU_MOON,
        rot_mat: Matrix3::new(
            0.5256992783481783,
            0.7849985804883132,
            0.327746086743286,
            -0.8502641586573962,
            0.4729767092674251,
            0.2309629688785363,
            0.0262893371319015,
            -0.4000878167626341,
            0.9160996723235273,
        ),
        rot_mat_dt: None,
    };

    assert_eq!(dcm.to, IAU_MOON);
    assert_eq!(dcm.from, J2000);

    assert!(
        (dcm.rot_mat - spice_dcm.rot_mat).norm() < 1e-5,
        "dcm error! got: {}want:{}err = {:.3e}: {:.3e}",
        dcm.rot_mat,
        spice_dcm.rot_mat,
        (dcm.rot_mat - spice_dcm.rot_mat).norm(),
        dcm.rot_mat - spice_dcm.rot_mat
    );
}

#[test]
fn regression_test_issue_112_test_iau_jupiter() {
    use core::str::FromStr;

    let almanac = Almanac::new("../data/pck11.pca").unwrap();

    let epoch = Epoch::from_str("2030-01-01 00:00:00").unwrap();

    let dcm = almanac
        .rotate(JUPITER_BARYCENTER_J2000, IAU_JUPITER_FRAME, epoch)
        .unwrap();

    let spice_dcm = DCM {
        from: J2000,
        to: IAU_JUPITER,
        rot_mat: Matrix3::new(
            -0.1371949263739366,
            -0.893256221707872,
            -0.4281014769390864,
            0.9904365275051731,
            -0.130075026655604,
            -0.0459997002603103,
            -0.0145957925699357,
            -0.4303182657298211,
            0.9025592241058392,
        ),
        rot_mat_dt: None,
    };

    assert_eq!(dcm.to, IAU_JUPITER);
    assert_eq!(dcm.from, J2000);

    assert!(
        (dcm.rot_mat - spice_dcm.rot_mat).norm() < 1e-9,
        "dcm error! got: {}want:{}err = {:.3e}: {:.3e}",
        dcm.rot_mat,
        spice_dcm.rot_mat,
        (dcm.rot_mat - spice_dcm.rot_mat).norm(),
        dcm.rot_mat - spice_dcm.rot_mat
    );
}
