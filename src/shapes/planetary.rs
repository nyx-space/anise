/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::structure::{
    array::DataArray,
    planetocentric::{phaseangle::PhaseAngle, trigangle::TrigAngle},
};

use super::ellipsoid::Ellipsoid;

/// Planetary constants can store the same data as the SPICE textual PCK files
#[derive(Clone, Debug, PartialEq)]
pub struct PlanetaryConstants<'a> {
    /// Name is used as the input for the hashing function.
    pub name: &'a str,
    /// Generic comments field
    pub comments: &'a str,
    /// Gravitational parameter (μ) of this planetary object.
    pub mu_km3_s2: f64,
    /// The shape is always a tri axial ellipsoid
    pub shape: Option<Ellipsoid>,
    ///     TODO: Create a PoleOrientation structure which is optional. If defined, it includes the stuff below, and none optional (DataArray can be empty).
    pub pole_right_ascension: Option<PhaseAngle>,
    pub pole_declination: Option<PhaseAngle>,
    pub prime_meridian: Option<PhaseAngle>,
    pub nut_prec_angles: Option<DataArray<'a, TrigAngle>>,
}
