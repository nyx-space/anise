/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::{
    constants::{
        celestial_objects::EARTH,
        frames::*,
        orientations::{ECLIPJ2000, FK4, ITRF93, J2000},
    },
    math::{
        cartesian::CartesianState,
        rotation::{Quaternion, DCM},
        Matrix3, Vector3,
    },
    naif::kpl::parser::convert_tpc,
    prelude::{Almanac, Frame, BPC},
};
use hifitime::{Duration, Epoch, TimeSeries, TimeUnits};
use spice::cstr;

// Allow up to two arcsecond of error (or 0.12 microradians), but check test results for actualized error
const MAX_ERR_DEG: f64 = 7.2e-6;
const DCM_EPSILON: f64 = 1e-9;

// IAU Moon rotates fast. This shows the difference between SPICE's and Hifitime's implementation of time because SPICE has a rounding error
// when computing the centuries past J2000 ET.
const IAU_MOON_DCM_EPSILON: f64 = 1e-5;
const IAU_MOON_MAX_ERR_DEG: f64 = 1e-3;
// Absolute error tolerance between ANISE and SPICE for the same state rotation.
const POSITION_ERR_TOL_KM: f64 = 2e-5;
const VELOCITY_ERR_TOL_KM_S: f64 = 5e-7;
// Return absolute tolerance, i.e. perform the same rotation from A to B and B to A, and check that the norm error is less than that.
const RTN_POSITION_EPSILON_KM: f64 = 1e-10;
const RTN_VELOCITY_EPSILON_KM_S: f64 = 1e-10;

