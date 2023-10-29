/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::f64::EPSILON;

use anise::{
    constants::{
        celestial_objects::EARTH,
        frames::*,
        orientations::{ECLIPJ2000, ITRF93, J2000},
    },
    math::{
        rotation::{Quaternion, DCM},
        Matrix3,
    },
    naif::kpl::parser::convert_tpc,
    prelude::{Almanac, Frame, BPC},
};
use hifitime::{Duration, Epoch, TimeSeries, TimeUnits};
use spice::cstr;

// Allow up to two arcsecond of error (or 0.12 microradians), but check test results for actualized error
const MAX_ERR_DEG: f64 = 7.2e-6;
const DCM_EPSILON: f64 = 1e-9;

/// This test converts the PCK file into its ANISE equivalent format, loads it into an Almanac, and compares the rotations computed by the Almanac and by SPICE
/// It only check the IAU rotations to its J2000 parent, and accounts for nutation and precession coefficients where applicable.
#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn validate_iau_rotation_to_parent() {
    // Known bug with nutation and precession angles: https://github.com/nyx-space/anise/issues/122
    let pck = "data/pck00008.tpc";
    spice::furnsh(pck);
    let planetary_data = convert_tpc(pck, "data/gm_de431.tpc").unwrap();

    let almanac = Almanac {
        planetary_data,
        ..Default::default()
    };

    for frame in [
        IAU_MERCURY_FRAME,
        IAU_VENUS_FRAME,
        IAU_EARTH_FRAME,
        IAU_MARS_FRAME,
        IAU_JUPITER_FRAME,
        IAU_SATURN_FRAME,
        // IAU_NEPTUNE_FRAME, // Bug: https://github.com/nyx-space/anise/issues/122
        // IAU_URANUS_FRAME,
    ] {
        for (num, epoch) in TimeSeries::inclusive(
            Epoch::from_tdb_duration(Duration::ZERO),
            Epoch::from_tdb_duration(0.2.centuries()),
            1.days(),
        )
        .enumerate()
        {
            let dcm = almanac.rotation_to_parent(frame, epoch).unwrap();

            let mut rot_data: [[f64; 6]; 6] = [[0.0; 6]; 6];
            unsafe {
                spice::c::sxform_c(
                    cstr!("J2000"),
                    cstr!(format!("{frame:o}")),
                    epoch.to_tdb_seconds(),
                    rot_data.as_mut_ptr(),
                );
            }

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
            assert!(
                deg_err.abs() < MAX_ERR_DEG,
                "#{num} @ {epoch} rotation error for {frame}: {deg_err:e} deg"
            );

            assert!(
                (dcm.rot_mat - spice_mat).norm() < DCM_EPSILON,
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
        }
    }
}

#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn validate_bpc_rotation_to_parent() {
    let pck = "data/earth_latest_high_prec.bpc";
    spice::furnsh(pck);

    let bpc = BPC::load(pck).unwrap();
    let almanac = Almanac::from_bpc(bpc).unwrap();

    let frame = Frame::from_ephem_orient(EARTH, ITRF93);

    // This BPC file start in 2011 and ends in 2022.
    for (num, epoch) in TimeSeries::inclusive(
        Epoch::from_tdb_duration(0.11.centuries()),
        Epoch::from_tdb_duration(0.2.centuries()),
        1.days(),
    )
    .enumerate()
    {
        let dcm = almanac.rotation_to_parent(frame, epoch).unwrap();

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
            "#{num} @ {epoch} unit vector angle error for {frame}: {uvec_angle_deg_err:e} deg"
        );
        assert!(
            deg_err.abs() < MAX_ERR_DEG,
            "#{num} @ {epoch} rotation error for {frame}: {deg_err:e} deg"
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
            (dcm.rot_mat - rot_mat).norm() < EPSILON,
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

        assert_eq!(dcm.from, ECLIPJ2000);
        assert_eq!(dcm.to, J2000);
    }
}

#[ignore = "Requires Rust SPICE -- must be executed serially"]
#[test]
fn validate_bpc_rotations() {
    let pck = "data/earth_latest_high_prec.bpc";
    spice::furnsh(pck);

    let bpc = BPC::load(pck).unwrap();
    let almanac = Almanac::from_bpc(bpc).unwrap();

    let frame = Frame::from_ephem_orient(EARTH, ITRF93);

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
        let dcm = almanac
            .rotate_from_to(EARTH_ITRF93, EME2000, epoch)
            .unwrap();

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
    let pck = "data/pck00008.tpc";
    let bpc = "data/earth_latest_high_prec.bpc";
    spice::furnsh(bpc);
    spice::furnsh(pck);
    let planetary_data = convert_tpc(pck, "data/gm_de431.tpc").unwrap();

    let almanac = Almanac {
        planetary_data,
        ..Default::default()
    };
    let almanac = almanac.load_bpc(BPC::load(bpc).unwrap()).unwrap();

    println!("{almanac}");

    let mut actual_max_uvec_err_deg = 0.0;
    let mut actual_max_err_deg = 0.0;

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
        for (num, epoch) in TimeSeries::inclusive(start, end, 1.days()).enumerate() {
            let dcm = almanac.rotate_from_to(EARTH_ITRF93, frame, epoch).unwrap();

            let mut rot_data: [[f64; 6]; 6] = [[0.0; 6]; 6];
            unsafe {
                spice::c::sxform_c(
                    cstr!("ITRF93"),
                    cstr!(format!("{frame:o}")),
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
                println!("ANISE: {dcm}");
                println!("SPICE: {spice_dcm}");

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
                (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm()
                    < DCM_EPSILON * 0.1,
                "#{num} {epoch}\ngot: {}want:{}err = {:.3e}: {:.3e}",
                dcm.rot_mat_dt.unwrap(),
                spice_dcm.rot_mat_dt.unwrap(),
                (dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()).norm(),
                dcm.rot_mat_dt.unwrap() - spice_dcm.rot_mat_dt.unwrap()
            );
        }
    }
    println!("actualized max error in rotation angle = {actual_max_err_deg:.3e} deg");
    println!("actualized max error in rotation direction = {actual_max_uvec_err_deg:.3e} deg");
}
