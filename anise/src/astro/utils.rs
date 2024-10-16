/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use std::f64::consts::{PI, TAU};

use crate::errors::{MathError, PhysicsError};

use super::PhysicsResult;

/// Mean anomaly f64::EPSILON
pub const MA_EPSILON: f64 = 1e-12;

/// Computes the true anomaly from the given mean anomaly for an orbit.
///
/// The computation process varies depending on whether the orbit is elliptical (eccentricity less than or equal to 1)
/// or hyperbolic (eccentricity greater than 1). In each case, the method uses an iterative algorithm to find a
/// sufficiently accurate approximation of the true anomaly.
///
/// # Arguments
///
/// * `ma_radians` - The mean anomaly in radians.
/// * `ecc` - The eccentricity of the orbit.
///
/// # Remarks
///
/// This function uses GTDS MathSpec Equations 3-180, 3-181, and 3-186 for the iterative computation process.
///
/// Source: GMAT source code (`compute_mean_to_true_anomaly`)
pub fn compute_mean_to_true_anomaly_rad(ma_radians: f64, ecc: f64) -> PhysicsResult<f64> {
    let rm = ma_radians;
    if ecc <= 1.0 {
        // Elliptical orbit
        let mut e2 = rm + ecc * rm.sin(); // GTDS MathSpec Equation 3-182

        let mut iter = 0;

        loop {
            iter += 1;
            if iter > 1000 {
                return Err(PhysicsError::AppliedMath {
                    source: MathError::MaxIterationsReached {
                        iter,
                        action: "computing the true anomaly from the mean anomaly",
                    },
                });
            }

            // GTDS MathSpec Equation 3-180  Note: a little difference here is that it uses Cos(E) instead of Cos(E-0.5*f)
            let normalized_anomaly = 1.0 - ecc * e2.cos();

            if normalized_anomaly.abs() < MA_EPSILON {
                return Err(PhysicsError::AppliedMath {
                    source: MathError::DomainError {
                        value: normalized_anomaly,
                        msg: "normalized anomaly too small",
                    },
                });
            }

            // GTDS MathSpec Equation 3-181
            let e1 = e2 - (e2 - ecc * e2.sin() - rm) / normalized_anomaly;

            if (e2 - e1).abs() < MA_EPSILON {
                break;
            }

            e2 = e1;
        }

        let mut e = e2;

        if e < 0.0 {
            e += TAU;
        }

        let c = (e - PI).abs();

        let mut ta = if c >= 1.0e-08 {
            let normalized_anomaly = 1.0 - ecc;

            if (normalized_anomaly).abs() < MA_EPSILON {
                return Err(PhysicsError::AppliedMath {
                    source: MathError::DomainError {
                        value: normalized_anomaly,
                        msg: "normalized anomaly too small",
                    },
                });
            }

            let eccentricity_ratio = (1.0 + ecc) / normalized_anomaly; // temp2 = (1+ecc)/(1-ecc)

            if eccentricity_ratio < 0.0 {
                return Err(PhysicsError::AppliedMath {
                    source: MathError::DomainError {
                        value: eccentricity_ratio,
                        msg: "eccentricity ratio too small",
                    },
                });
            }

            let f = eccentricity_ratio.sqrt();
            let g = (e / 2.0).tan();
            // tan(TA/2) = Sqrt[(1+ecc)/(1-ecc)] * tan(E/2)
            2.0 * (f * g).atan()
        } else {
            e
        };

        if ta < 0.0 {
            ta += TAU;
        }
        Ok(ta)
    } else {
        //---------------------------------------------------------
        // hyperbolic orbit
        //---------------------------------------------------------

        // For hyperbolic orbit, anomaly is nolonger to be an angle so we cannot use mod of 2*PI to mean anomaly.
        // We need to keep its original value for calculation.
        //if (rm > PI)                       // incorrect
        //   rm = rm - TWO_PI;               // incorrect

        //f2 = ecc * Sinh(rm) - rm;          // incorrect
        //f2 = rm / 2;                       // incorrect  // GTDS MathSpec Equation 3-186
        let mut f2: f64 = 0.0; // This is the correct initial value for hyperbolic eccentric anomaly.
        let mut iter = 0;

        loop {
            iter += 1;
            if iter > 1000 {
                return Err(PhysicsError::AppliedMath {
                    source: MathError::MaxIterationsReached {
                        iter,
                        action: "computing the true anomaly from the mean anomaly",
                    },
                });
            }

            let normalizer = ecc * f2.cosh() - 1.0;

            if normalizer.abs() < MA_EPSILON {
                return Err(PhysicsError::AppliedMath {
                    source: MathError::DomainError {
                        value: normalizer,
                        msg: "normalized anomaly too small (hyperbolic case)",
                    },
                });
            }

            let f1 = f2 - (ecc * f2.sinh() - f2 - rm) / normalizer; // GTDS MathSpec Equation 3-186
            if (f2 - f1).abs() < MA_EPSILON {
                break;
            }
            f2 = f1;
        }

        let f = f2;
        let normalized_anomaly = ecc - 1.0;

        if normalized_anomaly.abs() < MA_EPSILON {
            return Err(PhysicsError::AppliedMath {
                source: MathError::DomainError {
                    value: normalized_anomaly,
                    msg: "normalized anomaly too small (hyperbolic case)",
                },
            });
        }

        let eccentricity_ratio = (ecc + 1.0) / normalized_anomaly; // temp2 = (ecc+1)/(ecc-1)

        if eccentricity_ratio < 0.0 {
            return Err(PhysicsError::AppliedMath {
                source: MathError::DomainError {
                    value: eccentricity_ratio,
                    msg: "eccentricity ratio too small (hyperbolic case)",
                },
            });
        }

        let e = eccentricity_ratio.sqrt();
        let g = (f / 2.0).tanh();
        let mut ta = 2.0 * (e * g).atan(); // tan(TA/2) = Sqrt[(ecc+1)/(ecc-1)] * Tanh(F/2)    where: F is hyperbolic centric anomaly

        if ta < 0.0 {
            ta += TAU;
        }
        Ok(ta)
    }
}
