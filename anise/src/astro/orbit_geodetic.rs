/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::PhysicsResult;
use crate::{
    math::{
        angles::{between_0_360, between_pm_180},
        cartesian::CartesianState,
        Vector3,
    },
    prelude::Frame,
};
use hifitime::Epoch;
use log::error;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyType;

impl CartesianState {
    /// Creates a new Orbit from the provided semi-major axis altitude in kilometers
    #[allow(clippy::too_many_arguments)]
    pub fn try_keplerian_altitude(
        sma_altitude: f64,
        ecc: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian(
            sma_altitude + frame.mean_equatorial_radius_km()?,
            ecc,
            inc,
            raan,
            aop,
            ta,
            epoch,
            frame,
        )
    }

    /// Creates a new Orbit from the provided altitudes of apoapsis and periapsis, in kilometers
    #[allow(clippy::too_many_arguments)]
    pub fn try_keplerian_apsis_altitude(
        apo_alt: f64,
        peri_alt: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian_apsis_radii(
            apo_alt + frame.mean_equatorial_radius_km()?,
            peri_alt + frame.mean_equatorial_radius_km()?,
            inc,
            raan,
            aop,
            ta,
            epoch,
            frame,
        )
    }

    /// Creates a new Orbit from the latitude (φ), longitude (λ) and height (in km) with respect to the frame's ellipsoid given the angular velocity.
    ///
    /// **Note:** The mean Earth angular velocity is `0.004178079012116429` deg/s.
    ///
    /// NOTE: This computation differs from the spherical coordinates because we consider the flattening of body.
    /// Reference: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016
    pub fn try_latlongalt(
        latitude_deg: f64,
        longitude_deg: f64,
        height_km: f64,
        angular_velocity_deg_s: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        let e2 = 2.0 * frame.flattening()? - frame.flattening()?.powi(2);
        let (sin_long, cos_long) = longitude_deg.to_radians().sin_cos();
        let (sin_lat, cos_lat) = latitude_deg.to_radians().sin_cos();
        // page 144
        let c_body = frame.semi_major_radius_km()? / ((1.0 - e2 * sin_lat.powi(2)).sqrt());
        let s_body = (frame.semi_major_radius_km()? * (1.0 - frame.flattening()?).powi(2))
            / ((1.0 - e2 * sin_lat.powi(2)).sqrt());
        let ri = (c_body + height_km) * cos_lat * cos_long;
        let rj = (c_body + height_km) * cos_lat * sin_long;
        let rk = (s_body + height_km) * sin_lat;
        let radius = Vector3::new(ri, rj, rk);
        let velocity = Vector3::new(0.0, 0.0, angular_velocity_deg_s).cross(&radius);
        Ok(Self::new(
            radius[0],
            radius[1],
            radius[2],
            velocity[0],
            velocity[1],
            velocity[2],
            epoch,
            frame,
        ))
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl CartesianState {
    /// Creates a new Orbit from the provided semi-major axis altitude in kilometers
    #[cfg(feature = "python")]
    #[classmethod]
    pub fn from_keplerian_altitude(
        _cls: &PyType,
        sma_altitude: f64,
        ecc: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian_altitude(sma_altitude, ecc, inc, raan, aop, ta, epoch, frame)
    }

    /// Creates a new Orbit from the provided altitudes of apoapsis and periapsis, in kilometers
    #[cfg(feature = "python")]
    #[classmethod]
    pub fn from_keplerian_apsis_altitude(
        _cls: &PyType,
        apo_alt: f64,
        peri_alt: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian_apsis_altitude(apo_alt, peri_alt, inc, raan, aop, ta, epoch, frame)
    }

    /// Creates a new Orbit from the latitude (φ), longitude (λ) and height (in km) with respect to the frame's ellipsoid given the angular velocity.
    ///
    /// **Units:** degrees, degrees, km, rad/s
    /// NOTE: This computation differs from the spherical coordinates because we consider the flattening of body.
    /// Reference: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016
    #[cfg(feature = "python")]
    #[classmethod]
    pub fn from_latlongalt(
        _cls: &PyType,
        latitude_deg: f64,
        longitude_deg: f64,
        height_km: f64,
        angular_velocity: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_latlongalt(
            latitude_deg,
            longitude_deg,
            height_km,
            angular_velocity,
            epoch,
            frame,
        )
    }

    /// Returns the SMA altitude in km
    pub fn sma_altitude_km(&self) -> PhysicsResult<f64> {
        Ok(self.sma_km()? - self.frame.mean_equatorial_radius_km()?)
    }

    /// Returns the altitude of periapsis (or perigee around Earth), in kilometers.
    pub fn periapsis_altitude_km(&self) -> PhysicsResult<f64> {
        Ok(self.periapsis_km()? - self.frame.mean_equatorial_radius_km()?)
    }

    /// Returns the altitude of apoapsis (or apogee around Earth), in kilometers.
    pub fn apoapsis_altitude_km(&self) -> PhysicsResult<f64> {
        Ok(self.apoapsis_km()? - self.frame.mean_equatorial_radius_km()?)
    }

    /// Returns the geodetic longitude (λ) in degrees. Value is between 0 and 360 degrees.
    ///
    /// # Frame warning
    /// If the state is NOT in a body fixed frame (i.e. ITRF93), then this computation is INVALID.
    ///
    /// Although the reference is not Vallado, the math from Vallado proves to be equivalent.
    /// Reference: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016
    pub fn geodetic_longitude_deg(&self) -> f64 {
        between_0_360(self.radius_km.y.atan2(self.radius_km.x).to_degrees())
    }

    /// Returns the geodetic latitude (φ) in degrees. Value is between -180 and +180 degrees.
    ///
    /// # Frame warning
    /// If the state is NOT in a body fixed frame (i.e. ITRF93), then this computation is INVALID.
    ///
    /// Reference: Vallado, 4th Ed., Algorithm 12 page 172.
    pub fn geodetic_latitude_deg(&self) -> PhysicsResult<f64> {
        let eps = 1e-12;
        let max_attempts = 20;
        let mut attempt_no = 0;
        let r_delta = (self.radius_km.x.powi(2) + self.radius_km.y.powi(2)).sqrt();
        let mut latitude = (self.radius_km.z / self.rmag_km()).asin();
        let e2 = self.frame.flattening()? * (2.0 - self.frame.flattening()?);
        loop {
            attempt_no += 1;
            let c_earth =
                self.frame.semi_major_radius_km()? / ((1.0 - e2 * (latitude).sin().powi(2)).sqrt());
            let new_latitude = (self.radius_km.z + c_earth * e2 * (latitude).sin()).atan2(r_delta);
            if (latitude - new_latitude).abs() < eps {
                return Ok(between_pm_180(new_latitude.to_degrees()));
            } else if attempt_no >= max_attempts {
                error!(
                    "geodetic latitude failed to converge -- error = {}",
                    (latitude - new_latitude).abs()
                );
                return Ok(between_pm_180(new_latitude.to_degrees()));
            }
            latitude = new_latitude;
        }
    }

    /// Returns the geodetic height in km.
    ///
    /// Reference: Vallado, 4th Ed., Algorithm 12 page 172.
    pub fn geodetic_height_km(&self) -> PhysicsResult<f64> {
        let e2 = self.frame.flattening()? * (2.0 - self.frame.flattening()?);
        let latitude = self.geodetic_latitude_deg()?.to_radians();
        let sin_lat = latitude.sin();
        if (latitude - 1.0).abs() < 0.1 {
            // We are near poles, let's use another formulation.
            let s_earth = (self.frame.semi_major_radius_km()?
                * (1.0 - self.frame.flattening()?).powi(2))
                / ((1.0 - e2 * sin_lat.powi(2)).sqrt());
            Ok(self.radius_km.z / latitude.sin() - s_earth)
        } else {
            let c_earth =
                self.frame.semi_major_radius_km()? / ((1.0 - e2 * sin_lat.powi(2)).sqrt());
            let r_delta = (self.radius_km.x.powi(2) + self.radius_km.y.powi(2)).sqrt();
            Ok(r_delta / latitude.cos() - c_earth)
        }
    }
}
