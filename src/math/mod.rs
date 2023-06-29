/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

// Vector3 is nalgebra's Vector3 with a 64-bit floating point representation.
pub type Vector3 = nalgebra::Vector3<f64>;
pub type Vector6 = nalgebra::Vector6<f64>;
pub type Matrix3 = nalgebra::Matrix3<f64>;
pub type Matrix6 = nalgebra::Matrix3<f64>;

pub mod angles;
pub mod cartesian;
pub mod interpolation;
pub mod polyfit;
pub mod rotation;
pub mod units;
pub mod utils;

/// Returns the projection of a onto b
pub fn projv(a: &Vector3, b: &Vector3) -> Vector3 {
    b * a.dot(b) / b.dot(b)
}

/// Returns the components of vector a orthogonal to b
pub fn perpv(a: &Vector3, b: &Vector3) -> Vector3 {
    let big_a = a[0].abs().max(a[1].abs().max(a[2].abs()));
    let big_b = b[0].abs().max(b[1].abs().max(b[2].abs()));
    if big_a < f64::EPSILON {
        Vector3::zeros()
    } else if big_b < f64::EPSILON {
        *a
    } else {
        let a_scl = a / big_a;
        let b_scl = b / big_b;
        let v = projv(&a_scl, &b_scl);
        big_a * (a_scl - v)
    }
}
