use std::path::PathBuf;

use anise::constants::frames::{
    EARTH_ITRF93, EARTH_J2000, EME2000, GCRF, IAU_JUPITER_FRAME, IAU_MOON_FRAME,
    JUPITER_BARYCENTER_J2000, MOON_J2000, MOON_ME_DE440_ME421_FRAME, MOON_PA_DE421_FRAME,
    MOON_PA_DE440_FRAME,
};
use anise::constants::orientations::{
    ECLIPJ2000, IAU_JUPITER, IAU_MOON, ITRF93, J2000, MOON_PA_DE440,
};
use anise::constants::usual_planetary_constants::MEAN_EARTH_ANGULAR_VELOCITY_DEG_S;
use anise::math::rotation::{EulerParameter, DCM};
use anise::math::{Matrix3, Vector3};
use anise::naif::kpl::parser::convert_tpc;

use anise::f64_eq_tol;
use anise::structure::PlanetaryDataSet;
use anise::{file2heap, prelude::*};

mod validation;

#[test]
fn fetch_crc32() {
    let data_set = PlanetaryDataSet::from_bytes(file2heap!("../data/pck11.pca").unwrap());
    println!("{:x}", data_set.crc32());
}

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
fn test_single_bpc_dcm() {
    use core::str::FromStr;
    let bpc = BPC::load("../data/earth_latest_high_prec.bpc").unwrap();
    let almanac = Almanac::from_bpc(bpc);

    // Test the BPC domain since a BPC is loaded here.
    let (start, end) = almanac.bpc_domain(3000).unwrap();
    assert!(
        (start - Epoch::from_gregorian_utc_at_midnight(2000, 1, 1)).abs() < 1.0_f64.microseconds(),
        "wrong start epoch"
    );
    assert!(
        (end - Epoch::from_gregorian_utc_at_midnight(2023, 1, 11)).abs() < 50.0_f64.microseconds(),
        "wrong end epoch: {end:?}"
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
        (dcm.rot_mat - spice_dcm.rot_mat).norm() < 2.9e-9,
        "dcm error! got: {}want:{}err = {:.3e}: {:.3e}",
        dcm.rot_mat,
        spice_dcm.rot_mat,
        (dcm.rot_mat - spice_dcm.rot_mat).norm(),
        dcm.rot_mat - spice_dcm.rot_mat
    );

    // Check the derivative
    assert!(
        (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm() < 2.1e-13,
        "derivative error! got: {}want:{}derivative err = {:.3e}: {:.3e}",
        dcm.rot_mat_dt.unwrap(),
        spice_dcm.rot_mat_dt.unwrap(),
        (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm(),
        dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
    );

    // Check the DCM to EP reciprocity
    let orig_q: EulerParameter = dcm.into();
    let (orig_uvec, orig_angle_rad) = orig_q.uvec_angle_rad();
    let rtn_dcm: DCM = orig_q.into();
    assert!((rtn_dcm.rot_mat - dcm.rot_mat).norm() < 2e-16);
    let rtn_q: EulerParameter = rtn_dcm.into();
    let (rtn_uvec, rtn_angle_rad) = rtn_q.uvec_angle_rad();

    assert!((rtn_uvec - orig_uvec).norm() < f64::EPSILON);
    assert!((rtn_angle_rad - orig_angle_rad).abs() < f64::EPSILON);
}

#[test]
fn test_itrf93_to_j2k() {
    use core::str::FromStr;
    let bpc = BPC::load("../data/earth_latest_high_prec.bpc").unwrap();
    let almanac = Almanac::from_bpc(bpc);

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
        (dcm.rot_mat - spice_dcm.rot_mat).norm() < 2.9e-9,
        "dcm error! got: {}want:{}err = {:.3e}: {:.3e}",
        dcm.rot_mat,
        spice_dcm.rot_mat,
        (dcm.rot_mat - spice_dcm.rot_mat).norm(),
        dcm.rot_mat - spice_dcm.rot_mat
    );

    // Check the derivative
    assert!(
        (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm() < 2.1e-13,
        "derivative error! got: {}want:{}derivative err = {:.3e}: {:.3e}",
        dcm.rot_mat_dt.unwrap(),
        spice_dcm.rot_mat_dt.unwrap(),
        (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm(),
        dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
    );

    // Check that the angular rate of the ITRF93 frame wrt to the J2000 frame is very close to the mean value.
    let omega_deg_s = almanac
        .angular_velocity_wrt_j2000_deg_s(EARTH_ITRF93, epoch)
        .unwrap();

    assert!(
        dbg!(omega_deg_s.norm()) - dbg!(MEAN_EARTH_ANGULAR_VELOCITY_DEG_S).abs() < 1e-8,
        "incorrect mean Earth angular velocity"
    );
}

#[test]
fn test_j2k_to_itrf93() {
    use core::str::FromStr;
    let bpc = BPC::load("../data/earth_latest_high_prec.bpc").unwrap();
    let almanac = Almanac::from_bpc(bpc);

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
        (dcm.rot_mat - spice_dcm.rot_mat).norm() < 2.9e-9,
        "dcm error! got: {}want:{}err = {:.3e}: {:.3e}",
        dcm.rot_mat,
        spice_dcm.rot_mat,
        (dcm.rot_mat - spice_dcm.rot_mat).norm(),
        dcm.rot_mat - spice_dcm.rot_mat
    );

    // Check the derivative
    assert!(
        (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm() < 2.1e-13,
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

#[test]
fn regression_test_issue_357_test_moon_me_j2k() {
    use core::str::FromStr;

    let almanac = Almanac::new("../data/pck11.pca")
        .unwrap()
        .load("../data/moon_fk_de440.epa")
        .unwrap()
        .load("../data/moon_pa_de440_200625.bpc")
        .unwrap()
        .load("../data/de440s.bsp")
        .unwrap();

    let epoch = Epoch::from_str("2024-01-01 22:28:39").unwrap();

    let dcm = almanac
        .rotate(MOON_PA_DE440_FRAME, MOON_J2000, epoch)
        .unwrap();

    /*
        In [10]: sp.sxform("MOON_PA_DE440", "J2000", my_et)
    Out[10]:
    array([[ 9.78289320e-01,  2.07027066e-01, -9.47625902e-03,
             0.00000000e+00,  0.00000000e+00,  0.00000000e+00],
           [-1.95463789e-01,  9.06520407e-01, -3.74185328e-01,
             0.00000000e+00,  0.00000000e+00,  0.00000000e+00],
           [-6.88760685e-02,  3.67913775e-01,  9.27305527e-01,
             0.00000000e+00,  0.00000000e+00,  0.00000000e+00],
           [ 5.51091888e-07, -2.60415126e-06, -2.62517851e-10,
             9.78289320e-01,  2.07027066e-01, -9.47625902e-03],
           [ 2.41301211e-06,  5.20281183e-07, -2.93451776e-11,
            -1.95463789e-01,  9.06520407e-01, -3.74185328e-01],
           [ 9.79597415e-07,  1.83424192e-07, -1.45240394e-11,
            -6.88760685e-02,  3.67913775e-01,  9.27305527e-01]])

         */

    let spice_dcm = DCM {
        from: MOON_PA_DE440,
        to: J2000,
        rot_mat: Matrix3::new(
            9.78289320e-01,
            2.07027066e-01,
            -9.47625902e-03,
            -1.95463789e-01,
            9.06520407e-01,
            -3.74185328e-01,
            -6.88760685e-02,
            3.67913775e-01,
            9.27305527e-01,
        ),
        rot_mat_dt: Some(Matrix3::new(
            5.51091888e-07,
            -2.60415126e-06,
            -2.62517851e-10,
            2.41301211e-06,
            5.20281183e-07,
            -2.93451776e-11,
            9.79597415e-07,
            1.83424192e-07,
            -1.45240394e-11,
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

    /*
     Frame Name                  Relative to          Type   Frame ID
    -------------------------   -----------------    -----  --------
    MOON_PA                     MOON_PA_DE440        FIXED  31010
    MOON_ME                     MOON_ME_DE440_ME421  FIXED  31011
    MOON_PA_DE440               ICRF/J2000           PCK    31008
    MOON_ME_DE440_ME421         MOON_PA_DE440        FIXED  31009
      */

    // Repeat for Moon ME

    // Check the path first
    let moon_pa_path = almanac
        .orientation_path_to_root(MOON_PA_DE440_FRAME, epoch)
        .unwrap();

    assert_eq!(moon_pa_path.0, 1, "Moon PA is defined wrt J2000");
    assert_eq!(
        moon_pa_path.1[0].unwrap(),
        1,
        "Moon PA is defined wrt J2000"
    );

    let moon_me_path = almanac
        .orientation_path_to_root(MOON_ME_DE440_ME421_FRAME, epoch)
        .unwrap();
    assert_eq!(
        moon_me_path.0, 2,
        "Moon ME is defined wrt Moon PA: {:?}",
        moon_me_path.1
    );
    assert_eq!(
        moon_me_path.1[0].unwrap(),
        31008,
        "Moon ME is defined wrt Moon PA"
    );
    assert_eq!(
        moon_me_path.1[1].unwrap(),
        1,
        "Moon PA is defined wrt J2000"
    );

    let dcm = almanac
        .rotate(MOON_PA_DE440_FRAME, MOON_ME_DE440_ME421_FRAME, epoch)
        .unwrap();

    /*
            In [9]: sp.sxform("MOON_PA_DE440", "MOON_ME", my_et)
    Out[9]:
    array([[ 9.99999873e-01, -3.28958658e-04,  3.81521208e-04,
             0.00000000e+00,  0.00000000e+00,  0.00000000e+00],
           [ 3.28959197e-04,  9.99999946e-01, -1.35020600e-06,
             0.00000000e+00,  0.00000000e+00,  0.00000000e+00],
           [-3.81520743e-04,  1.47571074e-06,  9.99999927e-01,
             0.00000000e+00,  0.00000000e+00,  0.00000000e+00],
           [ 0.00000000e+00,  0.00000000e+00,  0.00000000e+00,
             9.99999873e-01, -3.28958658e-04,  3.81521208e-04],
           [ 0.00000000e+00,  0.00000000e+00,  0.00000000e+00,
             3.28959197e-04,  9.99999946e-01, -1.35020600e-06],
           [ 0.00000000e+00,  0.00000000e+00,  0.00000000e+00,
            -3.81520743e-04,  1.47571074e-06,  9.99999927e-01]])
             */

    let spice_dcm = DCM {
        from: MOON_PA_DE440,
        to: J2000,
        rot_mat: Matrix3::new(
            9.99999873e-01,
            -3.28958658e-04,
            3.81521208e-04,
            3.28959197e-04,
            9.99999946e-01,
            -1.35020600e-06,
            -3.81520743e-04,
            1.47571074e-06,
            9.99999927e-01,
        ),
        rot_mat_dt: None,
    };

    assert!(
        (dcm.rot_mat - spice_dcm.rot_mat).norm() < 1e-9,
        "dcm error! got: {}want:{}err = {:.3e}: {:.3e}",
        dcm.rot_mat,
        spice_dcm.rot_mat,
        (dcm.rot_mat - spice_dcm.rot_mat).norm(),
        dcm.rot_mat - spice_dcm.rot_mat
    );

    // Verification of functionality.
    // Build an orbit in the Earth J2000 frame and transform it into the Moon ME frame to get its latitude and longitude.
    let epoch = Epoch::from_str("2024-09-22T08:45:22 UTC").unwrap();
    // This state is identical in ANISE and SPICE, queried from a BSP.
    let orbit_moon_j2k = Orbit::new(
        638.053603,
        -1776.813629,
        195.147575,
        -0.017910,
        -0.181449,
        -1.584180,
        epoch,
        MOON_J2000,
    );
    // Transform to Earth J2000.
    let orbit_earth_j2k = almanac
        .transform_to(orbit_moon_j2k, EARTH_J2000, None)
        .unwrap();
    // Compute the LLA in the Moon ME frame, used for cartography.
    let orbit_moon_me = almanac
        .transform_to(orbit_earth_j2k, MOON_ME_DE440_ME421_FRAME, None)
        .unwrap();
    let (lat, long, alt) = orbit_moon_me.latlongalt().unwrap();
    dbg!(lat, long, alt);
}

#[test]
fn regression_test_issue_431_test() {
    use core::str::FromStr;

    let almanac = Almanac::new("../data/pck11.pca")
        .unwrap()
        .load("../data/moon_fk_de440.epa")
        .unwrap()
        .load("../data/moon_pa_de440_200625.bpc")
        .unwrap()
        .load("../data/de440s.bsp")
        .unwrap();

    let epoch = Epoch::from_str("2022-06-29 00:00:00 TDB").unwrap();

    let expected = almanac
        .translate(EARTH_J2000, MOON_PA_DE421_FRAME, epoch, None)
        .unwrap();

    let computed = almanac
        .translate_state_to(
            Vector3::zeros(),
            Vector3::zeros(),
            EARTH_J2000,
            MOON_PA_DE421_FRAME,
            epoch,
            None,
            LengthUnit::Kilometer,
            TimeUnit::Second,
        )
        .unwrap();

    assert_eq!(expected, computed);
}

#[test]
fn icrs_frame_bias_magnitude_at_earth_surface() {
    use anise::constants::frames::{EME2000, GCRF};
    use core::str::FromStr;

    let almanac = Almanac::default().load("../data/pck11.pca").unwrap();
    let epoch = Epoch::from_str("2025-06-15T12:00:00 TDB").unwrap();

    // 6378 km along X in J2000 (Earth equatorial radius).
    let v_j2000 = Vector3::new(6378.0, 0.0, 0.0);
    let dcm = almanac.rotate(EME2000, GCRF, epoch).unwrap();
    let v_icrs = dcm.rot_mat * v_j2000;

    // Frame bias is ~0.04" total (~1.1e-7 rad); at 6378 km the
    // displacement is ~0.7 m = ~7e-4 km. Use a generous band.
    let delta_km = (v_icrs - v_j2000).norm();
    assert!(
        (1.0e-4..3.0e-3).contains(&delta_km),
        "expected bias 1e-4..3e-3 km (~0.1..3 m), got {delta_km:.3e} km"
    );
}

#[test]
fn icrs_chain_to_itrf93_differs_from_j2000_by_bias() {
    use anise::constants::frames::{EARTH_ITRF93, EME2000, GCRF};
    use core::str::FromStr;

    let almanac = Almanac::default()
        .load("../data/earth_latest_high_prec.bpc")
        .unwrap()
        .load("../data/pck11.pca")
        .unwrap();
    let epoch = Epoch::from_str("2020-06-15T12:00:00 TDB").unwrap();

    // GEO-altitude vector (~42164 km along X in EME2000).
    let v = Vector3::new(42_164.0, 0.0, 0.0);

    let dcm_eme = almanac.rotate(EME2000, EARTH_ITRF93, epoch).unwrap();
    let dcm_gcrf = almanac.rotate(GCRF, EARTH_ITRF93, epoch).unwrap();

    let v_via_eme = dcm_eme.rot_mat * v;
    let v_via_gcrf = dcm_gcrf.rot_mat * v;

    let delta_km = (v_via_gcrf - v_via_eme).norm();
    // Frame bias is ~1.1e-7 rad total; at 42164 km the displacement is ~4.5 m.
    assert!(
        (1.0e-3..1.0e-2).contains(&delta_km),
        "expected GCRF/EME2000 chain difference 1e-3..1e-2 km (~1..10 m), got {delta_km:.3e} km"
    );
}

#[cfg(feature = "validation")]
#[test]
fn icrs_matches_sofa_iaubp00() {
    use anise::constants::frames::{EME2000, GCRF};
    use core::str::FromStr;

    let almanac = Almanac::default().load("../data/pck11.pca").unwrap();
    let epoch = Epoch::from_str("2020-06-15T12:00:00 TDB").unwrap();

    // SOFA bp00 returns (rb, rp, rbp). We want rb (the bias-only matrix).
    // At J2000.0 TT the bias matrix is time-independent.
    let (rb, _rp, _rbp) = sofars::pnp::bp00(2451545.0, 0.0);

    let dcm = almanac.rotate(EME2000, GCRF, epoch).unwrap();

    for i in 0..3 {
        for j in 0..3 {
            let err = (dcm.rot_mat[(i, j)] - rb[i][j]).abs();
            assert!(
                err < 1e-14,
                "B[{i}][{j}]: anise={a:.18e}, sofa={s:.18e}, err={err:.3e}",
                a = dcm.rot_mat[(i, j)],
                s = rb[i][j],
            );
        }
    }
}

#[test]
fn icrs_angular_velocity_matches_eclipj2000_behaviour() {
    use anise::constants::frames::{EARTH_ECLIPJ2000, EME2000, GCRF};
    use core::str::FromStr;

    let almanac = Almanac::default().load("../data/pck11.pca").unwrap();
    let epoch = Epoch::from_str("2020-06-15T12:00:00 TDB").unwrap();

    let eclip_result = almanac.angular_velocity_rad_s(EARTH_ECLIPJ2000, EME2000, epoch);
    let icrs_result = almanac.angular_velocity_rad_s(GCRF, EME2000, epoch);

    assert_eq!(
        eclip_result.is_ok(),
        icrs_result.is_ok(),
        "ICRS angular velocity should match ECLIPJ2000 result kind: \
         eclip={eclip_result:?}, icrs={icrs_result:?}"
    );
    if let (Ok(ev), Ok(iv)) = (eclip_result.as_ref(), icrs_result.as_ref()) {
        assert!(
            ev.norm() < 1e-15,
            "ECLIPJ2000 angular velocity should be ~0"
        );
        assert!(iv.norm() < 1e-15, "ICRS angular velocity should be ~0");
    }
}

#[test]
fn body_inertial_frames() {
    use anise::constants::frames::{
        JUPITER_INERTIAL_FRAME, MARS_INERTIAL_FRAME, MERCURY_INERTIAL_FRAME, MOON_INERTIAL_FRAME,
        NEPTUNE_INERTIAL_FRAME, SATURN_INERTIAL_FRAME, URANUS_INERTIAL_FRAME, VENUS_INERTIAL_FRAME,
    };
    let almanac = Almanac::default().load("../data/pck11.pca").unwrap();
    let epoch = Epoch::from_str("2020-02-29T12:34:56 TDB").unwrap();

    for mut frame in [
        JUPITER_INERTIAL_FRAME,
        MARS_INERTIAL_FRAME,
        MERCURY_INERTIAL_FRAME,
        MOON_INERTIAL_FRAME,
        NEPTUNE_INERTIAL_FRAME,
        SATURN_INERTIAL_FRAME,
        URANUS_INERTIAL_FRAME,
        VENUS_INERTIAL_FRAME,
    ] {
        // Ensure that fetching the frame does not affect its properties.
        frame = almanac.frame_info(frame).unwrap();
        assert!(frame.force_inertial);
        assert!(frame.frozen_epoch.is_some());

        // Fetch the DCM and ensure the DCM derivative is zero and matches the DCM at J2000.
        let dcm = almanac.rotate(frame, GCRF, epoch).unwrap();
        assert!(dcm.rot_mat_dt.is_none());

        let mut ref_dcm = almanac
            .rotate(frame, GCRF, Epoch::from_et_seconds(0.0))
            .unwrap();
        ref_dcm.rot_mat_dt = None;

        assert_eq!(ref_dcm, dcm);

        println!("{frame}");
    }
}

#[test]
fn moon_tod_mod() {
    use anise::constants::frames::{MOON_MOD_FRAME, MOON_TOD_FRAME};
    let almanac = Almanac::default().load("../data/pck11.pca").unwrap();
    let epoch = Epoch::from_str("2020-02-29T12:34:56 TDB").unwrap();

    for mut frame in [MOON_MOD_FRAME, MOON_TOD_FRAME] {
        frame = almanac.frame_info(frame).unwrap();
        assert!(frame.force_inertial);
        assert!(frame.frozen_epoch.is_none());
        assert!(frame.is_dynamic());

        let dcm = almanac.rotate(frame, GCRF, epoch).unwrap();
        assert!(dcm.rot_mat_dt.is_none());
    }
    assert_eq!(format!("{MOON_MOD_FRAME}"), "Moon inertial MOD".to_string());
    assert_eq!(format!("{MOON_TOD_FRAME}"), "Moon inertial TOD".to_string());

    // Convert these to true of epoch frames
    let mut toe = MOON_TOD_FRAME;
    toe.frozen_epoch = Some(epoch);
    assert_eq!(
        format!("{toe}"),
        "Moon inertial TOE @ 2020-02-29T12:34:56 TDB".to_string()
    );

    let mut moe = MOON_MOD_FRAME;
    moe.frozen_epoch = Some(epoch);
    assert_eq!(
        format!("{moe}"),
        "Moon inertial MOE @ 2020-02-29T12:34:56 TDB".to_string()
    );
}

#[test]
fn earth_mean_of_date_mean_of_epoch() {
    use anise::constants::frames::{EARTH_MOD_FRAME, EARTH_MOD_LEGACY_FRAME};
    let almanac = Almanac::default()
        .load("../data/pck11.pca")
        .unwrap()
        .load("../data/de440s.bsp")
        .unwrap();
    let epoch = Epoch::from_str("2020-02-29T12:34:56 TDB").unwrap();

    // Ensure we can compute the rotation matrix to latest Earth MOD.
    let dcm_mod = almanac.rotate(EARTH_MOD_FRAME, GCRF, epoch).unwrap();
    assert!(dcm_mod.rot_mat_dt.is_none());
    println!("{dcm_mod}");

    let dcm_mod_legacy = almanac.rotate(EARTH_MOD_LEGACY_FRAME, GCRF, epoch).unwrap();
    assert!(dcm_mod_legacy.rot_mat_dt.is_none());
    println!("{dcm_mod_legacy}");

    let dcm_norm_delta = (dcm_mod.rot_mat - dcm_mod_legacy.rot_mat).norm();
    assert!(dcm_norm_delta > 0.0 && dcm_norm_delta < 1e-6);

    let mut of_epoch = EARTH_MOD_FRAME;
    // Must freeze at a different epoch than the evaluation epoch for this test
    of_epoch.frozen_epoch = Some(epoch - Unit::Day * 365.25);
    let dcm_moe = almanac.rotate(of_epoch, GCRF, epoch).unwrap();
    // We compare the exactly rotation matrices because the DCM structure itself has a larger epsilon.
    assert_ne!(dcm_moe.rot_mat, dcm_mod.rot_mat);
    assert!(dcm_mod.rot_mat_dt.is_none());
    // Note that the printed rotation name in the DCM structure only looses the knowledge that this
    // is an Of Epoch frame (it does not know that, it only knows the IDs).
    println!("{dcm_moe}");

    // Validation test case
    let epoch = Epoch::from_gregorian_utc_hms(2026, 5, 7, 18, 0, 0);
    let orbit = Orbit::new(7000.0, 0.0, 0.0, 0.0, 6.0, 0.0, epoch, EARTH_J2000);
    let orbit_xf = almanac
        .transform_to(orbit, EARTH_MOD_LEGACY_FRAME, None)
        .unwrap();
    f64_eq_tol!(orbit_xf.radius_km.x, 6999.855552, 1e-4, "validation failed");
    f64_eq_tol!(orbit_xf.radius_km.y, 41.244545, 1e-4, "validation failed");
    f64_eq_tol!(orbit_xf.radius_km.z, 17.920174, 1e-4, "validation failed");
    f64_eq_tol!(
        orbit_xf.velocity_km_s.x,
        -0.035352,
        1e-4,
        "validation failed"
    );
    f64_eq_tol!(
        orbit_xf.velocity_km_s.y,
        5.999896,
        1e-4,
        "validation failed"
    );
    f64_eq_tol!(
        orbit_xf.velocity_km_s.z,
        -0.000045,
        1e-4,
        "validation failed"
    );
}

#[test]
fn earth_true_of_date_true_of_epoch() {
    use anise::constants::frames::{EARTH_TOD_FRAME, EARTH_TOD_LEGACY_FRAME};
    use anise::constants::orientations::{EARTH_TOD_2000A, EARTH_TOD_2000B};
    let almanac = Almanac::default()
        .load("../data/pck11.pca")
        .unwrap()
        .load("../data/de440s.bsp")
        .unwrap();
    let epoch = Epoch::from_str("2020-02-29T12:34:56 TDB").unwrap();

    // Ensure we can compute the rotation matrix to latest Earth MOD.
    let dcm_mod = almanac.rotate(EARTH_TOD_FRAME, GCRF, epoch).unwrap();
    assert!(dcm_mod.rot_mat_dt.is_none());
    println!("{dcm_mod}");

    let dcm_mod_legacy = almanac.rotate(EARTH_TOD_LEGACY_FRAME, GCRF, epoch).unwrap();
    assert!(dcm_mod_legacy.rot_mat_dt.is_none());
    println!("{dcm_mod_legacy}");

    let dcm_norm_delta = (dcm_mod.rot_mat - dcm_mod_legacy.rot_mat).norm();
    assert!(dcm_norm_delta > 0.0 && dcm_norm_delta < 1e-6);

    // Compute the delta between both 2000 model
    let dcm_2000a = almanac
        .rotate(EARTH_TOD_FRAME.with_orient(EARTH_TOD_2000A), GCRF, epoch)
        .unwrap();
    assert!(dcm_2000a.rot_mat_dt.is_none());
    println!("{dcm_2000a}");

    let dcm_2000b = almanac
        .rotate(EARTH_TOD_FRAME.with_orient(EARTH_TOD_2000B), GCRF, epoch)
        .unwrap();
    assert!(dcm_2000b.rot_mat_dt.is_none());
    println!("{dcm_2000b}");

    let dcm_norm_delta = (dcm_2000a.rot_mat - dcm_2000b.rot_mat).norm();
    assert!(dcm_norm_delta > 0.0 && dcm_norm_delta < 2e-9);

    // Validation test case
    let epoch = Epoch::from_gregorian_utc_hms(2026, 5, 7, 18, 0, 0);
    let orbit = Orbit::new(7000.0, 0.0, 0.0, 0.0, 6.0, 0.0, epoch, EARTH_J2000);
    let orbit_xf = almanac
        .transform_to(orbit, EARTH_TOD_LEGACY_FRAME, None)
        .unwrap();
    println!("{orbit_xf}");
    f64_eq_tol!(orbit_xf.radius_km.x, 6999.854255, 1e-4, "validation failed");
    f64_eq_tol!(orbit_xf.radius_km.y, 41.428761, 1e-4, "validation failed");
    f64_eq_tol!(orbit_xf.radius_km.z, 18.001989, 1e-4, "validation failed");
    f64_eq_tol!(
        orbit_xf.velocity_km_s.x,
        -0.03551,
        1e-4,
        "validation failed"
    );
    f64_eq_tol!(
        orbit_xf.velocity_km_s.y,
        5.999895,
        1e-4,
        "validation failed"
    );
    f64_eq_tol!(
        orbit_xf.velocity_km_s.z,
        0.000193,
        1e-4,
        "validation failed"
    );
}

#[test]
fn earth_teme() {
    use anise::constants::frames::{EARTH_TEME_FRAME, EARTH_TEME_LEGACY_FRAME};
    let almanac = Almanac::default()
        .load("../data/pck11.pca")
        .unwrap()
        .load("../data/de440s.bsp")
        .unwrap();
    let epoch = Epoch::from_str("2020-02-29T12:34:56 TDB").unwrap();

    // Ensure we can compute the rotation matrix to latest Earth MOD.
    let dcm_mod = almanac.rotate(EARTH_TEME_FRAME, GCRF, epoch).unwrap();
    assert!(dcm_mod.rot_mat_dt.is_none());
    println!("{dcm_mod}");

    let dcm_mod_legacy = almanac
        .rotate(EARTH_TEME_LEGACY_FRAME, GCRF, epoch)
        .unwrap();
    assert!(dcm_mod_legacy.rot_mat_dt.is_none());
    println!("{dcm_mod_legacy}");

    let dcm_norm_delta = (dcm_mod.rot_mat - dcm_mod_legacy.rot_mat).norm();
    assert!(dcm_norm_delta > 0.0 && dcm_norm_delta < 1e-6);

    // Validation test case
    let epoch = Epoch::from_gregorian_utc_hms(2026, 5, 7, 18, 0, 0);
    let orbit = Orbit::new(7000.0, 0.0, 0.0, 0.0, 6.0, 0.0, epoch, EARTH_J2000);
    let orbit_xf = almanac
        .transform_to(orbit, EARTH_TEME_LEGACY_FRAME, None)
        .unwrap();
    f64_eq_tol!(orbit_xf.radius_km.x, 6999.855346, 1e-4, "validation failed");
    f64_eq_tol!(orbit_xf.radius_km.y, 41.243867, 1e-4, "validation failed");
    f64_eq_tol!(orbit_xf.radius_km.z, 18.001989, 1e-4, "validation failed");
    f64_eq_tol!(
        orbit_xf.velocity_km_s.x,
        -0.035352,
        1e-4,
        "validation failed"
    );
    f64_eq_tol!(
        orbit_xf.velocity_km_s.y,
        5.999896,
        1e-4,
        "validation failed"
    );
    f64_eq_tol!(
        orbit_xf.velocity_km_s.z,
        0.000193,
        1e-4,
        "validation failed"
    );
}
