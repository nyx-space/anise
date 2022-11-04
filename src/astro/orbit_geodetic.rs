/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::GeodeticFrameTrait;
use crate::{
    errors::PhysicsErrorKind,
    math::{
        angles::{between_0_360, between_pm_180},
        cartesian::Cartesian,
    },
};
use hifitime::Epoch;
use log::{error, warn};

impl<F: GeodeticFrameTrait> Cartesian<F> {
    /// Creates a new Orbit from the provided semi-major axis altitude in kilometers
    pub fn keplerian_altitude(
        sma_altitude: f64,
        ecc: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        dt: Epoch,
        frame: F,
    ) -> Result<Self, PhysicsErrorKind> {
        Self::keplerian(
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
    pub fn keplerian_apsis_altitude(
        a_a: f64,
        a_p: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        dt: Epoch,
        frame: F,
    ) -> Result<Self, PhysicsErrorKind> {
        Self::keplerian_apsis_radii(
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
