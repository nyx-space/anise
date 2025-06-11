/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::utils::compute_mean_to_true_anomaly_rad;
use super::PhysicsResult;

use crate::{
    errors::{
        HyperbolicTrueAnomalySnafu, InfiniteValueSnafu, ParabolicEccentricitySnafu,
        ParabolicSemiParamSnafu, PhysicsError, RadiusSnafu, VelocitySnafu,
    },
    math::{
        angles::{between_0_360, between_pm_180},
        cartesian::CartesianState,
        rotation::DCM,
        Matrix3, Vector3, Vector6,
    },
    prelude::{uuid_from_epoch, Frame},
    NaifId,
};
use core::f64::consts::PI;

use core::fmt;
use hifitime::{Duration, Epoch, TimeUnits, Unit};
use log::{error, info, warn};
use snafu::ensure;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyType;

/// If an orbit has an eccentricity below the following value, it is considered circular (only affects warning messages)
pub const ECC_EPSILON: f64 = 1e-11;

/// A helper type alias, but no assumptions are made on the underlying validity of the frame.
pub type Orbit = CartesianState;

impl Orbit {
    /// Attempts to create a new Orbit around the provided Celestial or Geoid frame from the Keplerian orbital elements.
    ///
    /// **Units:** km, none, degrees, degrees, degrees, degrees
    ///
    /// WARNING: This function will return an error if the singularities in the conversion are encountered.
    /// NOTE: The state is defined in Cartesian coordinates as they are non-singular. This causes rounding
    /// errors when creating a state from its Keplerian orbital elements (cf. the state tests).
    /// One should expect these errors to be on the order of 1e-12.
    #[allow(clippy::too_many_arguments)]
    pub fn try_keplerian(
        sma_km: f64,
        ecc: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ta_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        let mu_km3_s2 = frame.mu_km3_s2()?;
        if mu_km3_s2.abs() < f64::EPSILON {
            warn!("GM is near zero ({mu_km3_s2} km^3/s^2): expect rounding errors!",);
        }
        // Algorithm from GMAT's StateConversionUtil::KeplerianToCartesian
        let ecc = if ecc < 0.0 {
            warn!("eccentricity cannot be negative: sign of eccentricity changed");
            ecc * -1.0
        } else {
            ecc
        };
        let sma = if ecc > 1.0 && sma_km > 0.0 {
            warn!("eccentricity > 1 (hyperbolic) BUT SMA > 0 (elliptical): sign of SMA changed");
            sma_km * -1.0
        } else if ecc < 1.0 && sma_km < 0.0 {
            warn!("eccentricity < 1 (elliptical) BUT SMA < 0 (hyperbolic): sign of SMA changed");
            sma_km * -1.0
        } else {
            sma_km
        };
        if (sma * (1.0 - ecc)).abs() < 1e-3 {
            // GMAT errors below one meter. Let's warn for below that, but not panic, might be useful for landing scenarios?
            warn!("radius of periapsis is less than one meter");
        }
        ensure!(
            (1.0 - ecc).abs() >= ECC_EPSILON,
            ParabolicEccentricitySnafu { limit: ECC_EPSILON }
        );
        if ecc > 1.0 {
            let ta_deg = between_0_360(ta_deg);
            ensure!(
                ta_deg <= (PI - (1.0 / ecc).acos()).to_degrees(),
                HyperbolicTrueAnomalySnafu { ta_deg }
            );
        }
        ensure!(
            (1.0 + ecc * ta_deg.to_radians().cos()).is_finite(),
            InfiniteValueSnafu {
                action: "computing radius of orbit"
            }
        );

        // Done with all the warnings and errors supported by GMAT
        // The conversion algorithm itself comes from GMAT's StateConversionUtil::ComputeKeplToCart
        // NOTE: GMAT supports mean anomaly instead of true anomaly, but only for backward compatibility reasons
        // so it isn't supported here.
        let inc_rad = inc_deg.to_radians();
        let raan_rad = raan_deg.to_radians();
        let aop_rad = aop_deg.to_radians();
        let ta_rad = ta_deg.to_radians();
        let p_km = sma * (1.0 - ecc.powi(2));

        ensure!(p_km.abs() >= f64::EPSILON, ParabolicSemiParamSnafu { p_km });

        // NOTE: At this point GMAT computes 1+ecc**2 and checks whether it's very small.
        // It then reports that the radius may be too large. We've effectively already done
        // this check above (and panicked if needed), so it isn't repeated here.
        let radius = p_km / (1.0 + ecc * ta_rad.cos());
        let (sin_aop_ta, cos_aop_ta) = (aop_rad + ta_rad).sin_cos();
        let (sin_inc, cos_inc) = inc_rad.sin_cos();
        let (sin_raan, cos_raan) = raan_rad.sin_cos();
        let (sin_aop, cos_aop) = aop_rad.sin_cos();
        let x = radius * (cos_aop_ta * cos_raan - cos_inc * sin_aop_ta * sin_raan);
        let y = radius * (cos_aop_ta * sin_raan + cos_inc * sin_aop_ta * cos_raan);
        let z = radius * sin_aop_ta * sin_inc;
        let sqrt_gm_p = (mu_km3_s2 / p_km).sqrt();
        let cos_ta_ecc = ta_rad.cos() + ecc;
        let sin_ta = ta_rad.sin();

        let vx = sqrt_gm_p * cos_ta_ecc * (-sin_aop * cos_raan - cos_inc * sin_raan * cos_aop)
            - sqrt_gm_p * sin_ta * (cos_aop * cos_raan - cos_inc * sin_raan * sin_aop);
        let vy = sqrt_gm_p * cos_ta_ecc * (-sin_aop * sin_raan + cos_inc * cos_raan * cos_aop)
            - sqrt_gm_p * sin_ta * (cos_aop * sin_raan + cos_inc * cos_raan * sin_aop);
        let vz = sqrt_gm_p * (cos_ta_ecc * sin_inc * cos_aop - sin_ta * sin_inc * sin_aop);

        Ok(Self {
            radius_km: Vector3::new(x, y, z),
            velocity_km_s: Vector3::new(vx, vy, vz),
            epoch,
            frame,
        })
    }

    /// Attempts to create a new Orbit from the provided radii of apoapsis and periapsis, in kilometers
    #[allow(clippy::too_many_arguments)]
    pub fn try_keplerian_apsis_radii(
        r_a_km: f64,
        r_p_km: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ta_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        ensure!(
            r_a_km > f64::EPSILON,
            RadiusSnafu {
                action: "radius of apoapsis is negative"
            }
        );
        ensure!(
            r_p_km > f64::EPSILON,
            RadiusSnafu {
                action: "radius of periapsis is negative"
            }
        );
        // The two checks above ensure that sma > 0
        let sma = (r_a_km + r_p_km) / 2.0;
        let ecc = r_a_km / sma - 1.0;
        Self::try_keplerian(sma, ecc, inc_deg, raan_deg, aop_deg, ta_deg, epoch, frame)
    }

    /// Attempts to create a new Orbit around the provided frame from the borrowed state vector
    ///
    /// The state vector **must** be sma, ecc, inc, raan, aop, ta. This function is a shortcut to `cartesian`
    /// and as such it has the same unit requirements.
    pub fn try_keplerian_vec(state: &Vector6, epoch: Epoch, frame: Frame) -> PhysicsResult<Self> {
        Self::try_keplerian(
            state[0], state[1], state[2], state[3], state[4], state[5], epoch, frame,
        )
    }

