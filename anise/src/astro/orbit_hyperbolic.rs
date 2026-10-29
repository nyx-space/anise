/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::PhysicsResult;

use crate::{
    astro::orbit::Orbit,
    errors::{InfiniteValueSnafu, NotHyperbolicSnafu, PhysicsError},
    math::{
        Vector3,
        angles::{between_0_360, between_pm_180},
    },
    prelude::Frame,
};

use hifitime::{Duration, Epoch, TimeUnits};
use snafu::ensure;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python", pymethods)]
impl Orbit {
    /// Returns the hyperbolic anomaly in degrees between 0 and 360.0
    /// Returns an error if the orbit is not hyperbolic.
    ///
    /// :rtype: float
    pub fn hyperbolic_anomaly_deg(&self) -> PhysicsResult<f64> {
        let ecc = self.ecc()?;
        if ecc <= 1.0 {
            Err(PhysicsError::NotHyperbolic { ecc })
        } else {
            let (sin_ta, cos_ta) = self.ta_deg()?.to_radians().sin_cos();
            let sinh_h = (sin_ta * (ecc.powi(2) - 1.0).sqrt()) / (1.0 + ecc * cos_ta);
            Ok(between_0_360(sinh_h.asinh().to_degrees()))
        }
    }

    /// Computes the planetary targeting B-Plane (Body Plane) for a hyperbolic trajectory. Refer to [BPlane] for details.
    /// :rtype: BPlane
    pub fn b_plane(&self) -> PhysicsResult<BPlane> {
        BPlane::new(*self)
    }

    /// Component of the impact parameter vector along the $\hat{\mathbf{R}}$ axis ($\mathbf{B} \cdot \hat{\mathbf{R}}$), in kilometers.
    /// Negative values indicate a flyby over the northern hemisphere relative to the frame's $+Z$ pole.
    /// :rtype: float
    pub fn b_r_km(&self) -> PhysicsResult<f64> {
        Ok(BPlane::new(*self)?.b_r_km)
    }

    /// Component of the impact parameter vector along the $\hat{\mathbf{T}}$ axis ($\mathbf{B} \cdot \hat{\mathbf{T}}$), in kilometers.
    /// Positive values indicate a prograde pass; negative values indicate a retrograde pass.
    /// :rtype: float
    pub fn b_t_km(&self) -> PhysicsResult<f64> {
        Ok(BPlane::new(*self)?.b_t_km)
    }

    /// Linearized Time of Flight (LTOF) along the incoming asymptote to B-Plane crossing:
    /// $\text{LTOF} = -(\mathbf{r} \cdot \hat{\mathbf{S}}) / v_\infty$.
    /// Positive when the spacecraft is inbound (prior to B-Plane crossing) and negative after crossing.
    /// :rtype: time.Duration
    pub fn b_ltof(&self) -> PhysicsResult<Duration> {
        Ok(BPlane::new(*self)?.ltof)
    }

    /// Returns the B plane angle in degrees between -180 and 180
    /// :rtype: float
    pub fn b_plane_angle_deg(&self) -> PhysicsResult<f64> {
        Ok(BPlane::new(*self)?.angle_deg())
    }

    /// Returns the B plane vector magnitude, in kilometers
    /// :rtype: float
    pub fn b_plane_mag_km(&self) -> PhysicsResult<f64> {
        Ok(BPlane::new(*self)?.magnitude_km())
    }
}

