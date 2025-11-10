/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{orbit::Orbit, PhysicsResult};

use crate::prelude::Frame;

use hifitime::Epoch;

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub(crate) fn equinoctial_to_keplerian(
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

impl Orbit {
    /// Attempts to create a new Orbit from the Equinoctial orbital elements.
    ///
    /// Note that this function computes the Keplerian elements from the equinoctial and then
    /// calls the try_keplerian_mean_anomaly initializer.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::neg_multiply)]
    pub fn try_equinoctial(
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
            equinoctial_to_keplerian(sma_km, h, k, p, q, lambda_deg);

        Self::try_keplerian_mean_anomaly(
            sma_km, ecc, inc_deg, raan_deg, aop_deg, ma_deg, epoch, frame,
        )
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Orbit {
    /// Returns the equinoctial semi-major axis (a) in km.
    ///
    /// :rtype: float
    pub fn equinoctial_a_km(&self) -> PhysicsResult<f64> {
        self.sma_km()
    }

    /// Returns the equinoctial element h (ecc * sin(aop + raan)).
    ///
    /// :rtype: float
    pub fn equinoctial_h(&self) -> PhysicsResult<f64> {
        Ok(self.ecc()? * (self.aop_deg()?.to_radians() + self.raan_deg()?.to_radians()).sin())
    }

    /// Returns the equinoctial element k (ecc * cos(aop + raan)).
    ///
    /// :rtype: float
    pub fn equinoctial_k(&self) -> PhysicsResult<f64> {
        Ok(self.ecc()? * (self.aop_deg()?.to_radians() + self.raan_deg()?.to_radians()).cos())
    }

    /// Returns the equinoctial element p (sin(inc/2) * sin(raan)).
    ///
    /// :rtype: float
    pub fn equinoctial_p(&self) -> PhysicsResult<f64> {
        Ok((self.inc_deg()?.to_radians() / 2.0).sin() * self.raan_deg()?.to_radians().sin())
    }

    /// Returns the equinoctial element q (sin(inc/2) * cos(raan)).
    ///
    /// :rtype: float
    pub fn equinoctial_q(&self) -> PhysicsResult<f64> {
        Ok((self.inc_deg()?.to_radians() / 2.0).sin() * self.raan_deg()?.to_radians().cos())
    }

    /// Returns the equinoctial mean longitude (lambda = raan + aop + ma) in degrees.
    ///
    /// :rtype: float
    pub fn equinoctial_lambda_mean_deg(&self) -> PhysicsResult<f64> {
        // C++ version `aeq[5]=kep[3]+kep[4]+kep[5]` sums degrees.
        Ok(self.raan_deg()? + self.aop_deg()? + self.ma_deg()?)
    }
}