    /// Creates (without error checking) a new Orbit around the provided Celestial or Geoid frame from the Keplerian orbital elements.
    ///
    /// **Units:** km, none, degrees, degrees, degrees, degrees
    ///
    /// WARNING: This function will panic if the singularities in the conversion are expected.
    /// NOTE: The state is defined in Cartesian coordinates as they are non-singular. This causes rounding
    /// errors when creating a state from its Keplerian orbital elements (cf. the state tests).
    /// One should expect these errors to be on the order of 1e-12.
    #[allow(clippy::too_many_arguments)]
    pub fn keplerian(
        sma_km: f64,
        ecc: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ta_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> Self {
        Self::try_keplerian(
            sma_km, ecc, inc_deg, raan_deg, aop_deg, ta_deg, epoch, frame,
        )
        .unwrap()
    }

    /// Creates a new Orbit from the provided radii of apoapsis and periapsis, in kilometers
    #[allow(clippy::too_many_arguments)]
    pub fn keplerian_apsis_radii(
        r_a_km: f64,
        r_p_km: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ta_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> Self {
        Self::try_keplerian_apsis_radii(
            r_a_km, r_p_km, inc_deg, raan_deg, aop_deg, ta_deg, epoch, frame,
        )
        .unwrap()
    }

    /// Initializes a new orbit from the Keplerian orbital elements using the mean anomaly instead of the true anomaly.
    ///
    /// # Implementation notes
    /// This function starts by converting the mean anomaly to true anomaly, and then it initializes the orbit
    /// using the keplerian(..) method.
    /// The conversion is from GMAT's MeanToTrueAnomaly function, transliterated originally by Claude and GPT4 with human adjustments.
    #[allow(clippy::too_many_arguments)]
    pub fn try_keplerian_mean_anomaly(
        sma_km: f64,
        ecc: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ma_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        // Start by computing the true anomaly
        let ta_rad = compute_mean_to_true_anomaly_rad(ma_deg.to_radians(), ecc)?;

        Self::try_keplerian(
            sma_km,
            ecc,
            inc_deg,
            raan_deg,
            aop_deg,
            ta_rad.to_degrees(),
            epoch,
            frame,
        )
    }

    /// Creates a new Orbit around the provided frame from the borrowed state vector
    ///
    /// The state vector **must** be sma, ecc, inc, raan, aop, ta. This function is a shortcut to `cartesian`
    /// and as such it has the same unit requirements.
    pub fn keplerian_vec(state: &Vector6, epoch: Epoch, frame: Frame) -> Self {
        Self::try_keplerian_vec(state, epoch, frame).unwrap()
    }

    /// Returns this state as a Keplerian Vector6 in [km, none, degrees, degrees, degrees, degrees]
    ///
    /// Note that the time is **not** returned in the vector.
    pub fn to_keplerian_vec(self) -> PhysicsResult<Vector6> {
        Ok(Vector6::new(
            self.sma_km()?,
            self.ecc()?,
            self.inc_deg()?,
            self.raan_deg()?,
            self.aop_deg()?,
            self.ta_deg()?,
        ))
    }

    /// Returns the orbital momentum vector
    pub fn hvec(&self) -> PhysicsResult<Vector3> {
        ensure!(
            self.rmag_km() > f64::EPSILON,
            RadiusSnafu {
                action: "cannot compute orbital momentum vector with zero radius"
            }
        );
        ensure!(
            self.vmag_km_s() > f64::EPSILON,
            VelocitySnafu {
                action: "cannot compute orbital momentum vector with zero velocity"
            }
        );
        Ok(self.radius_km.cross(&self.velocity_km_s))
    }

    /// Returns the eccentricity vector (no unit)
    pub fn evec(&self) -> Result<Vector3, PhysicsError> {
        let r = self.radius_km;
        ensure!(
            self.rmag_km() > f64::EPSILON,
            RadiusSnafu {
                action: "cannot compute eccentricity vector with zero radial state"
            }
        );
        let v = self.velocity_km_s;
        Ok(
            ((v.norm().powi(2) - self.frame.mu_km3_s2()? / r.norm()) * r - (r.dot(&v)) * v)
                / self.frame.mu_km3_s2()?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::Frame;
    use hifitime::{Epoch, Duration, Unit};
    use crate::constants::celestial_objects::EARTH;
    use crate::constants::orientations::J2000;
    use hifitime::assert_approx_eq;
    use crate::errors::{PhysicsError, RadiusError, InfiniteValue}; // Assuming these are the correct error variants from errors.rs
    use std::f64::consts::PI; // PI is used in calculations

    const MU_EARTH: f64 = 398600.435436; // km^3/s^2
    const TEST_EPS_DURATION: f64 = 1e-2; // seconds, fairly tolerant for multi-step calcs
    const TEST_EPS_RADIUS: f64 = 1e-3;   // km

    fn get_test_epoch() -> Epoch {
        Epoch::from_gregorian_tai_components(2000, 1, 1, 0, 0, 0, 0)
    }

    fn get_test_frame() -> Frame {
        Frame::new(EARTH, J2000).with_mu_km3_s2(MU_EARTH)
    }

    fn create_orbit(sma_km: f64, ecc: f64, ta_deg: f64, epoch: Epoch, frame: Frame) -> Orbit {
        Orbit::try_keplerian(sma_km, ecc, 0.0, 0.0, 0.0, ta_deg, epoch, frame).unwrap_or_else(|e| panic!("Failed to create orbit for test: {:?}", e))
    }

    #[test]
    fn test_duration_to_radius_elliptical() {
        let epoch = get_test_epoch();
        let frame = get_test_frame();
        let sma_km = 10000.0;
        let ecc = 0.1;

        let rp_km = sma_km * (1.0 - ecc); // 9000 km
        let ra_km = sma_km * (1.0 + ecc); // 11000 km
        let period_s = 2.0 * PI * (sma_km.powi(3) / MU_EARTH).sqrt(); // Approx 9951.9652 s

        // Test 1a: Time from periapsis (TA=0) to apoapsis
        let orbit_at_peri = create_orbit(sma_km, ecc, 0.0, epoch, frame);
        let res1a = orbit_at_peri.duration_to_radius(ra_km);
        assert!(res1a.is_ok(), "Test 1a failed: {:?}", res1a.err());
        assert_approx_eq!(res1a.unwrap().to_seconds(), period_s / 2.0, TEST_EPS_DURATION);

        // Test 1b: Orbit at TA=90 deg. Time to apoapsis.
        // Expected duration pre-calculated: 2804.368 s
        let orbit_at_ta90 = create_orbit(sma_km, ecc, 90.0, epoch, frame);
        // current r for TA=90: p / (1 + e*cos(90)) = sma(1-e^2) = 10000(1-0.01) = 9900km
        let _current_r_ta90 = orbit_at_ta90.rmag_km(); // approx 9900 km
        let res1b = orbit_at_ta90.duration_to_radius(ra_km);
        assert!(res1b.is_ok(), "Test 1b failed: {:?}", res1b.err());
        assert_approx_eq!(res1b.unwrap().to_seconds(), 2804.368, TEST_EPS_DURATION);

        // Test 1c: Orbit at TA=0. Target radius_km = rp. Expected duration: 0.0.
        let res1c = orbit_at_peri.duration_to_radius(rp_km);
        assert!(res1c.is_ok(), "Test 1c failed: {:?}", res1c.err());
        assert_approx_eq!(res1c.unwrap().to_seconds(), 0.0, TEST_EPS_DURATION);

        // Test 1d: Orbit at TA=0. Target radius_km = 9000.0 + TEST_EPS_RADIUS (slightly after periapsis).
        // Expected: small positive duration.
        let target_r_slightly_after_peri = rp_km + TEST_EPS_RADIUS;
        let res1d = orbit_at_peri.duration_to_radius(target_r_slightly_after_peri);
        assert!(res1d.is_ok(), "Test 1d failed: {:?}", res1d.err());
        assert!(res1d.unwrap().to_seconds() > 0.0, "Duration should be positive");
        // A more precise value would require calculation, e.g. for rp_km + 1km, it's ~17.6s
        // For rp_km + 0.001km, it's very small. Let's check it's less than a few seconds.
        assert!(res1d.unwrap().to_seconds() < 20.0, "Duration seems too large for small radius change from periapsis");


        // Test 1e: Orbit at TA=180 deg (apoapsis). Time to periapsis.
        let orbit_at_apo = create_orbit(sma_km, ecc, 180.0, epoch, frame);
        let res1e = orbit_at_apo.duration_to_radius(rp_km);
        assert!(res1e.is_ok(), "Test 1e failed: {:?}", res1e.err());
        assert_approx_eq!(res1e.unwrap().to_seconds(), period_s / 2.0, TEST_EPS_DURATION);
    }

    #[test]
    fn test_duration_to_radius_circular() {
        let epoch = get_test_epoch();
        let frame = get_test_frame();
        let sma_km = 10000.0;

        // Test 2a: Near-circular orbit. Target radius_km = sma_km. Expected: Duration 0.0.
        let ecc_near_circular = ECC_EPSILON / 10.0; // Ensure it's treated as circular by the function's logic
        let circ_orbit = create_orbit(sma_km, ecc_near_circular, 0.0, epoch, frame);
        let res2a = circ_orbit.duration_to_radius(sma_km);
        assert!(res2a.is_ok(), "Test 2a failed: {:?}", res2a.err());
        assert_approx_eq!(res2a.unwrap().to_seconds(), 0.0, TEST_EPS_DURATION);

        // Test 2b: Target radius_km different from sma_km. Expected: RadiusError.
        let res2b = circ_orbit.duration_to_radius(sma_km + 1.0);
        assert!(res2b.is_err());
        match res2b.err().unwrap() {
            PhysicsError::RadiusError { action } => {
                assert_eq!(action, "Target radius for circular orbit must be (nearly) equal to orbit radius");
            }
            e => panic!("Test 2b: Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_duration_to_radius_hyperbolic() {
        let epoch = get_test_epoch();
        let frame = get_test_frame();
        let sma_km = -5000.0; // Negative for hyperbolic
        let ecc = 2.0;
        let rp_km = sma_km.abs() * (ecc - 1.0); // 5000 * (2-1) = 5000 km

        // Test 3a: From periapsis (TA=0) to radius_km = 10000 km.
        // Expected duration pre-calculated: 721.74 s
        let orbit_at_peri = create_orbit(sma_km, ecc, 0.0, epoch, frame);
        let res3a = orbit_at_peri.duration_to_radius(10000.0);
        assert!(res3a.is_ok(), "Test 3a failed: {:?}", res3a.err());
        assert_approx_eq!(res3a.unwrap().to_seconds(), 721.74, TEST_EPS_DURATION);

        // Test 3b: Target radius_km = rp_km. Expected duration: 0.0.
        let res3b = orbit_at_peri.duration_to_radius(rp_km);
        assert!(res3b.is_ok(), "Test 3b failed: {:?}", res3b.err());
        assert_approx_eq!(res3b.unwrap().to_seconds(), 0.0, TEST_EPS_DURATION);

        // Test 3c: Orbit at TA=30 deg. Target radius rp_km. Expected: RadiusError (past).
        let orbit_at_ta30 = create_orbit(sma_km, ecc, 30.0, epoch, frame);
        let res3c = orbit_at_ta30.duration_to_radius(rp_km);
        assert!(res3c.is_err());
        match res3c.err().unwrap() {
            PhysicsError::RadiusError { action } => {
                assert_eq!(action, "Radius (on [0,PI] TA arc) in past for hyperbolic orbit");
            }
            e => panic!("Test 3c: Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_duration_to_radius_error_conditions() {
        let epoch = get_test_epoch();
        let frame = get_test_frame();
        let sma_elliptical = 10000.0;
        let ecc_elliptical = 0.1;
        let rp_elliptical = sma_elliptical * (1.0 - ecc_elliptical); // 9000
        let ra_elliptical = sma_elliptical * (1.0 + ecc_elliptical); // 11000
        let elliptical_orbit = create_orbit(sma_elliptical, ecc_elliptical, 0.0, epoch, frame);

        let sma_hyperbolic = -5000.0;
        let ecc_hyperbolic = 2.0;
        let rp_hyperbolic = sma_hyperbolic.abs() * (ecc_hyperbolic - 1.0); // 5000
        let hyperbolic_orbit = create_orbit(sma_hyperbolic, ecc_hyperbolic, 0.0, epoch, frame);

        // Test 4a: radius_km = -100.0
        let res4a = elliptical_orbit.duration_to_radius(-100.0);
        assert!(res4a.is_err());
        match res4a.err().unwrap() {
            PhysicsError::RadiusError { action } => assert_eq!(action, "Target radius must be positive"),
            e => panic!("Test 4a: Unexpected error type: {:?}", e),
        }

        // Test 4b: Elliptical orbit. Target radius_km less than periapsis.
        let res4b = elliptical_orbit.duration_to_radius(rp_elliptical - 100.0);
        assert!(res4b.is_err());
        match res4b.err().unwrap() {
            PhysicsError::RadiusError { action } => assert_eq!(action, "Radius outside reachable range for elliptical orbit"),
            e => panic!("Test 4b: Unexpected error type: {:?}", e),
        }

        // Test 4c: Elliptical orbit. Target radius_km greater than apoapsis.
        let res4c = elliptical_orbit.duration_to_radius(ra_elliptical + 100.0);
        assert!(res4c.is_err());
        match res4c.err().unwrap() {
            PhysicsError::RadiusError { action } => assert_eq!(action, "Radius outside reachable range for elliptical orbit"),
            e => panic!("Test 4c: Unexpected error type: {:?}", e),
        }

        // Test 4d: Hyperbolic orbit. Target radius_km less than periapsis.
        let res4d = hyperbolic_orbit.duration_to_radius(rp_hyperbolic - 100.0);
        assert!(res4d.is_err());
        match res4d.err().unwrap() {
            PhysicsError::RadiusError { action } => assert_eq!(action, "Radius below periapsis for hyperbolic/parabolic orbit"),
            e => panic!("Test 4d: Unexpected error type: {:?}", e),
        }

        // Test 4e: cos(nu) out of bounds. This is tricky to hit precisely due to prior reachability checks.
        // If ecc=0.1, p = 10000*(1-0.01)=9900. For r=p, cos(nu)=0. For r > p, cos(nu) < 0. For r < p, cos(nu) > 0.
        // To make (p/r - 1)/e out of bounds, e.g. > 1: (p/r - 1) > e => p/r > 1+e => p > r(1+e).
        // If r = p/(1+e+epsilon_small), then (p/r-1)/e = (1+e+eps-1)/e = (e+eps)/e = 1+eps/e. This is > 1.
        // This radius is rp = p/(1+e). So r = rp / (1+eps_small_prime). This is smaller than periapsis.
        // So, this condition is typically caught by reachability checks (4b, 4d) for valid orbits.
        // For example, if target radius is less than periapsis, it's caught by "Radius outside reachable range".
        // If we somehow bypassed that, e.g. by a faulty p or e value not from self, this could be hit.
        // For now, rely on 4b, 4c, 4d to cover most practical cases of this.

        // Test 4f: Nearly parabolic orbits (ecc close to 1.0)
        // Case 1: Elliptical, ecc = 1.0 - 2*ECC_EPSILON
        let ecc_near_para_ell = 1.0 - 2.0 * ECC_EPSILON; // Still elliptical
        // sma = rp / (1-ecc) = 7000 / (2*ECC_EPSILON) -> very large sma
        // Using try_keplerian_apsis_radii to avoid issues with large SMA direct input if not careful
        let rp_near_para = 7000.0;
        // For elliptical, ra = rp * (1+e)/(1-e)
        let ra_near_para_ell = rp_near_para * (1.0 + ecc_near_para_ell) / (1.0 - ecc_near_para_ell);
        let orbit_near_para_ell = Orbit::try_keplerian_apsis_radii(ra_near_para_ell, rp_near_para, 0.0,0.0,0.0,0.0, epoch, frame).expect("Near-para ell orbit");

        let res4f1 = orbit_near_para_ell.duration_to_radius(rp_near_para + 1000.0); // Target radius valid
        assert!(res4f1.is_ok(), "Test 4f1 (near-para ell) failed: {:?}", res4f1.err());
        assert!(res4f1.unwrap().to_seconds() > 0.0);

        // Case 2: Hyperbolic, ecc = 1.0 + 2*ECC_EPSILON
        let ecc_near_para_hyp = 1.0 + 2.0 * ECC_EPSILON; // Still hyperbolic
        // sma = rp / (1-ecc) = 7000 / (-2*ECC_EPSILON) -> large negative sma
        // Using try_keplerian_apsis_radii is for elliptical, need direct try_keplerian for hyperbolic SMA
        let sma_near_para_hyp = rp_near_para / (1.0 - ecc_near_para_hyp); // This will be negative
        let orbit_near_para_hyp = create_orbit(sma_near_para_hyp, ecc_near_para_hyp, 0.0, epoch, frame);

        let res4f2 = orbit_near_para_hyp.duration_to_radius(rp_near_para + 1000.0); // Target radius valid
        assert!(res4f2.is_ok(), "Test 4f2 (near-para hyp) failed: {:?}", res4f2.err());
        assert!(res4f2.unwrap().to_seconds() > 0.0);

        // Test 4f-alternative for InfiniteValue (parabolic orbit sma->inf, n->0)
        // Orbit::try_keplerian ensures (1.0 - ecc).abs() >= ECC_EPSILON, so ecc cannot be exactly 1.0.
        // If sma_km is extremely large (e.g. ecc very close to 1.0), n_rad_s becomes very small.
        // If sma_km were infinite (parabolic), n_rad_s would be 0.
        // The current check is ensure!(n_rad_s.is_finite() && n_rad_s > 0.0, InfiniteValue { ... })
        // This should catch n_rad_s = 0 if sma_km was Inf, but sma_km() itself would likely return Inf.
        // If sma_km.abs().powi(3) overflows to Inf, then n_rad_s is 0.
        let ecc_very_near_1 = 1.0 - ECC_EPSILON * 1.01; // Should pass try_keplerian
        let sma_very_large = rp_near_para / (1.0 - ecc_very_near_1);
        let orbit_large_sma = create_orbit(sma_very_large, ecc_very_near_1, 0.0, epoch, frame);
        let res_large_sma_time = orbit_large_sma.duration_to_radius(rp_near_para + 1.0);
        assert!(res_large_sma_time.is_ok()); // Should be ok, time will just be large or n very small.
                                             // The InfiniteValue for n_rad_s is more about mu/sma^3 being NaN/Inf or zero due to sma being zero or Inf.
                                             // If sma_km() returns Inf, then sma_km.abs().powi(3) is Inf, then n_rad_s is 0. Caught.
                                             // If sma_km() returns some error that leads to NaN sma, then n_rad_s is NaN. Caught.
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg_attr(feature = "python", pymethods)]
impl Orbit {
    /// Builds the rotation matrix that rotates from the topocentric frame (SEZ) into the body fixed frame of this state.
    ///
    /// # Frame warnings
    /// + If the state is NOT in a body fixed frame (i.e. ITRF93), then this computation is INVALID.
    /// + (Usually) no time derivative can be computed: the orbit is expected to be a body fixed frame where the `at_epoch` function will fail. Exceptions for Moon body fixed frames.
    ///
    /// # UNUSED Arguments
    /// + `from`: ID of this new frame. Only used to set the "from" frame of the DCM. -- No longer used since 0.5.3
    ///
    /// # Source
    /// From the GMAT MathSpec, page 30 section 2.6.9 and from `Calculate_RFT` in `TopocentricAxes.cpp`, this returns the
    /// rotation matrix from the topocentric frame (SEZ) to body fixed frame.
    /// In the GMAT MathSpec notation, R_{IF} is the DCM from body fixed to inertial. Similarly, R{FT} is from topocentric
    /// to body fixed.
    ///
    /// :type _from: float
    /// :rtype: DCM
    pub fn dcm_from_topocentric_to_body_fixed(&self, _from: NaifId) -> PhysicsResult<DCM> {
        let rot_mat_dt = if let Ok(pre) = self.at_epoch(self.epoch - Unit::Second * 1) {
            if let Ok(post) = self.at_epoch(self.epoch + Unit::Second * 1) {
                let dcm_pre = pre.dcm3x3_from_topocentric_to_body_fixed()?;
                let dcm_post = post.dcm3x3_from_topocentric_to_body_fixed()?;
                Some(0.5 * dcm_post.rot_mat - 0.5 * dcm_pre.rot_mat)
            } else {
                None
            }
        } else {
            None
        };

        Ok(DCM {
            rot_mat: self.dcm3x3_from_topocentric_to_body_fixed()?.rot_mat,
            rot_mat_dt,
            from: uuid_from_epoch(self.frame.orientation_id, self.epoch),
            to: self.frame.orientation_id,
        })
    }

    /// Builds the rotation matrix that rotates from the topocentric frame (SEZ) into the body fixed frame of this state.
    ///
    /// # Frame warning
    /// If the state is NOT in a body fixed frame (i.e. ITRF93), then this computation is INVALID.
    ///
    /// # Source
    /// From the GMAT MathSpec, page 30 section 2.6.9 and from `Calculate_RFT` in `TopocentricAxes.cpp`, this returns the
    /// rotation matrix from the topocentric frame (SEZ) to body fixed frame.
    /// In the GMAT MathSpec notation, R_{IF} is the DCM from body fixed to inertial. Similarly, R{FT} is from topocentric
    /// to body fixed.
    ///
    /// :rtype: DCM
    pub fn dcm3x3_from_topocentric_to_body_fixed(&self) -> PhysicsResult<DCM> {
        if (self.radius_km.x.powi(2) + self.radius_km.y.powi(2)).sqrt() < 1e-3 {
            warn!("SEZ frame ill-defined when close to the poles");
        }
        let phi = self.latitude_deg()?.to_radians();
        let lambda = self.longitude_deg().to_radians();
        let z_hat = Vector3::new(
            phi.cos() * lambda.cos(),
            phi.cos() * lambda.sin(),
            phi.sin(),
        );
        // y_hat MUST be renormalized otherwise it's about 0.76 and therefore the rotation looses the norms conservation property.
        let mut y_hat = Vector3::new(0.0, 0.0, 1.0).cross(&z_hat);
        y_hat /= y_hat.norm();
        let x_hat = y_hat.cross(&z_hat);

        let rot_mat = Matrix3::new(
            x_hat[0], y_hat[0], z_hat[0], x_hat[1], y_hat[1], z_hat[1], x_hat[2], y_hat[2],
            z_hat[2],
        );

        Ok(DCM {
            rot_mat,
            rot_mat_dt: None,
            from: uuid_from_epoch(self.frame.orientation_id, self.epoch),
            to: self.frame.orientation_id,
        })
    }

    /// Builds the rotation matrix that rotates from this state's inertial frame to this state's RIC frame
    ///
    /// # Frame warning
    /// If the state is NOT in an inertial frame, then this computation is INVALID.
    ///
    /// # Algorithm
    /// 1. Compute the state data one millisecond before and one millisecond assuming two body dynamics
    /// 2. Compute the DCM for this state, and the pre and post states
    /// 3. Build the c vector as the normalized orbital momentum vector
    /// 4. Build the i vector as the cross product of \hat{r} and c
    /// 5. Build the RIC DCM as a 3x3 of the columns [\hat{r}, \hat{i}, \hat{c}], for the post, post, and current states
    /// 6. Compute the difference between the DCMs of the pre and post states, to build the DCM time derivative
    /// 7. Return the DCM structure with a 6x6 state DCM.
    ///
    /// # Note on the time derivative
    /// If the pre or post states cannot be computed, then the time derivative of the DCM will _not_ be set.
    /// Further note that most astrodynamics tools do *not* account for the time derivative in the RIC frame.
    ///
    /// :rtype: DCM
    pub fn dcm_from_ric_to_inertial(&self) -> PhysicsResult<DCM> {
        let rot_mat_dt = if let Ok(pre) = self.at_epoch(self.epoch - Unit::Millisecond * 1) {
            if let Ok(post) = self.at_epoch(self.epoch + Unit::Millisecond * 1) {
                let dcm_pre = pre.dcm3x3_from_ric_to_inertial()?;
                let dcm_post = post.dcm3x3_from_ric_to_inertial()?;
                Some(0.5 * dcm_post.rot_mat - 0.5 * dcm_pre.rot_mat)
            } else {
                None
            }
        } else {
            None
        };

        Ok(DCM {
            rot_mat: self.dcm3x3_from_ric_to_inertial()?.rot_mat,
            rot_mat_dt,
            from: uuid_from_epoch(self.frame.orientation_id, self.epoch),
            to: self.frame.orientation_id,
        })
    }

    /// Builds the rotation matrix that rotates from this state's inertial frame to this state's RIC frame
    ///
    /// # Frame warning
    /// If the state is NOT in an inertial frame, then this computation is INVALID.
    ///
    /// # Algorithm
    /// 1. Build the c vector as the normalized orbital momentum vector
    /// 2. Build the i vector as the cross product of \hat{r} and c
    /// 3. Build the RIC DCM as a 3x3 of the columns [\hat{r}, \hat{i}, \hat{c}]
    /// 4. Return the DCM structure **without** accounting for the transport theorem.
    ///
    /// :rtype: DCM
    pub fn dcm3x3_from_ric_to_inertial(&self) -> PhysicsResult<DCM> {
        let r_hat = self.r_hat();
        let c_hat = self.hvec()? / self.hmag()?;
        let i_hat = r_hat.cross(&c_hat);

        let rot_mat = Matrix3::from_columns(&[r_hat, i_hat, c_hat]);

        Ok(DCM {
            rot_mat,
            rot_mat_dt: None,
            from: uuid_from_epoch(self.frame.orientation_id, self.epoch),
            to: self.frame.orientation_id,
        })
    }

    /// Builds the rotation matrix that rotates from this state's inertial frame to this state's RCN frame (radial, cross, normal)
    ///
    /// # Frame warning
    /// If the stattion is NOT in an inertial frame, then this computation is INVALID.
    ///
    /// # Algorithm
    /// 1. Compute \hat{r}, \hat{h}, the unit vectors of the radius and orbital momentum.
    /// 2. Compute the cross product of these
    /// 3. Build the DCM with these unit vectors
    /// 4. Return the DCM structure
    ///
    /// :rtype: DCM
    pub fn dcm3x3_from_rcn_to_inertial(&self) -> PhysicsResult<DCM> {
        let r = self.r_hat();
        let n = self.hvec()? / self.hmag()?;
        let c = n.cross(&r);
        let rot_mat =
            Matrix3::new(r[0], r[1], r[2], c[0], c[1], c[2], n[0], n[1], n[2]).transpose();

        Ok(DCM {
            rot_mat,
            rot_mat_dt: None,
            from: uuid_from_epoch(self.frame.orientation_id, self.epoch),
            to: self.frame.orientation_id,
        })
    }

    /// Builds the rotation matrix that rotates from this state's inertial frame to this state's RCN frame (radial, cross, normal)
    ///
    /// # Frame warning
    /// If the stattion is NOT in an inertial frame, then this computation is INVALID.
    ///
    /// # Algorithm
    /// 1. Compute \hat{r}, \hat{h}, the unit vectors of the radius and orbital momentum.
    /// 2. Compute the cross product of these
    /// 3. Build the DCM with these unit vectors
    /// 4. Return the DCM structure with a 6x6 DCM with the time derivative of the VNC frame set.
    ///
    /// # Note on the time derivative
    /// If the pre or post states cannot be computed, then the time derivative of the DCM will _not_ be set.
    /// Further note that most astrodynamics tools do *not* account for the time derivative in the RIC frame.
    ///
    /// :rtype: DCM
    pub fn dcm_from_rcn_to_inertial(&self) -> PhysicsResult<DCM> {
        let rot_mat_dt = if let Ok(pre) = self.at_epoch(self.epoch - Unit::Millisecond * 1) {
            if let Ok(post) = self.at_epoch(self.epoch + Unit::Millisecond * 1) {
                let dcm_pre = pre.dcm3x3_from_rcn_to_inertial()?;
                let dcm_post = post.dcm3x3_from_rcn_to_inertial()?;
                Some(0.5 * dcm_post.rot_mat - 0.5 * dcm_pre.rot_mat)
            } else {
                None
            }
        } else {
            None
        };

        Ok(DCM {
            rot_mat: self.dcm3x3_from_rcn_to_inertial()?.rot_mat,
            rot_mat_dt,
            from: uuid_from_epoch(self.frame.orientation_id, self.epoch),
            to: self.frame.orientation_id,
        })
    }

    /// Builds the rotation matrix that rotates from this state's inertial frame to this state's VNC frame (velocity, normal, cross)
    ///
    /// # Frame warning
    /// If the stattion is NOT in an inertial frame, then this computation is INVALID.
    ///
    /// # Algorithm
    /// 1. Compute \hat{v}, \hat{h}, the unit vectors of the radius and orbital momentum.
    /// 2. Compute the cross product of these
    /// 3. Build the DCM with these unit vectors
    /// 4. Return the DCM structure.
    ///
    /// :rtype: DCM
    pub fn dcm3x3_from_vnc_to_inertial(&self) -> PhysicsResult<DCM> {
        let v = self.velocity_km_s / self.vmag_km_s();
        let n = self.hvec()? / self.hmag()?;
        let c = v.cross(&n);
        let rot_mat =
            Matrix3::new(v[0], v[1], v[2], n[0], n[1], n[2], c[0], c[1], c[2]).transpose();

        Ok(DCM {
            rot_mat,
            rot_mat_dt: None,
            from: uuid_from_epoch(self.frame.orientation_id, self.epoch),
            to: self.frame.orientation_id,
        })
    }

    /// Builds the rotation matrix that rotates from this state's inertial frame to this state's VNC frame (velocity, normal, cross)
    ///
    /// # Frame warning
    /// If the stattion is NOT in an inertial frame, then this computation is INVALID.
    ///
    /// # Algorithm
    /// 1. Compute \hat{v}, \hat{h}, the unit vectors of the radius and orbital momentum.
    /// 2. Compute the cross product of these
    /// 3. Build the DCM with these unit vectors
    /// 4. Compute the difference between the DCMs of the pre and post states (+/- 1 ms), to build the DCM time derivative
    /// 4. Return the DCM structure with a 6x6 DCM with the time derivative of the VNC frame set.
    ///
    /// # Note on the time derivative
    /// If the pre or post states cannot be computed, then the time derivative of the DCM will _not_ be set.
    /// Further note that most astrodynamics tools do *not* account for the time derivative in the RIC frame.
    ///
    /// :rtype: DCM
    pub fn dcm_from_vnc_to_inertial(&self) -> PhysicsResult<DCM> {
        let rot_mat_dt = if let Ok(pre) = self.at_epoch(self.epoch - Unit::Millisecond * 1) {
            if let Ok(post) = self.at_epoch(self.epoch + Unit::Millisecond * 1) {
                let dcm_pre = pre.dcm3x3_from_vnc_to_inertial()?;
                let dcm_post = post.dcm3x3_from_vnc_to_inertial()?;
                Some(0.5 * dcm_post.rot_mat - 0.5 * dcm_pre.rot_mat)
            } else {
                None
            }
        } else {
            None
        };

        Ok(DCM {
            rot_mat: self.dcm3x3_from_vnc_to_inertial()?.rot_mat,
            rot_mat_dt,
            from: uuid_from_epoch(self.frame.orientation_id, self.epoch),
            to: self.frame.orientation_id,
        })
    }

    /// Creates a new Orbit around the provided Celestial or Geoid frame from the Keplerian orbital elements.
    ///
    /// **Units:** km, none, degrees, degrees, degrees, degrees
    ///
    /// NOTE: The state is defined in Cartesian coordinates as they are non-singular. This causes rounding
    /// errors when creating a state from its Keplerian orbital elements (cf. the state tests).
    /// One should expect these errors to be on the order of 1e-12.
    ///
    /// :type sma_km: float
    /// :type ecc: float
    /// :type inc_deg: float
    /// :type raan_deg: float
    /// :type aop_deg: float
    /// :type ta_deg: float
    /// :type epoch: Epoch
    /// :type frame: Frame
    /// :rtype: Orbit
    #[cfg(feature = "python")]
    #[classmethod]
    pub fn from_keplerian(
        _cls: &Bound<'_, PyType>,
        sma_km: f64,
        ecc: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ta_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian(
            sma_km, ecc, inc_deg, raan_deg, aop_deg, ta_deg, epoch, frame,
        )
    }

    /// Attempts to create a new Orbit from the provided radii of apoapsis and periapsis, in kilometers
    ///
    /// :type r_a_km: float
    /// :type r_p_km: float
    /// :type inc_deg: float
    /// :type raan_deg: float
    /// :type aop_deg: float
    /// :type ta_deg: float
    /// :type epoch: Epoch
    /// :type frame: Frame
    /// :rtype: Orbit
    #[cfg(feature = "python")]
    #[classmethod]
    pub fn from_keplerian_apsis_radii(
        _cls: &Bound<'_, PyType>,
        r_a_km: f64,
        r_p_km: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ta_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian_apsis_radii(
            r_a_km, r_p_km, inc_deg, raan_deg, aop_deg, ta_deg, epoch, frame,
        )
    }

    /// Initializes a new orbit from the Keplerian orbital elements using the mean anomaly instead of the true anomaly.
    ///
    /// # Implementation notes
    /// This function starts by converting the mean anomaly to true anomaly, and then it initializes the orbit
    /// using the keplerian(..) method.
    /// The conversion is from GMAT's MeanToTrueAnomaly function, transliterated originally by Claude and GPT4 with human adjustments.
    ///
    /// :type sma_km: float
    /// :type ecc: float
    /// :type inc_deg: float
    /// :type raan_deg: float
    /// :type aop_deg: float
    /// :type ma_deg: float
    /// :type epoch: Epoch
    /// :type frame: Frame
    /// :rtype: Orbit
    #[cfg(feature = "python")]
    #[classmethod]
    pub fn from_keplerian_mean_anomaly(
        _cls: &Bound<'_, PyType>,
        sma_km: f64,
        ecc: f64,
        inc_deg: f64,
        raan_deg: f64,
        aop_deg: f64,
        ma_deg: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian_mean_anomaly(
            sma_km, ecc, inc_deg, raan_deg, aop_deg, ma_deg, epoch, frame,
        )
    }

    /// Returns the orbital momentum value on the X axis
    ///
    /// :rtype: float
    pub fn hx(&self) -> PhysicsResult<f64> {
        Ok(self.hvec()?[0])
    }

    /// Returns the orbital momentum value on the Y axis
    ///
    /// :rtype: float
    pub fn hy(&self) -> PhysicsResult<f64> {
        Ok(self.hvec()?[1])
    }

    /// Returns the orbital momentum value on the Z axis
    ///
    /// :rtype: float
    pub fn hz(&self) -> PhysicsResult<f64> {
        Ok(self.hvec()?[2])
    }

    /// Returns the norm of the orbital momentum
    ///
    /// :rtype: float
    pub fn hmag(&self) -> PhysicsResult<f64> {
        Ok(self.hvec()?.norm())
    }

    /// Returns the specific mechanical energy in km^2/s^2
    ///
    /// :rtype: float
    pub fn energy_km2_s2(&self) -> PhysicsResult<f64> {
        ensure!(
            self.rmag_km() > f64::EPSILON,
            RadiusSnafu {
                action: "cannot compute energy with zero radial state"
            }
        );
        Ok(self.vmag_km_s().powi(2) / 2.0 - self.frame.mu_km3_s2()? / self.rmag_km())
    }

    /// Returns the semi-major axis in km
    ///
    /// :rtype: float
    pub fn sma_km(&self) -> PhysicsResult<f64> {
        // Division by zero prevented in energy_km2_s2
        Ok(-self.frame.mu_km3_s2()? / (2.0 * self.energy_km2_s2()?))
    }

    /// Mutates this orbit to change the SMA
    ///
    /// :type new_sma_km: float
    /// :rtype: None
    pub fn set_sma_km(&mut self, new_sma_km: f64) -> PhysicsResult<()> {
        let me = Self::try_keplerian(
            new_sma_km,
            self.ecc()?,
            self.inc_deg()?,
            self.raan_deg()?,
            self.aop_deg()?,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        )?;

        *self = me;

        Ok(())
    }

    /// Returns a copy of the state with a new SMA
    ///
    /// :type new_sma_km: float
    /// :rtype: Orbit
    pub fn with_sma_km(&self, new_sma_km: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_sma_km(new_sma_km)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided SMA added to the current one
    ///
    /// :type delta_sma_km: float
    /// :rtype: Orbit
    pub fn add_sma_km(&self, delta_sma_km: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_sma_km(me.sma_km()? + delta_sma_km)?;
        Ok(me)
    }

    /// Returns the period in seconds
    ///
    /// :rtype: Duration
    pub fn period(&self) -> PhysicsResult<Duration> {
        Ok(2.0
            * PI
            * (self.sma_km()?.powi(3) / self.frame.mu_km3_s2()?)
                .sqrt()
                .seconds())
    }

    /// Returns the eccentricity (no unit)
    ///
    /// :rtype: float
    pub fn ecc(&self) -> PhysicsResult<f64> {
        Ok(self.evec()?.norm())
    }

    /// Mutates this orbit to change the ECC
    ///
    /// :type new_ecc: float
    /// :rtype: None
    pub fn set_ecc(&mut self, new_ecc: f64) -> PhysicsResult<()> {
        let me = Self::try_keplerian(
            self.sma_km()?,
            new_ecc,
            self.inc_deg()?,
            self.raan_deg()?,
            self.aop_deg()?,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        )?;

        *self = me;

        Ok(())
    }

    /// Returns a copy of the state with a new ECC
    ///
    /// :type new_ecc: float
    /// :rtype: Orbit
    pub fn with_ecc(&self, new_ecc: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_ecc(new_ecc)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided ECC added to the current one
    ///
    /// :type delta_ecc: float
    /// :rtype: Orbit
    pub fn add_ecc(&self, delta_ecc: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_ecc(me.ecc()? + delta_ecc)?;
        Ok(me)
    }

    /// Returns the inclination in degrees
    ///
    /// :rtype: float
    pub fn inc_deg(&self) -> PhysicsResult<f64> {
        Ok((self.hvec()?[2] / self.hmag()?).acos().to_degrees())
    }

    /// Mutates this orbit to change the INC
    ///
    /// :type new_inc_deg: float
    /// :rtype: None
    pub fn set_inc_deg(&mut self, new_inc_deg: f64) -> PhysicsResult<()> {
        let me = Self::try_keplerian(
            self.sma_km()?,
            self.ecc()?,
            new_inc_deg,
            self.raan_deg()?,
            self.aop_deg()?,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        )?;

        *self = me;

        Ok(())
    }

    /// Returns a copy of the state with a new INC
    ///
    /// :type new_inc_deg: float
    /// :rtype: Orbit
    pub fn with_inc_deg(&self, new_inc_deg: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_inc_deg(new_inc_deg)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided INC added to the current one
    ///
    /// :type delta_inc_deg: float
    /// :rtype: None
    pub fn add_inc_deg(&self, delta_inc_deg: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_inc_deg(me.inc_deg()? + delta_inc_deg)?;
        Ok(me)
    }

    /// Returns the argument of periapsis in degrees
    ///
    /// :rtype: float
    pub fn aop_deg(&self) -> PhysicsResult<f64> {
        let n = Vector3::new(0.0, 0.0, 1.0).cross(&self.hvec()?);
        let cos_aop = n.dot(&self.evec()?) / (n.norm() * self.ecc()?);
        let aop = cos_aop.acos();
        if aop.is_nan() {
            if cos_aop > 1.0 {
                Ok(180.0)
            } else {
                Ok(0.0)
            }
        } else if self.evec()?[2] < 0.0 {
            Ok((2.0 * PI - aop).to_degrees())
        } else {
            Ok(aop.to_degrees())
        }
    }

    /// Mutates this orbit to change the AOP
    ///
    /// :type new_aop_deg: float
    /// :rtype: None
    pub fn set_aop_deg(&mut self, new_aop_deg: f64) -> PhysicsResult<()> {
        let me = Self::try_keplerian(
            self.sma_km()?,
            self.ecc()?,
            self.inc_deg()?,
            self.raan_deg()?,
            new_aop_deg,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        )?;

        *self = me;

        Ok(())
    }

    /// Returns a copy of the state with a new AOP
    ///
    /// :type new_aop_deg: float
    /// :rtype: Orbit
    pub fn with_aop_deg(&self, new_aop_deg: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_aop_deg(new_aop_deg)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided AOP added to the current one
    ///
    /// :type delta_aop_deg: float
    /// :rtype: Orbit
    pub fn add_aop_deg(&self, delta_aop_deg: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_aop_deg(me.aop_deg()? + delta_aop_deg)?;
        Ok(me)
    }

    /// Returns the right ascension of the ascending node in degrees
    ///
    /// :rtype: float
    pub fn raan_deg(&self) -> PhysicsResult<f64> {
        let n = Vector3::new(0.0, 0.0, 1.0).cross(&self.hvec()?);
        let cos_raan = n[0] / n.norm();
        let raan = cos_raan.acos();
        if raan.is_nan() {
            if cos_raan > 1.0 {
                Ok(180.0)
            } else {
                Ok(0.0)
            }
        } else if n[1] < 0.0 {
            Ok((2.0 * PI - raan).to_degrees())
        } else {
            Ok(raan.to_degrees())
        }
    }

    /// Mutates this orbit to change the RAAN
    ///
    /// :type new_raan_deg: float
    /// :rtype: None
    pub fn set_raan_deg(&mut self, new_raan_deg: f64) -> PhysicsResult<()> {
        let me = Self::try_keplerian(
            self.sma_km()?,
            self.ecc()?,
            self.inc_deg()?,
            new_raan_deg,
            self.aop_deg()?,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        )?;

        *self = me;

        Ok(())
    }

    /// Returns a copy of the state with a new RAAN
    ///
    /// :type new_raan_deg: float
    /// :rtype: Orbit
    pub fn with_raan_deg(&self, new_raan_deg: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_raan_deg(new_raan_deg)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided RAAN added to the current one
    ///
    /// :type delta_raan_deg: float
    /// :rtype: Orbit
    pub fn add_raan_deg(&self, delta_raan_deg: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_raan_deg(me.raan_deg()? + delta_raan_deg)?;
        Ok(me)
    }

    /// Returns the true anomaly in degrees between 0 and 360.0
    ///
    /// NOTE: This function will emit a warning stating that the TA should be avoided if in a very near circular orbit
    /// Code from <https://github.com/ChristopherRabotin/GMAT/blob/80bde040e12946a61dae90d9fc3538f16df34190/src/gmatutil/util/StateConversionUtil.cpp#L6835>
    ///
    /// LIMITATION: For an orbit whose true anomaly is (very nearly) 0.0 or 180.0, this function may return either 0.0 or 180.0 with a very small time increment.
    /// This is due to the precision of the cosine calculation: if the arccosine calculation is out of bounds, the sign of the cosine of the true anomaly is used
    /// to determine whether the true anomaly should be 0.0 or 180.0. **In other words**, there is an ambiguity in the computation in the true anomaly exactly at 180.0 and 0.0.
    ///
    /// :rtype: float
    pub fn ta_deg(&self) -> PhysicsResult<f64> {
        if self.ecc()? < ECC_EPSILON {
            warn!(
                "true anomaly ill-defined for circular orbit (e = {})",
                self.ecc()?
            );
        }
        let cos_nu = self.evec()?.dot(&self.radius_km) / (self.ecc()? * self.rmag_km());
        // If we're close the valid bounds, let's just do a sign check and return the true anomaly
        let ta = cos_nu.acos();
        if ta.is_nan() {
            if cos_nu > 1.0 {
                Ok(180.0)
            } else {
                Ok(0.0)
            }
        } else if self.radius_km.dot(&self.velocity_km_s) < 0.0 {
            Ok((2.0 * PI - ta).to_degrees())
        } else {
            Ok(ta.to_degrees())
        }
    }

    /// Mutates this orbit to change the TA
    ///
    /// :type new_ta_deg: float
    /// :rtype: None
    pub fn set_ta_deg(&mut self, new_ta_deg: f64) -> PhysicsResult<()> {
        let me = Self::try_keplerian(
            self.sma_km()?,
            self.ecc()?,
            self.inc_deg()?,
            self.raan_deg()?,
            self.aop_deg()?,
            new_ta_deg,
            self.epoch,
            self.frame,
        )?;

        *self = me;

        Ok(())
    }

    /// Returns the time derivative of the true anomaly computed as the 360.0 degrees divided by the orbital period (in seconds).
    ///
    /// :rtype: float
    pub fn ta_dot_deg_s(&self) -> PhysicsResult<f64> {
        Ok(360.0 / self.period()?.to_seconds())
    }

    /// Returns a copy of the state with a new TA
    ///
    /// :type new_ta_deg: float
    /// :rtype: Orbit
    pub fn with_ta_deg(&self, new_ta_deg: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_ta_deg(new_ta_deg)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided TA added to the current one
    ///
    /// :type delta_ta_deg: float
    /// :rtype: Orbit
    pub fn add_ta_deg(&self, delta_ta_deg: f64) -> PhysicsResult<Self> {
        let mut me = *self;
        me.set_ta_deg(me.ta_deg()? + delta_ta_deg)?;
        Ok(me)
    }

    /// Returns a copy of this state with the provided apoasis and periapsis
    ///
    /// :type new_ra_km: float
    /// :type new_rp_km: float
    /// :rtype: Orbit
    pub fn with_apoapsis_periapsis_km(
        &self,
        new_ra_km: f64,
        new_rp_km: f64,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian_apsis_radii(
            new_ra_km,
            new_rp_km,
            self.inc_deg()?,
            self.raan_deg()?,
            self.aop_deg()?,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        )
    }

    /// Returns a copy of this state with the provided apoasis and periapsis added to the current values
    ///
    /// :type delta_ra_km: float
    /// :type delta_rp_km: float
    /// :rtype: Orbit
    pub fn add_apoapsis_periapsis_km(
        &self,
        delta_ra_km: f64,
        delta_rp_km: f64,
    ) -> PhysicsResult<Self> {
        Self::try_keplerian_apsis_radii(
            self.apoapsis_km()? + delta_ra_km,
            self.periapsis_km()? + delta_rp_km,
            self.inc_deg()?,
            self.raan_deg()?,
            self.aop_deg()?,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        )
    }

    /// Returns the true longitude in degrees
    ///
    /// :rtype: float
    pub fn tlong_deg(&self) -> PhysicsResult<f64> {
        // Angles already in degrees
        Ok(between_0_360(
            self.aop_deg()? + self.raan_deg()? + self.ta_deg()?,
        ))
    }

    /// Returns the argument of latitude in degrees
    ///
    /// NOTE: If the orbit is near circular, the AoL will be computed from the true longitude
    /// instead of relying on the ill-defined true anomaly.
    ///
    /// :rtype: float
    pub fn aol_deg(&self) -> PhysicsResult<f64> {
        Ok(between_0_360(if self.ecc()? < ECC_EPSILON {
            self.tlong_deg()? - self.raan_deg()?
        } else {
            self.aop_deg()? + self.ta_deg()?
        }))
    }

    /// Returns the radius of periapsis (or perigee around Earth), in kilometers.
    ///
    /// :rtype: float
    pub fn periapsis_km(&self) -> PhysicsResult<f64> {
        Ok(self.sma_km()? * (1.0 - self.ecc()?))
    }

    /// Returns the radius of apoapsis (or apogee around Earth), in kilometers.
    ///
    /// :rtype: float
    pub fn apoapsis_km(&self) -> PhysicsResult<f64> {
        Ok(self.sma_km()? * (1.0 + self.ecc()?))
    }

    /// Returns the eccentric anomaly in degrees
    ///
    /// This is a conversion from GMAT's StateConversionUtil::TrueToEccentricAnomaly
    ///
    /// :rtype: float
    pub fn ea_deg(&self) -> PhysicsResult<f64> {
        let (sin_ta, cos_ta) = self.ta_deg()?.to_radians().sin_cos();
        let ecc_cos_ta = self.ecc()? * cos_ta;
        let sin_ea = ((1.0 - self.ecc()?.powi(2)).sqrt() * sin_ta) / (1.0 + ecc_cos_ta);
        let cos_ea = (self.ecc()? + cos_ta) / (1.0 + ecc_cos_ta);
        // The atan2 function is a bit confusing: https://doc.rust-lang.org/std/primitive.f64.html#method.atan2 .
        Ok(sin_ea.atan2(cos_ea).to_degrees())
    }

    /// Returns the flight path angle in degrees
    ///
    /// :rtype: float
    pub fn fpa_deg(&self) -> PhysicsResult<f64> {
        let nu = self.ta_deg()?.to_radians();
        let ecc = self.ecc()?;
        let denom = (1.0 + 2.0 * ecc * nu.cos() + ecc.powi(2)).sqrt();
        let sin_fpa = ecc * nu.sin() / denom;
        let cos_fpa = 1.0 + ecc * nu.cos() / denom;
        Ok(sin_fpa.atan2(cos_fpa).to_degrees())
    }

    /// Returns the mean anomaly in degrees
    ///
    /// This is a conversion from GMAT's StateConversionUtil::TrueToMeanAnomaly
    ///
    /// :rtype: float
    pub fn ma_deg(&self) -> PhysicsResult<f64> {
        if self.ecc()?.abs() < ECC_EPSILON {
            Err(PhysicsError::ParabolicEccentricity { limit: ECC_EPSILON })
        } else if self.ecc()? < 1.0 {
            Ok(between_0_360(
                (self.ea_deg()?.to_radians() - self.ecc()? * self.ea_deg()?.to_radians().sin())
                    .to_degrees(),
            ))
        } else {
            // From GMAT's TrueToHyperbolicAnomaly
            Ok(
                ((self.ta_deg()?.to_radians().sin() * (self.ecc()?.powi(2) - 1.0)).sqrt()
                    / (1.0 + self.ecc()? * self.ta_deg()?.to_radians().cos()))
                .asinh()
                .to_degrees(),
            )
        }
    }

    /// Returns the semi parameter (or semilatus rectum)
    ///
    /// :rtype: float
    pub fn semi_parameter_km(&self) -> PhysicsResult<f64> {
        Ok(self.sma_km()? * (1.0 - self.ecc()?.powi(2)))
    }

    /// Returns whether this state satisfies the requirement to compute the Mean Brouwer Short orbital
    /// element set.
    ///
    /// This is a conversion from GMAT's StateConversionUtil::CartesianToBrouwerMeanShort.
    /// The details are at the log level `info`.
    /// NOTE: Mean Brouwer Short are only defined around Earth. However, `nyx` does *not* check the
    /// main celestial body around which the state is defined (GMAT does perform this verification).
    ///
    /// :rtype: bool
    pub fn is_brouwer_short_valid(&self) -> PhysicsResult<bool> {
        if self.inc_deg()? > 180.0 {
            info!("Brouwer Mean Short only applicable for inclinations less than 180.0");
            Ok(false)
        } else if self.ecc()? >= 1.0 || self.ecc()? < 0.0 {
            info!("Brouwer Mean Short only applicable for elliptical orbits");
            Ok(false)
        } else if self.periapsis_km()? < 3000.0 {
            // NOTE: GMAT emits a warning if the periagee is less than the Earth radius, but we do not do that here.
            info!("Brouwer Mean Short only applicable for if perigee is greater than 3000 km");
            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// Returns the right ascension of this orbit in degrees
    ///
    /// :rtype: float
    pub fn right_ascension_deg(&self) -> f64 {
        between_0_360((self.radius_km.y.atan2(self.radius_km.x)).to_degrees())
    }

    /// Returns the declination of this orbit in degrees
    ///
    /// :rtype: float
    pub fn declination_deg(&self) -> f64 {
        between_pm_180((self.radius_km.z / self.rmag_km()).asin().to_degrees())
    }

    /// Returns the semi minor axis in km, includes code for a hyperbolic orbit
    ///
    /// :rtype: float
    pub fn semi_minor_axis_km(&self) -> PhysicsResult<f64> {
        if self.ecc()? <= 1.0 {
            Ok(((self.sma_km()? * self.ecc()?).powi(2) - self.sma_km()?.powi(2)).sqrt())
        } else {
            Ok(self.hmag()?.powi(2)
                / (self.frame.mu_km3_s2()? * (self.ecc()?.powi(2) - 1.0).sqrt()))
        }
    }

    /// Returns the velocity declination of this orbit in degrees
    ///
    /// :rtype: float
    pub fn velocity_declination_deg(&self) -> f64 {
        between_pm_180(
            (self.velocity_km_s.z / self.vmag_km_s())
                .asin()
                .to_degrees(),
        )
    }

    /// Returns the $C_3$ of this orbit in km^2/s^2
    ///
    /// :rtype: float
    pub fn c3_km2_s2(&self) -> PhysicsResult<f64> {
        Ok(-self.frame.mu_km3_s2()? / self.sma_km()?)
    }

    /// Returns the radius of periapse in kilometers for the provided turn angle of this hyperbolic orbit.
    /// Returns an error if the orbit is not hyperbolic.
    ///
    /// :type turn_angle_degrees: float
    /// :rtype: float
    pub fn vinf_periapsis_km(&self, turn_angle_degrees: f64) -> PhysicsResult<f64> {
        let ecc = self.ecc()?;
        if ecc <= 1.0 {
            Err(PhysicsError::NotHyperbolic {
                ecc: self.ecc().unwrap(),
            })
        } else {
            let cos_rho = (0.5 * (PI - turn_angle_degrees.to_radians())).cos();
            Ok((1.0 / cos_rho - 1.0) * self.frame.mu_km3_s2()? / self.vmag_km_s().powi(2))
        }
    }

    /// Returns the turn angle in degrees for the provided radius of periapse passage of this hyperbolic orbit
    /// Returns an error if the orbit is not hyperbolic.
    ///
    /// :type periapsis_km: float
    /// :rtype: float
    pub fn vinf_turn_angle_deg(&self, periapsis_km: f64) -> PhysicsResult<f64> {
        let ecc = self.ecc()?;
        if ecc <= 1.0 {
            Err(PhysicsError::NotHyperbolic {
                ecc: self.ecc().unwrap(),
            })
        } else {
            let rho = (1.0
                / (1.0 + self.vmag_km_s().powi(2) * (periapsis_km / self.frame.mu_km3_s2()?)))
            .acos();
            Ok(between_0_360((PI - 2.0 * rho).to_degrees()))
        }
    }

    /// Returns the hyperbolic anomaly in degrees between 0 and 360.0
    /// Returns an error if the orbit is not hyperbolic.
    ///
    /// :rtype: float
    pub fn hyperbolic_anomaly_deg(&self) -> PhysicsResult<f64> {
        let ecc = self.ecc()?;
        if ecc <= 1.0 {
            Err(PhysicsError::NotHyperbolic {
                ecc: self.ecc().unwrap(),
            })
        } else {
            let (sin_ta, cos_ta) = self.ta_deg()?.to_radians().sin_cos();
            let sinh_h = (sin_ta * (ecc.powi(2) - 1.0).sqrt()) / (1.0 + ecc * cos_ta);
            Ok(between_0_360(sinh_h.asinh().to_degrees()))
        }
    }

    /// Adjusts the true anomaly of this orbit using the mean anomaly.
    ///
    /// # Astrodynamics note
    /// This is not a true propagation of the orbit. This is akin to a two body propagation ONLY without any other force models applied.
    /// Use Nyx for high fidelity propagation.
    ///
    /// :type new_epoch: Epoch
    /// :rtype: Orbit
    pub fn at_epoch(&self, new_epoch: Epoch) -> PhysicsResult<Self> {
        let m0_rad = self.ma_deg()?.to_radians();
        let mt_rad = m0_rad
            + (self.frame.mu_km3_s2()? / self.sma_km()?.powi(3)).sqrt()
                * (new_epoch - self.epoch).to_seconds();

        Self::try_keplerian_mean_anomaly(
            self.sma_km()?,
            self.ecc()?,
            self.inc_deg()?,
            self.raan_deg()?,
            self.aop_deg()?,
            mt_rad.to_degrees(),
            new_epoch,
            self.frame,
        )
    }

    /// Calculates the duration to reach a specific radius in the orbit.
    ///
    /// This function computes the time it will take for the orbiting body to reach
    /// the given `radius_km` from its current position. The calculation assumes
    /// two-body dynamics and considers the direction of motion.
    ///
    /// # Arguments
    ///
    /// * `radius_km` - The target radius from the central body in kilometers.
    ///
    /// # Returns
    ///
    /// A `PhysicsResult<Duration>` which is:
    /// - `Ok(Duration)`: The time duration to reach the target radius. This can be zero
    ///   if the orbit is circular and already at the target radius, or if the target
    ///   radius is very close to the current radius in the direction of motion.
    /// - `Err(PhysicsError)`: If the target radius is invalid (e.g., non-positive),
    ///   unreachable for the given orbit (e.g., outside apoapsis/periapsis limits
    ///   for an elliptical orbit, or below periapsis for a hyperbolic/parabolic one),
    ///   or if any other calculation error occurs.
    ///
    /// # Logic Details
    ///
    /// 1.  Retrieves orbital parameters (eccentricity, mu, SMA, periapsis, apoapsis, semi-latus rectum).
    /// 2.  Validates `radius_km` (must be positive).
    /// 3.  Handles circular/near-circular orbits separately: if the target radius matches the orbit's
    ///     radius, duration is zero; otherwise, it's an error.
    /// 4.  Checks if the `radius_km` is reachable within the orbit's geometry (between periapsis and
    ///     apoapsis for elliptical, above periapsis for hyperbolic/parabolic).
    /// 5.  Calculates the true anomaly (`nu_rad_at_radius`) corresponding to `radius_km`. This calculation
    ///     yields a `nu` in `[0, PI]`.
    /// 6.  Converts `nu_rad_at_radius` to mean anomaly (`m_rad_at_radius`) using `astro::utils`.
    /// 7.  Gets the current mean anomaly (`m_current_rad`).
    /// 8.  Calculates mean motion (`n_rad_s`).
    /// 9.  Computes time from periapsis to the target radius (`t_from_p_to_radius_s`) and to the
    ///     current position (`t_from_p_to_current_s`).
    /// 10. The initial time difference `delta_t_s` is `t_from_p_to_radius_s - t_from_p_to_current_s`.
    /// 11. Adjusts `delta_t_s`:
    ///     - If `delta_t_s` is significantly negative:
    ///         - For elliptical orbits, a full period is added (assuming the radius will be reached
    ///           in the next orbit, as `nu_rad_at_radius` is on the "advancing" half `[0, PI]`).
    ///         - For hyperbolic/parabolic orbits, it's an error because the radius on this specific
    ///           `[0, PI]` true anomaly arc was in the past and won't be met again on this arc.
    ///     - If `delta_t_s` is slightly negative (close to zero), it's set to `0.0`.
    /// 12. Returns the calculated `delta_t_s` as a `Duration`.
    ///
    /// # Assumptions & Limitations
    ///
    /// - Assumes two-body problem (Keplerian motion). No perturbations are considered.
    /// - For elliptical orbits, if the radius is reachable at two points (ascending and descending parts
    ///   of the orbit), this function calculates the time to reach the radius corresponding to the
    ///   true anomaly in `[0, PI]` (typically the ascending part or up to apoapsis if starting past periapsis).
    /// - For hyperbolic/parabolic orbits, the true anomaly at radius is also computed in `[0, PI]`. If this
    ///   point is in the past, the function returns an error, as it doesn't look for solutions on the
    ///   departing leg if `nu > PI` would be required (unless current TA is already > PI and target radius is further along).
    ///   The current implementation strictly uses the `acos` result, so `nu_rad_at_radius` is always `0 <= nu <= PI`.
    ///   This means it finds the time to reach the radius on the path from periapsis up to the point where true anomaly is PI.
    pub fn duration_to_radius(&self, radius_km: f64) -> PhysicsResult<Duration> {
        // Local constants as per subtask instructions
        let dist_epsilon_val: f64 = 1e-7;
        let similar_to_zero_val: f64 = 1e-9;
        let angle_epsilon_val: f64 = 1e-9;

        // Necessary imports (already present at file-level, but can be listed here for clarity if needed by tool/linter)
        // use crate::astro::utils; // Implicitly used via crate::astro::utils
        // use hifitime::Duration; // Already imported at file level
        // use crate::errors::{PhysicsError, PhysicsResult, RadiusError, InfiniteValue}; // PhysicsResult is return type, errors used with ensure!
        // use snafu::ensure; // Already imported at file level

        // Step 1: Retrieve eccentricity
        let ecc = self.ecc()?;

        // Step 2: Pre-condition check for radius_km
        ensure!(
            radius_km > dist_epsilon_val,
            RadiusError {
                action: "Target radius must be positive"
            }
        );

        // Step 3: Handle circular/near-circular orbits
        if ecc < ECC_EPSILON {
            let current_orbit_radius = self.sma_km()?; // For circular, sma is the radius
            ensure!(
                (radius_km - current_orbit_radius).abs() < dist_epsilon_val,
                RadiusError {
                    action: "Target radius for circular orbit must be (nearly) equal to orbit radius"
                }
            );
            return Ok(Duration::from_seconds(0.0));
        }

        // Step 4: Retrieve mu and sma
        let mu_km3_s2 = self.frame.mu_km3_s2()?;
        let sma_km = self.sma_km()?;

        // Step 5: Perform reachability checks
        let rp_km = self.periapsis_km()?;
        if ecc < 1.0 {
            // Elliptical
            let ra_km = self.apoapsis_km()?;
            ensure!(
                radius_km >= rp_km - dist_epsilon_val && radius_km <= ra_km + dist_epsilon_val,
                RadiusError {
                    action: "Radius outside reachable range for elliptical orbit"
                }
            );
        } else {
            // Hyperbolic/Parabolic
            ensure!(
                radius_km >= rp_km - dist_epsilon_val,
                RadiusError {
                    action: "Radius below periapsis for hyperbolic/parabolic orbit"
                }
            );
        }

        // Step 6: Retrieve semi-latus rectum
        let p_km = self.semi_parameter_km()?;

        // Step 7: Calculate cos_nu_val
        let cos_nu_val = (p_km / radius_km - 1.0) / ecc;

        // Step 8: Validate and clamp cos_nu_val
        ensure!(
            cos_nu_val >= -1.0 - angle_epsilon_val && cos_nu_val <= 1.0 + angle_epsilon_val,
            RadiusError {
                action: "Cannot compute true anomaly: cos(nu) out of bounds"
            }
        );
        let cos_nu_rad_at_radius = cos_nu_val.clamp(-1.0, 1.0);

        // Step 9: Calculate true anomaly at radius
        let nu_rad_at_radius = cos_nu_rad_at_radius.acos(); // Result in [0, PI]

        // Step 10: Calculate mean anomaly at target radius
        let m_rad_at_radius =
            crate::astro::utils::true_anomaly_to_mean_anomaly_rad(nu_rad_at_radius, ecc)
                .map_err(|e| PhysicsError::AppliedMath { source: e })?;

        // Step 11: Get current mean anomaly
        let m_current_rad = self.ma_deg()?.to_radians();

        // Step 12: Calculate mean motion n_rad_s
        let n_rad_s = (mu_km3_s2 / sma_km.abs().powi(3)).sqrt();
        ensure!(
            n_rad_s.is_finite() && n_rad_s > 0.0,
            InfiniteValue {
                action: "Mean motion calculation failed (non-finite or non-positive)"
            }
        );

        // Step 13: Calculate time from periapsis to target radius and current position
        let t_from_p_to_radius_s = m_rad_at_radius / n_rad_s;
        let t_from_p_to_current_s = m_current_rad / n_rad_s;

        // Step 14: Calculate initial delta time
        let mut delta_t_s = t_from_p_to_radius_s - t_from_p_to_current_s;

        // Step 15: Adjust delta_t_s
        if delta_t_s < -similar_to_zero_val {
            if ecc < 1.0 {
                // Elliptical: target radius (on the 0->PI true anomaly arc) will be reached in the next orbit.
                delta_t_s += self.period()?.to_seconds();
            } else {
                // Hyperbolic/Parabolic: nu_rad_at_radius is on the [0, PI] arc.
                // This specific point (on the 0->PI arc) is in the past.
                return RadiusError {
                    action: "Radius (on [0,PI] TA arc) in past for hyperbolic orbit",
                }
                .fail();
            }
        } else if delta_t_s < 0.0 {
            // If delta_t_s is negative but very close to zero (-similar_to_zero_val < delta_t_s < 0.0)
            delta_t_s = 0.0;
        }

        // Step 16: Return Ok(Duration::from_seconds(delta_t_s))
        Ok(Duration::from_seconds(delta_t_s))
    }

    /// Returns a Cartesian state representing the RIC difference between self and other, in position and velocity (with transport theorem).
    /// Refer to dcm_from_ric_to_inertial for details on the RIC frame.
    ///
    /// # Algorithm
    /// 1. Compute the RIC DCM of self
    /// 2. Rotate self into the RIC frame
    /// 3. Rotation other into the RIC frame
    /// 4. Compute the difference between these two states
    /// 5. Strip the astrodynamical information from the frame, enabling only computations from `CartesianState`
    ///
    /// :type other: Orbit
    /// :rtype: Orbit
    pub fn ric_difference(&self, other: &Self) -> PhysicsResult<Self> {
        let self_in_ric = (self.dcm_from_ric_to_inertial()?.transpose() * self)?;
        let other_in_ric = (self.dcm_from_ric_to_inertial()?.transpose() * other)?;
        let mut rslt = (self_in_ric - other_in_ric)?;
        rslt.frame.strip();
        Ok(rslt)
    }

    /// Returns a Cartesian state representing the VNC difference between self and other, in position and velocity (with transport theorem).
    /// Refer to dcm_from_vnc_to_inertial for details on the VNC frame.
    ///
    /// # Algorithm
    /// 1. Compute the VNC DCM of self
    /// 2. Rotate self into the VNC frame
    /// 3. Rotation other into the VNC frame
    /// 4. Compute the difference between these two states
    /// 5. Strip the astrodynamical information from the frame, enabling only computations from `CartesianState`
    ///
    /// :type other: Orbit
    /// :rtype: Orbit
    pub fn vnc_difference(&self, other: &Self) -> PhysicsResult<Self> {
        let self_in_vnc = (self.dcm_from_vnc_to_inertial()?.transpose() * self)?;
        let other_in_vnc = (self.dcm_from_vnc_to_inertial()?.transpose() * other)?;
        let mut rslt = (self_in_vnc - other_in_vnc)?;
        rslt.frame.strip();
        Ok(rslt)
    }
}

#[allow(clippy::format_in_format_args)]
impl fmt::LowerHex for Orbit {
    /// Prints the Keplerian orbital elements in floating point with units if frame is celestial,
    /// If frame is geodetic, prints the range, altitude, latitude, and longitude with respect to the planetocentric frame
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.frame.is_celestial() {
            error!("you must update the frame from the Almanac before printing this state's orbital parameters");
            Err(fmt::Error)
        } else {
            let decimals = f.precision().unwrap_or(6);

            write!(
                f,
                "[{:x}] {}\tsma = {} km\tecc = {}\tinc = {} deg\traan = {} deg\taop = {} deg\tta = {} deg",
                self.frame,
                self.epoch,
                format!("{:.*}", decimals, self.sma_km().map_err(|err| {
                    error!("{err}");
                    fmt::Error
                })?),
                format!("{:.*}", decimals, self.ecc().map_err(|err| {
                    error!("{err}");
                    fmt::Error
                })?),
                format!("{:.*}", decimals, self.inc_deg().map_err(|err| {
                    error!("{err}");
                    fmt::Error
                })?),
                format!("{:.*}", decimals, self.raan_deg().map_err(|err| {
                    error!("{err}");
                    fmt::Error
                })?),
                format!("{:.*}", decimals, self.aop_deg().map_err(|err| {
                    error!("{err}");
                    fmt::Error
                })?),
                format!("{:.*}", decimals, self.ta_deg().map_err(|err| {
                    error!("{err}");
                    fmt::Error
                })?),
            )
        }
    }
}

#[allow(clippy::format_in_format_args)]
impl fmt::UpperHex for Orbit {
    /// Prints the prints the range, altitude, latitude, and longitude with respect to the planetocentric frame in floating point with units if frame is celestial,
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.frame.is_geodetic() {
            error!("you must update the frame from the Almanac before printing this state's planetocentric parameters");
            Err(fmt::Error)
        } else {
            let decimals = f.precision().unwrap_or(3);
            write!(
                f,
                "[{:x}] {}\trange = {} km\talt. = {} km\tlatitude = {} deg\tlongitude = {} deg",
                self.frame,
                self.epoch,
                format!("{:.*}", decimals, self.rmag_km()),
                format!(
                    "{:.*}",
                    decimals,
                    self.height_km().map_err(|err| {
                        error!("{err}");
                        fmt::Error
                    })?
                ),
                format!(
                    "{:.*}",
                    decimals,
                    self.latitude_deg().map_err(|err| {
                        error!("{err}");
                        fmt::Error
                    })?
                ),
                format!("{:.*}", decimals, self.longitude_deg()),
            )
        }
    }
}
