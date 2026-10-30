/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use anise::constants::frames::{EARTH_ICRS, IAU_EARTH_FRAME, IAU_MOON_FRAME, MOON_ICRS};
use anise::math::cartesian::CartesianState;
use anise::prelude::{Almanac, LengthUnit, TimeUnit};
use hifitime::Epoch;
use std::path::PathBuf;

/// Regression for #911: translation in J2000 axes must not be combined with
/// components in a rotating input frame before the initial rotation.
#[test]
fn provided_state_transform_matches_cartesian_entry_point() {
    let data_dir = std::env::var_os("ANISE_911_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data"));
    let almanac = Almanac::new(&data_dir.join("de440s.bsp").to_string_lossy())
        .unwrap()
        .load(&data_dir.join("pck08.pca").to_string_lossy())
        .unwrap();

    for epoch in [
        Epoch::from_gregorian_utc_at_noon(2025, 1, 1),
        Epoch::from_gregorian_utc_at_noon(2026, 6, 1),
    ] {
        for source in [IAU_MOON_FRAME, MOON_ICRS] {
            for observer in [IAU_EARTH_FRAME, EARTH_ICRS] {
                let state =
                    CartesianState::new(1200.0, 2400.0, -550.0, 0.1, -0.2, 0.3, epoch, source);
                let expected = almanac.transform_to(state, observer, None).unwrap();

                for (distance_unit, time_unit, distance_scale, time_scale) in [
                    (LengthUnit::Kilometer, TimeUnit::Second, 1.0, 1.0),
                    (LengthUnit::Meter, TimeUnit::Second, 1000.0, 1.0),
                    (LengthUnit::Meter, TimeUnit::Minute, 1000.0, 60.0),
                ] {
                    let actual = almanac
                        .transform_state_to(
                            state.radius_km * distance_scale,
                            state.velocity_km_s * distance_scale * time_scale,
                            source,
                            observer,
                            epoch,
                            None,
                            distance_unit,
                            time_unit,
                        )
                        .unwrap();
                    let pos_error_km = expected.rss_radius_km(&actual).unwrap();
                    let vel_error_km_s = expected.rss_velocity_km_s(&actual).unwrap();
                    assert!(
                        pos_error_km < 1e-8,
                        "source={source:?} observer={observer:?} epoch={epoch} units={distance_unit}/{time_unit:?} position error={pos_error_km} km"
                    );
                    assert!(
                        vel_error_km_s < 1e-11,
                        "source={source:?} observer={observer:?} epoch={epoch} units={distance_unit}/{time_unit:?} velocity error={vel_error_km_s} km/s"
                    );
                }
            }
        }
    }
}
