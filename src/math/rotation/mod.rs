/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

/// The smallest difference between two radians is set to one arcsecond.
pub(crate) const EPSILON_RAD: f64 = 4.8e-6;
/// Equality of f64 for rotations
pub(crate) const EPSILON: f64 = 1e-12;

mod dcm;
mod mrp;
mod quaternion;
pub use dcm::DCM;
pub use mrp::MRP;
pub use quaternion::Quaternion;

pub trait Rotation: TryInto<Quaternion> {}

/// Generates the angles for the test
#[cfg(test)]
pub(crate) fn generate_angles() -> Vec<f64> {
    use core::f64::consts::TAU;
    let mut angles = Vec::new();
    let mut angle = -TAU;
    loop {
        angles.push(angle);
        angle += 0.01 * TAU;
        if angle > TAU {
            break;
        }
    }
    angles
}

#[cfg(test)]
use super::Vector3;
/// Returns whether two vectors can be considered equal after a rotation
#[cfg(test)]
pub(crate) fn vec3_eq(a: Vector3, b: Vector3, msg: String) {
    let rslt = (a - b).norm();
    assert!(rslt < 1e-3, "{msg}:{rslt:.e}\ta = {a}\tb = {b}")
}