/// Defines a planetary targeting B-Plane (Body Plane) for a hyperbolic trajectory.
///
/// The B-Plane is an analytical target frame attached to the
/// central body's center of mass and oriented perpendicular to the incoming hyperbolic asymptote.
/// It linearizes the nonlinear two-body gravity well at the sphere of influence: the impact parameter
/// vector $\mathbf{B}$ points from the central body's focus to the point where the unperturbed incoming
/// asymptote pierces the B-Plane.
///
/// # Reference Triad ($\hat{\mathbf{S}}, \hat{\mathbf{T}}, \hat{\mathbf{R}}$)
///
/// Given a hyperbolic `Orbit` with eccentricity vector $\mathbf{e} = e \hat{\mathbf{e}}$, angular
/// momentum unit vector $\hat{\mathbf{h}}$, semi-minor axis $b = -a \sqrt{e^2 - 1}$, and in-plane
/// normal $\hat{\mathbf{n}} = \hat{\mathbf{h}} \times \hat{\mathbf{e}}$, the orthonormal right-handed
/// B-Plane triad is constructed relative to the frame's reference pole $\hat{\mathbf{k}} = [0, 0, 1]^T$:
///
/// 1. **Incoming Asymptote ($\hat{\mathbf{S}}$):** Unit vector parallel to the incoming hyperbolic
///    excess velocity $\mathbf{v}_{\infty}^{-}$, normal to the B-Plane:
///    $$\hat{\mathbf{S}} = \frac{1}{e}\hat{\mathbf{e}} + \sqrt{1 - \frac{1}{e^2}}\hat{\mathbf{n}}$$
/// 2. **Transverse Axis ($\hat{\mathbf{T}}$):** Lies in the reference equatorial plane (orthogonal to
///    both $\hat{\mathbf{S}}$ and the reference pole $\hat{\mathbf{k}} = [0, 0, 1]^T$), pointing eastward:
///    $$\hat{\mathbf{T}} = \frac{\hat{\mathbf{S}} \times \hat{\mathbf{k}}}{\|\hat{\mathbf{S}} \times \hat{\mathbf{k}}\|}$$
/// 3. **Radial / Meridian Axis ($\hat{\mathbf{R}}$):** Completes the right-handed triad
///    ($\hat{\mathbf{S}} \times \hat{\mathbf{T}} = \hat{\mathbf{R}}$):
///    $$\hat{\mathbf{R}} = \hat{\mathbf{S}} \times \hat{\mathbf{T}}$$
///
/// **Hemisphere Sign Convention:** Because $(\hat{\mathbf{S}}, \hat{\mathbf{T}}, \hat{\mathbf{R}})$ is
/// right-handed and $\hat{\mathbf{T}} \propto \hat{\mathbf{S}} \times \hat{\mathbf{k}}$, the $\hat{\mathbf{R}}$
/// axis points generally **southward** ($\hat{\mathbf{R}} \cdot \hat{\mathbf{k}} < 0$), matching JPL Monte,
/// NASA GMAT, and AGI STK conventions. Consequently:
/// * **Northern hemisphere flyby:** $B_R < 0$
/// * **Southern hemisphere flyby:** $B_R > 0$
/// * **Prograde equatorial flyby:** $B_T > 0$, $B_R = 0$
/// * **Retrograde equatorial flyby:** $B_T < 0$, $B_R = 0$
///
/// # Impact Parameter Vector ($\mathbf{B}$)
///
/// The impact parameter vector lies strictly within the $(\hat{\mathbf{T}}, \hat{\mathbf{R}})$ plane
/// ($\mathbf{B} \cdot \hat{\mathbf{S}} \equiv 0$) with magnitude equal to the hyperbolic semi-minor axis $b$:
/// $$\mathbf{B} = b \left(\sqrt{1 - \frac{1}{e^2}}\hat{\mathbf{e}} - \frac{1}{e}\hat{\mathbf{n}}\right) = B_T \hat{\mathbf{T}} + B_R \hat{\mathbf{R}}$$
///
/// # Frame Requirements Warning
///
/// The input `Orbit` **must** be expressed in an inertial (or pseudo-inertial, frozen at epoch) frame
/// whose Z-axis $[0, 0, 1]^T$ corresponds to the desired reference pole (e.g., EME2000, Moon ICRS, or
/// ECLIPJ2000). **Never** pass an `Orbit` transformed into a rotating body-fixed frame via the transport
/// theorem, as the rotational velocity term corrupts the osculating angular momentum, eccentricity, and $C_3$.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "python", pyclass(from_py_object, get_all, set_all))]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct BPlane {
    /// Component of the impact parameter vector along the $\hat{\mathbf{R}}$ axis ($\mathbf{B} \cdot \hat{\mathbf{R}}$), in kilometers.
    /// Negative values indicate a flyby over the northern hemisphere relative to the frame's $+Z$ pole.
    pub b_r_km: f64,

    /// Component of the impact parameter vector along the $\hat{\mathbf{T}}$ axis ($\mathbf{B} \cdot \hat{\mathbf{T}}$), in kilometers.
    /// Positive values indicate a prograde pass; negative values indicate a retrograde pass.
    pub b_t_km: f64,

    /// Linearized Time of Flight (LTOF) along the incoming asymptote to B-Plane crossing:
    /// $\text{LTOF} = -(\mathbf{r} \cdot \hat{\mathbf{S}}) / v_\infty$.
    /// Positive when the spacecraft is inbound (prior to B-Plane crossing) and negative after crossing.
    pub ltof: Duration,

    /// The inertial reference frame in which the input `Orbit` was evaluated, whose $+Z$ axis
    /// defines the reference pole $\hat{\mathbf{k}} = [0, 0, 1]^T$ for the $\hat{\mathbf{T}}$ and $\hat{\mathbf{R}}$ axes.
    pub frame: Frame,

    /// The osculating epoch of the trajectory state used to compute this B-Plane.
    pub epoch: Epoch,
}

