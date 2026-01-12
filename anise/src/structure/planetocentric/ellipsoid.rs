/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::math::Vector3;
use core::fmt;
use der::{Decode, Encode, Reader, Writer};
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "metaload")]
use serde_dhall::StaticType;

#[cfg(feature = "python")]
use pyo3::exceptions::PyTypeError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::pyclass::CompareOp;

/// Only the tri-axial Ellipsoid shape model is currently supported by ANISE.
/// This is directly inspired from SPICE PCK.
/// > For each body, three radii are listed: The first number is
/// > the largest equatorial radius (the length of the semi-axis
/// > containing the prime meridian), the second number is the smaller
/// > equatorial radius, and the third is the polar radius.
///
/// Example: Radii of the Earth.
///
///    BODY399_RADII     = ( 6378.1366   6378.1366   6356.7519 )
///
/// :type semi_major_equatorial_radius_km: float
/// :type polar_radius_km: float, optional
/// :type semi_minor_equatorial_radius_km: float, optional
/// :rtype: Ellipsoid
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "metaload", derive(StaticType))]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct Ellipsoid {
    pub semi_major_equatorial_radius_km: f64,
    pub semi_minor_equatorial_radius_km: f64,
    pub polar_radius_km: f64,
}

impl Ellipsoid {
    /// Builds an ellipsoid as if it were a sphere
    pub fn from_sphere(radius_km: f64) -> Self {
        Self {
            semi_major_equatorial_radius_km: radius_km,
            semi_minor_equatorial_radius_km: radius_km,
            polar_radius_km: radius_km,
        }
    }

    /// Builds an ellipsoid as if it were a spheroid, where only the polar axis has a different radius
    pub fn from_spheroid(equatorial_radius_km: f64, polar_radius_km: f64) -> Self {
        Self {
            semi_major_equatorial_radius_km: equatorial_radius_km,
            semi_minor_equatorial_radius_km: equatorial_radius_km,
            polar_radius_km,
        }
    }

    /// Computes the intersection of a ray defined by `view_point` and `view_direction` with the ellipsoid.
    ///
    /// This is functionally equivalent to the SPICE routine `surfpt_c`.
    ///
    /// # Arguments
    /// * `view_point` - The origin of the ray (e.g. spacecraft position in Body Fixed frame).
    /// * `view_direction` - The direction vector of the ray (e.g. instrument boresight in Body Fixed frame).
    ///
    /// # Returns
    /// * `Some(Vector3)` - The Cartesian coordinates of the first intersection point on the surface.
    /// * `None` - If the ray does not intersect the ellipsoid.
    pub fn intersect(&self, view_point: Vector3, view_direction: Vector3) -> Option<Vector3> {
        let a = self.semi_major_equatorial_radius_km;
        let b = self.semi_minor_equatorial_radius_km;
        let c = self.polar_radius_km;

        // 1. Scale to Unit Sphere Space: P' = [x/a, y/b, z/c]
        // We do this manually to avoid constructing a scaling matrix
        let origin = Vector3::new(view_point.x / a, view_point.y / b, view_point.z / c);
        let direction = Vector3::new(
            view_direction.x / a,
            view_direction.y / b,
            view_direction.z / c,
        );

        // 2. Quadratic Equation: |O' + t*D'|^2 = 1
        // (D' . D')t^2 + 2(O' . D')t + (O' . O' - 1) = 0
        let a_coeff = direction.dot(&direction);
        let b_coeff = 2.0 * origin.dot(&direction);
        let c_coeff = origin.dot(&origin) - 1.0;

        let discriminant = b_coeff * b_coeff - 4.0 * a_coeff * c_coeff;

        if discriminant < 0.0 {
            return None; // Ray misses
        }

        // 3. Solve for t
        let sqrt_disc = discriminant.sqrt();
        let t1 = (-b_coeff - sqrt_disc) / (2.0 * a_coeff);
        let t2 = (-b_coeff + sqrt_disc) / (2.0 * a_coeff);

        // 4. Select closest positive t
        // Use a small epsilon to avoid finding the "origin" if we are already on the surface
        let t = if t1 > 1e-9 {
            t1
        } else if t2 > 1e-9 {
            t2
        } else {
            return None; // Intersection is behind
        };

        // 5. Unscale
        Some(view_point + view_direction * t)
    }

