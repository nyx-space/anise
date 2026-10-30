/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{PhysicsResult, orbit::INC_EPSILON_DEG, orbit::Orbit};

use crate::errors::{
    EquinoctialInclinationSnafu, InfiniteValueSnafu, ParabolicSemiParamSnafu,
    RadiusSnafu, VelocitySnafu,
};
use crate::math::{Vector3, angles::between_0_360};
use crate::prelude::Frame;

use core::f64::consts::PI;
use hifitime::Epoch;
use snafu::ensure;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyType;

pub(crate) fn walker_equinoctial_to_keplerian(
    sma_km: f64,
    h: f64,
    k: f64,
    p: f64,
    q: f64,
    lambda_deg: f64,
) -> (f64, f64, f64, f64, f64, f64) {
    let ecc = (h * h + k * k).sqrt();

    let s_sq: f64 = p * p + q * q; // sin^2(i/2)

    // Handle potential rounding errors > 1.0
    let inc_rad = if s_sq <= 1.0 {
        (1.0 - 2.0 * s_sq).acos()
    } else {
        (1.0 - 2.0_f64).acos() // Match C++ logic
    };
    let inc_deg = inc_rad.to_degrees();

    let raan_deg = p.atan2(q).to_degrees();
    let aop_plus_raan = h.atan2(k).to_degrees();

    let aop_deg = aop_plus_raan - raan_deg;
    let ma_deg = lambda_deg - aop_plus_raan;

    (sma_km, ecc, inc_deg, raan_deg, aop_deg, ma_deg)
}

/// Converts a Cartesian state into the Modified Equinoctial elements `[p_km, f, g, h, k, L_deg]`.
///
/// + `p_km`: the semi-latus rectum, in km: `p = a(1 - e²)`
/// + `f`: eccentricity vector (x-component): `f = e cos(ω + Ω)`
/// + `g`: eccentricity vector (y-component): `g = e sin(ω + Ω)`
/// + `h`: orbit plane tilt (x-component): `h = tan(i/2) cos(Ω)`
/// + `k`: orbit plane tilt (y-component): `k = tan(i/2) sin(Ω)`
/// + `true_lon_deg`: the true longitude, in degrees: `L = ω + Ω + ν`
///
/// This function uses the same formulas from FortranAstrodynamicsToolkit from Jacob Williams:
/// https://github.com/jacobwilliams/Fortran-Astrodynamics-Toolkit/blob/ab8d9c017835d3f52bd691835bd4e8ad10da8467/src/modified_equinoctial_module.f90#L55
pub(crate) fn cartesian_to_modified_equinoctial(
    radius_km: Vector3,
    velocity_km_s: Vector3,
    mu_km3_s2: f64,
) -> PhysicsResult<[f64; 6]> {
    let r_dot_v = radius_km.dot(&velocity_km_s);
    let r_hat = radius_km.normalize();
    let h_vec_km2_per_s = radius_km.cross(&velocity_km_s);
    let h_mag_km2_per_s = h_vec_km2_per_s.norm();
    ensure!(
        h_mag_km2_per_s > f64::EPSILON,
        VelocitySnafu {
            action: "cannot compute modified equinoctial elements with zero angular momentum"
        }
    );
    let h_hat = h_vec_km2_per_s.normalize();

    // Equinoctial orbital elements are degenerate near 180 degree inclination
    let inc_rad = h_hat[2].clamp(-1.0, 1.0).acos();
    ensure!(
        (PI - inc_rad).abs() > INC_EPSILON_DEG.to_radians(),
        EquinoctialInclinationSnafu {
            inclination_deg: inc_rad.to_degrees(),
            tolerance_deg: INC_EPSILON_DEG,
        }
    );

    // Transverse unit vector of the RTN frame
    let transverse_dir = (radius_km.norm() * velocity_km_s - r_dot_v * r_hat) / h_mag_km2_per_s;
    let p_km = h_mag_km2_per_s.powi(2) / mu_km3_s2;
    let k = h_hat[0] / (1.0 + h_hat[2]);
    let h = -h_hat[1] / (1.0 + h_hat[2]);

    // Unit vectors spanning the orbit plane.
    let sec2_half_inclination = 1.0 + h.powi(2) + k.powi(2);
    let f_hat = Vector3::new(
        1.0 - k.powi(2) + h.powi(2),
        2.0 * k * h,
        -2.0 * k
    ) / sec2_half_inclination;
    let g_hat = Vector3::new(
        2.0 * k * h,
        1.0 + k.powi(2) - h.powi(2),
        2.0 * h
    ) / sec2_half_inclination;

    let ecc_vec = velocity_km_s.cross(&h_vec_km2_per_s) / mu_km3_s2 - r_hat;
    let f = ecc_vec.dot(&f_hat);
    let g = ecc_vec.dot(&g_hat);
    let true_lon_rad = (r_hat[1] - transverse_dir[0]).atan2(r_hat[0] + transverse_dir[1]);

    Ok([p_km, f, g, h, k, between_0_360(true_lon_rad.to_degrees())])
}

