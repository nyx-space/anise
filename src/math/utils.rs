/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::f64::EPSILON;

/// Returns the absolute difference between two floats as per the approx crate
pub fn abs_diff(a: f64, b: f64) -> f64 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

pub fn rel_diff(a: f64, b: f64, max_relative: f64) -> f64 {
    if a == b {
        return 0.0;
    }

    // Handle remaining infinities
    if a.is_infinite() || b.is_infinite() {
        // We are far from equal so return a big number
        return f64::INFINITY;
    }

    let abs_diff = (a - b).abs();

    // For when the numbers are really close together
    if abs_diff <= EPSILON {
        return abs_diff;
    }

    let abs_a = a.abs();
    let abs_b = b.abs();

    let largest = if abs_b > abs_a { abs_b } else { abs_a };

    // Use a relative difference comparison
    largest * max_relative
}

// Normalize between -1.0 and 1.0
pub fn normalize(x: f64, min_x: f64, max_x: f64) -> f64 {
    2.0 * (x - min_x) / (max_x - min_x) - 1.0
}

// Denormalize between -1.0 and 1.0
pub fn denormalize(xp: f64, min_x: f64, max_x: f64) -> f64 {
    (max_x - min_x) * (xp + 1.0) / 2.0 + min_x
}
