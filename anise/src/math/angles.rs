/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

/// Returns the provided angle bounded between 0.0 and 360.0.
///
/// This function takes an angle (in degrees) and normalizes it to the range [0, 360).
/// If the angle is negative, it will be converted to a positive angle in the equivalent position.
/// For example, an angle of -90 degrees will be converted to 270 degrees.
///
/// # Arguments
///
/// * `angle` - An angle in degrees.
///
pub fn between_0_360(angle: f64) -> f64 {
    angle.rem_euclid(360.0)
}

/// Returns the provided angle bounded between -180.0 and +180.0
pub fn between_pm_180(angle: f64) -> f64 {
    between_pm_x(angle, 180.0)
}

/// Returns the provided angle bounded between -x and +x.
///
/// This function takes an angle (in degrees) and normalizes it to the range [-x, x).
/// If the angle is outside this range, it will be converted to an equivalent angle within this range.
/// For example, if x is 180, an angle of 270 degrees will be converted to -90 degrees.
///
/// # Arguments
///
/// * `angle` - An angle in degrees.
/// * `x` - The boundary for the angle normalization.
pub fn between_pm_x(angle: f64, x: f64) -> f64 {
    let mut bounded = angle.rem_euclid(2.0 * x);
    if bounded >= x {
        bounded -= 2.0 * x;
    }
    bounded
}
