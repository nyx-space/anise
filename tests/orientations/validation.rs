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
    math::Matrix3,
    naif::kpl::parser::convert_tpc,
    prelude::{Almanac, Frame},
};
use hifitime::{Duration, Epoch, TimeSeries, TimeUnits};

/// This test converts the PCK file into its ANISE equivalent format, loads it into an Almanac, and compares the rotations computed by the Almanac and by SPICE
#[test]
fn pck00008_validation() {
    let pck = "data/pck00008.tpc";
    spice::furnsh(pck);
    let planetary_data = convert_tpc(pck, "data/gm_de431.tpc").unwrap();

    let mut almanac = Almanac::default();
    almanac.planetary_data = planetary_data;

    for epoch in TimeSeries::inclusive(
        Epoch::from_tdb_duration(Duration::ZERO),
        Epoch::from_tdb_duration(0.2.centuries()),
        1.days(),
    ) {
        let rot_data = spice::pxform("J2000", "IAU_EARTH", epoch.to_tdb_seconds());
        // Confirmed that the M3x3 below is the correct representation from SPICE by using the mxv spice function and compare that to the nalgebra equivalent computation.
        let spice_dcm = Matrix3::new(
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

        let dcm = almanac
            .rotation_to_parent(Frame::from_ephem_orient(399, 399), epoch)
            .unwrap();

        assert!(
            (dcm.rot_mat - spice_dcm).norm() < EPSILON,
            "{epoch}\ngot: {}want:{spice_dcm}err: {}",
            dcm.rot_mat,
            dcm.rot_mat - spice_dcm
        );
    }
}