    /// Computes the unit normal vector at a specific point on the surface of the ellipsoid.
    ///
    /// The input `surface_point` must be in the same frame as the ellipsoid definition
    /// (typically the Body-Fixed frame).
    ///
    /// # Math
    /// For an ellipsoid (x/a)^2 + (y/b)^2 + (z/c)^2 = 1, the gradient vector is:
    /// ∇f = [ 2x/a^2, 2y/b^2, 2z/c^2 ]
    pub fn surface_normal(&self, surface_point: Vector3) -> Vector3 {
        Vector3::new(
            surface_point.x / self.semi_major_equatorial_radius_km.powi(2),
            surface_point.y / self.semi_minor_equatorial_radius_km.powi(2),
            surface_point.z / self.polar_radius_km.powi(2),
        )
        .normalize()
    }

    /// Computes the emission angle (epsilon) at a surface point.
    ///
    /// This is the angle between the surface normal and the vector from the surface point
    /// to the observer (spacecraft).
    ///
    /// * 0.0 degrees means the observer is looking straight down (Nadir).
    /// * 90.0 degrees means the observer is looking from the horizon (grazing).
    /// * > 90.0 degrees means the point is not visible (on the back side).
    pub fn emission_angle_deg(&self, surface_point: Vector3, observer_pos_body: Vector3) -> f64 {
        let normal = self.surface_normal(surface_point);
        let vec_to_observer = (observer_pos_body - surface_point).normalize();

        // Clamp dot product to [-1.0, 1.0] to avoid NaN from acos due to float errors
        normal
            .dot(&vec_to_observer)
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees()
    }

    /// Computes the solar incidence angle (iota) at a surface point.
    ///
    /// This is the angle between the surface normal and the vector from the surface point
    /// to the Sun.
    ///
    /// * 0.0 degrees means the Sun is directly overhead (Noon).
    /// * 90.0 degrees means the Sun is at the horizon (Terminator).
    /// * > 90.0 degrees means the point is in shadow (Night).
    pub fn solar_incidence_angle_deg(&self, surface_point: Vector3, sun_pos_body: Vector3) -> f64 {
        let normal = self.surface_normal(surface_point);
        let vec_to_sun = (sun_pos_body - surface_point).normalize();

        normal.dot(&vec_to_sun).clamp(-1.0, 1.0).acos().to_degrees()
    }
}