/// Converts Modified Equinoctial elements into a Cartesian position (km) and velocity (km/s).
///
/// + `p_km`: the semi-latus rectum, in km: `p = a(1 - e²)`
/// + `f`: eccentricity vector (x-component): `f = e cos(ω + Ω)`
/// + `g`: eccentricity vector (y-component): `g = e sin(ω + Ω)`
/// + `h`: orbit plane tilt (x-component): `h = tan(i/2) cos(Ω)`
/// + `k`: orbit plane tilt (y-component): `k = tan(i/2) sin(Ω)`
/// + `true_lon_deg`: the true longitude, in degrees: `L = ω + Ω + ν`
/// + `mu_km3_s2`: the gravitational parameter of the central body, in km^3/s^2
///
/// Reference:
/// https://github.com/jacobwilliams/Fortran-Astrodynamics-Toolkit/blob/ab8d9c017835d3f52bd691835bd4e8ad10da8467/src/modified_equinoctial_module.f90#L105
#[allow(clippy::too_many_arguments)]
pub(crate) fn modified_equinoctial_to_cartesian(
    p_km: f64,
    f: f64,
    g: f64,
    h: f64,
    k: f64,
    true_lon_deg: f64,
    mu_km3_s2: f64,
) -> PhysicsResult<(Vector3, Vector3)> {
    ensure!(
        p_km > f64::EPSILON,
        RadiusSnafu { action: "semi-latus rectum must be positive" }
    );

    let (sin_l, cos_l) = true_lon_deg.to_radians().sin_cos();
    let w = 1.0 + f * cos_l + g * sin_l;
    let r_mag_km = p_km / w;

    // Bates, Mueller, and White - Fundamentals of Astrodynamics 2.5-4 pg73
    let velocity_scaling_km_per_s = (mu_km3_s2 / p_km).sqrt();

    // Unit vectors spanning the orbit plane.
    let sec2_half_inclination = 1.0 + h.powi(2) + k.powi(2);
    let f_hat = Vector3::new(
        1.0 - k.powi(2) + h.powi(2),
        2.0 * k * h,
        -2.0 * k
    ) / sec2_half_inclination;
    let g_hat = Vector3::new(
        2.0 * k * h,
        1.0 + k.powi(2) - h.powi(2),
        2.0 * h
    ) / sec2_half_inclination;

    let x = r_mag_km * cos_l;
    let y = r_mag_km * sin_l;
    let x_dot = -velocity_scaling_km_per_s * (g + sin_l);
    let y_dot = velocity_scaling_km_per_s * (f + cos_l);

    Ok((x * f_hat + y * g_hat, x_dot * f_hat + y_dot * g_hat))
}

/// A state in Modified Equinoctial elements, parameterized at an epoch and frame.
///
/// + `p_km`: the semi-latus rectum, in km: `p = a(1 - e²)` (must be strictly positive)
/// + `f`: eccentricity vector (x-component): `f = e cos(ω + Ω)`
/// + `g`: eccentricity vector (y-component): `g = e sin(ω + Ω)`
/// + `h`: orbit plane tilt (x-component): `h = tan(i/2) cos(Ω)`
/// + `k`: orbit plane tilt (y-component): `k = tan(i/2) sin(Ω)`
/// + `true_lon_deg`: the true longitude, in degrees: `L = ω + Ω + ν`
///
/// NOTE: Modified equinoctial elements are singular for 180deg inclination
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ModifiedEquinoctialState {
    pub p_km: f64,
    pub f: f64,
    pub g: f64,
    pub h: f64,
    pub k: f64,
    pub true_lon_deg: f64,
    pub epoch: Epoch,
    pub frame: Frame,
}

impl ModifiedEquinoctialState {
    /// Attempts to construct a [ModifiedEquinoctialState] with range validation.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        p_km: f64,
        f: f64,
        g: f64,
        h: f64,
        k: f64,
        true_lon_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        ensure!(
            [p_km, f, g, h, k, true_lon_deg].iter().all(|v| v.is_finite()),
            InfiniteValueSnafu {
                action: "construct ModifiedEquinoctialState"
            }
        );

