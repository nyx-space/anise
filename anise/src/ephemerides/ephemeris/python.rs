/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{Almanac, Covariance, EphemEntry, Ephemeris, EphemerisError, LocalFrame, Orbit};
use crate::naif::daf::data_types::DataType;
use crate::naif::daf::DafDataType;
use hifitime::Epoch;
use pyo3::prelude::*;
use pyo3::types::PyType;
use std::collections::BTreeMap;

#[pymethods]
impl Ephemeris {
    /// :rtype: str
    #[getter]
    fn get_object_id(&self) -> String {
        self.object_id.clone()
    }

    #[getter]
    fn get_interpolation(&self) -> String {
        match self.interpolation {
            DataType::Type9LagrangeUnequalStep => "LAGRANGE".to_string(),
            DataType::Type13HermiteUnequalStep => "HERMITE".to_string(),
            _ => unreachable!(),
        }
    }

    #[new]
    fn py_new(orbit_list: Vec<Orbit>, object_id: String) -> Self {
        let mut state_data = BTreeMap::new();

        for orbit in orbit_list {
            state_data.insert(orbit.epoch, EphemEntry { orbit, covar: None });
        }

        Self {
            state_data,
            object_id,
            interpolation: DafDataType::Type13HermiteUnequalStep,
            degree: 7,
        }
    }

    /// Initializes a new Ephemeris from a file path to CCSDS OEM file.
    ///
    /// :type path: str
    /// :rtype: Ephemeris
    #[classmethod]
    #[pyo3(name = "from_ccsds_oem_file", signature=(path))]
    fn py_from_ccsds_oem_file(_cls: Bound<'_, PyType>, path: &str) -> Result<Self, EphemerisError> {
        Self::from_ccsds_oem_file(path)
    }

    /// Exports this Ephemeris to CCSDS OEM at the provided path, optionally specifying an originator and/or an object name
    ///
    /// :type path: str
    /// :type originator: str, optional
    /// :type object_name: str, optional
    fn py_to_ccsds_oem_file(
        &self,
        path: &str,
        originator: Option<String>,
        object_name: Option<String>,
    ) -> Result<(), EphemerisError> {
        self.to_ccsds_oem_file(path, originator, object_name)
    }

    /// Returns the time domain of this ephemeris.
    ///
    /// :rtype: tuple
    pub fn py_domain(&self) -> Result<(Epoch, Epoch), EphemerisError> {
        self.domain()
    }

    /// Returns whether all of the data in this ephemeris includes the covariance.
    ///
    /// :rtype: bool
    pub fn py_includes_covariance(&self) -> bool {
        self.includes_covariance()
    }

    /// Inserts a new ephemeris entry to this ephemeris (it is automatically sorted chronologically).
    ///
    /// :type entry: EphemEntry
    pub fn py_insert(&mut self, entry: EphemEntry) {
        self.insert(entry);
    }

    /// Inserts a new orbit (without covariance) to this ephemeris (it is automatically sorted chronologically).
    ///
    /// :type orbit: Orbit
    pub fn py_insert_orbit(&mut self, orbit: Orbit) {
        self.insert_orbit(orbit);
    }

    /// Returns the nearest entry before the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: EphemEntry
    pub fn py_nearest_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<EphemEntry, EphemerisError> {
        self.nearest_before(epoch, almanac)
    }

    /// Returns the nearest entry after the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: EphemEntry
    pub fn py_nearest_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<EphemEntry, EphemerisError> {
        self.nearest_after(epoch, almanac)
    }

    /// Returns the nearest orbit before the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: Orbit
    pub fn py_nearest_orbit_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Orbit, EphemerisError> {
        self.nearest_orbit_before(epoch, almanac)
    }

    /// Returns the nearest orbit after the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: Orbit
    pub fn py_nearest_orbit_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Orbit, EphemerisError> {
        self.nearest_orbit_after(epoch, almanac)
    }

    /// Returns the nearest covariance before the provided epoch as a tuple (Epoch, Covariance)
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: tuple
    pub fn py_nearest_covar_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Option<(Epoch, Covariance)>, EphemerisError> {
        self.nearest_covar_before(epoch, almanac)
    }

    /// Returns the nearest covariance after the provided epoch as a tuple (Epoch, Covariance)
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: tuple
    pub fn py_nearest_covar_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Option<(Epoch, Covariance)>, EphemerisError> {
        self.nearest_covar_after(epoch, almanac)
    }

    /// Interpolates the ephemeris state and covariance at the provided epoch.
    ///
    /// # Orbit Interpolation
    /// The orbital state is interpolated using high-fidelity numeric methods consistent
    /// with SPICE standards:
    /// * **Type 9 (Lagrange):** Uses an Nth-order Lagrange polynomial interpolation on
    ///   unequal time steps. It interpolates each of the 6 state components (position
    ///   and velocity) independently.
    /// * **Type 13 (Hermite):** Uses an Nth-order Hermite interpolation. This method
    ///   explicitly uses the velocity data (derivatives) to constrain the interpolation
    ///   of the position, ensuring that the resulting position curve is smooth and
    ///   dynamically consistent with the velocity.
    ///
    /// # Covariance Interpolation (Log-Euclidean)
    /// If covariance data is available, this method performs **Log-Euclidean Riemannian
    /// Interpolation**. Unlike standard linear element-wise interpolation, this approach
    /// respects the geometric manifold of Symmetric Positive Definite (SPD) matrices.
    ///
    /// This guarantees that:
    /// 1. **Positive Definiteness:** The interpolated covariance matrix is always mathematically
    ///    valid (all eigenvalues are strictly positive), preventing numerical crashes in downstream filters.
    /// 2. **Volume Preservation:** It prevents the artificial "swelling" (determinant increase)
    ///    of uncertainty that occurs when linearly interpolating between two valid matrices.
    ///    The interpolation follows the "geodesic" (shortest path) on the curved surface of
    ///    covariance matrices.
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: EphemEntry
    pub fn py_at(&self, epoch: Epoch, almanac: &Almanac) -> Result<EphemEntry, EphemerisError> {
        self.at(epoch, almanac)
    }

    /// Interpolate the ephemeris at the provided epoch, returning only the orbit.
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: Orbit
    pub fn py_orbit_at(&self, epoch: Epoch, almanac: &Almanac) -> Result<Orbit, EphemerisError> {
        self.orbit_at(epoch, almanac)
    }

    /// Interpolate the ephemeris at the provided epoch, returning only the covariance.
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: Covariance
    pub fn py_covar_at(
        &self,
        epoch: Epoch,
        local_frame: LocalFrame,
        almanac: &Almanac,
    ) -> Result<Option<Covariance>, EphemerisError> {
        self.covar_at(epoch, local_frame, almanac)
    }
}
