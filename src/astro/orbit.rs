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
    errors::{
        HyperbolicTrueAnomalySnafu, InfiniteValueSnafu, ParabolicEccentricitySnafu,
        ParabolicSemiParamSnafu, PhysicsError, RadiusSnafu,
    },
    math::{
        angles::{between_0_360, between_pm_180},
        cartesian::CartesianState,
        Vector3, Vector6,
    },
    prelude::Frame,
};
use core::f64::consts::PI;
use core::f64::EPSILON;
use core::fmt;
use hifitime::{Duration, Epoch, TimeUnits};
use log::{error, info, warn};
use snafu::ensure;

/// If an orbit has an eccentricity below the following value, it is considered circular (only affects warning messages)
pub const ECC_EPSILON: f64 = 1e-11;

/// A helper type alias, but no assumptions are made on the underlying validity of the frame.
pub type Orbit = CartesianState;

impl CartesianState {
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
        sma: f64,
        ecc: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        let mu_km3_s2 = frame.mu_km3_s2()?;
        if mu_km3_s2.abs() < EPSILON {
            warn!("GM is near zero ({mu_km3_s2} km^3/s^2): expect rounding errors!",);
        }
        // Algorithm from GMAT's StateConversionUtil::KeplerianToCartesian
        let ecc = if ecc < 0.0 {
            warn!("eccentricity cannot be negative: sign of eccentricity changed");
            ecc * -1.0
        } else {
            ecc
        };
        let sma = if ecc > 1.0 && sma > 0.0 {
            warn!("eccentricity > 1 (hyperbolic) BUT SMA > 0 (elliptical): sign of SMA changed");
            sma * -1.0
        } else if ecc < 1.0 && sma < 0.0 {
            warn!("eccentricity < 1 (elliptical) BUT SMA < 0 (hyperbolic): sign of SMA changed");
            sma * -1.0
        } else {
            sma
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
            let ta_deg = between_0_360(ta);
            ensure!(
                ta_deg <= (PI - (1.0 / ecc).acos()).to_degrees(),
                HyperbolicTrueAnomalySnafu { ta_deg }
            );
        }
        ensure!(
            (1.0 + ecc * ta.to_radians().cos()).is_finite(),
            InfiniteValueSnafu {
                action: "computing radius of orbit"
            }
        );

        // Done with all the warnings and errors supported by GMAT
        // The conversion algorithm itself comes from GMAT's StateConversionUtil::ComputeKeplToCart
        // NOTE: GMAT supports mean anomaly instead of true anomaly, but only for backward compatibility reasons
        // so it isn't supported here.
        let inc = inc.to_radians();
        let raan = raan.to_radians();
        let aop = aop.to_radians();
        let ta = ta.to_radians();
        let p = sma * (1.0 - ecc.powi(2));

        ensure!(p.abs() >= EPSILON, ParabolicSemiParamSnafu { p });

