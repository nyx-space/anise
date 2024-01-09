/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use core::f64::EPSILON;
use core::fmt;
use der::{Decode, Encode, Reader, Writer};
use serde_derive::{Deserialize, Serialize};

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
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(get_all, set_all))]
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
}

#[cfg_attr(feature = "python", pymethods)]
impl Ellipsoid {
    /// Initializes a new [Ellipsoid] shape provided at least its semi major equatorial radius, optionally its semi minor equatorial radius, and optionally its polar radius.
    /// All units are in kilometers. If the semi minor equatorial radius is not provided, a bi-axial spheroid will be created using the semi major equatorial radius as
    /// the equatorial radius and using the provided polar axis radius. If only the semi major equatorial radius is provided, a perfect sphere will be built.
    #[cfg(feature = "python")]
    #[new]
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

    #[cfg(feature = "python")]
    fn __str__(&self) -> String {
        format!("{self}")
    }

    #[cfg(feature = "python")]
    fn __repr__(&self) -> String {
        format!("{self} (@{self:p})")
    }

    #[cfg(feature = "python")]
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
    #[cfg(feature = "python")]
    fn __getnewargs__(&self) -> Result<(f64, Option<f64>, Option<f64>), PyErr> {
        Ok((
            self.semi_major_equatorial_radius_km,
            Some(self.polar_radius_km),
            Some(self.semi_minor_equatorial_radius_km),
        ))
    }

    /// Returns the mean equatorial radius in kilometers
    pub fn mean_equatorial_radius_km(&self) -> f64 {
        (self.semi_major_equatorial_radius_km + self.semi_minor_equatorial_radius_km) / 2.0
    }

    pub fn is_sphere(&self) -> bool {
        self.is_spheroid()
            && (self.polar_radius_km - self.semi_minor_equatorial_radius_km).abs() < EPSILON
    }

    pub fn is_spheroid(&self) -> bool {
        (self.semi_major_equatorial_radius_km - self.semi_minor_equatorial_radius_km).abs()
            < EPSILON
    }

    /// Returns the flattening ratio, computed from the mean equatorial radius and the polar radius
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
