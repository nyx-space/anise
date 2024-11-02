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
    math::{
        angles::{between_0_360, between_pm_180},
        cartesian::CartesianState,
        Vector3,
    },
    prelude::Frame,
};
use hifitime::Epoch;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyType;

impl CartesianState {
    /// Creates a new Orbit from the provided semi-major axis altitude in kilometers
    #[allow(clippy::too_many_arguments)]
    pub fn try_keplerian_altitude(
        sma_altitude_km: f64,
        ecc: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ta_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian(
            sma_altitude_km + frame.mean_equatorial_radius_km()?,
            ecc,
            inc_deg,
            raan_deg,
            aop_deg,
            ta_deg,
            epoch,
            frame,
        )
    }

    /// Creates a new Orbit from the provided altitudes of apoapsis and periapsis, in kilometers
    #[allow(clippy::too_many_arguments)]
    pub fn try_keplerian_apsis_altitude(
        apo_alt_km: f64,
        peri_alt_km: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ta_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian_apsis_radii(
            apo_alt_km + frame.mean_equatorial_radius_km()?,
            peri_alt_km + frame.mean_equatorial_radius_km()?,
            inc_deg,
            raan_deg,
            aop_deg,
            ta_deg,
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
    ///
    /// :type sma_altitude_km: float
    /// :type ecc: float
    /// :type inc_deg: float
    /// :type raan_deg: float
    /// :type aop_deg: float
    /// :type ta_deg: float
    /// :type epoch: Epoch
    /// :type frame: Frame
    /// :rtype: Orbit
    #[allow(clippy::too_many_arguments)]
    #[cfg(feature = "python")]
    #[classmethod]
    pub fn from_keplerian_altitude(
        _cls: &Bound<'_, PyType>,
        sma_altitude_km: f64,
        ecc: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ta_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian_altitude(
            sma_altitude_km,
            ecc,
            inc_deg,
            raan_deg,
            aop_deg,
            ta_deg,
            epoch,
            frame,
        )
    }

    /// Creates a new Orbit from the provided altitudes of apoapsis and periapsis, in kilometers
    ///
    /// :type apo_alt_km: float
    /// :type peri_alt_km: float
    /// :type inc_deg: float
    /// :type raan_deg: float
    /// :type aop_deg: float
    /// :type ta_deg: float
    /// :type epoch: Epoch
    /// :type frame: Frame
    /// :rtype: Orbit
    #[allow(clippy::too_many_arguments)]
    #[cfg(feature = "python")]
    #[classmethod]
    pub fn from_keplerian_apsis_altitude(
        _cls: &Bound<'_, PyType>,
        apo_alt_km: f64,
        peri_alt_km: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ta_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian_apsis_altitude(
            apo_alt_km,
            peri_alt_km,
            inc_deg,
            raan_deg,
            aop_deg,
            ta_deg,
            epoch,
            frame,
        )
    }

    /// Creates a new Orbit from the latitude (φ), longitude (λ) and height (in km) with respect to the frame's ellipsoid given the angular velocity.
    ///
    /// **Units:** degrees, degrees, km, rad/s
    /// NOTE: This computation differs from the spherical coordinates because we consider the flattening of body.
    /// Reference: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016
    ///
    ///
    /// :type latitude_deg: float
    /// :type longitude_deg: float
    /// :type height_km: float
    /// :type angular_velocity: float
    /// :type epoch: Epoch
    /// :type frame: Frame
    /// :rtype: Orbit
    #[cfg(feature = "python")]
    #[classmethod]
    pub fn from_latlongalt(
        _cls: &Bound<'_, PyType>,
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
    ///
    /// :rtype: float
    pub fn sma_altitude_km(&self) -> PhysicsResult<f64> {
        Ok(self.sma_km()? - self.frame.mean_equatorial_radius_km()?)
    }

    /// Returns the altitude of periapsis (or perigee around Earth), in kilometers.
    ///
    /// :rtype: float
    pub fn periapsis_altitude_km(&self) -> PhysicsResult<f64> {
        Ok(self.periapsis_km()? - self.frame.mean_equatorial_radius_km()?)
    }

    /// Returns the altitude of apoapsis (or apogee around Earth), in kilometers.
    ///
    /// :rtype: float
    pub fn apoapsis_altitude_km(&self) -> PhysicsResult<f64> {
        Ok(self.apoapsis_km()? - self.frame.mean_equatorial_radius_km()?)
    }

    /// Returns the geodetic latitude, geodetic longitude, and geodetic height, respectively in degrees, degrees, and kilometers.
    ///
    /// # Algorithm
    /// This uses the Heikkinen procedure, which is not iterative. The results match Vallado and GMAT.
    ///
    /// :rtype: typing.Tuple
    pub fn latlongalt(&self) -> PhysicsResult<(f64, f64, f64)> {
        let a_km = self.frame.mean_equatorial_radius_km()?;
        let b_km = self.frame.shape.unwrap().polar_radius_km;
        let e2 = (a_km.powi(2) - b_km.powi(2)) / a_km.powi(2);
        let e_prime2 = (a_km.powi(2) - b_km.powi(2)) / b_km.powi(2);
        let p = (self.radius_km.x.powi(2) + self.radius_km.y.powi(2)).sqrt();
        let big_f = 54.0 * b_km.powi(2) * self.radius_km.z.powi(2);
        let big_g =
            p.powi(2) + (1.0 - e2) * self.radius_km.z.powi(2) - e2 * (a_km.powi(2) - b_km.powi(2));
        let c = (e2.powi(2) * big_f * p.powi(2)) / big_g.powi(3);
        let s = (1.0 + c + (c.powi(2) + 2.0 * c).sqrt()).powf(1.0 / 3.0);
        let k = s + 1.0 + 1.0 / s;
        let big_p = big_f / (3.0 * k.powi(2) * big_g.powi(2));
        let big_q = (1.0 + 2.0 * e2.powi(2) * big_p).sqrt();
        let r0 = (-big_p * e2 * p) / (1.0 + big_q)
            + (0.5 * a_km.powi(2) * (1.0 + 1.0 / big_q)
                - (big_p * (1.0 - e2) * self.radius_km.z.powi(2)) / (big_q * (1.0 + big_q))
                - 0.5 * big_p * p.powi(2))
            .sqrt();
        let big_u = ((p - e2 * r0).powi(2) + self.radius_km.z.powi(2)).sqrt();
        let big_v = ((p - e2 * r0).powi(2) + (1.0 - e2) * self.radius_km.z.powi(2)).sqrt();
        let z0 = b_km.powi(2) * self.radius_km.z / (a_km * big_v);

        let alt_km = big_u * (1.0 - b_km.powi(2) / (a_km * big_v));
        let lat_deg =
            between_pm_180((((self.radius_km.z + e_prime2 * z0) / p).atan()).to_degrees());
        let long_deg = between_0_360(self.radius_km.y.atan2(self.radius_km.x).to_degrees());

        Ok((lat_deg, long_deg, alt_km))
    }

    /// Returns the geodetic longitude (λ) in degrees. Value is between -180 and 180 degrees.
    ///
    /// # Frame warning
    /// This state MUST be in the body fixed frame (e.g. ITRF93) prior to calling this function, or the computation is **invalid**.
    ///
    /// :rtype: float
    pub fn longitude_deg(&self) -> f64 {
        between_pm_180(self.radius_km.y.atan2(self.radius_km.x).to_degrees())
    }

    /// Returns the geodetic longitude (λ) in degrees. Value is between 0 and 360 degrees.
    ///
    /// # Frame warning
    /// This state MUST be in the body fixed frame (e.g. ITRF93) prior to calling this function, or the computation is **invalid**.
    ///
    /// :rtype: float
    pub fn longitude_360_deg(&self) -> f64 {
        between_0_360(self.radius_km.y.atan2(self.radius_km.x).to_degrees())
    }

    /// Returns the geodetic latitude (φ) in degrees. Value is between -180 and +180 degrees.
    ///
    /// # Frame warning
    /// This state MUST be in the body fixed frame (e.g. ITRF93) prior to calling this function, or the computation is **invalid**.
    ///
    /// :rtype: float
    pub fn latitude_deg(&self) -> PhysicsResult<f64> {
        Ok(self.latlongalt()?.0)
    }

    /// Returns the geodetic height in km.
    ///
    /// Reference: Vallado, 4th Ed., Algorithm 12 page 172.
    ///
    /// :rtype: float
    pub fn height_km(&self) -> PhysicsResult<f64> {
        Ok(self.latlongalt()?.2)
    }
}
