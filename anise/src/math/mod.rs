/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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

use nalgebra::allocator::Allocator;
use nalgebra::{DefaultAllocator, DimName, OVector};

/// Returns the root sum squared (RSS) between two vectors of any dimension N.
pub fn root_sum_squared<N: DimName>(vec_a: &OVector<f64, N>, vec_b: &OVector<f64, N>) -> f64
where
    DefaultAllocator: Allocator<N>,
{
    vec_a
        .iter()
        .zip(vec_b.iter())
        .map(|(&x, &y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// Returns the root mean squared (RSS) between two vectors of any dimension N.
pub fn root_mean_squared<N: DimName>(vec_a: &OVector<f64, N>, vec_b: &OVector<f64, N>) -> f64
where
    DefaultAllocator: Allocator<N>,
{
    let sum_of_squares = vec_a
        .iter()
        .zip(vec_b.iter())
        .map(|(&x, &y)| (x - y).powi(2))
        .sum::<f64>();

    let mean_of_squares = sum_of_squares / vec_a.len() as f64;
    mean_of_squares.sqrt()
}

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
pub fn rotate_vector(a: &Vector3, axis: &Vector3, theta_rad: f64) -> Vector3 {
    let k_hat = axis.normalize();
    a.scale(theta_rad.cos())
        + k_hat.cross(a).scale(theta_rad.sin())
        + k_hat.scale(k_hat.dot(a) * (1.0 - theta_rad.cos()))
}

#[cfg(test)]
mod math_ut {
    use super::{rotate_vector, Vector3};
    #[test]
    fn test_rotate_vector() {
        use approx::assert_abs_diff_eq;
        let a = Vector3::new(1.0, 0.0, 0.0);
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let theta_rad = std::f64::consts::PI / 2.0;
        let result = rotate_vector(&a, &axis, theta_rad);
        assert_abs_diff_eq!(result, Vector3::new(0.0, 1.0, 0.0), epsilon = 1e-7);
    }
}
