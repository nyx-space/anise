/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use serde::{Deserialize, Serialize};
use snafu::ResultExt;

#[cfg(feature = "python")]
use pyo3::exceptions::PyException;
#[cfg(feature = "python")]
use pyo3::prelude::*;

use super::{AnalysisError, PhysicsOrbitElSnafu};
use crate::prelude::Orbit;

/// Orbital element defines all of the supported orbital elements in ANISE, which are all built from a State.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.analysis"))]
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum OrbitalElement {
    /// Argument of Latitude (deg)
    AoL,
    /// Argument of Periapse (deg)
    AoP,
    /// Radius of apoapsis (km)
    ApoapsisRadius,
    /// Altitude of apoapsis (km)
    ApoapsisAltitude,
    /// C_3 in (km/s)^2
    C3,
    /// Declination (deg) (also called elevation if in a body fixed frame)
    Declination,
    /// Eccentric anomaly (deg)
    EccentricAnomaly,
    /// Eccentricity (no unit)
    Eccentricity,
    /// Specific energy
    Energy,
    /// Flight path angle (deg)
    FlightPathAngle,
    /// Geodetic height (km)
    Height,
    /// Geodetic latitude (deg)
    Latitude,
    /// Geodetic longitude (deg)
    Longitude,
    /// Orbital momentum
    Hmag,
    /// X component of the orbital momentum vector
    HX,
    /// Y component of the orbital momentum vector
    HY,
    /// Z component of the orbital momentum vector
    HZ,
    /// Hyperbolic anomaly (deg), only valid for hyperbolic orbits
    HyperbolicAnomaly,
    /// Inclination (deg)
    Inclination,
    /// Mean anomaly (deg)
    MeanAnomaly,
    /// Radius of periapse (km)
    PeriapsisRadius,
    /// Altitude of periapse (km)
    PeriapsisAltitude,
    /// Orbital period (s)
    Period,
    /// Right ascension (deg)
    RightAscension,
    /// Right ascension of the ascending node (deg)
    RAAN,
    /// Norm of the radius vector
    Rmag,
    /// Semi parameter (km)
    SemiParameter,
    /// Semi major axis (km)
    SemiMajorAxis,
    /// Semi minor axis (km)
    SemiMinorAxis,
    /// True anomaly
    TrueAnomaly,
    /// True longitude
    TrueLongitude,
    /// Velocity declination (deg)
    VelocityDeclination,
    /// Norm of the velocity vector (km/s)
    Vmag,
    /// X component of the radius (km)
    X,
    /// Y component of the radius (km)
    Y,
    /// Z component of the radius (km)
    Z,
    /// X component of the velocity (km/s)
    VX,
    /// Y component of the velocity (km/s)
    VY,
    /// Z component of the velocity (km/s)
    VZ,
}

impl OrbitalElement {
    /// Computes this scalar expression for the provided orbit.
    pub fn evaluate(self, orbit: Orbit) -> Result<f64, AnalysisError> {
        match self {
            Self::AoL => orbit
                .aol_deg()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::AoP => orbit
                .aop_deg()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::ApoapsisRadius => orbit
                .apoapsis_km()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::ApoapsisAltitude => orbit
                .apoapsis_altitude_km()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::C3 => orbit
                .c3_km2_s2()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::Declination => Ok(orbit.declination_deg()),
            Self::EccentricAnomaly => orbit
                .ea_deg()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::Eccentricity => orbit.ecc().context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::Energy => orbit
                .energy_km2_s2()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::FlightPathAngle => orbit
                .fpa_deg()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::Height => orbit
                .height_km()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::Latitude => orbit
                .latitude_deg()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::Longitude => Ok(orbit.longitude_deg()),
            Self::Hmag => orbit
                .hmag()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::HX => orbit.hx().context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::HY => orbit.hy().context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::HZ => orbit.hz().context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::HyperbolicAnomaly => orbit
                .hyperbolic_anomaly_deg()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::Inclination => orbit
                .inc_deg()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::MeanAnomaly => orbit
                .ma_deg()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::PeriapsisRadius => orbit
                .periapsis_km()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::PeriapsisAltitude => orbit
                .periapsis_altitude_km()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::Period => Ok(orbit
                .period()
                .context(PhysicsOrbitElSnafu { el: self, orbit })?
                .to_seconds()),
            Self::RightAscension => Ok(orbit.right_ascension_deg()),
            Self::RAAN => orbit
                .raan_deg()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::Rmag => Ok(orbit.rmag_km()),
            Self::SemiParameter => orbit
                .semi_parameter_km()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::SemiMajorAxis => orbit
                .sma_km()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::SemiMinorAxis => orbit
                .semi_minor_axis_km()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::TrueAnomaly => orbit
                .ta_deg()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::TrueLongitude => orbit
                .tlong_deg()
                .context(PhysicsOrbitElSnafu { el: self, orbit }),
            Self::VelocityDeclination => Ok(orbit.velocity_declination_deg()),
            Self::Vmag => Ok(orbit.vmag_km_s()),
            Self::VX => Ok(orbit.velocity_km_s.x),
            Self::VY => Ok(orbit.velocity_km_s.y),
            Self::VZ => Ok(orbit.velocity_km_s.z),
            Self::X => Ok(orbit.radius_km.x),
            Self::Y => Ok(orbit.radius_km.y),
            Self::Z => Ok(orbit.radius_km.z),
        }
    }

    pub const fn is_angle(&self) -> bool {
        matches!(
            self,
            Self::AoL
                | Self::AoP
                | Self::Declination
                | Self::EccentricAnomaly
                | Self::FlightPathAngle
                | Self::Latitude
                | Self::Longitude
                | Self::HyperbolicAnomaly
                | Self::Inclination
                | Self::MeanAnomaly
                | Self::RightAscension
                | Self::RAAN
                | Self::TrueAnomaly
                | Self::TrueLongitude
                | Self::VelocityDeclination
        )
    }

    pub const fn unit(&self) -> &'static str {
        match self {
            // Angles
            Self::AoL
            | Self::AoP
            | Self::Declination
            | Self::Latitude
            | Self::Longitude
            | Self::FlightPathAngle
            | Self::Inclination
            | Self::RightAscension
            | Self::RAAN
            | Self::TrueLongitude
            | Self::VelocityDeclination
            | Self::MeanAnomaly
            | Self::EccentricAnomaly
            | Self::HyperbolicAnomaly
            | Self::TrueAnomaly => "deg",

            // Distances
            Self::ApoapsisRadius
            | Self::ApoapsisAltitude
            | Self::Height
            | Self::Hmag
            | Self::HX
            | Self::HY
            | Self::HZ
            | Self::PeriapsisRadius
            | Self::PeriapsisAltitude
            | Self::Rmag
            | Self::SemiParameter
            | Self::SemiMajorAxis
            | Self::SemiMinorAxis
            | Self::X
            | Self::Y
            | Self::Z => "km",

            // Velocities
            Self::VX | Self::VY | Self::VZ | Self::Vmag => "km/s",

            Self::C3 | Self::Energy => "km^2/s^2",
            Self::Eccentricity => "unitless",
            Self::Period => "s",
        }
    }
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "python", pymethods)]
impl OrbitalElement {
    /// Evaluate the orbital element enum variant for the provided orbit
    #[pyo3(name = "evaluate", signature=(orbit))]
    pub fn py_evaluate(&self, orbit: Orbit) -> Result<f64, PyErr> {
        self.evaluate(orbit)
            .map_err(|e| PyException::new_err(e.to_string()))
    }
    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
    fn __ne__(&self, other: &Self) -> bool {
        self != other
    }
}