        ensure!(p_km > f64::EPSILON, ParabolicSemiParamSnafu { p_km });

        // Modified eoe are invalid at 180deg inclination since tan(180/2) = inf
        let inc_rad = 2.0 * (h.powi(2) + k.powi(2)).sqrt().atan();
        ensure!(
            (PI - inc_rad).abs() > INC_EPSILON_DEG.to_radians(),
            EquinoctialInclinationSnafu {
                inclination_deg: inc_rad.to_degrees(),
                tolerance_deg: INC_EPSILON_DEG,
            }
        );

        Ok(Self {
            p_km,
            f,
            g,
            h,
            k,
            true_lon_deg,
            epoch,
            frame,
        })
    }
}

impl Orbit {
    /// Attempts to create a new Orbit from the Walker equinoctial orbital elements
    ///
    /// + `sma_km`: the semi-major axis, in km: `a = (r_apo + r_peri) / 2`
    /// + `h`: eccentricity vector (y-component): `h = e sin(ω + Ω)`
    /// + `k`: eccentricity vector (x-component): `k = e cos(ω + Ω)`
    /// + `p`: orbit plane tilt (y-component): `p = sin(i/2) sin(Ω)`
    /// + `q`: orbit plane tilt (x-component): `q = sin(i/2) cos(Ω)`
    /// + `lambda_deg`: the mean longitude, in degrees: `λ = M + ω + Ω`
    /// NOTE: It routes through the Keplerian elements, so it is singular for circular
    /// and equatorial orbits.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::neg_multiply)]
    pub fn try_from_walker_equinoctial(
        sma_km: f64,
        h: f64,
        k: f64,
        p: f64,
        q: f64,
        lambda_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        let (sma_km, ecc, inc_deg, raan_deg, aop_deg, ma_deg) =
            walker_equinoctial_to_keplerian(sma_km, h, k, p, q, lambda_deg);

        Self::try_keplerian_mean_anomaly(
            sma_km, ecc, inc_deg, raan_deg, aop_deg, ma_deg, epoch, frame,
        )
    }

