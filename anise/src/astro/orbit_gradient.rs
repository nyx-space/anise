/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::analysis::prelude::OrbitalElement;
use crate::astro::orbit::ECC_EPSILON;
use crate::astro::PhysicsResult;
use crate::errors::PhysicsError;
use crate::prelude::{Frame, Orbit};
use crate::time::Epoch;
use core::f64::consts::TAU;
use core::fmt;
use hyperdual::linalg::norm;
use hyperdual::{Float, OHyperdual};
use log::{debug, error, warn};
use nalgebra::{Vector3, U7};

/// Define the gradient of an Orbit with respect to its of its Cartesian elements.
#[derive(Copy, Clone, Debug)]
pub struct OrbitGrad {
    pub x_km: OHyperdual<f64, U7>,
    pub y_km: OHyperdual<f64, U7>,
    pub z_km: OHyperdual<f64, U7>,
    pub vx_km_s: OHyperdual<f64, U7>,
    pub vy_km_s: OHyperdual<f64, U7>,
    pub vz_km_s: OHyperdual<f64, U7>,
    pub epoch: Epoch,
    pub frame: Frame,
}

impl From<Orbit> for OrbitGrad {
    /// Initialize a new OrbitDual from an orbit, no other initializers
    fn from(orbit: Orbit) -> Self {
        Self {
            x_km: OHyperdual::from_slice(&[orbit.radius_km.x, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            y_km: OHyperdual::from_slice(&[orbit.radius_km.y, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]),
            z_km: OHyperdual::from_slice(&[orbit.radius_km.z, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]),
            vx_km_s: OHyperdual::from_slice(&[orbit.velocity_km_s.x, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            vy_km_s: OHyperdual::from_slice(&[orbit.velocity_km_s.y, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]),
            vz_km_s: OHyperdual::from_slice(&[orbit.velocity_km_s.z, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
            epoch: orbit.epoch,
            frame: orbit.frame,
        }
    }
}

/// A type which stores the partial of an element
#[derive(Copy, Clone, Debug)]
pub struct OrbitalElementPartials {
    pub param: OrbitalElement,
    pub dual: OHyperdual<f64, U7>,
}

impl OrbitalElementPartials {
    /// Returns the real value of this parameter
    pub fn real(&self) -> f64 {
        self.dual[0]
    }
    /// The partial of this parameter with respect to X
    pub fn wtr_x(&self) -> f64 {
        self.dual[1]
    }
    /// The partial of this parameter with respect to Y
    pub fn wtr_y(&self) -> f64 {
        self.dual[2]
    }
    /// The partial of this parameter with respect to Z
    pub fn wtr_z(&self) -> f64 {
        self.dual[3]
    }
    /// The partial of this parameter with respect to VX
    pub fn wtr_vx(&self) -> f64 {
        self.dual[4]
    }
    /// The partial of this parameter with respect to VY
    pub fn wtr_vy(&self) -> f64 {
        self.dual[5]
    }
    /// The partial of this parameter with respect to VZ
    pub fn wtr_vz(&self) -> f64 {
        self.dual[6]
    }
}

impl fmt::Display for OrbitalElementPartials {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {}", self.param, self.dual)
    }
}

impl OrbitGrad {
    /// Returns the radius vector of this Orbit in [km, km, km]
    pub(crate) fn radius_km(&self) -> Vector3<OHyperdual<f64, U7>> {
        Vector3::new(self.x_km, self.y_km, self.z_km)
    }

    /// Returns the velocity vector of this Orbit in [km/s, km/s, km/s]
    pub(crate) fn velocity_km_s(&self) -> Vector3<OHyperdual<f64, U7>> {
        Vector3::new(self.vx_km_s, self.vy_km_s, self.vz_km_s)
    }

    /// Returns the orbital momentum vector
    pub(crate) fn hvec(&self) -> Vector3<OHyperdual<f64, U7>> {
        self.radius_km().cross(&self.velocity_km_s())
    }

    /// Returns the eccentricity vector (no unit)
    pub(crate) fn evec(&self) -> PhysicsResult<Vector3<OHyperdual<f64, U7>>> {
        let r = self.radius_km();
        let v = self.velocity_km_s();
        let hgm = OHyperdual::from(self.frame.mu_km3_s2()?);
        // Split up this operation because it doesn't seem to be implemented
        // ((norm(&v).powi(2) - hgm / norm(&r)) * r - (r.dot(&v)) * v) / hgm
        Ok(Vector3::new(
            ((norm(&v).powi(2) - hgm / norm(&r)) * r[0] - (r.dot(&v)) * v[0]) / hgm,
            ((norm(&v).powi(2) - hgm / norm(&r)) * r[1] - (r.dot(&v)) * v[1]) / hgm,
            ((norm(&v).powi(2) - hgm / norm(&r)) * r[2] - (r.dot(&v)) * v[2]) / hgm,
        ))
    }

    pub fn partial_for(&self, param: OrbitalElement) -> PhysicsResult<OrbitalElementPartials> {
        match param {
            OrbitalElement::X => Ok(OrbitalElementPartials {
                dual: self.x_km,
                param: OrbitalElement::X,
            }),
            OrbitalElement::Y => Ok(OrbitalElementPartials {
                dual: self.y_km,
                param: OrbitalElement::Y,
            }),
            OrbitalElement::Z => Ok(OrbitalElementPartials {
                dual: self.z_km,
                param: OrbitalElement::Z,
            }),
            OrbitalElement::VX => Ok(OrbitalElementPartials {
                dual: self.vx_km_s,
                param: OrbitalElement::VX,
            }),
            OrbitalElement::VY => Ok(OrbitalElementPartials {
                dual: self.vy_km_s,
                param: OrbitalElement::VY,
            }),
            OrbitalElement::VZ => Ok(OrbitalElementPartials {
                dual: self.vz_km_s,
                param: OrbitalElement::VZ,
            }),
            OrbitalElement::Rmag => Ok(self.rmag_km()),
            OrbitalElement::Vmag => Ok(self.vmag_km_s()),
            OrbitalElement::HX => Ok(self.hx()),
            OrbitalElement::HY => Ok(self.hy()),
            OrbitalElement::HZ => Ok(self.hz()),
            OrbitalElement::Hmag => Ok(self.hmag()),
            OrbitalElement::Energy => self.energy_km2_s2(),
            OrbitalElement::SemiMajorAxis => self.sma_km(),
            OrbitalElement::Eccentricity => self.ecc(),
            OrbitalElement::Inclination => Ok(self.inc_deg()),
            OrbitalElement::AoP => self.aop_deg(),
            OrbitalElement::AoL => self.aol_deg(),
            OrbitalElement::RAAN => Ok(self.raan_deg()),
            OrbitalElement::PeriapsisRadius => self.periapsis_km(),
            OrbitalElement::ApoapsisRadius => self.apoapsis_km(),
            OrbitalElement::PeriapsisAltitude => self.periapsis_altitude_km(),
            OrbitalElement::ApoapsisAltitude => self.apoapsis_altitude_km(),
            OrbitalElement::TrueLongitude => self.tlong_deg(),
            OrbitalElement::FlightPathAngle => self.fpa_deg(),
            OrbitalElement::MeanAnomaly => self.ma_deg(),
            OrbitalElement::EccentricAnomaly => self.ea_deg(),
            OrbitalElement::Height => self.height_km(),
            OrbitalElement::Latitude => self.latitude_deg(),
            OrbitalElement::Longitude => Ok(self.longitude_deg()),
            OrbitalElement::C3 => self.c3(),
            OrbitalElement::RightAscension => Ok(self.right_ascension_deg()),
            OrbitalElement::Declination => Ok(self.declination_deg()),
            OrbitalElement::HyperbolicAnomaly => self.hyperbolic_anomaly_deg(),
            OrbitalElement::SemiParameter => self.semi_parameter_km(),
            OrbitalElement::SemiMinorAxis => self.semi_minor_axis_km(),
            OrbitalElement::TrueAnomaly => self.ta_deg(),
            OrbitalElement::Period => self.period_s(),
            _ => Err(PhysicsError::PartialsNotYetDefined),
        }
    }

    /// Returns the magnitude of the radius vector in km
    pub fn rmag_km(&self) -> OrbitalElementPartials {
        OrbitalElementPartials {
            param: OrbitalElement::Rmag,
            dual: (self.x_km.powi(2) + self.y_km.powi(2) + self.z_km.powi(2)).sqrt(),
        }
    }

    /// Returns the magnitude of the velocity vector in km/s
    pub fn vmag_km_s(&self) -> OrbitalElementPartials {
        OrbitalElementPartials {
            param: OrbitalElement::Vmag,
            dual: (self.vx_km_s.powi(2) + self.vy_km_s.powi(2) + self.vz_km_s.powi(2)).sqrt(),
        }
    }

    /// Returns the orbital momentum value on the X axis
    pub fn hx(&self) -> OrbitalElementPartials {
        OrbitalElementPartials {
            dual: self.hvec()[0],
            param: OrbitalElement::HX,
        }
    }

    /// Returns the orbital momentum value on the Y axis
    pub fn hy(&self) -> OrbitalElementPartials {
        OrbitalElementPartials {
            dual: self.hvec()[1],
            param: OrbitalElement::HY,
        }
    }

    /// Returns the orbital momentum value on the Z axis
    pub fn hz(&self) -> OrbitalElementPartials {
        OrbitalElementPartials {
            dual: self.hvec()[2],
            param: OrbitalElement::HZ,
        }
    }

    /// Returns the norm of the orbital momentum
    pub fn hmag(&self) -> OrbitalElementPartials {
        OrbitalElementPartials {
            dual: norm(&self.hvec()),
            param: OrbitalElement::Hmag,
        }
    }

    /// Returns the specific mechanical energy
    pub fn energy_km2_s2(&self) -> PhysicsResult<OrbitalElementPartials> {
        Ok(OrbitalElementPartials {
            dual: self.vmag_km_s().dual.powi(2) / OHyperdual::from(2.0)
                - OHyperdual::from(self.frame.mu_km3_s2()?) / self.rmag_km().dual,
            param: OrbitalElement::Energy,
        })
    }

    /// Returns the semi-major axis in km
    pub fn sma_km(&self) -> PhysicsResult<OrbitalElementPartials> {
        Ok(OrbitalElementPartials {
            dual: -OHyperdual::from(self.frame.mu_km3_s2()?)
                / (OHyperdual::from(2.0) * self.energy_km2_s2()?.dual),
            param: OrbitalElement::SemiMajorAxis,
        })
    }

    /// Returns the eccentricity (no unit)
    pub fn ecc(&self) -> PhysicsResult<OrbitalElementPartials> {
        Ok(OrbitalElementPartials {
            dual: norm(&self.evec()?),
            param: OrbitalElement::Eccentricity,
        })
    }

    /// Returns the inclination in degrees
    pub fn inc_deg(&self) -> OrbitalElementPartials {
        OrbitalElementPartials {
            dual: (self.hvec()[(2, 0)] / self.hmag().dual).acos().to_degrees(),
            param: OrbitalElement::Inclination,
        }
    }

    /// Returns the argument of periapsis in degrees
    pub fn aop_deg(&self) -> PhysicsResult<OrbitalElementPartials> {
        let n = Vector3::new(
            OHyperdual::from(0.0),
            OHyperdual::from(0.0),
            OHyperdual::from(1.0),
        )
        .cross(&self.hvec());
        let aop = (n.dot(&self.evec()?) / (norm(&n) * self.ecc()?.dual)).acos();
        if aop.is_nan() {
            warn!("AoP is NaN");
            Ok(OrbitalElementPartials {
                dual: OHyperdual::from(0.0),
                param: OrbitalElement::AoP,
            })
        } else if self.evec()?[2].real() < 0.0 {
            Ok(OrbitalElementPartials {
                dual: (OHyperdual::from(TAU) - aop).to_degrees(),
                param: OrbitalElement::AoP,
            })
        } else {
            Ok(OrbitalElementPartials {
                dual: aop.to_degrees(),
                param: OrbitalElement::AoP,
            })
        }
    }

    /// Returns the right ascension of ther ascending node in degrees
    pub fn raan_deg(&self) -> OrbitalElementPartials {
        let n = Vector3::new(
            OHyperdual::from(0.0),
            OHyperdual::from(0.0),
            OHyperdual::from(1.0),
        )
        .cross(&self.hvec());
        let raan = (n[(0, 0)] / norm(&n)).acos();
        if raan.is_nan() {
            warn!("RAAN is NaN");
            OrbitalElementPartials {
                dual: OHyperdual::from(0.0),
                param: OrbitalElement::RAAN,
            }
        } else if n[(1, 0)] < 0.0 {
            OrbitalElementPartials {
                dual: (OHyperdual::from(TAU) - raan).to_degrees(),
                param: OrbitalElement::RAAN,
            }
        } else {
            OrbitalElementPartials {
                dual: raan.to_degrees(),
                param: OrbitalElement::RAAN,
            }
        }
    }

    /// Returns the true anomaly in degrees between 0 and 360.0
    pub fn ta_deg(&self) -> PhysicsResult<OrbitalElementPartials> {
        if self.ecc()?.real() < ECC_EPSILON {
            warn!(
                "true anomaly ill-defined for circular orbit (e = {})",
                self.ecc()?
            );
        }

        // Compute cosine of true anomaly
        let cos_nu = self.evec()?.dot(&self.radius_km()) / (self.ecc()?.dual * self.rmag_km().dual);

        // Align with orbit.rs: Try acos() first.
        // If cos_nu is slightly > 1.0 or < -1.0 due to noise, acos returns NaN.
        let ta = cos_nu.acos();

        if ta.real().is_nan() {
            // Handle the numerical noise edge cases
            if cos_nu.real() > 1.0 {
                Ok(OrbitalElementPartials {
                    dual: OHyperdual::from(180.0),
                    param: OrbitalElement::TrueAnomaly,
                })
            } else {
                Ok(OrbitalElementPartials {
                    dual: OHyperdual::from(0.0),
                    param: OrbitalElement::TrueAnomaly,
                })
            }
        } else if self.radius_km().dot(&self.velocity_km_s()).real() < 0.0 {
            // Quadrant check
            Ok(OrbitalElementPartials {
                dual: (OHyperdual::from(TAU) - ta).to_degrees(),
                param: OrbitalElement::TrueAnomaly,
            })
        } else {
            Ok(OrbitalElementPartials {
                dual: ta.to_degrees(),
                param: OrbitalElement::TrueAnomaly,
            })
        }
    }

    /// Returns the true longitude in degrees
    pub fn tlong_deg(&self) -> PhysicsResult<OrbitalElementPartials> {
        // Angles already in degrees
        Ok(OrbitalElementPartials {
            dual: self.aop_deg()?.dual + self.raan_deg().dual + self.ta_deg()?.dual,
            param: OrbitalElement::TrueLongitude,
        })
    }

    /// Returns the argument of latitude in degrees
    ///
    /// NOTE: If the orbit is near circular, the AoL will be computed from the true longitude
    /// instead of relying on the ill-defined true anomaly.
    pub fn aol_deg(&self) -> PhysicsResult<OrbitalElementPartials> {
        if self.ecc()?.real() < ECC_EPSILON {
            Ok(OrbitalElementPartials {
                dual: self.tlong_deg()?.dual - self.raan_deg().dual,
                param: OrbitalElement::AoL,
            })
        } else {
            Ok(OrbitalElementPartials {
                dual: self.aop_deg()?.dual + self.ta_deg()?.dual,
                param: OrbitalElement::AoL,
            })
        }
    }

    /// Returns the radius of periapsis (or perigee around Earth), in kilometers.
    pub fn periapsis_km(&self) -> PhysicsResult<OrbitalElementPartials> {
        Ok(OrbitalElementPartials {
            dual: self.sma_km()?.dual * (OHyperdual::from(1.0) - self.ecc()?.dual),
            param: OrbitalElement::PeriapsisRadius,
        })
    }

    /// Returns the radius of apoapsis (or apogee around Earth), in kilometers.
    pub fn apoapsis_km(&self) -> PhysicsResult<OrbitalElementPartials> {
        Ok(OrbitalElementPartials {
            dual: self.sma_km()?.dual * (OHyperdual::from(1.0) + self.ecc()?.dual),
            param: OrbitalElement::ApoapsisRadius,
        })
    }

    /// Returns the eccentric anomaly in degrees
    ///
    /// This is a conversion from GMAT's StateConversionUtil::TrueToEccentricAnomaly
    pub fn ea_deg(&self) -> PhysicsResult<OrbitalElementPartials> {
        let (sin_ta, cos_ta) = self.ta_deg()?.dual.to_radians().sin_cos();
        let ecc_cos_ta = self.ecc()?.dual * cos_ta;
        let sin_ea = ((OHyperdual::from(1.0) - self.ecc()?.dual.powi(2)).sqrt() * sin_ta)
            / (OHyperdual::from(1.0) + ecc_cos_ta);
        let cos_ea = (self.ecc()?.dual + cos_ta) / (OHyperdual::from(1.0) + ecc_cos_ta);
        // The atan2 function is a bit confusing: https://doc.rust-lang.org/std/primitive.f64.html#method.atan2 .
        Ok(OrbitalElementPartials {
            dual: sin_ea.atan2(cos_ea).to_degrees(),
            param: OrbitalElement::EccentricAnomaly,
        })
    }

    /// Returns the flight path angle in degrees
    pub fn fpa_deg(&self) -> PhysicsResult<OrbitalElementPartials> {
        let nu = self.ta_deg()?.dual.to_radians();
        let ecc = self.ecc()?.dual;
        let denom =
            (OHyperdual::from(1.0) + OHyperdual::from(2.0) * ecc * nu.cos() + ecc.powi(2)).sqrt();
        let sin_fpa = ecc * nu.sin() / denom;
        let cos_fpa = OHyperdual::from(1.0) + ecc * nu.cos() / denom;
        Ok(OrbitalElementPartials {
            dual: sin_fpa.atan2(cos_fpa).to_degrees(),
            param: OrbitalElement::FlightPathAngle,
        })
    }

    /// Returns the mean anomaly in degrees
    pub fn ma_deg(&self) -> PhysicsResult<OrbitalElementPartials> {
        if self.ecc()?.real().abs() < ECC_EPSILON {
            Err(PhysicsError::ParabolicEccentricity { limit: ECC_EPSILON })
        } else if self.ecc()?.real() < 1.0 {
            Ok(OrbitalElementPartials {
                dual: (self.ea_deg()?.dual.to_radians()
                    - self.ecc()?.dual * self.ea_deg()?.dual.to_radians().sin())
                .to_degrees(),
                param: OrbitalElement::MeanAnomaly,
            })
        } else {
            debug!("computing the hyperbolic anomaly");
            // From GMAT's TrueToHyperbolicAnomaly
            Ok(OrbitalElementPartials {
                dual: ((self.ta_deg()?.dual.to_radians().sin()
                    * (self.ecc()?.dual.powi(2) - OHyperdual::from(1.0)))
                .sqrt()
                    / (OHyperdual::from(1.0)
                        + self.ecc()?.dual * self.ta_deg()?.dual.to_radians().cos()))
                .asinh()
                .to_degrees(),
                param: OrbitalElement::MeanAnomaly,
            })
        }
    }

    /// Returns the semi parameter (or semilatus rectum)
    pub fn semi_parameter_km(&self) -> PhysicsResult<OrbitalElementPartials> {
        Ok(OrbitalElementPartials {
            dual: self.sma_km()?.dual * (OHyperdual::from(1.0) - self.ecc()?.dual.powi(2)),
            param: OrbitalElement::SemiParameter,
        })
    }

    /// Returns the geodetic longitude (λ) in degrees. Value is between 0 and 360 degrees.
    ///
    /// Although the reference is not Vallado, the math from Vallado proves to be equivalent.
    /// Reference: G. Xu and Y. Xu, "GPS", DOI 10.1007/978-3-662-50367-6_2, 2016
    pub fn longitude_deg(&self) -> OrbitalElementPartials {
        OrbitalElementPartials {
            dual: self.y_km.atan2(self.x_km).to_degrees(),
            param: OrbitalElement::Longitude,
        }
    }

    /// Returns the geodetic latitude (φ) in degrees. Value is between -180 and +180 degrees.
    ///
    /// Reference: Vallado, 4th Ed., Algorithm 12 page 172.
    pub fn latitude_deg(&self) -> PhysicsResult<OrbitalElementPartials> {
        let flattening = self.frame.flattening()?;
        let eps = 1e-12;
        let max_attempts = 20;
        let mut attempt_no = 0;
        let r_delta = (self.x_km.powi(2) + self.y_km.powi(2)).sqrt();
        let mut latitude = (self.z_km / self.rmag_km().dual).asin();
        let e2 = OHyperdual::from(flattening * (2.0 - flattening));
        loop {
            attempt_no += 1;
            let c_earth = OHyperdual::from(self.frame.semi_major_radius_km()?)
                / ((OHyperdual::from(1.0) - e2 * (latitude).sin().powi(2)).sqrt());
            let new_latitude = (self.z_km + c_earth * e2 * (latitude).sin()).atan2(r_delta);
            if (latitude - new_latitude).abs() < eps {
                return Ok(OrbitalElementPartials {
                    dual: new_latitude.to_degrees(),
                    param: OrbitalElement::Latitude,
                });
            } else if attempt_no >= max_attempts {
                error!(
                    "geodetic latitude failed to converge -- error = {}",
                    (latitude - new_latitude).abs()
                );
                return Ok(OrbitalElementPartials {
                    dual: new_latitude.to_degrees(),
                    param: OrbitalElement::Latitude,
                });
            }
            latitude = new_latitude;
        }
    }

    /// Returns the geodetic height in km.
    ///
    /// Reference: Vallado, 4th Ed., Algorithm 12 page 172.
    pub fn height_km(&self) -> PhysicsResult<OrbitalElementPartials> {
        let flattening = self.frame.flattening()?;
        let semi_major_radius = self.frame.semi_major_radius_km()?;
        let e2 = OHyperdual::from(flattening * (2.0 - flattening));
        let latitude = self.latitude_deg()?.dual.to_radians();
        let sin_lat = latitude.sin();
        if (latitude - OHyperdual::from(1.0)).abs() < 0.1 {
            // We are near poles, let's use another formulation.
            let s_earth0: f64 = semi_major_radius * (1.0 - flattening).powi(2);
            let s_earth = OHyperdual::from(s_earth0)
                / ((OHyperdual::from(1.0) - e2 * sin_lat.powi(2)).sqrt());
            Ok(OrbitalElementPartials {
                dual: self.z_km / latitude.sin() - s_earth,
                param: OrbitalElement::Height,
            })
        } else {
            let c_earth = OHyperdual::from(semi_major_radius)
                / ((OHyperdual::from(1.0) - e2 * sin_lat.powi(2)).sqrt());
            let r_delta = (self.x_km.powi(2) + self.y_km.powi(2)).sqrt();
            Ok(OrbitalElementPartials {
                dual: r_delta / latitude.cos() - c_earth,
                param: OrbitalElement::Height,
            })
        }
    }

    /// Returns the right ascension of this orbit in degrees
    #[allow(clippy::eq_op)]
    pub fn right_ascension_deg(&self) -> OrbitalElementPartials {
        OrbitalElementPartials {
            dual: (self.y_km.atan2(self.x_km)).to_degrees(),
            param: OrbitalElement::RightAscension,
        }
    }

    /// Returns the declination of this orbit in degrees
    #[allow(clippy::eq_op)]
    pub fn declination_deg(&self) -> OrbitalElementPartials {
        OrbitalElementPartials {
            dual: (self.z_km / self.rmag_km().dual).asin().to_degrees(),
            param: OrbitalElement::Declination,
        }
    }

    /// Returns the semi minor axis in km, includes code for a hyperbolic orbit
    pub fn semi_minor_axis_km(&self) -> PhysicsResult<OrbitalElementPartials> {
        if self.ecc()?.real() <= 1.0 {
            Ok(OrbitalElementPartials {
                dual: (self.sma_km()?.dual.powi(2)
                    - (self.sma_km()?.dual * self.ecc()?.dual).powi(2))
                .sqrt(),
                param: OrbitalElement::SemiMinorAxis,
            })
        } else {
            Ok(OrbitalElementPartials {
                dual: self.hmag().dual.powi(2)
                    / (OHyperdual::from(self.frame.mu_km3_s2()?)
                        * (self.ecc()?.dual.powi(2) - OHyperdual::from(1.0)).sqrt()),
                param: OrbitalElement::SemiMinorAxis,
            })
        }
    }

    /// Returns the velocity declination of this orbit in degrees
    pub fn velocity_declination(&self) -> OrbitalElementPartials {
        OrbitalElementPartials {
            dual: (self.vz_km_s / self.vmag_km_s().dual).asin().to_degrees(),
            param: OrbitalElement::VelocityDeclination,
        }
    }

    /// Returns the $C_3$ of this orbit
    pub fn c3(&self) -> PhysicsResult<OrbitalElementPartials> {
        Ok(OrbitalElementPartials {
            dual: -OHyperdual::from(self.frame.mu_km3_s2()?) / self.sma_km()?.dual,
            param: OrbitalElement::C3,
        })
    }

    /// Returns the hyperbolic anomaly in degrees between 0 and 360.0
    pub fn hyperbolic_anomaly_deg(&self) -> PhysicsResult<OrbitalElementPartials> {
        let ecc = self.ecc()?;
        if ecc.real() <= 1.0 {
            Err(PhysicsError::NotHyperbolic { ecc: ecc.real() })
        } else {
            let (sin_ta, cos_ta) = self.ta_deg()?.dual.to_radians().sin_cos();
            let sinh_h = (sin_ta * (ecc.dual.powi(2) - OHyperdual::from(1.0)).sqrt())
                / (OHyperdual::from(1.0) + ecc.dual * cos_ta);
            Ok(OrbitalElementPartials {
                dual: sinh_h.asinh().to_degrees(),
                param: OrbitalElement::HyperbolicAnomaly,
            })
        }
    }

    /// Returns the altitude of periapsis (or perigee around Earth), in kilometers.
    pub fn periapsis_altitude_km(&self) -> PhysicsResult<OrbitalElementPartials> {
        Ok(OrbitalElementPartials {
            dual: self.sma_km()?.dual * (OHyperdual::from(1.0) - self.ecc()?.dual)
                - OHyperdual::from(self.frame.mean_equatorial_radius_km()?),
            param: OrbitalElement::PeriapsisRadius,
        })
    }

    /// Returns the altitude of apoapsis (or apogee around Earth), in kilometers.
    pub fn apoapsis_altitude_km(&self) -> PhysicsResult<OrbitalElementPartials> {
        Ok(OrbitalElementPartials {
            dual: self.sma_km()?.dual * (OHyperdual::from(1.0) + self.ecc()?.dual)
                - OHyperdual::from(self.frame.mean_equatorial_radius_km()?),
            param: OrbitalElement::ApoapsisRadius,
        })
    }

    /// Returns the period in seconds
    pub fn period_s(&self) -> PhysicsResult<OrbitalElementPartials> {
        Ok(OrbitalElementPartials {
            dual: OHyperdual::from(TAU)
                * (self.sma_km()?.dual.powi(3) / OHyperdual::from(self.frame.mu_km3_s2()?)).sqrt(),
            param: OrbitalElement::ApoapsisRadius,
        })
    }
}