impl BPlane {
    /// Compute the B-Plane parameters of this hyperbolic orbit.
    ///
    /// NOTE: The `Orbit` must be expressed in an inertial (non-rotating) frame whose Z-axis
    /// defines the B-Plane reference pole `k_ref = [0, 0, 1]`. Do NOT transform the orbit into
    /// a rotating body-fixed frame via the transport theorem (`v_rot = v_inert - w x r`), as
    /// that corrupts the osculating orbital elements; apply only the pure rotation DCM if aligning
    /// with a body pole.
    pub fn new(orbit: Orbit) -> PhysicsResult<Self> {
        let ecc = orbit.ecc()?;
        ensure!(ecc > 1.0, NotHyperbolicSnafu { ecc });

        let e_hat = orbit.evec()? / ecc;
        let h_hat = orbit.hvec()? / orbit.hmag()?;
        let n_hat = h_hat.cross(&e_hat);

        let cos_nu_inf = 1.0 / ecc;
        let sin_nu_inf = (1.0 - cos_nu_inf.powi(2)).sqrt();

        let s = cos_nu_inf * e_hat + sin_nu_inf * n_hat;
        let s_hat = s / s.norm();

        let b_vec = orbit.semi_minor_axis_km()? * (sin_nu_inf * e_hat - cos_nu_inf * n_hat);

        // Cross product with k_ref = [0, 0, 1]: s_hat x k_ref = [s_y, -s_x, 0]
        let t = Vector3::new(s_hat.y, -s_hat.x, 0.0);
        let t_norm = t.norm();
        ensure!(
            t_norm > 1e-12,
            InfiniteValueSnafu {
                action: "computing incoming asymptote; is it parallel to the [0, 0, 1] reference axis?"
            }
        );

        let t_hat = t / t_norm;
        let r_hat = s_hat.cross(&t_hat);

        let b_r_km = b_vec.dot(&r_hat);
        let b_t_km = b_vec.dot(&t_hat);

        let v_inf_km_s = orbit.c3_km2_s2()?.sqrt();
        let ltof = (-orbit.radius_km.dot(&s_hat) / v_inf_km_s).seconds();

        Ok(BPlane {
            b_r_km,
            b_t_km,
            ltof,
            frame: orbit.frame,
            epoch: orbit.epoch,
        })
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl BPlane {
    /// Returns the B plane angle in degrees between -180 and 180
    /// :rtype: float
    pub fn angle_deg(&self) -> f64 {
        between_pm_180(self.b_r_km.atan2(self.b_t_km).to_degrees())
    }

    /// Returns the B plane vector magnitude, in kilometers
    /// :rtype: float
    pub fn magnitude_km(&self) -> f64 {
        (self.b_t_km.powi(2) + self.b_r_km.powi(2)).sqrt()
    }
}
