/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{
    constants::SPEED_OF_LIGHT_KM_S,
    errors::{AberrationSnafu, VelocitySnafu},
    math::{rotate_vector, Vector3},
};

use core::fmt;

#[cfg(feature = "python")]
use pyo3::prelude::*;
use snafu::ensure;

use super::PhysicsResult;
use crate::errors::PhysicsError;

/// Represents the aberration correction options in ANISE.
///
/// In space science and engineering, accurately pointing instruments (like optical cameras or radio antennas) at a target is crucial. This task is complicated by the finite speed of light, necessitating corrections for the apparent position of the target.
///
/// This structure holds parameters for aberration corrections applied to a target's position or state vector. These corrections account for the difference between the target's geometric (true) position and its apparent position as observed.
///
/// # Rule of tumb
/// In most Earth orbits, one does _not_ need to provide any aberration corrections. Light time to the target is less than one second (the Moon is about one second away).
/// In near Earth orbits, e.g. inner solar system, preliminary analysis can benefit from enabling unconverged light time correction. Stellar aberration is probably not required.
/// For deep space missions, preliminary analysis would likely require both light time correction and stellar aberration. Mission planning and operations will definitely need converged light-time calculations.
///
/// For more details, <https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/abcorr.html>.
///
/// # SPICE Validation
///
/// The validation test `validate_jplde_de440s_aberration_lt` checks 101,000 pairs of ephemeris computations and shows that the unconverged Light Time computation matches the SPICE computations almost all the time.
/// More specifically, the 99th percentile of error is less than 5 meters, the 75th percentile is less than one meter, and the median error is less than 2 millimeters.
///
/// :type name: str
/// :rtype: Aberration
#[derive(Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
pub struct Aberration {
    /// Indicates whether the light time calculations should be iterated upon (more precise but three times as many CPU cycles).
    pub converged: bool,
    /// Flag to denote if stellar aberration correction is applied. Stellar aberration is due to the motion of the observer (caused by Earth's orbit around the Sun).
    pub stellar: bool,
    /// Specifies whether in reception or transmission mode. True for 'transmit' mode, indicating the correction is applied to the transmitted signal from the observer to the target. False for 'receive' mode, for signals received from the target.
    pub transmit_mode: bool,
}

impl Aberration {
    /// Disables aberration corrections, e.g. all translations are geometric only (typical use case).
    pub const NONE: Option<Self> = None;
    /// Unconverged light time correction in reception mode without stellar aberration (e.g. a ground station targeting a spacecraft near the Moon)
    pub const LT: Option<Self> = Some(Self {
        converged: false,
        stellar: false,
        transmit_mode: false,
    });
    /// Unconverged light time correction in reception mode with stellar aberration
    pub const LT_S: Option<Self> = Some(Self {
        converged: false,
        stellar: true,
        transmit_mode: false,
    });
    /// Converged light time correction in reception mode without stellar aberration
    pub const CN: Option<Self> = Some(Self {
        converged: true,
        stellar: false,
        transmit_mode: false,
    });
    /// Converged light time correction in reception mode with stellar aberration
    pub const CN_S: Option<Self> = Some(Self {
        converged: true,
        stellar: true,
        transmit_mode: false,
    });
    /// Unconverged light time correction in transmission mode without stellar aberration (e.g. a Moon orbiter contacting a ground station)
    pub const XLT: Option<Self> = Some(Self {
        converged: false,
        stellar: false,
        transmit_mode: true,
    });
    /// Unconverged light time correction in transmission mode with stellar aberration
    pub const XLT_S: Option<Self> = Some(Self {
        converged: false,
        stellar: true,
        transmit_mode: true,
    });
    /// Converged light time correction in transmission mode without stellar aberration
    pub const XCN: Option<Self> = Some(Self {
        converged: true,
        stellar: false,
        transmit_mode: true,
    });
    /// Converged light time correction in transmission mode with stellar aberration
    pub const XCN_S: Option<Self> = Some(Self {
        converged: true,
        stellar: true,
        transmit_mode: true,
    });

