/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use anise::constants::frames::SUN_J2000;
use anise::ephemerides::ephemeris::Ephemeris;
use anise::math::Vector6;
use anise::naif::daf::DafDataType;
use anise::prelude::*;
use hifitime::{Epoch, TimeUnits};

#[test]
fn test_spk_type8_roundtrip() {
    let _ = pretty_env_logger::try_init();

    // 1. Create an Ephemeris with equal time steps
    let mut ephem = Ephemeris::new("Type8Test".to_string());
    ephem.degree = 3;
    // Set to something else first to ensure to_spice_bsp uses what we request
    ephem.interpolation = DafDataType::Type13HermiteUnequalStep;

    let t0 = Epoch::from_et_seconds(0.0);
    let h = 60.0;
    let num_records = 100;

    for i in 0..num_records {
        let t = (i as f64) * h;
        let epoch = t0 + t.seconds();
        // Constant velocity motion: x = 100 + 1*t
        let orbit = Orbit::from_cartesian_pos_vel(
            Vector6::new(100.0 + t, 200.0, 300.0, 1.0, 0.0, 0.0),
            epoch,
            SUN_J2000,
        );
        ephem.insert_orbit(orbit);
    }

    // 2. Export to SPK Type 8
    let spk = ephem
        .to_spice_bsp(-999, Some(DafDataType::Type8LagrangeEqualStep))
        .expect("Failed to export to SPK Type 8");

    // 3. Load into an Almanac
    let almanac = Almanac::default().with_spk(spk);

    // 4. Verify accuracy at various points
    for i in 0..(num_records - 1) {
        let t_mid = (i as f64 + 0.5) * h;
        let epoch_mid = t0 + t_mid.seconds();

        let state = almanac.state_of(-999, SUN_J2000, epoch_mid, None).expect("Failed to query state");

        // Since it's constant velocity, Lagrange interpolation of any degree >= 1 should be exact (within precision)
        assert!((state.radius_km.x - (100.0 + t_mid)).abs() < 1e-10, "X position mismatch at t={t_mid}: got {}, exp {}", state.radius_km.x, 100.0 + t_mid);
        assert!((state.radius_km.y - 200.0).abs() < 1e-10);
        assert!((state.velocity_km_s.x - 1.0).abs() < 1e-10);
    }

    // Test boundaries
    let state_start = almanac.state_of(-999, SUN_J2000, t0, None).expect("Failed to query start state");
    assert!((state_start.radius_km.x - 100.0).abs() < 1e-10);

    let t_end = (num_records as f64 - 1.0) * h;
    let state_end = almanac.state_of(-999, SUN_J2000, t0 + t_end.seconds(), None).expect("Failed to query end state");
    assert!((state_end.radius_km.x - (100.0 + t_end)).abs() < 1e-10);
}