#[cfg_attr(feature = "python", pymethods)]
#[cfg(feature = "python")]
impl Ellipsoid {
    /// Initializes a new [Ellipsoid] shape provided at least its semi major equatorial radius, optionally its semi minor equatorial radius, and optionally its polar radius.
    /// All units are in kilometers. If the semi minor equatorial radius is not provided, a bi-axial spheroid will be created using the semi major equatorial radius as
    /// the equatorial radius and using the provided polar axis radius. If only the semi major equatorial radius is provided, a perfect sphere will be built.
    #[new]
    #[pyo3(signature=(semi_major_equatorial_radius_km, polar_radius_km=None, semi_minor_equatorial_radius_km=None))]
    fn py_new(
        semi_major_equatorial_radius_km: f64,
        polar_radius_km: Option<f64>,
        semi_minor_equatorial_radius_km: Option<f64>,
    ) -> Self {
        match polar_radius_km {
            Some(polar_radius_km) => match semi_minor_equatorial_radius_km {
                Some(semi_minor_equatorial_radius_km) => Self {
                    semi_major_equatorial_radius_km,
                    semi_minor_equatorial_radius_km,
                    polar_radius_km,
                },
                None => Self::from_spheroid(semi_major_equatorial_radius_km, polar_radius_km),
            },
            None => Self::from_sphere(semi_major_equatorial_radius_km),
        }
    }

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self} (@{self:p})")
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> Result<bool, PyErr> {
        match op {
            CompareOp::Eq => Ok(self == other),
            CompareOp::Ne => Ok(self != other),
            _ => Err(PyErr::new::<PyTypeError, _>(format!(
                "{op:?} not available"
            ))),
        }
    }

    /// Allows for pickling the object
    ///
    /// :rtype: typing.Tuple
    fn __getnewargs__(&self) -> Result<(f64, Option<f64>, Option<f64>), PyErr> {
        Ok((
            self.semi_major_equatorial_radius_km,
            Some(self.polar_radius_km),
            Some(self.semi_minor_equatorial_radius_km),
        ))
    }

    /// :rtype: float
    #[getter]
    fn get_semi_major_equatorial_radius_km(&self) -> PyResult<f64> {
        Ok(self.semi_major_equatorial_radius_km)
    }
    /// :type semi_major_equatorial_radius_km: float
    #[setter]
    fn set_semi_major_equatorial_radius_km(
        &mut self,
        semi_major_equatorial_radius_km: f64,
    ) -> PyResult<()> {
        self.semi_major_equatorial_radius_km = semi_major_equatorial_radius_km;
        Ok(())
    }
    /// :rtype: float
    #[getter]
    fn get_polar_radius_km(&self) -> PyResult<f64> {
        Ok(self.polar_radius_km)
    }
    /// :type polar_radius_km: float
    #[setter]
    fn set_polar_radius_km(&mut self, polar_radius_km: f64) -> PyResult<()> {
        self.polar_radius_km = polar_radius_km;
        Ok(())
    }
    /// :rtype: float
    #[getter]
    fn get_semi_minor_equatorial_radius_km(&self) -> PyResult<f64> {
        Ok(self.semi_minor_equatorial_radius_km)
    }
    /// :type semi_minor_equatorial_radius_km: float
    #[setter]
    fn set_semi_minor_equatorial_radius_km(
        &mut self,
        semi_minor_equatorial_radius_km: f64,
    ) -> PyResult<()> {
        self.semi_minor_equatorial_radius_km = semi_minor_equatorial_radius_km;
        Ok(())
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Ellipsoid {
    /// Returns the mean equatorial radius in kilometers
    ///
    /// :rtype: float
    pub fn mean_equatorial_radius_km(&self) -> f64 {
        (self.semi_major_equatorial_radius_km + self.semi_minor_equatorial_radius_km) / 2.0
    }

    /// Returns true if the polar radius is equal to the semi minor radius.
    ///
    /// :rtype: bool
    pub fn is_sphere(&self) -> bool {
        self.is_spheroid()
            && (self.polar_radius_km - self.semi_minor_equatorial_radius_km).abs() < f64::EPSILON
    }

    /// Returns true if the semi major and minor radii are equal
    ///
    /// :rtype: bool
    pub fn is_spheroid(&self) -> bool {
        (self.semi_major_equatorial_radius_km - self.semi_minor_equatorial_radius_km).abs()
            < f64::EPSILON
    }

    /// Returns the flattening ratio, computed from the mean equatorial radius and the polar radius
    ///
    /// :rtype: float
    pub fn flattening(&self) -> f64 {
        (self.mean_equatorial_radius_km() - self.polar_radius_km) / self.mean_equatorial_radius_km()
    }
}

impl fmt::Display for Ellipsoid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if self.is_sphere() {
            write!(f, "radius = {} km", self.semi_major_equatorial_radius_km)
        } else if self.is_spheroid() {
            write!(
                f,
                "eq. radius = {} km, polar radius = {} km, f = {}",
                self.semi_major_equatorial_radius_km,
                self.polar_radius_km,
                self.flattening()
            )
        } else {
            write!(
                f,
                "major radius = {} km, minor radius = {} km, polar radius = {} km, f = {}",
                self.semi_major_equatorial_radius_km,
                self.semi_minor_equatorial_radius_km,
                self.polar_radius_km,
                self.flattening()
            )
        }
    }
}

impl Encode for Ellipsoid {
    fn encoded_len(&self) -> der::Result<der::Length> {
        self.semi_major_equatorial_radius_km.encoded_len()?
            + self.semi_minor_equatorial_radius_km.encoded_len()?
            + self.polar_radius_km.encoded_len()?
    }

    fn encode(&self, encoder: &mut impl Writer) -> der::Result<()> {
        self.semi_major_equatorial_radius_km.encode(encoder)?;
        self.semi_minor_equatorial_radius_km.encode(encoder)?;
        self.polar_radius_km.encode(encoder)
    }
}

impl<'a> Decode<'a> for Ellipsoid {
    fn decode<R: Reader<'a>>(decoder: &mut R) -> der::Result<Self> {
        Ok(Self {
            semi_major_equatorial_radius_km: decoder.decode()?,
            semi_minor_equatorial_radius_km: decoder.decode()?,
            polar_radius_km: decoder.decode()?,
        })
    }
}
