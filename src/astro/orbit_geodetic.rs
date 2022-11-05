/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{GeodeticFrame, GeodeticFrameTrait};
use crate::{
    errors::PhysicsErrorKind,
    math::{
        angles::{between_0_360, between_pm_180},
        cartesian::Cartesian,
        Vector3,
    },
};
use hifitime::Epoch;
use log::{error, warn};

pub type GeodeticOrbit = Cartesian<GeodeticFrame>;

impl<F: GeodeticFrameTrait> Cartesian<F> {
    /// Creates a new Orbit from the provided semi-major axis altitude in kilometers
    pub fn try_keplerian_altitude(
        sma_altitude: f64,
        ecc: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        dt: Epoch,
        frame: F,
    ) -> Result<Self, PhysicsErrorKind> {
        Self::try_keplerian(
            sma_altitude + frame.equatorial_radius_km(),
            ecc,
            inc,
            raan,
            aop,
            ta,
            dt,
            frame,
        )
    }

    /// Creates a new Orbit from the provided altitudes of apoapsis and periapsis, in kilometers
    pub fn try_keplerian_apsis_altitude(
        a_a: f64,
        a_p: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        dt: Epoch,
        frame: F,
    ) -> Result<Self, PhysicsErrorKind> {
        Self::try_keplerian_apsis_radii(
            a_a + frame.equatorial_radius_km(),
            a_p + frame.equatorial_radius_km(),
            inc,
            raan,
            aop,
            ta,
            dt,
            frame,
        )
    }

    /// Creates a new Orbit from the geodetic latitude (φ), longitude (λ) and height with respect to the ellipsoid of the frame.
    ///
    /// **Units:** degrees, degrees, km
    /// NOTE: This computation differs from the spherical coordinates because we consider the flattening of body.
    /// Reference: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016
    /// **WARNING:** This uses the rotational rates known to Nyx. For other objects, use `from_altlatlong` for other celestial bodies.
    pub fn from_geodesic(latitude: f64, longitude: f64, height: f64, dt: Epoch, frame: F) -> Self {
        Self::from_altlatlong(
            latitude,
            longitude,
            height,
            frame.angular_velocity_deg(),
            dt,
            frame,
        )
    }

    /// Creates a new Orbit from the latitude (φ), longitude (λ) and height with respect to the frame's ellipsoid.
    ///
    /// **Units:** degrees, degrees, km, rad/s
    /// NOTE: This computation differs from the spherical coordinates because we consider the flattening of body.
    /// Reference: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016
    pub fn from_altlatlong(
        latitude_deg: f64,
        longitude_deg: f64,
        height_km: f64,
        angular_velocity: f64,
        dt: Epoch,
        frame: F,
    ) -> Self {
        let e2 = 2.0 * frame.flattening() - frame.flattening().powi(2);
        let (sin_long, cos_long) = longitude_deg.to_radians().sin_cos();
        let (sin_lat, cos_lat) = latitude_deg.to_radians().sin_cos();
        // page 144
        let c_body = frame.semi_major_radius_km() / ((1.0 - e2 * sin_lat.powi(2)).sqrt());
        let s_body = (frame.semi_major_radius_km() * (1.0 - frame.flattening()).powi(2))
            / ((1.0 - e2 * sin_lat.powi(2)).sqrt());
        let ri = (c_body + height_km) * cos_lat * cos_long;
        let rj = (c_body + height_km) * cos_lat * sin_long;
        let rk = (s_body + height_km) * sin_lat;
        let radius = Vector3::new(ri, rj, rk);
        let velocity = Vector3::new(0.0, 0.0, angular_velocity).cross(&radius);
        Self::cartesian(
            radius[0],
            radius[1],
            radius[2],
            velocity[0],
            velocity[1],
            velocity[2],
            dt,
            frame,
        )
    }

    /// Returns the SMA altitude in km
    pub fn sma_altitude(&self) -> f64 {
        self.sma_km() - self.frame.equatorial_radius_km()
    }

    /// Returns the altitude of periapsis (or perigee around Earth), in kilometers.
    pub fn periapsis_altitude(&self) -> f64 {
        self.periapsis_km() - self.frame.equatorial_radius_km()
    }

    /// Returns the altitude of apoapsis (or apogee around Earth), in kilometers.
    pub fn apoapsis_altitude(&self) -> f64 {
        self.apoapsis_km() - self.frame.equatorial_radius_km()
    }

    /// Returns the geodetic longitude (λ) in degrees. Value is between 0 and 360 degrees.
    ///
    /// Although the reference is not Vallado, the math from Vallado proves to be equivalent.
    /// Reference: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016
    pub fn geodetic_longitude(&self) -> f64 {
        between_0_360(self.radius_km.y.atan2(self.radius_km.x).to_degrees())
    }

    /// Returns the geodetic latitude (φ) in degrees. Value is between -180 and +180 degrees.
    ///
    /// Reference: Vallado, 4th Ed., Algorithm 12 page 172.
    pub fn geodetic_latitude(&self) -> f64 {
        if !self.frame.is_body_fixed() {
            warn!("computation of geodetic latitude must be done in a body fixed frame and {:?} is not one!", self.frame);
        }
        let eps = 1e-12;
        let max_attempts = 20;
        let mut attempt_no = 0;
        let r_delta = (self.radius_km.x.powi(2) + self.radius_km.y.powi(2)).sqrt();
        let mut latitude = (self.radius_km.z / self.rmag_km()).asin();
        let e2 = self.frame.flattening() * (2.0 - self.frame.flattening());
        loop {
            attempt_no += 1;
            let c_earth =
                self.frame.semi_major_radius_km() / ((1.0 - e2 * (latitude).sin().powi(2)).sqrt());
            let new_latitude = (self.radius_km.z + c_earth * e2 * (latitude).sin()).atan2(r_delta);
            if (latitude - new_latitude).abs() < eps {
                return between_pm_180(new_latitude.to_degrees());
            } else if attempt_no >= max_attempts {
                error!(
                    "geodetic latitude failed to converge -- error = {}",
                    (latitude - new_latitude).abs()
                );
                return between_pm_180(new_latitude.to_degrees());
            }
            latitude = new_latitude;
        }
    }

    /// Returns the geodetic height in km.
    ///
    /// Reference: Vallado, 4th Ed., Algorithm 12 page 172.
    pub fn geodetic_height(&self) -> f64 {
        if !self.frame.is_body_fixed() {
            warn!("Computation of geodetic height must be done in a body fixed frame and {:?} is not one!", self.frame);
        }
        let e2 = self.frame.flattening() * (2.0 - self.frame.flattening());
        let latitude = self.geodetic_latitude().to_radians();
        let sin_lat = latitude.sin();
        if (latitude - 1.0).abs() < 0.1 {
            // We are near poles, let's use another formulation.
            let s_earth = (self.frame.semi_major_radius_km()
                * (1.0 - self.frame.flattening()).powi(2))
                / ((1.0 - e2 * sin_lat.powi(2)).sqrt());
            self.radius_km.z / latitude.sin() - s_earth
        } else {
            let c_earth = self.frame.semi_major_radius_km() / ((1.0 - e2 * sin_lat.powi(2)).sqrt());
            let r_delta = (self.radius_km.x.powi(2) + self.radius_km.y.powi(2)).sqrt();
            r_delta / latitude.cos() - c_earth
        }
    }
}