        // NOTE: At this point GMAT computes 1+ecc**2 and checks whether it's very small.
        // It then reports that the radius may be too large. We've effectively already done
        // this check above (and panicked if needed), so it isn't repeated here.
        let radius = p / (1.0 + ecc * ta.cos());
        let (sin_aop_ta, cos_aop_ta) = (aop + ta).sin_cos();
        let (sin_inc, cos_inc) = inc.sin_cos();
        let (sin_raan, cos_raan) = raan.sin_cos();
        let (sin_aop, cos_aop) = aop.sin_cos();
        let x = radius * (cos_aop_ta * cos_raan - cos_inc * sin_aop_ta * sin_raan);
        let y = radius * (cos_aop_ta * sin_raan + cos_inc * sin_aop_ta * cos_raan);
        let z = radius * sin_aop_ta * sin_inc;
        let sqrt_gm_p = (mu_km3_s2 / p).sqrt();
        let cos_ta_ecc = ta.cos() + ecc;
        let sin_ta = ta.sin();

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
        r_a: f64,
        r_p: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> PhysicsResult<Self> {
        ensure!(
            r_a > EPSILON,
            RadiusSnafu {
                action: "radius of apoapsis is negative"
            }
        );
        ensure!(
            r_p > EPSILON,
            RadiusSnafu {
                action: "radius of periapsis is negative"
            }
        );
        // The two checks above ensure that sma > 0
        let sma = (r_a + r_p) / 2.0;
        let ecc = r_a / sma - 1.0;
        Self::try_keplerian(sma, ecc, inc, raan, aop, ta, epoch, frame)
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
        sma: f64,
        ecc: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> Self {
        Self::try_keplerian(sma, ecc, inc, raan, aop, ta, epoch, frame).unwrap()
    }

    /// Creates a new Orbit from the provided radii of apoapsis and periapsis, in kilometers
    #[allow(clippy::too_many_arguments)]
    pub fn keplerian_apsis_radii(
        r_a: f64,
        r_p: f64,
        inc: f64,
        raan: f64,
        aop: f64,
        ta: f64,
        epoch: Epoch,
        frame: Frame,
    ) -> Self {
        Self::try_keplerian_apsis_radii(r_a, r_p, inc, raan, aop, ta, epoch, frame).unwrap()
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
            self.rmag_km() > EPSILON,
            RadiusSnafu {
                action: "cannot compute orbital momentum vector with zero radius"
            }
        );
        ensure!(
            self.vmag_km_s() > EPSILON,
            RadiusSnafu {
                action: "cannot compute orbital momentum vector with zero velocity"
            }
        );
        Ok(self.radius_km.cross(&self.velocity_km_s))
    }

    /// Returns the orbital momentum value on the X axis
    pub fn hx(&self) -> PhysicsResult<f64> {
        Ok(self.hvec()?[0])
    }

    /// Returns the orbital momentum value on the Y axis
    pub fn hy(&self) -> PhysicsResult<f64> {
        Ok(self.hvec()?[1])
    }

    /// Returns the orbital momentum value on the Z axis
    pub fn hz(&self) -> PhysicsResult<f64> {
        Ok(self.hvec()?[2])
    }

    /// Returns the norm of the orbital momentum
    pub fn hmag(&self) -> PhysicsResult<f64> {
        Ok(self.hvec()?.norm())
    }

    /// Returns the specific mechanical energy in km^2/s^2
    pub fn energy_km2_s2(&self) -> PhysicsResult<f64> {
        ensure!(
            self.rmag_km() > EPSILON,
            RadiusSnafu {
                action: "cannot compute energy with zero radial state"
            }
        );
        Ok(self.vmag_km_s().powi(2) / 2.0 - self.frame.mu_km3_s2()? / self.rmag_km())
    }

    /// Returns the semi-major axis in km
    pub fn sma_km(&self) -> PhysicsResult<f64> {
        // Division by zero prevented in energy_km2_s2
        Ok(-self.frame.mu_km3_s2()? / (2.0 * self.energy_km2_s2()?))
    }

    /// Mutates this orbit to change the SMA
    pub fn set_sma(&mut self, new_sma_km: f64) -> PhysicsResult<()> {
        let me = Self::keplerian(
            new_sma_km,
            self.ecc()?,
            self.inc_deg()?,
            self.raan_deg()?,
            self.aop_deg()?,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        );

        *self = me;

        Ok(())
    }