    /// Attempts to create a new Orbit from the Modified Equinoctial orbital elements.
    ///
    /// + `semi_parameter_km`: the semi-latus rectum, in km: `p = a(1 - e²)`
    /// + `f`: eccentricity vector (x-component): `f = e cos(ω + Ω)`
    /// + `g`: eccentricity vector (y-component): `g = e sin(ω + Ω)`
    /// + `h`: orbit plane tilt (x-component): `h = tan(i/2) cos(Ω)`
    /// + `k`: orbit plane tilt (y-component): `k = tan(i/2) sin(Ω)`
    /// + `true_longitude_deg`: the true longitude, in degrees: `L = ω + Ω + ν`
    #[allow(clippy::too_many_arguments)]
    pub fn try_from_modified_equinoctial(
        semi_parameter_km: f64,
        f: f64,
        g: f64,
        h: f64,
        k: f64,
        true_longitude_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        let (radius_km, velocity_km_s) = modified_equinoctial_to_cartesian(
            semi_parameter_km,
            f,
            g,
            h,
            k,
            true_longitude_deg,
            frame.mu_km3_s2()?,
        )?;

        Ok(Self {
            radius_km,
            velocity_km_s,
            epoch,
            frame,
        })
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Orbit {
    /// Returns the Walker equinoctial semi-major axis (a) in km.
    ///
    /// :rtype: float
    pub fn walker_equinoctial_a_km(&self) -> PhysicsResult<f64> {
        self.sma_km()
    }

    /// Returns the Walker equinoctial element h (ecc * sin(aop + raan)).
    ///
    /// :rtype: float
    pub fn walker_equinoctial_h(&self) -> PhysicsResult<f64> {
        Ok(self.ecc()? * (self.aop_deg()?.to_radians() + self.raan_deg()?.to_radians()).sin())
    }

    /// Returns the Walker equinoctial element k (ecc * cos(aop + raan)).
    ///
    /// :rtype: float
    pub fn walker_equinoctial_k(&self) -> PhysicsResult<f64> {
        Ok(self.ecc()? * (self.aop_deg()?.to_radians() + self.raan_deg()?.to_radians()).cos())
    }

    /// Returns the Walker equinoctial element p (sin(inc/2) * sin(raan)).
    ///
    /// :rtype: float
    pub fn walker_equinoctial_p(&self) -> PhysicsResult<f64> {
        Ok((self.inc_deg()?.to_radians() / 2.0).sin() * self.raan_deg()?.to_radians().sin())
    }

    /// Returns the Walker equinoctial element q (sin(inc/2) * cos(raan)).
    ///
    /// :rtype: float
    pub fn walker_equinoctial_q(&self) -> PhysicsResult<f64> {
        Ok((self.inc_deg()?.to_radians() / 2.0).sin() * self.raan_deg()?.to_radians().cos())
    }

    /// Returns the Walker equinoctial mean longitude (lambda = raan + aop + ma) in degrees.
    ///
    /// :rtype: float
    pub fn walker_equinoctial_lambda_mean_deg(&self) -> PhysicsResult<f64> {
        // C++ version `aeq[5]=kep[3]+kep[4]+kep[5]` sums degrees.
        Ok(self.raan_deg()? + self.aop_deg()? + self.ma_deg()?)
    }

    /// Returns Walker equinoctial elements in order as a vector.
    ///
    /// + `sma`: the semi-major axis, in km: `a = (r_apo + r_peri) / 2`
    /// + `h`: eccentricity vector (y-component): `h = e sin(ω + Ω)`
    /// + `k`: eccentricity vector (x-component): `k = e cos(ω + Ω)`
    /// + `p`: orbit plane tilt (y-component): `p = sin(i/2) sin(Ω)` (bounded at i = 180°)
    /// + `q`: orbit plane tilt (x-component): `q = sin(i/2) cos(Ω)` (bounded at i = 180°)
    /// + `lambda`: the mean longitude, in degrees: `λ = M + ω + Ω`
    ///
    /// These are derived from the Keplerian elements, so this returns an error for an exactly
    /// circular orbit; use [Orbit::modified_equinoctial_elements] there.
    ///
    /// :rtype: list[float]
    pub fn walker_equinoctial_elements(&self) -> PhysicsResult<Vec<f64>> {
        Ok(vec![
            self.walker_equinoctial_a_km()?,
            self.walker_equinoctial_h()?,
            self.walker_equinoctial_k()?,
            self.walker_equinoctial_p()?,
            self.walker_equinoctial_q()?,
            self.walker_equinoctial_lambda_mean_deg()?,
        ])
    }

    /// Returns the six Modified Equinoctial elements, computed directly from the Cartesian state
    /// so that they stay well conditioned for circular and equatorial orbits.
    ///
    /// + `p`: the semi-latus rectum, in km: `p = a(1 - e²)` (finite for a parabolic orbit)
    /// + `f`: eccentricity vector (x-component): `f = e cos(ω + Ω)`
    /// + `g`: eccentricity vector (y-component): `g = e sin(ω + Ω)`
    /// + `h`: orbit plane tilt (x-component): `h = tan(i/2) cos(Ω)` (diverges at i = 180°)
    /// + `k`: orbit plane tilt (y-component): `k = tan(i/2) sin(Ω)` (diverges at i = 180°)
    /// + `L`: the true longitude, in degrees: `L = ω + Ω + ν`
    ///
    /// :rtype: list[float]
    pub fn modified_equinoctial_elements(&self) -> PhysicsResult<Vec<f64>> {
        Ok(cartesian_to_modified_equinoctial(
            self.radius_km,
            self.velocity_km_s,
            self.frame.mu_km3_s2()?,
        )?
        .to_vec())
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl Orbit {
    /// Python bind for [Orbit::try_from_walker_equinoctial].
    #[cfg(feature = "python")]
    #[classmethod]
    #[allow(clippy::too_many_arguments)]
    pub fn from_walker_equinoctial(
        _cls: &Bound<'_, PyType>,
        sma_km: f64,
        h: f64,
        k: f64,
        p: f64,
        q: f64,
        lambda_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_from_walker_equinoctial(sma_km, h, k, p, q, lambda_deg, epoch, frame)
    }

    /// Python bind for [Orbit::try_from_modified_equinoctial].
    #[cfg(feature = "python")]
    #[classmethod]
    #[allow(clippy::too_many_arguments)]
    pub fn from_modified_equinoctial(
        _cls: &Bound<'_, PyType>,
        semi_parameter_km: f64,
        f: f64,
        g: f64,
        h: f64,
        k: f64,
        true_longitude_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_from_modified_equinoctial(
            semi_parameter_km,
            f,
            g,
            h,
            k,
            true_longitude_deg,
            epoch,
            frame,
        )
    }
}