    /// Initializes a new Aberration structure from one of the following (SPICE compatibility):
    /// + `NONE`: No correction
    /// + `LT`: unconverged light time, no stellar aberration, reception mode
    /// + `LT+S`: unconverged light time, with stellar aberration, reception mode
    /// + `CN`: converged light time, no stellar aberration, reception mode
    /// + `CN+S`: converged light time, with stellar aberration, reception mode
    /// + `XLT`: unconverged light time, no stellar aberration, transmission mode
    /// + `XLT+S`: unconverged light time, with stellar aberration, transmission mode
    /// + `XCN`: converged light time, no stellar aberration, transmission mode
    /// + `XCN+S`: converged light time, with stellar aberration, transmission mode
    pub fn new(flag: &str) -> PhysicsResult<Option<Self>> {
        match flag.trim() {
            "NONE" => Ok(Self::NONE),
            "LT" => Ok(Self::LT),
            "LT+S" => Ok(Self::LT_S),
            "CN" => Ok(Self::CN),
            "CN+S" => Ok(Self::CN_S),
            "XLT" => Ok(Self::XLT),
            "XLT+S" => Ok(Self::XLT_S),
            "XCN" => Ok(Self::XCN),
            "XCN+S" => Ok(Self::XCN_S),
            _ => Err(PhysicsError::AberrationError {
                action: "unknown aberration configuration name",
            }),
        }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl Aberration {
    /// Initializes a new Aberration structure from one of the following (SPICE compatibility):
    /// + `NONE`: No correction
    /// + `LT`: unconverged light time, no stellar aberration, reception mode
    /// + `LT+S`: unconverged light time, with stellar aberration, reception mode
    /// + `CN`: converged light time, no stellar aberration, reception mode
    /// + `CN+S`: converged light time, with stellar aberration, reception mode
    /// + `XLT`: unconverged light time, no stellar aberration, transmission mode
    /// + `XLT+S`: unconverged light time, with stellar aberration, transmission mode
    /// + `XCN`: converged light time, no stellar aberration, transmission mode
    /// + `XCN+S`: converged light time, with stellar aberration, transmission mode
    #[new]
    fn py_new(name: String) -> PhysicsResult<Self> {
        match Self::new(&name)? {
            Some(ab_corr) => Ok(ab_corr),
            None => Err(PhysicsError::AberrationError {
                action: "just uses `None` in Python instead",
            }),
        }
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self:?} (@{self:p})")
    }

    // Manual getters and setters for the stubs

    /// :rtype: bool
    #[getter]
    fn get_converged(&self) -> PyResult<bool> {
        Ok(self.converged)
    }

    /// :type converged: bool
    #[setter]
    fn set_converged(&mut self, converged: bool) -> PyResult<()> {
        self.converged = converged;
        Ok(())
    }
    /// :rtype: bool
    #[getter]
    fn get_stellar(&self) -> PyResult<bool> {
        Ok(self.stellar)
    }
    /// :type stellar: bool
    #[setter]
    fn set_stellar(&mut self, stellar: bool) -> PyResult<()> {
        self.stellar = stellar;
        Ok(())
    }
    /// :rtype: bool
    #[getter]
    fn get_transmit_mode(&self) -> PyResult<bool> {
        Ok(self.transmit_mode)
    }
    /// :type transmit_mode: bool
    #[setter]
    fn set_transmit_mode(&mut self, transmit_mode: bool) -> PyResult<()> {
        self.transmit_mode = transmit_mode;
        Ok(())
    }
}

impl fmt::Debug for Aberration {
    /// Prints this configuration as the SPICE name.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.transmit_mode {
            write!(f, "X")?;
        }
        if self.converged {
            write!(f, "CN")?;
        } else {
            write!(f, "LT")?;
        }
        if self.stellar {
            write!(f, "+S")?;
        }
        Ok(())
    }
}

