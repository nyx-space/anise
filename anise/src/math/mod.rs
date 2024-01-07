/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

// Vector3 is nalgebra's Vector3 with a 64-bit floating point representation.
pub type Vector3 = nalgebra::Vector3<f64>;
pub type Vector4 = nalgebra::Vector4<f64>;
pub type Vector6 = nalgebra::Vector6<f64>;
pub type Matrix3 = nalgebra::Matrix3<f64>;
pub type Matrix6 = nalgebra::Matrix6<f64>;

pub mod angles;
pub mod cartesian;
#[cfg(feature = "python")]
mod cartesian_py;
pub mod interpolation;
pub mod rotation;
pub mod units;

/// Returns the projection of a onto b
/// Converted from NAIF SPICE's `projv`
pub fn project_vector(a: &Vector3, b: &Vector3) -> Vector3 {
    b * a.dot(b) / b.dot(b)
}

/// Returns the components of vector a orthogonal to b
/// Converted from NAIF SPICE's `prepv`
pub fn perp_vector(a: &Vector3, b: &Vector3) -> Vector3 {
    let big_a = a[0].abs().max(a[1].abs().max(a[2].abs()));
    let big_b = b[0].abs().max(b[1].abs().max(b[2].abs()));
    if big_a < f64::EPSILON {
        Vector3::zeros()
    } else if big_b < f64::EPSILON {
        *a
    } else {
        let a_scl = a / big_a;
        let b_scl = b / big_b;
        let v = project_vector(&a_scl, &b_scl);
        big_a * (a_scl - v)
    }
}

/// Rotate the vector a around the provided axis by angle theta.
/// Converted from NAIF SPICE's `vrotv`
pub fn rotate_vector(a: &Vector3, axis: &Vector3, theta_rad: f64) -> Vector3 {
    // Compute the unit vector that lies in the direction of the AXIS.
    let x = axis.normalize();

    // Compute the projection of V onto AXIS.
    let p = project_vector(a, &x);

    // Compute the component of V orthogonal to the AXIS.
    let v1 = a - p;

    // Rotate V1 by 90 degrees about the AXIS and call the result V2.
    let v2 = a.cross(&v1);

    // Compute COS(THETA)*V1 + SIN(THETA)*V2. This is V1 rotated about the AXIS in the plane normal to the axis.
    let r_plane = v1 * theta_rad.cos() + v2 * theta_rad.sin();

    // Add the rotated component in the normal plane to AXIS to the projection of V onto AXIS (P) to obtain R.
    r_plane + p
}
