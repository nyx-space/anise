/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::utils::mean_anomaly_to_true_anomaly_rad;
use super::{orbit::Orbit, orbit_equinoctial::equinoctial_to_keplerian, PhysicsResult};

use crate::{
    errors::MeanElementSnafu,
    math::angles::{between_0_360, between_0_tau},
};
use core::f64::consts::PI;

use log::warn;
use snafu::ensure;

#[cfg(feature = "python")]
use pyo3::prelude::*;

const J2_EARTH: f64 = 1.082_626_925_638_815E-3;

/// Converts from Brouwer-Lyddane Mean Elements (short period terms only) to Osculating Keplerian Elements.
/// Warning: this function does not perform any verification on in the validity of the inputs.
///
/// Port of GMAT's StateConversionUtil::BrouwerMeanShortToOsculatingElements
///
/// @param mu   Gravitational parameter (km^3/s^2)
/// @param blms Brouwer-Lyddane Mean Elements [SMA, ECC, INC, RAAN, AOP, MA]
///             (angles in degrees)
/// @return     Osculating Keplerian Elements [SMA, ECC, INC, RAAN, AOP, MA]
///             (angles in degrees)
/// ------------------------------------------------------------------------------
#[allow(clippy::too_many_arguments)]
fn brouwer_mean_short_to_osculating_kep(
    sma_km: f64,
    ecc: f64,
    inc_deg: f64,
    raan_deg: f64,
    aop_deg: f64,
    ma_deg: f64,
    re_km: f64,
    j2: f64,
) -> PhysicsResult<[f64; 6]> {
    let k2 = 0.5 * j2;

    let smap = sma_km / re_km; // Normalized SMA
    let mut eccp = ecc;
    let mut incp = inc_deg.to_radians();
    let mut raanp = raan_deg.to_radians();
    let mut aopp = aop_deg.to_radians();
    let mut mean_anom = ma_deg.to_radians();

    if eccp < 0.0 {
        eccp = -eccp;
        mean_anom -= PI;
        aopp += PI;
        warn!(
            "eccentricity is negative (!?) so the current apoapsis will be taken to be new periapsis."
        );
    }

    ensure!(
        eccp <= 1.0,
        MeanElementSnafu {
            detail: "Brouwer Mean Short not applicable to hyperbolic orbits"
        }
    );

    // --- Pseudo-state and Normalization ---
    let mut pseudostate = 0;
    if incp > 175.0_f64.to_radians() {
        incp = PI - incp; // INC = 180 - INC
        raanp = -raanp; // RAAN = -RAAN
        aopp = -aopp;
        pseudostate = 1;
    }

    raanp = between_0_tau(raanp);
    aopp = between_0_tau(aopp);
    mean_anom = between_0_tau(mean_anom);

    // --- Intermediate Terms ---
    let eta = (1.0 - eccp.powi(2)).sqrt();
    let theta = incp.cos();
    let p = smap * eta.powi(2);
    let gm2 = k2 / smap.powi(2);
    let gm2p = gm2 / eta.powi(4);

    let tap = mean_anomaly_to_true_anomaly_rad(mean_anom, eccp)?;
    let rp = p / (1.0 + eccp * tap.cos());
    let adr = smap / rp;

    let sin_incp = incp.sin();
    // let sin_incp_sq = sin_incp.powi(2);
    let theta_sq = theta.powi(2);

    // --- Brouwer Equations (Short-Period Perturbations) ---
    // Note: These are direct transcriptions from the C++ code.

    let sma1 = smap
        + smap
            * gm2
            * ((adr.powi(3) - 1.0 / eta.powi(3)) * (-1.0 + 3.0 * theta_sq)
                + 3.0 * (1.0 - theta_sq) * adr.powi(3) * (2.0 * aopp + 2.0 * tap).cos());

    let decc = eta.powi(2) / 2.0
        * ((3.0
            * (1.0 / eta.powi(6))
            * gm2
            * (1.0 - theta_sq)
            * (2.0 * aopp + 2.0 * tap).cos()
            * (3.0 * eccp * tap.cos().powi(2)
                + 3.0 * tap.cos()
                + eccp.powi(2) * tap.cos().powi(3)
                + eccp))
            - (gm2p
                * (1.0 - theta_sq)
                * (3.0 * (2.0 * aopp + tap).cos() + (3.0 * tap + 2.0 * aopp).cos()))
            + (3.0 * theta_sq - 1.0) * gm2 / eta.powi(6)
                * (eccp * eta
                    + eccp / (1.0 + eta)
                    + 3.0 * eccp * tap.cos().powi(2)
                    + 3.0 * tap.cos()
                    + eccp.powi(2) * tap.cos().powi(3)));

    let dinc = gm2p / 2.0
        * theta
        * sin_incp
        * (3.0 * (2.0 * aopp + 2.0 * tap).cos()
            + 3.0 * eccp * (2.0 * aopp + tap).cos()
            + eccp * (2.0 * aopp + 3.0 * tap).cos());

    let draan = -gm2p / 2.0
        * theta
        * (6.0 * (tap - mean_anom + eccp * tap.sin())
            - 3.0 * (2.0 * aopp + 2.0 * tap).sin()
            - 3.0 * eccp * (2.0 * aopp + tap).sin()
            - eccp * (2.0 * aopp + 3.0 * tap).sin());

    // Human note: these variables are UNUSED but in the GMAT code
    // Note: The C++ code for aop1 and ma1 has J2 terms.
    // The C++ `aop1` is not `aopp + daopp`, it's a full recalculation.
    // We follow the C++ logic exactly.
    // let aop1 = aopp
    //     + 3.0 * j2 / 2.0 / p.powi(2)
    //         * ((2.0 - 5.0 / 2.0 * sin_incp_sq) * (tap - mean_anom + eccp * tap.sin())
    //             + (1.0 - 3.0 / 2.0 * sin_incp_sq)
    //                 * (1.0 / eccp * (1.0 - 1.0 / 4.0 * eccp.powi(2)) * tap.sin()
    //                     + 1.0 / 2.0 * (2.0 * tap).sin()
    //                     + eccp / 12.0 * (3.0 * tap).sin())
    //             - 1.0 / eccp
    //                 * (1.0 / 4.0 * sin_incp_sq
    //                     + (1.0 / 2.0 - 15.0 / 16.0 * sin_incp_sq) * eccp.powi(2))
    //                 * (tap + 2.0 * aopp).sin()
    //             + eccp / 16.0 * sin_incp_sq * (tap - 2.0 * aopp).sin()
    //             - 1.0 / 2.0 * (1.0 - 5.0 / 2.0 * sin_incp_sq) * (2.0 * tap + 2.0 * aopp).sin()
    //             + 1.0 / eccp
    //                 * (7.0 / 12.0 * sin_incp_sq
    //                     - 1.0 / 6.0 * (1.0 - 19.0 / 8.0 * sin_incp_sq) * eccp.powi(2))
    //                 * (3.0 * tap + 2.0 * aopp).sin()
    //             + 3.0 / 8.0 * sin_incp_sq * (4.0 * tap + 2.0 * aopp).sin()
    //             + eccp / 16.0 * sin_incp_sq * (5.0 * tap + 2.0 * aopp).sin());

    // `ma1` in C++ is also a full recalculation, not `mean_anom + dma`
    // let mut ma1 = mean_anom
    //     + 3.0 * j2 * eta / 2.0 / eccp / p.powi(2)
    //         * (-(1.0 - 3.0 / 2.0 * sin_incp_sq)
    //             * ((1.0 - eccp.powi(2) / 4.0) * tap.sin()
    //                 + eccp / 2.0 * (2.0 * tap).sin()
    //                 + eccp.powi(2) / 12.0 * (3.0 * tap).sin())
    //             + sin_incp_sq
    //                 * (1.0 / 4.0 * (1.0 + 5.0 / 4.0 * eccp.powi(2)) * (tap + 2.0 * aopp).sin()
    //                     - eccp.powi(2) / 16.0 * (tap - 2.0 * aopp).sin()
    //                     - 7.0 / 12.0
    //                         * (1.0 - eccp.powi(2) / 28.0)
    //                         * (3.0 * tap + 2.0 * aopp).sin()
    //                     - 3.0 * eccp / 8.0 * (4.0 * tap + 2.0 * aopp).sin()
    //                     - eccp.powi(2) / 16.0 * (5.0 * tap + 2.0 * aopp).sin()));

    let lgh = raanp
        + aopp
        + mean_anom
        + gm2p / 4.0
            * (6.0 * (-1.0 - 2.0 * theta + 5.0 * theta_sq) * (tap - mean_anom + eccp * tap.sin())
                + (3.0 + 2.0 * theta - 5.0 * theta_sq)
                    * (3.0 * (2.0 * aopp + 2.0 * tap).sin()
                        + 3.0 * eccp * (2.0 * aopp + tap).sin()
                        + eccp * (2.0 * aopp + 3.0 * tap).sin()))
        + gm2p / 4.0 * eta.powi(2) / (eta + 1.0)
            * eccp
            * (3.0
                * (1.0 - theta_sq)
                * ((3.0 * tap + 2.0 * aopp).sin() * (1.0 / 3.0 + adr.powi(2) * eta.powi(2) + adr)
                    + (2.0 * aopp + tap).sin() * (1.0 - adr.powi(2) * eta.powi(2) - adr))
                + 2.0
                    * tap.sin()
                    * (3.0 * theta_sq - 1.0)
                    * (1.0 + adr.powi(2) * eta.powi(2) + adr));

    let eccpdl = -eta.powi(3) / 4.0
        * gm2p
        * (2.0 * (-1.0 + 3.0 * theta_sq) * (adr.powi(2) * eta.powi(2) + adr + 1.0) * tap.sin()
            + 3.0
                * (1.0 - theta_sq)
                * ((-adr.powi(2) * eta.powi(2) - adr + 1.0) * (2.0 * aopp + tap).sin()
                    + (adr.powi(2) * eta.powi(2) + adr + 1.0 / 3.0)
                        * (2.0 * aopp + 3.0 * tap).sin()));

    // --- Recalculate ECC and MA from vector components ---
    let ecosl = (eccp + decc) * mean_anom.cos() - eccpdl * mean_anom.sin();
    let esinl = (eccp + decc) * mean_anom.sin() + eccpdl * mean_anom.cos();
    let ecc1 = (ecosl.powi(2) + esinl.powi(2)).sqrt();

    let ma1 = if ecc1 < 1.0e-11 {
        0.0
    } else {
        between_0_tau(esinl.atan2(ecosl))
    };

    // --- Recalculate INC and RAAN from vector components ---
    // C++: 1.0/2.0*Sin(incp)/Cos(incp/2.0) simplifies to Sin(incp/2.0)
    let sin_half_i = (0.5 * incp).sin();
    let cos_half_i = (0.5 * incp).cos();

    let sinhalfisinh =
        (sin_half_i + cos_half_i * 0.5 * dinc) * raanp.sin() + sin_half_i * draan * raanp.cos();
    let sinhalficosh =
        (sin_half_i + cos_half_i * 0.5 * dinc) * raanp.cos() - sin_half_i * draan * raanp.sin();

    let sin_half_i_new_sq = sinhalfisinh.powi(2) + sinhalficosh.powi(2);
    let sin_half_i_new = sin_half_i_new_sq.sqrt().clamp(-1.0, 1.0); // Clamp to [-1, 1]

    let inc1 = 2.0 * sin_half_i_new.asin();
    let raan1_final;
    let mut aop1_final = if inc1.abs() < 1.0e-9 || (inc1 - PI).abs() < 1.0e-9 {
        raan1_final = 0.0;
        lgh - ma1 - raan1_final
    } else {
        let raan1 = sinhalfisinh.atan2(sinhalficosh);
        raan1_final = between_0_tau(raan1);
        lgh - ma1 - raan1_final
    };

    aop1_final = between_0_tau(aop1_final);

    // --- Final Assembly ---
    let mut kepl = [
        sma1 * re_km, // De-normalize SMA
        ecc1,
        inc1.to_degrees(),
        raan1_final.to_degrees(),
        aop1_final.to_degrees(),
        ma1.to_degrees(),
    ];

    if pseudostate != 0 {
        kepl[2] = 180.0 - kepl[2];
        kepl[3] = 360.0 - kepl[3];
    }

    // Final normalization
    kepl[3] = between_0_360(kepl[3]);
    kepl[4] = between_0_360(kepl[4]);
    kepl[5] = between_0_360(kepl[5]);

    Ok(kepl)
}

