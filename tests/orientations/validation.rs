/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::{
    math::{
        rotation::{Quaternion, DCM},
        Matrix3,
    },
    naif::kpl::parser::convert_tpc,
    prelude::{Almanac, Frame},
};
use hifitime::{Duration, Epoch, TimeSeries, TimeUnits};

// Allow up to one arcsecond of error
const MAX_ERR_DEG: f64 = 3.6e-6;
const DCM_EPSILON: f64 = 1e-10;

/// This test converts the PCK file into its ANISE equivalent format, loads it into an Almanac, and compares the rotations computed by the Almanac and by SPICE
/// It only check the IAU Earth rotation to its J2000 parent, which does not account for nutation and precession coefficients.
#[test]
fn pck00008_iau_earth_validation() {
    let pck = "data/pck00008.tpc";
    spice::furnsh(pck);
    let planetary_data = convert_tpc(pck, "data/gm_de431.tpc").unwrap();

    let mut almanac = Almanac::default();
    almanac.planetary_data = planetary_data;

    let iau_earth = Frame::from_ephem_orient(399, 399);

    for (num, epoch) in TimeSeries::inclusive(
        Epoch::from_tdb_duration(Duration::ZERO),
        Epoch::from_tdb_duration(0.2.centuries()),
        1.days(),
    )
    .enumerate()
    {
        let rot_data = spice::pxform("J2000", "IAU_EARTH", epoch.to_tdb_seconds());
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

        let dcm = almanac.rotation_to_parent(iau_earth, epoch).unwrap();

        let spice_dcm = DCM {
            rot_mat: spice_mat,
            from: dcm.from,
            to: dcm.to,
            rot_mat_dt: None,
        };

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
            "#{num} @ {epoch} unit vector angle error"
        );
        assert!(
            deg_err.abs() < MAX_ERR_DEG,
            "#{num} @ {epoch} rotation error"
        );

        assert!(
            (dcm.rot_mat - spice_mat).norm() < DCM_EPSILON,
            "#{num} {epoch}\ngot: {}want:{spice_mat}err: {:.3e}",
            dcm.rot_mat,
            (dcm.rot_mat - spice_mat).norm()
        );
    }
}