impl fmt::Display for Aberration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.converged {
            write!(f, "converged ")?;
        } else {
            write!(f, "unconverged ")?;
        }
        write!(f, "light-time ")?;
        if self.stellar {
            write!(f, "and stellar aberration")?;
        } else {
            write!(f, "aberration")?;
        }
        if self.transmit_mode {
            write!(f, " in transmit mode")?;
        }
        Ok(())
    }
}

/// Returns the provided target [Orbit] with respect to any observer corrected for steller aberration.
///
/// # Arguments
///
/// + `target_pos_km`: the position of a target object with respect to the observer in kilometers
/// + `obs_wrt_ssb_vel_km_s`: the velocity of the observer with respect to the Solar System Barycenter in kilometers per second
/// + `ab_corr`: the [Aberration] correction
///
/// # Errors
///
/// This function will return an error in the following cases:
/// 1. the aberration is not set to include stellar corrections;
/// 1. the `target` is moving faster than the speed of light.
///
/// # Algorithm
/// Source: this algorithm and documentation were rewritten from NAIF's [`stelab`](https://github.com/nasa/kepler-pipeline/blob/f58b21df2c82969d8bd3e26a269bd7f5b9a770e1/source-code/matlab/fc/cspice-src-i686/cspice/stelab.c#L13) function:
///
/// Let r be the vector from the observer to the object, and v be the velocity of the observer with respect to the Solar System barycenter.
/// Let w be the angle between them. The aberration angle phi is given by
///
/// `sin(phi) = v sin(w) / c`
///
/// Let h be the vector given by the cross product
///
/// `h = r X v`
///
/// Rotate r by phi radians about h to obtain the apparent position of the object.
///
///
pub fn stellar_aberration(
    target_pos_km: Vector3,
    obs_wrt_ssb_vel_km_s: Vector3,
    ab_corr: Aberration,
) -> PhysicsResult<Vector3> {
    ensure!(
        ab_corr.stellar,
        AberrationSnafu {
            action: "stellar correction not available for this aberration"
        }
    );

    // Obtain the negative of the observer's velocity. This velocity, combined
    // with the target's position, will yield the inverse of the usual stellar
    // aberration correction, which is exactly what we seek.

    let obs_velocity_km_s = if ab_corr.transmit_mode {
        -obs_wrt_ssb_vel_km_s
    } else {
        obs_wrt_ssb_vel_km_s
    };

    // Get the velocity vector scaled with respect to the speed of light (v/c)
    let vbyc = obs_velocity_km_s / SPEED_OF_LIGHT_KM_S;

    ensure!(
        vbyc.dot(&vbyc) < 1.0,
        VelocitySnafu {
            action: "observer moving at or faster than light, cannot compute stellar aberration"
        }
    );

    // Get a unit vector that points in the direction of the object
    let u = target_pos_km.normalize();

    // Compute u_obj x (v/c)
    let h = u.cross(&vbyc);

    // Correct for stellar aberration
    let mut app_target_pos_km = target_pos_km;
    let sin_phi = h.norm();
    if sin_phi > f64::EPSILON {
        let phi = sin_phi.asin();
        app_target_pos_km = rotate_vector(&target_pos_km, &h, phi);
    }

    Ok(app_target_pos_km)
}

#[cfg(test)]
mod ut_aberration {
    #[test]
    fn test_display() {
        use super::Aberration;

        assert_eq!(format!("{:?}", Aberration::LT.unwrap()), "LT");
        assert_eq!(format!("{:?}", Aberration::LT_S.unwrap()), "LT+S");
        assert_eq!(format!("{:?}", Aberration::CN.unwrap()), "CN");
        assert_eq!(format!("{:?}", Aberration::CN_S.unwrap()), "CN+S");

        assert_eq!(format!("{:?}", Aberration::XLT.unwrap()), "XLT");
        assert_eq!(format!("{:?}", Aberration::XLT_S.unwrap()), "XLT+S");
        assert_eq!(format!("{:?}", Aberration::XCN.unwrap()), "XCN");
        assert_eq!(format!("{:?}", Aberration::XCN_S.unwrap()), "XCN+S");
    }
}
