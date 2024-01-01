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

use crate::{
    constants::SPEED_OF_LIGHT_KM_S,
    errors::{AberrationSnafu, VelocitySnafu},
    math::{rotate_vector, Vector3},
};

#[cfg(feature = "python")]
use pyo3::prelude::*;
use snafu::ensure;

use super::PhysicsResult;

/// Defines the available aberration corrections.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise."))]
pub enum Aberration {
    NoCorrection,
    LightTime,
    ConvergedLightTime,
    LightTimeStellar,
    ConvergedLightTimeStellar,
    TxLightTime,
    TxConvergedLightTime,
    TxLightTimeStellar,
    TxConvergedLightTimeStellar,
}

#[cfg_attr(feature = "python", pymethods)]
impl Aberration {
    /// Returns whether this Aberration setting uses a Newtonian convergence criteria.
    pub const fn is_converged(&self) -> bool {
        matches!(
            self,
            Self::ConvergedLightTime
                | Self::ConvergedLightTimeStellar
                | Self::TxConvergedLightTime
                | Self::TxConvergedLightTimeStellar
        )
    }

    /// Returns whether this Aberration setting computes the transmittion case.
    pub const fn is_transmit(&self) -> bool {
        matches!(
            self,
            Self::TxLightTime
                | Self::TxConvergedLightTime
                | Self::TxLightTimeStellar
                | Self::TxConvergedLightTimeStellar
        )
    }

    /// Returns whether this Aberration setting computes stellar corrections.
    pub const fn has_stellar(&self) -> bool {
        matches!(
            self,
            Self::LightTimeStellar
                | Self::ConvergedLightTimeStellar
                | Self::TxLightTimeStellar
                | Self::TxConvergedLightTimeStellar
        )
    }
}

/// Returns the provided target [Orbit] with respect to any observer corrected for steller aberration.
///
/// # Arguments
///
/// + `target_pos_km`: the position of a target object with respect to the observer in kilometers
/// + `obs_wrt_ssb_vel_km_s`: the velocity of the observer with respect to the Solar System Barycenter in kilometers per second
/// + `ab_corr`: the [Aberration] correction
///
/// # Errors
///
/// This function will return an error in the following cases:
/// 1. the aberration is not set to include stellar corrections;
/// 1. the `target` is moving faster than the speed of light.
///
/// # Algorithm
/// Source: this algorithm and documentation were rewritten from NAIF's [`stelab`](https://github.com/nasa/kepler-pipeline/blob/f58b21df2c82969d8bd3e26a269bd7f5b9a770e1/source-code/matlab/fc/cspice-src-i686/cspice/stelab.c#L13) function:
///
/// Let r be the vector from the observer to the object, and v be the velocity of the observer with respect to the Solar System barycenter.
/// Let w be the angle between them. The aberration angle phi is given by
///
/// `sin(phi) = v sin(w) / c`
///
/// Let h be the vector given by the cross product
///
/// `h = r X v`
///
/// Rotate r by phi radians about h to obtain the apparent position of the object.
///
///
pub fn stellar_aberration(
    target_pos_km: Vector3,
    obs_wrt_ssb_vel_km_s: Vector3,
    ab_corr: Aberration,
) -> PhysicsResult<Vector3> {
    ensure!(
        ab_corr.has_stellar(),
        AberrationSnafu {
            action: "stellar correction not available for this aberration"
        }
    );

    // Obtain the negative of the observer's velocity. This velocity, combined
    // with the target's position, will yield the inverse of the usual stellar
    // aberration correction, which is exactly what we seek.

    let obs_velocity_km_s = if ab_corr.is_transmit() {
        -obs_wrt_ssb_vel_km_s
    } else {
        obs_wrt_ssb_vel_km_s
    };

    // Get a unit vector that points in the direction of the object (u_obj)
    let u = target_pos_km.normalize();
    // Get the velocity vector scaled with respect to the speed of light (v/c)
    let onebyc = 1.0 / SPEED_OF_LIGHT_KM_S;
    let vbyc = onebyc * obs_velocity_km_s;

    ensure!(
        vbyc.dot(&vbyc) < 1.0,
        VelocitySnafu {
            action: "observer moving faster than light, cannot compute stellar aberration"
        }
    );

    // Compute u_obj x (v/c)
    let h = u.cross(&vbyc);

    // Correct for stellar aberration
    let mut app_target_pos_km = target_pos_km;
    let sin_phi = h.norm().abs();
    if sin_phi > EPSILON {
        let phi = sin_phi.asin();
        app_target_pos_km = rotate_vector(&target_pos_km, &h, phi);
    }

    Ok(app_target_pos_km)
}