    /// Returns a copy of the state with a new SMA
    pub fn with_sma(self, new_sma_km: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_sma(new_sma_km)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided SMA added to the current one
    pub fn add_sma(self, delta_sma: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_sma(me.sma_km()? + delta_sma)?;
        Ok(me)
    }

    /// Returns the period in seconds
    pub fn period(&self) -> PhysicsResult<Duration> {
        Ok(2.0
            * PI
            * (self.sma_km()?.powi(3) / self.frame.mu_km3_s2()?)
                .sqrt()
                .seconds())
    }

    /// Returns the eccentricity vector (no unit)
    pub fn evec(&self) -> Result<Vector3, PhysicsError> {
        let r = self.radius_km;
        ensure!(
            self.rmag_km() > EPSILON,
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

    /// Returns the eccentricity (no unit)
    pub fn ecc(&self) -> PhysicsResult<f64> {
        Ok(self.evec()?.norm())
    }

    /// Mutates this orbit to change the ECC
    pub fn set_ecc(&mut self, new_ecc: f64) -> PhysicsResult<()> {
        let me = Self::keplerian(
            self.sma_km()?,
            new_ecc,
            self.inc_deg()?,
            self.raan_deg()?,
            self.aop_deg()?,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        );

        *self = me;

        Ok(())
    }

    /// Returns a copy of the state with a new ECC
    pub fn with_ecc(self, new_ecc: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_ecc(new_ecc)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided ECC added to the current one
    pub fn add_ecc(self, delta_ecc: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_ecc(me.ecc()? + delta_ecc)?;
        Ok(me)
    }

    /// Returns the inclination in degrees
    pub fn inc_deg(&self) -> PhysicsResult<f64> {
        Ok((self.hvec()?[2] / self.hmag()?).acos().to_degrees())
    }

    /// Mutates this orbit to change the INC
    pub fn set_inc_deg(&mut self, new_inc_deg: f64) -> PhysicsResult<()> {
        let me = Self::keplerian(
            self.sma_km()?,
            self.ecc()?,
            new_inc_deg,
            self.raan_deg()?,
            self.aop_deg()?,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        );

        *self = me;

        Ok(())
    }

    /// Returns a copy of the state with a new INC
    pub fn with_inc_deg(self, new_inc_deg: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_inc_deg(new_inc_deg)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided INC added to the current one
    pub fn add_inc_deg(self, delta_inc_deg: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_inc_deg(me.inc_deg()? + delta_inc_deg)?;
        Ok(me)
    }

    /// Returns the argument of periapsis in degrees
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
    pub fn set_aop_deg(&mut self, new_aop_deg: f64) -> PhysicsResult<()> {
        let me = Self::keplerian(
            self.sma_km()?,
            self.ecc()?,
            self.inc_deg()?,
            self.raan_deg()?,
            new_aop_deg,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        );

        *self = me;

        Ok(())
    }

    /// Returns a copy of the state with a new AOP
    pub fn with_aop_deg(self, new_aop_deg: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_aop_deg(new_aop_deg)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided AOP added to the current one
    pub fn add_aop_deg(self, delta_aop_deg: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_aop_deg(me.aop_deg()? + delta_aop_deg)?;
        Ok(me)
    }

    /// Returns the right ascension of ther ascending node in degrees
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
    pub fn set_raan_deg(&mut self, new_raan_deg: f64) -> PhysicsResult<()> {
        let me = Self::keplerian(
            self.sma_km()?,
            self.ecc()?,
            self.inc_deg()?,
            new_raan_deg,
            self.aop_deg()?,
            self.ta_deg()?,
            self.epoch,
            self.frame,
        );

        *self = me;

        Ok(())
    }

    /// Returns a copy of the state with a new RAAN
    pub fn with_raan_deg(self, new_raan_deg: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_raan_deg(new_raan_deg)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided RAAN added to the current one
    pub fn add_raan_deg(self, delta_raan_deg: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_raan_deg(me.raan_deg()? + delta_raan_deg)?;
        Ok(me)
    }

    /// Returns the true anomaly in degrees between 0 and 360.0
    ///
    /// NOTE: This function will emit a warning stating that the TA should be avoided if in a very near circular orbit
    /// Code from https://github.com/ChristopherRabotin/GMAT/blob/80bde040e12946a61dae90d9fc3538f16df34190/src/gmatutil/util/StateConversionUtil.cpp#L6835
    ///
    /// LIMITATION: For an orbit whose true anomaly is (very nearly) 0.0 or 180.0, this function may return either 0.0 or 180.0 with a very small time increment.
    /// This is due to the precision of the cosine calculation: if the arccosine calculation is out of bounds, the sign of the cosine of the true anomaly is used
    /// to determine whether the true anomaly should be 0.0 or 180.0. **In other words**, there is an ambiguity in the computation in the true anomaly exactly at 180.0 and 0.0.
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
    pub fn set_ta_deg(&mut self, new_ta_deg: f64) -> PhysicsResult<()> {
        let me = Self::keplerian(
            self.sma_km()?,
            self.ecc()?,
            self.inc_deg()?,
            self.raan_deg()?,
            self.aop_deg()?,
            new_ta_deg,
            self.epoch,
            self.frame,
        );

        *self = me;

        Ok(())
    }

    /// Returns a copy of the state with a new TA
    pub fn with_ta_deg(self, new_ta_deg: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_ta_deg(new_ta_deg)?;
        Ok(me)
    }

    /// Returns a copy of the state with a provided TA added to the current one
    pub fn add_ta_deg(self, delta_ta_deg: f64) -> PhysicsResult<Self> {
        let mut me = self;
        me.set_ta_deg(me.ta_deg()? + delta_ta_deg)?;
        Ok(me)
    }

    /// Returns a copy of this state with the provided apoasis and periapsis
    pub fn with_apoapsis_periapsis_km(self, new_ra_km: f64, new_rp_km: f64) -> PhysicsResult<Self> {
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
    pub fn add_apoapsis_periapsis_km(
        self,
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
    pub fn aol_deg(&self) -> PhysicsResult<f64> {
        Ok(between_0_360(if self.ecc()? < ECC_EPSILON {
            self.tlong_deg()? - self.raan_deg()?
        } else {
            self.aop_deg()? + self.ta_deg()?
        }))
    }

    /// Returns the radius of periapsis (or perigee around Earth), in kilometers.
    pub fn periapsis_km(&self) -> PhysicsResult<f64> {
        Ok(self.sma_km()? * (1.0 - self.ecc()?))
    }

    /// Returns the radius of apoapsis (or apogee around Earth), in kilometers.
    pub fn apoapsis_km(&self) -> PhysicsResult<f64> {
        Ok(self.sma_km()? * (1.0 + self.ecc()?))
    }

    /// Returns the eccentric anomaly in degrees
    ///
    /// This is a conversion from GMAT's StateConversionUtil::TrueToEccentricAnomaly
    pub fn ea_deg(&self) -> PhysicsResult<f64> {
        let (sin_ta, cos_ta) = self.ta_deg()?.to_radians().sin_cos();
        let ecc_cos_ta = self.ecc()? * cos_ta;
        let sin_ea = ((1.0 - self.ecc()?.powi(2)).sqrt() * sin_ta) / (1.0 + ecc_cos_ta);
        let cos_ea = (self.ecc()? + cos_ta) / (1.0 + ecc_cos_ta);
        // The atan2 function is a bit confusing: https://doc.rust-lang.org/std/primitive.f64.html#method.atan2 .
        Ok(sin_ea.atan2(cos_ea).to_degrees())
    }

    /// Returns the flight path angle in degrees
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
    pub fn right_ascension_deg(&self) -> f64 {
        between_0_360((self.radius_km.y.atan2(self.radius_km.x)).to_degrees())
    }

    /// Returns the declination of this orbit in degrees
    pub fn declination_deg(&self) -> f64 {
        between_pm_180((self.radius_km.z / self.rmag_km()).asin().to_degrees())
    }

    /// Returns the semi minor axis in km, includes code for a hyperbolic orbit
    pub fn semi_minor_axis_km(&self) -> PhysicsResult<f64> {
        if self.ecc()? <= 1.0 {
            Ok(((self.sma_km()? * self.ecc()?).powi(2) - self.sma_km()?.powi(2)).sqrt())
        } else {
            Ok(self.hmag()?.powi(2)
                / (self.frame.mu_km3_s2()? * (self.ecc()?.powi(2) - 1.0).sqrt()))
        }
    }

    /// Returns the velocity declination of this orbit in degrees
    pub fn velocity_declination_deg(&self) -> f64 {
        between_pm_180(
            (self.velocity_km_s.z / self.vmag_km_s())
                .asin()
                .to_degrees(),
        )
    }

    /// Returns the $C_3$ of this orbit in km^2/s^2
    pub fn c3_km2_s2(&self) -> PhysicsResult<f64> {
        Ok(-self.frame.mu_km3_s2()? / self.sma_km()?)
    }
}

#[allow(clippy::format_in_format_args)]
impl fmt::LowerHex for Orbit {
    // Prints the Keplerian orbital elements in floating point with units
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