/// This test converts the PCK file into its ANISE equivalent format, loads it into an Almanac, and compares the rotations computed by the Almanac and by SPICE
/// It only check the IAU rotations to its J2000 parent, and accounts for nutation and precession coefficients where applicable.
#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn validate_iau_rotation_to_parent() {
    let pck = "../data/pck00008.tpc";
    spice::furnsh(pck);
    let planetary_data = convert_tpc(pck, "../data/gm_de431.tpc").unwrap();

    let almanac = Almanac {
        planetary_data,
        ..Default::default()
    };

    for frame in [
        IAU_MERCURY_FRAME,
        IAU_VENUS_FRAME,
        IAU_EARTH_FRAME,
        IAU_MOON_FRAME,
        IAU_MARS_FRAME,
        IAU_JUPITER_FRAME,
        IAU_SATURN_FRAME,
        // IAU_URANUS_FRAME, // TODO: https://github.com/nyx-space/anise/issues/185
        // IAU_NEPTUNE_FRAME,
    ] {
        for (num, epoch) in TimeSeries::inclusive(
            Epoch::from_tdb_duration(Duration::ZERO),
            Epoch::from_tdb_duration(0.2.centuries()),
            1.days(),
        )
        .enumerate()
        {
            let dcm = almanac
                .rotate(frame.with_orient(J2000), frame, epoch)
                .unwrap();

            let mut rot_data: [[f64; 6]; 6] = [[0.0; 6]; 6];
            unsafe {
                spice::c::sxform_c(
                    cstr!("J2000"),
                    cstr!(format!("{frame:o}")),
                    epoch.to_tdb_seconds(),
                    rot_data.as_mut_ptr(),
                );
            }

            // Parent rotation of Earth IAU frame is 3 not J2000, etc.
            assert!(
                [J2000, FK4, 4, 5, 6].contains(&dcm.from),
                "unexpected DCM from frame {}",
                dcm.from
            );
            assert_eq!(dcm.to, frame.orientation_id);

            // Confirmed that the M3x3 below is the correct representation from SPICE by using the mxv spice function and compare that to the nalgebra equivalent computation.
            let spice_mat = Matrix3::new(
                rot_data[0][0],
                rot_data[0][1],
                rot_data[0][2],
                rot_data[1][0],
                rot_data[1][1],
                rot_data[1][2],
                rot_data[2][0],
                rot_data[2][1],
                rot_data[2][2],
            );

            let rot_mat_dt = Some(Matrix3::new(
                rot_data[3][0],
                rot_data[3][1],
                rot_data[3][2],
                rot_data[4][0],
                rot_data[4][1],
                rot_data[4][2],
                rot_data[5][0],
                rot_data[5][1],
                rot_data[5][2],
            ));

            let spice_dcm = DCM {
                rot_mat: spice_mat,
                from: dcm.from,
                to: dcm.to,
                rot_mat_dt,
            };

            // Print out the error at its greatest, since we're the furthest away from J2000 reference epoch.
            if epoch == Epoch::from_tdb_duration(0.2.centuries()) {
                println!("ANISE: {dcm}{}", dcm.rot_mat_dt.unwrap());
                println!("SPICE: {spice_dcm}{}", spice_dcm.rot_mat_dt.unwrap());

                println!("DCM error\n{:e}", dcm.rot_mat - spice_dcm.rot_mat);

                println!(
                    "derivative error\n{:e}",
                    dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
                );
            }

            // Compute the different in PRV and rotation angle
            let q_anise = Quaternion::from(dcm);
            let q_spice = Quaternion::from(spice_dcm);

            let (anise_uvec, anise_angle) = q_anise.uvec_angle();
            let (spice_uvec, spice_angle) = q_spice.uvec_angle();

            let uvec_angle_deg_err = anise_uvec.dot(&spice_uvec).acos().to_degrees();
            let deg_err = (anise_angle - spice_angle).to_degrees();

            // In some cases, the arc cos of the angle between the unit vectors is NaN (because the dot product is rounded just past -1 or +1)
            // so we allow NaN.
            // However, we also check the rotation about that unit vector AND we check that the DCMs match too.
            let angular_err = if frame == IAU_MOON_FRAME {
                IAU_MOON_MAX_ERR_DEG
            } else {
                MAX_ERR_DEG
            };

            assert!(
                uvec_angle_deg_err.abs() < angular_err || uvec_angle_deg_err.is_nan(),
                "#{num} @ {epoch} unit vector angle error for {frame}: {uvec_angle_deg_err:e} deg"
            );
            assert!(
                deg_err.abs() < angular_err,
                "#{num} @ {epoch} rotation error for {frame}: {deg_err:e} deg"
            );

            let dcm_err = if frame == IAU_MOON_FRAME {
                IAU_MOON_DCM_EPSILON
            } else {
                DCM_EPSILON
            };

            assert!(
                (dcm.rot_mat - spice_mat).norm() < dcm_err,
                "#{num} {epoch}\ngot: {}want:{spice_mat}err = {:.3e}: {:.3e}",
                dcm.rot_mat,
                (dcm.rot_mat - spice_mat).norm(),
                dcm.rot_mat - spice_mat
            );

            // Check the derivative
            assert!(
                (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm() < DCM_EPSILON,
                "#{num} {epoch}\ngot: {}want:{}err = {:.3e}: {:.3e}",
                dcm.rot_mat_dt.unwrap(),
                spice_dcm.rot_mat_dt.unwrap(),
                (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm(),
                dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
            );

            // Check the transpose
            let dcm_t = almanac
                .rotate(frame, frame.with_orient(J2000), epoch)
                .unwrap();
            assert_eq!(dcm.transpose(), dcm_t);
        }
    }
}

#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn validate_bpc_rotation_to_parent() {
    let pck = "../data/earth_latest_high_prec.bpc";
    spice::furnsh(pck);

    let bpc = BPC::load(pck).unwrap();
    let almanac = Almanac::from_bpc(bpc).unwrap();

    // This BPC file start in 2011 and ends in 2022.
    for (num, epoch) in TimeSeries::inclusive(
        Epoch::from_tdb_duration(0.11.centuries()),
        Epoch::from_tdb_duration(0.2.centuries()),
        1.days(),
    )
    .enumerate()
    {
        let dcm = almanac.rotation_to_parent(EARTH_ITRF93, epoch).unwrap();

        if num == 0 {
            println!("{dcm}");
        }

        let mut rot_data: [[f64; 6]; 6] = [[0.0; 6]; 6];
        unsafe {
            spice::c::sxform_c(
                cstr!("ECLIPJ2000"),
                cstr!("ITRF93"),
                epoch.to_tdb_seconds(),
                rot_data.as_mut_ptr(),
            );
        }

        // Confirmed that the M3x3 below is the correct representation from SPICE by using the mxv spice function and compare that to the nalgebra equivalent computation.
        let rot_mat = Matrix3::new(
            rot_data[0][0],
            rot_data[0][1],
            rot_data[0][2],
            rot_data[1][0],
            rot_data[1][1],
            rot_data[1][2],
            rot_data[2][0],
            rot_data[2][1],
            rot_data[2][2],
        );

        let rot_mat_dt = Some(Matrix3::new(
            rot_data[3][0],
            rot_data[3][1],
            rot_data[3][2],
            rot_data[4][0],
            rot_data[4][1],
            rot_data[4][2],
            rot_data[5][0],
            rot_data[5][1],
            rot_data[5][2],
        ));

        let spice_dcm = DCM {
            rot_mat,
            from: dcm.from,
            to: dcm.to,
            rot_mat_dt,
        };

        if num == 0 {
            println!("ANISE: {dcm}{}", dcm.rot_mat_dt.unwrap());
            println!("SPICE: {spice_dcm}{}", spice_dcm.rot_mat_dt.unwrap());

            println!("DCM error\n{:e}", dcm.rot_mat - spice_dcm.rot_mat);

            println!(
                "derivative error\n{:e}",
                dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
            );
        }

        // Compute the different in PRV and rotation angle
        let q_anise = Quaternion::from(dcm);
        let q_spice = Quaternion::from(spice_dcm);

        let (anise_uvec, anise_angle) = q_anise.uvec_angle();
        let (spice_uvec, spice_angle) = q_spice.uvec_angle();

        let uvec_angle_deg_err = anise_uvec.dot(&spice_uvec).acos().to_degrees();
        let deg_err = (anise_angle - spice_angle).to_degrees();

        // In some cases, the arc cos of the angle between the unit vectors is NaN (because the dot product is rounded just past -1 or +1)
        // so we allow NaN.
        // However, we also check the rotation about that unit vector AND we check that the DCMs match too.
        assert!(
            uvec_angle_deg_err.abs() < MAX_ERR_DEG || uvec_angle_deg_err.is_nan(),
            "#{num} @ {epoch} unit vector angle error: {uvec_angle_deg_err:e} deg"
        );
        assert!(
            deg_err.abs() < MAX_ERR_DEG,
            "#{num} @ {epoch} rotation error: {deg_err:e} deg"
        );

        assert!(
            (dcm.rot_mat - rot_mat).norm() < DCM_EPSILON,
            "#{num} {epoch}\ngot: {}want:{rot_mat}err = {:.3e}: {:.3e}",
            dcm.rot_mat,
            (dcm.rot_mat - rot_mat).norm(),
            dcm.rot_mat - rot_mat
        );

        // Check the derivative
        assert!(
            (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm() < 1e-13,
            "#{num} {epoch}\ngot: {}want:{}err = {:.3e}: {:.3e}",
            dcm.rot_mat_dt.unwrap(),
            spice_dcm.rot_mat_dt.unwrap(),
            (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm(),
            dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
        );

        // Check the frames
        assert_eq!(dcm.from, ECLIPJ2000);
        assert_eq!(dcm.to, ITRF93);
    }
}

/// Ensure that our rotation for [ECLIPJ2000] to [J2000] matches the one from SPICE.
#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn validate_j2000_ecliptic() {
    // The eclipj2000 to j2000 rotation is embedded, so we don't need to load anything.
    let almanac = Almanac::default();

    for (num, epoch) in TimeSeries::inclusive(
        Epoch::from_tdb_duration(0.11.centuries()),
        Epoch::from_tdb_duration(0.2.centuries()),
        100.days(),
    )
    .enumerate()
    {
        let dcm = almanac.rotation_to_parent(EARTH_ECLIPJ2000, epoch).unwrap();

        let mut rot_data: [[f64; 6]; 6] = [[0.0; 6]; 6];
        unsafe {
            spice::c::sxform_c(
                cstr!("J2000"),
                cstr!("ECLIPJ2000"),
                epoch.to_tdb_seconds(),
                rot_data.as_mut_ptr(),
            );
        }

        assert_eq!(dcm.from, J2000);
        assert_eq!(dcm.to, ECLIPJ2000);

        // Confirmed that the M3x3 below is the correct representation from SPICE by using the mxv spice function and compare that to the nalgebra equivalent computation.
        let rot_mat = Matrix3::new(
            rot_data[0][0],
            rot_data[0][1],
            rot_data[0][2],
            rot_data[1][0],
            rot_data[1][1],
            rot_data[1][2],
            rot_data[2][0],
            rot_data[2][1],
            rot_data[2][2],
        );

        let rot_mat_dt = Matrix3::new(
            rot_data[3][0],
            rot_data[3][1],
            rot_data[3][2],
            rot_data[4][0],
            rot_data[4][1],
            rot_data[4][2],
            rot_data[5][0],
            rot_data[5][1],
            rot_data[5][2],
        );

        let spice_dcm = DCM {
            rot_mat,
            from: dcm.from,
            to: dcm.to,
            rot_mat_dt: if rot_mat_dt.norm() == 0.0 {
                // I know this will be the case.
                None
            } else {
                Some(rot_mat_dt)
            },
        };

        assert!(
            (dcm.rot_mat - rot_mat).norm() < f64::EPSILON,
            "#{num} {epoch}\ngot: {}want:{rot_mat}err = {:.3e}: {:.3e}",
            dcm.rot_mat,
            (dcm.rot_mat - rot_mat).norm(),
            dcm.rot_mat - rot_mat
        );

        // Check the derivative
        assert_eq!(
            dcm.rot_mat_dt, spice_dcm.rot_mat_dt,
            "expected both derivatives to be unuset"
        );
    }
}

#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn validate_bpc_rotations() {
    let pck = "../data/earth_latest_high_prec.bpc";
    spice::furnsh(pck);

    let bpc = BPC::load(pck).unwrap();
    let almanac = Almanac::from_bpc(bpc).unwrap();

    let frame = Frame::new(EARTH, ITRF93);

    let mut actual_max_uvec_err_deg = 0.0;
    let mut actual_max_err_deg = 0.0;

    // This BPC file start in 2011 and ends in 2022.
    for (num, epoch) in TimeSeries::inclusive(
        Epoch::from_tdb_duration(0.11.centuries()),
        Epoch::from_tdb_duration(0.2.centuries()),
        1.days(),
    )
    .enumerate()
    {
        let dcm = almanac.rotate(EARTH_ITRF93, EME2000, epoch).unwrap();

        let mut rot_data: [[f64; 6]; 6] = [[0.0; 6]; 6];
        unsafe {
            spice::c::sxform_c(
                cstr!("ITRF93"),
                cstr!("J2000"),
                epoch.to_tdb_seconds(),
                rot_data.as_mut_ptr(),
            );
        }

        // Confirmed that the M3x3 below is the correct representation from SPICE by using the mxv spice function and compare that to the nalgebra equivalent computation.
        let rot_mat = Matrix3::new(
            rot_data[0][0],
            rot_data[0][1],
            rot_data[0][2],
            rot_data[1][0],
            rot_data[1][1],
            rot_data[1][2],
            rot_data[2][0],
            rot_data[2][1],
            rot_data[2][2],
        );

        let rot_mat_dt = Some(Matrix3::new(
            rot_data[3][0],
            rot_data[3][1],
            rot_data[3][2],
            rot_data[4][0],
            rot_data[4][1],
            rot_data[4][2],
            rot_data[5][0],
            rot_data[5][1],
            rot_data[5][2],
        ));

        let spice_dcm = DCM {
            rot_mat,
            from: ITRF93,
            to: J2000,
            rot_mat_dt,
        };

        if num == 0 {
            println!("ANISE: {dcm}{}", dcm.rot_mat_dt.unwrap());
            println!("SPICE: {spice_dcm}{}", spice_dcm.rot_mat_dt.unwrap());

            println!("DCM error\n{:e}", dcm.rot_mat - spice_dcm.rot_mat);

            println!(
                "derivative error\n{:e}",
                dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
            );
        }

        // Compute the different in PRV and rotation angle
        let q_anise = Quaternion::from(dcm);
        let q_spice = Quaternion::from(spice_dcm);

        let (anise_uvec, anise_angle) = q_anise.uvec_angle();
        let (spice_uvec, spice_angle) = q_spice.uvec_angle();

        let uvec_angle_deg_err = anise_uvec.dot(&spice_uvec).acos().to_degrees();
        let deg_err = (anise_angle - spice_angle).to_degrees();

        // In some cases, the arc cos of the angle between the unit vectors is NaN (because the dot product is rounded just past -1 or +1)
        // so we allow NaN.
        // However, we also check the rotation about that unit vector AND we check that the DCMs match too.
        assert!(
            uvec_angle_deg_err.abs() < MAX_ERR_DEG || uvec_angle_deg_err.is_nan(),
            "#{num} @ {epoch} unit vector angle error for {frame}: {uvec_angle_deg_err:e} deg"
        );

        if uvec_angle_deg_err.abs() > actual_max_uvec_err_deg {
            actual_max_uvec_err_deg = uvec_angle_deg_err.abs();
        }

        assert!(
            deg_err.abs() < MAX_ERR_DEG,
            "#{num} @ {epoch} rotation error for {frame}: {deg_err:e} deg"
        );

        if deg_err.abs() > actual_max_err_deg {
            actual_max_err_deg = deg_err.abs();
        }

        assert!(
            (dcm.rot_mat - rot_mat).norm() < DCM_EPSILON,
            "#{num} {epoch}\ngot: {}want:{rot_mat}err = {:.3e}: {:.3e}",
            dcm.rot_mat,
            (dcm.rot_mat - rot_mat).norm(),
            dcm.rot_mat - rot_mat
        );

        // Check the derivative
        assert!(
            (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm() < 1e-13,
            "#{num} {epoch}\ngot: {}want:{}err = {:.3e}: {:.3e}",
            dcm.rot_mat_dt.unwrap(),
            spice_dcm.rot_mat_dt.unwrap(),
            (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm(),
            dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
        );
    }
    println!("actualized max error in rotation angle = {actual_max_err_deg:.3e} deg");
    println!("actualized max error in rotation direction = {actual_max_uvec_err_deg:.3e} deg");
}

#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn validate_bpc_to_iau_rotations() {
    let pck = "../data/pck00008.tpc";
    let bpc = "../data/earth_latest_high_prec.bpc";
    spice::furnsh(bpc);
    spice::furnsh(pck);
    let planetary_data = convert_tpc(pck, "../data/gm_de431.tpc").unwrap();

    let almanac = Almanac {
        planetary_data,
        ..Default::default()
    };
    let almanac = almanac.with_bpc(BPC::load(bpc).unwrap()).unwrap();

    println!("{almanac}");

    let mut actual_max_uvec_err_deg = 0.0;
    let mut actual_max_err_deg = 0.0;
    let mut actual_pos_err_km = 0.0;
    let mut actual_vel_err_km_s = 0.0;

    let start = Epoch::from_tdb_duration(0.11.centuries());
    let end = Epoch::from_tdb_duration(0.20.centuries());

    for frame in [
        IAU_MERCURY_FRAME,
        IAU_VENUS_FRAME,
        IAU_EARTH_FRAME,
        IAU_MARS_FRAME,
        IAU_JUPITER_FRAME,
        IAU_SATURN_FRAME,
    ] {
        for (num, epoch) in TimeSeries::inclusive(start, end, 27.days()).enumerate() {
            let dcm = almanac.rotate(EARTH_ITRF93, frame, epoch).unwrap();

            let mut rot_data: [[f64; 6]; 6] = [[0.0; 6]; 6];
            let spice_name = format!("{frame:o}");
            unsafe {
                spice::c::sxform_c(
                    cstr!("ITRF93"),
                    cstr!(spice_name),
                    epoch.to_tdb_seconds(),
                    rot_data.as_mut_ptr(),
                );
            }

            // Confirmed that the M3x3 below is the correct representation from SPICE by using the mxv spice function and compare that to the nalgebra equivalent computation.
            let rot_mat = Matrix3::new(
                rot_data[0][0],
                rot_data[0][1],
                rot_data[0][2],
                rot_data[1][0],
                rot_data[1][1],
                rot_data[1][2],
                rot_data[2][0],
                rot_data[2][1],
                rot_data[2][2],
            );

            let rot_mat_dt = Some(Matrix3::new(
                rot_data[3][0],
                rot_data[3][1],
                rot_data[3][2],
                rot_data[4][0],
                rot_data[4][1],
                rot_data[4][2],
                rot_data[5][0],
                rot_data[5][1],
                rot_data[5][2],
            ));

            let spice_dcm = DCM {
                rot_mat,
                from: ITRF93,
                to: frame.orientation_id,
                rot_mat_dt,
            };

            if num == 0 {
                println!("ANISE: {dcm}");
                println!("SPICE: {spice_dcm}");

                println!("DCM error\n{:e}", dcm.rot_mat - spice_dcm.rot_mat);

                println!(
                    "derivative error\n{:e}",
                    dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
                );
            }

            assert_eq!(dcm.from, EARTH_ITRF93.orientation_id);
            assert_eq!(dcm.to, frame.orientation_id);

            // Compute the different in PRV and rotation angle
            let q_anise = Quaternion::from(dcm);
            let q_spice = Quaternion::from(spice_dcm);

            let (anise_uvec, anise_angle) = q_anise.uvec_angle();
            let (spice_uvec, spice_angle) = q_spice.uvec_angle();

            let uvec_angle_deg_err = anise_uvec.dot(&spice_uvec).acos().to_degrees();
            let deg_err = (anise_angle - spice_angle).to_degrees();

            // In some cases, the arc cos of the angle between the unit vectors is NaN (because the dot product is rounded just past -1 or +1)
            // so we allow NaN.
            // However, we also check the rotation about that unit vector AND we check that the DCMs match too.
            assert!(
                uvec_angle_deg_err.abs() < MAX_ERR_DEG || uvec_angle_deg_err.is_nan(),
                "#{num} @ {epoch} unit vector angle error for {frame}: {uvec_angle_deg_err:e} deg"
            );

            if uvec_angle_deg_err.abs() > actual_max_uvec_err_deg {
                actual_max_uvec_err_deg = uvec_angle_deg_err.abs();
            }

            assert!(
                deg_err.abs() < MAX_ERR_DEG,
                "#{num} @ {epoch} rotation error for {frame}: {deg_err:e} deg"
            );

            if deg_err.abs() > actual_max_err_deg {
                actual_max_err_deg = deg_err.abs();
            }

            assert!(
                (dcm.rot_mat - rot_mat).norm() < DCM_EPSILON,
                "#{num} {epoch}\ngot: {}want:{rot_mat}err = {:.3e}: {:.3e}",
                dcm.rot_mat,
                (dcm.rot_mat - rot_mat).norm(),
                dcm.rot_mat - rot_mat
            );

            // Check the derivative with a slightly tighet constraint
            assert!(
                (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm()
                    < DCM_EPSILON * 0.1,
                "#{num} {epoch}\ngot: {}want:{}err = {:.3e}: {:.3e}",
                dcm.rot_mat_dt.unwrap(),
                spice_dcm.rot_mat_dt.unwrap(),
                (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm(),
                dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
            );

            // Check that we match the SXFORM documentation on the DCM * CartesianState multiplication
            let state = CartesianState {
                radius_km: Vector3::new(1234.0, 5678.9, 1234.0),
                velocity_km_s: Vector3::new(1.2340, 5.6789, 1.2340),
                epoch,
                frame: EARTH_ITRF93,
            };

            let spice_out = (spice_dcm * state).unwrap();
            let anise_out = (dcm * state).unwrap();

            assert_eq!(spice_out.frame, anise_out.frame);
            let pos_err_km = (spice_out.radius_km - anise_out.radius_km).norm();
            assert!(
                pos_err_km < POSITION_ERR_TOL_KM,
                "#{num} {epoch}: pos error is {pos_err_km:.3e} km/s"
            );
            let vel_err_km_s = (spice_out.velocity_km_s - anise_out.velocity_km_s).norm();
            assert!(
                vel_err_km_s < VELOCITY_ERR_TOL_KM_S,
                "#{num} {epoch}: vel error is {vel_err_km_s:.3e} km/s"
            );

            if pos_err_km > actual_pos_err_km {
                actual_pos_err_km = pos_err_km;
            }

            if vel_err_km_s > actual_vel_err_km_s {
                actual_vel_err_km_s = vel_err_km_s;
            }

            // Grab the transposed DCM
            let dcm_t = almanac.rotate(frame, EARTH_ITRF93, epoch).unwrap();

            let mut rot_data: [[f64; 6]; 6] = [[0.0; 6]; 6];
            unsafe {
                spice::c::sxform_c(
                    cstr!(format!("{frame:o}")),
                    cstr!("ITRF93"),
                    epoch.to_tdb_seconds(),
                    rot_data.as_mut_ptr(),
                );
            }

            // Confirmed that the M3x3 below is the correct representation from SPICE by using the mxv spice function and compare that to the nalgebra equivalent computation.
            let rot_mat = Matrix3::new(
                rot_data[0][0],
                rot_data[0][1],
                rot_data[0][2],
                rot_data[1][0],
                rot_data[1][1],
                rot_data[1][2],
                rot_data[2][0],
                rot_data[2][1],
                rot_data[2][2],
            );

            let rot_mat_dt = Some(Matrix3::new(
                rot_data[3][0],
                rot_data[3][1],
                rot_data[3][2],
                rot_data[4][0],
                rot_data[4][1],
                rot_data[4][2],
                rot_data[5][0],
                rot_data[5][1],
                rot_data[5][2],
            ));

            let spice_dcm_t = DCM {
                rot_mat,
                from: dcm_t.from,
                to: dcm_t.to,
                rot_mat_dt,
            };

            let spice_rtn = (spice_dcm_t * spice_out).unwrap();
            let anise_rtn = (dcm_t * anise_out).unwrap();

            assert_eq!(spice_rtn.frame, anise_rtn.frame);
            assert!((spice_rtn.radius_km - state.radius_km).norm() < RTN_POSITION_EPSILON_KM);
            assert!(
                (spice_rtn.velocity_km_s - state.velocity_km_s).norm() < RTN_VELOCITY_EPSILON_KM_S
            );
            assert!((anise_rtn.radius_km - state.radius_km).norm() < RTN_POSITION_EPSILON_KM);
            assert!(
                (anise_rtn.velocity_km_s - state.velocity_km_s).norm() < RTN_VELOCITY_EPSILON_KM_S
            );
        }
    }
    println!("actualized max error in rotation angle = {actual_max_err_deg:.3e} deg");
    println!("actualized max error in rotation direction = {actual_max_uvec_err_deg:.3e} deg");
    println!("actualized max error in position = {actual_pos_err_km:.6e} km");
    println!("actualized max error in velocity = {actual_vel_err_km_s:.6e} km/s");
}