impl Orbit {
    /// This is the private helper function that performs the main conversion.
    /// The public methods will call this.
    fn calculate_brouwer_mean_short_elements(
        &self,
    ) -> PhysicsResult<(f64, f64, f64, f64, f64, f64)> {
        // --- Inner Helper Functions for Solver ---

        /// Converts Keplerian elements [SMA, ECC, INC, RAAN, AOP, MA] (degrees)
        /// to Equinoctial elements [a, h, k, p, q, lambda_mean_deg].
        /// This is needed for intermediate solver variables, not for `self`.
        fn kep_to_aeq(kep: &[f64; 6]) -> [f64; 6] {
            let (sma, ecc, inc_deg, raan_deg, aop_deg, ma_deg) =
                (kep[0], kep[1], kep[2], kep[3], kep[4], kep[5]);

            let inc_rad = (inc_deg / 2.0).to_radians();
            let raan_rad = raan_deg.to_radians();
            let aop_raan_rad = (aop_deg + raan_deg).to_radians();

            [
                sma,
                ecc * aop_raan_rad.sin(),
                ecc * aop_raan_rad.cos(),
                inc_rad.sin() * raan_rad.sin(),
                inc_rad.sin() * raan_rad.cos(),
                raan_deg + aop_deg + ma_deg, // Mean longitude in degrees
            ]
        }

        // --- Constants ---
        const TOLERANCE: f64 = 1.0e-8;
        const MAX_ITER: u32 = 100;

        // --- 1. Validation ---
        ensure!(
            self.frame.ephem_origin_id_match(399),
            MeanElementSnafu {
                detail: "Brouwer Mean short only applies for Earth centered objects"
            }
        );

        let osc_sma = self.sma_km()?;
        let osc_ecc = self.ecc()?;
        let osc_inc = self.inc_deg()?;

        ensure!(
            osc_inc <= 180.0,
            MeanElementSnafu {
                detail: "Brouwer Mean Short only applies for prograde orbits (inc < 180.0)"
            }
        );

        ensure!(self.periapsis_km()? > 3000.0, MeanElementSnafu{detail: "Brouwer Mean Short only applies for orbits whose periapsis is greater than 3,000 km"});

        ensure!(
            (0.0..1.0).contains(&self.ecc()?),
            MeanElementSnafu {
                detail: "Brouwer Mean Short only applies for non hyperbolic orbits"
            }
        );

        let periapsis_km = self.periapsis_km()?;

        if periapsis_km < self.frame.mean_equatorial_radius_km()? {
            warn!("Brouwer Mean Short might be inaccurate because orbit intersects Earth");
        }

        // --- 2. Get Initial Osculating Elements ---
        // We get [SMA, ECC, INC, RAAN, AOP, MA]
        let mut osc_kep_ma = [
            osc_sma,
            osc_ecc,
            osc_inc,
            self.raan_deg()?,
            self.aop_deg()?,
            self.ma_deg()?, // GMAT C++ converts TA -> MA, we can get MA directly
        ];

        // --- 3. Handle Pseudo-state for high inclination ---
        let mut pseudostate = 0;
        let (cart, aeq); // Declare target cartesian and equinoctial

        // `osc_kep_ma` holds the elements we will use for the initial guess
        // AND for defining the target state. We flip it *in-place* if needed.
        if osc_kep_ma[2] > 175.0 {
            pseudostate = 1;
            osc_kep_ma[2] = 180.0 - osc_kep_ma[2]; // INC = 180 - INC
            osc_kep_ma[3] = -osc_kep_ma[3]; // RAAN = -RAAN
            osc_kep_ma[4] = -osc_kep_ma[4];

            // Re-generate `Orbit` state from flipped Keplerian elements
            let flipped_orbit_state = Self::try_keplerian_mean_anomaly(
                osc_kep_ma[0],
                osc_kep_ma[1],
                osc_kep_ma[2],
                osc_kep_ma[3],
                osc_kep_ma[4],
                osc_kep_ma[5],
                self.epoch,
                self.frame,
            )?;

            // Set *both* targets from the flipped state
            cart = flipped_orbit_state.to_cartesian_pos_vel();
            aeq = [
                flipped_orbit_state.equinoctial_a_km()?,
                flipped_orbit_state.equinoctial_h()?,
                flipped_orbit_state.equinoctial_k()?,
                flipped_orbit_state.equinoctial_p()?,
                flipped_orbit_state.equinoctial_q()?,
                flipped_orbit_state.equinoctial_lambda_mean_deg()?,
            ];
        } else {
            // Set *both* targets from the original state (`self`)
            cart = self.to_cartesian_pos_vel();
            aeq = [
                self.equinoctial_a_km()?,
                self.equinoctial_h()?,
                self.equinoctial_k()?,
                self.equinoctial_p()?,
                self.equinoctial_q()?,
                self.equinoctial_lambda_mean_deg()?,
            ];
        };

        // --- 4. Iterative Solver ---

        // `blmean_guess` (C++) is the initial guess for mean elements.
        // We guess that mean elements = osculating elements.
        let blmean_guess = osc_kep_ma;

        // `aeqmean` (C++) is the equinoctial version of the initial guess
        // We must use the *private helper* for this.
        let mut aeqmean = kep_to_aeq(&blmean_guess);

        let kep2 = brouwer_mean_short_to_osculating_kep(
            blmean_guess[0],
            blmean_guess[1],
            blmean_guess[2],
            blmean_guess[3],
            blmean_guess[4],
            blmean_guess[5],
            self.frame.mean_equatorial_radius_km()?,
            J2_EARTH,
        )?;

        // `aeq2` (C++) is equinoctial version of `kep2`
        // We must use the *private helper* for this.
        let mut aeq2 = kep_to_aeq(&kep2);

        // `aeqmean2` (C++) is the *next* guess, `x_k+1`
        let mut aeqmean2 = [0.0; 6];
        for i in 0..6 {
            aeqmean2[i] = aeqmean[i] + (aeq[i] - aeq2[i]);
        }

        let mut emag = 0.9; // Dummy value to start loop
        let mut emag_old = 1.0;
        let mut ii = 0;

        while emag > TOLERANCE {
            // `blmean2` (C++) is the mean Keplerian state from our "next" guess
            let (a, h, k, p, q, lambda_deg) = (
                aeqmean2[0],
                aeqmean2[1],
                aeqmean2[2],
                aeqmean2[3],
                aeqmean2[4],
                aeqmean2[5],
            );
            let (sma_km, ecc, inc_deg, raan_deg, aop_deg, ma_deg) =
                equinoctial_to_keplerian(a, h, k, p, q, lambda_deg);

            // `kep2` (C++) is the osculating state from `blmean2`
            let kep2 = brouwer_mean_short_to_osculating_kep(
                sma_km,
                ecc,
                inc_deg,
                raan_deg,
                aop_deg,
                ma_deg,
                self.frame.mean_equatorial_radius_km()?,
                J2_EARTH,
            )?;

            // `cart2` (C++) is the Cartesian state from `kep2`
            let state2 = Orbit::try_keplerian_mean_anomaly(
                kep2[0], kep2[1], kep2[2], kep2[3], kep2[4], kep2[5], self.epoch, self.frame,
            )?;
            let cart2 = state2.to_cartesian_pos_vel();

            // Calculate normalized error
            let cart_err = cart - cart2;
            emag = cart_err.norm() / cart.norm();

            // `aeq2` (C++) is the osculating equinoctial state from `kep2`
            aeq2 = kep_to_aeq(&kep2);

            if emag_old > emag {
                // Converging
                emag_old = emag;
                aeqmean = aeqmean2;

                // `aeqmean2` (C++) `x_k+1` = `x_k` + (target - f(x_k))
                for i in 0..6 {
                    aeqmean2[i] = aeqmean[i] + (aeq[i] - aeq2[i]);
                }
            } else {
                warn!("Brouwer Mean Short algorithm convergence not improving, current rel. error {emag_old}");
                break;
            }

            if ii > MAX_ITER {
                warn!(
                    "Brouwer Mean Short iterations stopped after {MAX_ITER} -- may be inaccurate"
                );
                break;
            }
            ii += 1;
        }

        // --- 5. Final Conversion & Post-Processing ---
        let (sma_km, mut ecc, mut inc_deg, mut raan_deg, mut aop_deg, mut ma_deg) =
            equinoctial_to_keplerian(
                aeqmean2[0],
                aeqmean2[1],
                aeqmean2[2],
                aeqmean2[3],
                aeqmean2[4],
                aeqmean2[5],
            );

        // Handle negative eccentricity
        if ecc < 0.0 {
            ecc = -ecc;
            aop_deg += 180.0;
            ma_deg -= 180.0;
        }

        // Undo pseudo-state
        if pseudostate != 0 {
            inc_deg = 180.0 - inc_deg;
            raan_deg = -raan_deg;
        }

        // Normalize angles
        raan_deg = between_0_360(raan_deg);
        aop_deg = between_0_360(aop_deg);
        ma_deg = between_0_360(ma_deg);

        Ok((sma_km, ecc, inc_deg, raan_deg, aop_deg, ma_deg))
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Orbit {
    /// Returns the Brouwer-short mean semi-major axis in km.
    ///
    /// :rtype: float
    pub fn sma_brouwer_short_km(&self) -> PhysicsResult<f64> {
        Ok(self.calculate_brouwer_mean_short_elements()?.0)
    }

    /// Returns the Brouwer-short mean eccentricity.
    ///
    /// :rtype: float
    pub fn ecc_brouwer_short(&self) -> PhysicsResult<f64> {
        Ok(self.calculate_brouwer_mean_short_elements()?.1)
    }

    /// Returns the Brouwer-short mean inclination in degrees.
    ///
    /// :rtype: float
    pub fn inc_brouwer_short_deg(&self) -> PhysicsResult<f64> {
        Ok(self.calculate_brouwer_mean_short_elements()?.2)
    }

    /// Returns the Brouwer-short mean Right Ascension of the Ascending Node in degrees.
    ///
    /// :rtype: float
    pub fn raan_brouwer_short_deg(&self) -> PhysicsResult<f64> {
        Ok(self.calculate_brouwer_mean_short_elements()?.3)
    }

    /// Returns the Brouwer-short mean Argument of Perigee in degrees.
    ///
    /// :rtype: float
    pub fn aop_brouwer_short_deg(&self) -> PhysicsResult<f64> {
        Ok(self.calculate_brouwer_mean_short_elements()?.4)
    }

    /// Returns the Brouwer-short mean Mean Anomaly in degrees.
    ///
    /// :rtype: float
    pub fn ma_brouwer_short_deg(&self) -> PhysicsResult<f64> {
        Ok(self.calculate_brouwer_mean_short_elements()?.5)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::frames::EARTH_J2000;
    use crate::prelude::{Almanac, Frame};
    use hifitime::Epoch;
    use rstest::{fixture, rstest};

    #[fixture]
    fn almanac() -> Almanac {
        Almanac::new("../data/pck08.pca").unwrap()
    }

    #[fixture]
    fn epoch() -> Epoch {
        Epoch::from_gregorian_utc_at_midnight(2000, 1, 1)
    }

    #[fixture]
    fn eme2k(almanac: Almanac) -> Frame {
        almanac.frame_info(EARTH_J2000).unwrap()
    }

    // Helper function to check all elements at once.
    fn assert_brouwer_roundtrip(
        mean_sma_km: f64,
        mean_ecc: f64,
        mean_inc_deg: f64,
        mean_raan_deg: f64,
        mean_aop_deg: f64,
        mean_ma_deg: f64,
        eme2k: Frame,
    ) {
        // 1. Convert Mean -> Osculating
        // This calls the `brouwer_mean_short_to_osculating_kep` function
        let osc_kep = brouwer_mean_short_to_osculating_kep(
            mean_sma_km,
            mean_ecc,
            mean_inc_deg,
            mean_raan_deg,
            mean_aop_deg,
            mean_ma_deg,
            eme2k.mean_equatorial_radius_km().unwrap(),
            J2_EARTH,
        )
        .unwrap();

        let epoch = Epoch::from_gregorian_tai_at_midnight(2025, 11, 9);
        // 2. Create a State from the osculating elements
        let osc_state = Orbit::try_keplerian_mean_anomaly(
            osc_kep[0], osc_kep[1], osc_kep[2], osc_kep[3], osc_kep[4], osc_kep[5], epoch, eme2k,
        )
        .unwrap();

        // 3. Convert Osculating -> Mean (using the new public functions)
        // These calls all trigger `calculate_brouwer_mean_short_elements`
        let mean_sma_out = osc_state.sma_brouwer_short_km().unwrap();
        let mean_ecc_out = osc_state.ecc_brouwer_short().unwrap();
        let mean_inc_out = osc_state.inc_brouwer_short_deg().unwrap();
        let mean_raan_out = osc_state.raan_brouwer_short_deg().unwrap();
        let mean_aop_out = osc_state.aop_brouwer_short_deg().unwrap();
        let mean_ma_out = osc_state.ma_brouwer_short_deg().unwrap();

        // 4. Compare the input mean elements to the output mean elements.
        // Tolerances are based on the solver's precision.
        let sma_tol = 4.0; // km
        let ecc_tol = 1e-2; // unitless
        let ang_tol = 7e-1; // degrees

        assert!(
            (mean_sma_km - mean_sma_out).abs() < sma_tol,
            "SMA mismatch: in={mean_sma_km}, out={mean_sma_out}"
        );
        assert!(
            (mean_ecc - mean_ecc_out).abs() < ecc_tol,
            "ECC mismatch: in={mean_ecc}, out={mean_ecc_out}"
        );
        assert!(
            (mean_inc_deg - mean_inc_out).abs() < ang_tol,
            "INC mismatch: in={mean_inc_deg}, out={mean_inc_out}"
        );
        assert!(
            (mean_raan_deg - mean_raan_out).abs() < ang_tol,
            "RAAN mismatch: in={mean_raan_deg}, out={mean_raan_out}"
        );
        assert!(
            (mean_aop_deg - mean_aop_out).abs() < ang_tol,
            "AOP mismatch: in={mean_aop_deg}, out={mean_aop_out}"
        );
        assert!(
            (mean_ma_deg - mean_ma_out).abs() < ang_tol,
            "MA mismatch: in={mean_ma_deg}, out={mean_ma_out}"
        );
    }

    #[rstest]
    fn test_brouwer_short_roundtrip_prograde_leo(eme2k: Frame) {
        // A typical LEO orbit
        assert_brouwer_roundtrip(
            7000.0, // mean SMA (km)
            0.01,   // mean ECC
            51.6,   // mean INC (deg)
            30.0,   // mean RAAN (deg)
            20.0,   // mean AOP (deg)
            10.0,   // mean MA (deg)
            eme2k,
        );
    }

    #[rstest]
    fn test_brouwer_short_roundtrip_retrograde(eme2k: Frame) {
        // A high-inclination retrograde orbit to test the `pseudostate` logic
        assert_brouwer_roundtrip(
            7200.0, // mean SMA (km)
            0.02,   // mean ECC
            177.0,  // mean INC (deg) - tests the > 175 deg logic
            45.0,   // mean RAAN (deg)
            10.0,   // mean AOP (deg)
            5.0,    // mean MA (deg)
            eme2k,
        );
    }

    #[rstest]
    fn test_brouwer_short_roundtrip_gto(eme2k: Frame) {
        // A GTO-like orbit, pushing the eccentricity
        assert_brouwer_roundtrip(
            24365.0, // mean SMA (km)
            0.7,     // mean ECC
            28.5,    // mean INC (deg)
            60.0,    // mean RAAN (deg)
            90.0,    // mean AOP (deg)
            25.0,    // mean MA (deg)
            eme2k,
        );
    }
}
