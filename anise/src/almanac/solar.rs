/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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
    /// This computes the Sun Probe Earth angle (SPE) if the probe is in a loaded SPK, its ID is the "observer_id", and the target is set to its central body.
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
    ///
    /// :type target_id: int
    /// :type observer_id: int
    /// :type epoch: Epoch
    /// :rtype: float
    pub fn sun_angle_deg(
        &self,
        target_id: NaifId,
        observer_id: NaifId,
        epoch: Epoch,
    ) -> Result<f64, EphemerisError> {
        let obs_to_sun =
            self.translate_geometric(SUN_J2000, Frame::from_ephem_j2000(observer_id), epoch)?;
        let obs_to_target = self.translate_geometric(
            Frame::from_ephem_j2000(target_id),
            Frame::from_ephem_j2000(observer_id),
            epoch,
        )?;

        Ok(obs_to_sun
            .r_hat()
            .dot(&obs_to_target.r_hat())
            .acos()
            .to_degrees())
    }

    /// Convenience function that calls `sun_angle_deg` with the provided frames instead of the ephemeris ID.
    ///
    /// :type target: Frame
    /// :type observer: Frame
    /// :type epoch: Epoch
    /// :rtype: float
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
mod ut_solar {
    use crate::{
        constants::{
            celestial_objects::EARTH,
            frames::{EARTH_J2000, IAU_EARTH_FRAME, SUN_J2000},
            usual_planetary_constants::MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
        },
        prelude::*,
    };

    /// Load a BSP of a spacecraft trajectory, compute the sun elevation at different points on the surface below that spacecraft
    /// and ensure that it matches with the SPE calculation.
    #[test]
    fn verify_geometry() {
        let ctx = Almanac::default()
            .load("../data/de440s.bsp")
            .and_then(|ctx| ctx.load("../data/gmat-hermite.bsp"))
            .and_then(|ctx| ctx.load("../data/pck11.pca"))
            .unwrap();

        let epoch = Epoch::from_gregorian_hms(2000, 1, 1, 12, 0, 0, TimeScale::UTC);

        let sc_id = -10000001;

        let my_sc_j2k = Frame::from_ephem_j2000(sc_id);

        // Grab the state in the J2000 frame
        let state = ctx.transform(my_sc_j2k, EARTH_J2000, epoch, None).unwrap();

        // We'll check at four different points in the orbit
        for epoch in TimeSeries::inclusive(
            epoch,
            epoch + state.period().unwrap(),
            0.05 * state.period().unwrap(),
        ) {
            let spe_deg = ctx.sun_angle_deg(EARTH, sc_id, epoch).unwrap();
            assert_eq!(
                spe_deg,
                ctx.sun_angle_deg_from_frame(EARTH_J2000, my_sc_j2k, epoch)
                    .unwrap()
            );

            let iau_earth = ctx.frame_from_uid(IAU_EARTH_FRAME).unwrap();

            // Compute this state in the body fixed frame.
            let state_bf = ctx
                .transform(my_sc_j2k, IAU_EARTH_FRAME, epoch, None)
                .unwrap();
            // Build a local point on the surface below this spacecraft
            let nadir_surface_point = Orbit::try_latlongalt(
                state_bf.latitude_deg().unwrap(),
                state_bf.longitude_deg(),
                0.0,
                MEAN_EARTH_ANGULAR_VELOCITY_DEG_S,
                epoch,
                iau_earth,
            )
            .unwrap();
            // Fetch the state of the sun at this time.
            let sun_state = ctx
                .transform(SUN_J2000, IAU_EARTH_FRAME, epoch, None)
                .unwrap();

            // Compute the Sun elevation from that point.
            let sun_elevation_deg = ctx
                .azimuth_elevation_range_sez(sun_state, nadir_surface_point, None, None)
                .unwrap()
                .elevation_deg;

            println!(
                "{epoch}\tsun el = {sun_elevation_deg:.3} deg\tlat = {:.3} deg\tlong = {:.3} deg\t-->\tSPE = {spe_deg:.3} deg",
                nadir_surface_point.latitude_deg().unwrap(),
                nadir_surface_point.longitude_deg()
            );

            // Test: the sun elevation from the ground should be about 90 degrees _less_ than the SPE
            // The "about" is because the SPE effectively assumes a spherical celestial object, but the frame
            // from the Almanac accounts for the flattening when computing the exact location below the vehicle.
            // Hence, there is a small difference (we accept up to 0.05 degrees of difference).
            assert!((sun_elevation_deg + 90.0 - spe_deg).abs() < 5e-2)
        }
    }
}
