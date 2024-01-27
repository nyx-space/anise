/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{constants::frames::SUN_J2000, ephemerides::EphemerisError, prelude::Frame, NaifId};

use super::Almanac;

use hifitime::Epoch;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python", pymethods)]
impl Almanac {
    /// Returns the angle (between 0 and 180 degrees) between the observer and the Sun, and the observer and the target body ID.
    /// This computes the Sun Probe Earth angle (SPE) if the probe is in a loaded, its ID is the "observer_id", and the target is set to its central body.
    ///
    /// # Geometry
    /// If the SPE is greater than 90 degrees, then the celestial object below the probe is in sunlight.
    ///
    /// ## Sunrise at nadir
    /// ```text
    /// Sun
    ///  |  \      
    ///  |   \
    ///  |    \
    ///  Obs. -- Target
    /// ```
    /// ## Sun high at nadir
    /// ```text
    /// Sun
    ///  \        
    ///   \  __ θ > 90
    ///    \     \
    ///     Obs. ---------- Target
    /// ```
    ///
    /// ## Sunset at nadir
    /// ```text
    ///          Sun
    ///        /  
    ///       /  __ θ < 90
    ///      /    /
    ///  Obs. -- Target
    /// ```
    ///
    /// # Algorithm
    /// 1. Compute the position of the Sun as seen from the observer
    /// 2. Compute the position of the target as seen from the observer
    /// 3. Return the arccosine of the dot product of the norms of these vectors.
    pub fn sun_angle_deg(
        &self,
        target_id: NaifId,
        observer_id: NaifId,
        epoch: Epoch,
    ) -> Result<f64, EphemerisError> {
        let obs_to_sun =
            self.translate_geometric(SUN_J2000, Frame::from_ephem_j2000(observer_id), epoch)?;
        let obs_to_target = self.translate_geometric(
            Frame::from_ephem_j2000(observer_id),
            Frame::from_ephem_j2000(target_id),
            epoch,
        )?;

        Ok(obs_to_sun
            .r_hat()
            .dot(&obs_to_target.r_hat())
            .acos()
            .to_degrees())
    }

    /// Convenience function that calls `sun_angle_deg` with the provided frames instead of the ephemeris ID.
    pub fn sun_angle_deg_from_frame(
        &self,
        target: Frame,
        observer: Frame,
        epoch: Epoch,
    ) -> Result<f64, EphemerisError> {
        self.sun_angle_deg(target.ephemeris_id, observer.ephemeris_id, epoch)
    }
}

#[cfg(test)]
mod ut_solar {}
