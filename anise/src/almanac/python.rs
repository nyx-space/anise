/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{
    planetary::{PlanetaryDataError, PlanetaryDataSetSnafu},
    Almanac,
};
use crate::{
    astro::{Aberration, AzElRange, Occultation},
    ephemerides::EphemerisError,
    errors::AlmanacResult,
    math::{cartesian::CartesianState, rotation::DCM},
    orientations::OrientationError,
    prelude::{Frame, Orbit},
    NaifId,
};
use hifitime::{Epoch, TimeScale, TimeSeries};
use pyo3::prelude::*;
use rayon::prelude::*;
use snafu::prelude::*;

#[pymethods]
impl Almanac {
    /// Returns the frame information (gravitational param, shape) as defined in this Almanac from an empty frame
    /// :type uid: Frame
    /// :rtype: Frame
    fn frame_info(&self, uid: Frame) -> Result<Frame, PlanetaryDataError> {
        Ok(self
            .planetary_data
            .get_by_id(uid.ephemeris_id)
            .context(PlanetaryDataSetSnafu {
                action: "fetching frame by its UID via ephemeris_id",
            })?
            .to_frame(uid.into()))
    }

    /// Initializes a new Almanac from the provided file path, guessing at the file type
    #[new]
    fn py_new(path: &str) -> AlmanacResult<Self> {
        Self::new(path)
    }

    fn __str__(&self) -> String {
        format!("{self}")
    }

    fn __repr__(&self) -> String {
        format!("{self} (@{self:p})")
    }

    /// Pretty prints the description of this Almanac, showing everything by default. Default time scale is TDB.
    /// If any parameter is set to true, then nothing other than that will be printed.
    ///
    /// :type spk: bool, optional
    /// :type bpc: bool, optional
    /// :type planetary: bool, optional
    /// :type eulerparams: bool, optional
    /// :type time_scale: TimeScale, optional
    /// :type round_time: bool, optional
    /// :rtype: None
    #[pyo3(name = "describe", signature=(
        spk=None,
        bpc=None,
        planetary=None,
        eulerparams=None,
        time_scale=None,
        round_time=None,
    ))]
    fn py_describe(
        &self,
        spk: Option<bool>,
        bpc: Option<bool>,
        planetary: Option<bool>,
        eulerparams: Option<bool>,
        time_scale: Option<TimeScale>,
        round_time: Option<bool>,
    ) {
        self.describe(spk, bpc, planetary, eulerparams, time_scale, round_time)
    }

    /// Generic function that tries to load the provided path guessing to the file type.
    ///
    /// :type path: str
    /// :rtype: Almanac
    #[pyo3(name = "load")]
    fn py_load(&self, path: &str) -> AlmanacResult<Self> {
        self.load(path)
    }

    /// Computes the azimuth (in degrees), elevation (in degrees), and range (in kilometers) of the
    /// receiver state (`rx`) seen from the transmitter state (`tx`), once converted into the SEZ frame of the transmitter.
    ///
    /// # Warning
    /// The obstructing body _should_ be a tri-axial ellipsoid body, e.g. IAU_MOON_FRAME.
    ///
    /// # Algorithm
    /// 1. If any obstructing_bodies are provided, ensure that none of these are obstructing the line of sight between the receiver and transmitter.
    /// 2. Compute the SEZ (South East Zenith) frame of the transmitter.
    /// 3. Rotate the receiver position vector into the transmitter SEZ frame.
    /// 4. Rotate the transmitter position vector into that same SEZ frame.
    /// 5. Compute the range as the norm of the difference between these two position vectors.
    /// 6. Compute the elevation, and ensure it is between +/- 180 degrees.
    /// 7. Compute the azimuth with a quadrant check, and ensure it is between 0 and 360 degrees.
    ///
    /// :type rx: Orbit
    /// :type tx: Orbit
    /// :type obstructing_body: Frame, optional
    /// :type ab_corr: Aberration, optional
    /// :rtype: AzElRange
    #[pyo3(name = "azimuth_elevation_range_sez", signature=(rx, tx, obstructing_body=None, ab_corr=None))]
    pub fn py_azimuth_elevation_range_sez(
        &self,
        rx: Orbit,
        tx: Orbit,
        obstructing_body: Option<Frame>,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<AzElRange> {
        self.azimuth_elevation_range_sez(rx, tx, obstructing_body, ab_corr)
    }

    /// Computes the azimuth (in degrees), elevation (in degrees), and range (in kilometers) of the
    /// receiver states (first item in tuple) seen from the transmitter state (second item in states tuple), once converted into the SEZ frame of the transmitter.
    ///
    /// Note: if any computation fails, the error will be printed to the stderr.
    /// Note: the output AER will be chronologically sorted, regardless of transmitter.
    ///
    /// Refer to [azimuth_elevation_range_sez] for details.
    ///
    /// :type rx_tx_states: List[Orbit]
    /// :type obstructing_body: Frame, optional
    /// :type ab_corr: Aberration, optional
    /// :rtype: List[AzElRange]
    #[pyo3(name = "azimuth_elevation_range_sez_many", signature=(
        rx_tx_states,
        obstructing_body=None, ab_corr=None
    ))]
    fn py_azimuth_elevation_range_sez_many(
        &self,
        py: Python,
        rx_tx_states: Vec<(CartesianState, CartesianState)>,
        obstructing_body: Option<Frame>,
        ab_corr: Option<Aberration>,
    ) -> Vec<AzElRange> {
        py.allow_threads(|| {
            let mut rslt = rx_tx_states
                .par_iter()
                .filter_map(|(rx, tx)| {
                    self.azimuth_elevation_range_sez(*rx, *tx, obstructing_body, ab_corr)
                        .map_or_else(
                            |e| {
                                println!("{e}");
                                None
                            },
                            |aer| Some(aer),
                        )
                })
                .collect::<Vec<AzElRange>>();
            rslt.sort_by(|aer_a, aer_b| aer_a.epoch.cmp(&aer_b.epoch));
            rslt
        })
    }

    /// Computes whether the line of sight between an observer and an observed Cartesian state is obstructed by the obstructing body.
    /// Returns true if the obstructing body is in the way, false otherwise.
    ///
    /// For example, if the Moon is in between a Lunar orbiter (observed) and a ground station (observer), then this function returns `true`
    /// because the Moon (obstructing body) is indeed obstructing the line of sight.
    ///
    /// ```text
    /// Observed
    ///   o  -
    ///    +    -
    ///     +      -
    ///      + ***   -
    ///     * +    *   -
    ///     *  + + * + + o
    ///     *     *     Observer
    ///       ****
    ///```
    ///
    /// Key Elements:
    /// - `o` represents the positions of the observer and observed objects.
    /// - The dashed line connecting the observer and observed is the line of sight.
    ///
    /// Algorithm (source: Algorithm 35 of Vallado, 4th edition, page 308.):
    /// - `r1` and `r2` are the transformed radii of the observed and observer objects, respectively.
    /// - `r1sq` and `r2sq` are the squared magnitudes of these vectors.
    /// - `r1dotr2` is the dot product of `r1` and `r2`.
    /// - `tau` is a parameter that determines the intersection point along the line of sight.
    /// - The condition `(1.0 - tau) * r1sq + r1dotr2 * tau <= ob_mean_eq_radius_km^2` checks if the line of sight is within the obstructing body's radius, indicating an obstruction.
    ///
    /// :type observer: Orbit
    /// :type observed: Orbit
    /// :type obstructing_body: Frame
    /// :type ab_corr: Aberration, optional
    /// :rtype: bool
    #[pyo3(name = "line_of_sight_obstructed", signature=(
        observer,
        observed,
        obstructing_body,
        ab_corr=None,
    ))]
    fn py_line_of_sight_obstructed(
        &self,
        observer: Orbit,
        observed: Orbit,
        obstructing_body: Frame,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<bool> {
        self.line_of_sight_obstructed(observer, observed, obstructing_body, ab_corr)
    }

    /// Computes the occultation percentage of the `back_frame` object by the `front_frame` object as seen from the observer, when according for the provided aberration correction.
    ///
    /// A zero percent occultation means that the back object is fully visible from the observer.
    /// A 100%  percent occultation means that the back object is fully hidden from the observer because of the front frame (i.e. _umbra_ if the back object is the Sun).
    /// A value in between means that the back object is partially hidden from the observser (i.e. _penumbra_ if the back object is the Sun).
    /// Refer to the [MathSpec](https://nyxspace.com/nyxspace/MathSpec/celestial/eclipse/) for modeling details.
    ///
    /// :type back_frame: Frame
    /// :type front_frame: Frame
    /// :type observer: Orbit
    /// :type ab_corr: Aberration, optional
    /// :rtype: Occultation
    #[pyo3(name = "occultation", signature=(
        back_frame,
        front_frame,
        observer,
        ab_corr=None,
    ))]
    fn py_occultation(
        &self,
        back_frame: Frame,
        front_frame: Frame,
        observer: Orbit,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<Occultation> {
        self.occultation(back_frame, front_frame, observer, ab_corr)
    }

    /// Computes the solar eclipsing of the observer due to the eclipsing_frame.
    ///
    /// This function calls `occultation` where the back object is the Sun in the J2000 frame, and the front object
    /// is the provided eclipsing frame.
    ///
    /// :type eclipsing_frame: Frame
    /// :type observer: Orbit
    /// :type ab_corr: Aberration, optional
    /// :rtype: Occultation
    #[pyo3(name = "solar_eclipsing", signature=(
        eclipsing_frame,
        observer,
        ab_corr=None,
    ))]
    fn py_solar_eclipsing(
        &self,
        eclipsing_frame: Frame,
        observer: Orbit,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<Occultation> {
        self.solar_eclipsing(eclipsing_frame, observer, ab_corr)
    }

    /// Computes the solar eclipsing of all the observers due to the eclipsing_frame, computed in parallel under the hood.
    ///
    /// Note: if any computation fails, the error will be printed to the stderr.
    /// Note: the output AER will be chronologically sorted, regardless of transmitter.
    ///
    /// Refer to [solar_eclipsing] for details.
    ///
    /// :type eclipsing_frame: Frame
    /// :type observers: List[Orbit]
    /// :type ab_corr: Aberration, optional
    /// :rtype: List[Occultation]
    #[pyo3(name = "solar_eclipsing_many", signature=(
        eclipsing_frame,
        observers,
        ab_corr=None,
    ))]
    fn py_solar_eclipsing_many(
        &self,
        py: Python,
        eclipsing_frame: Frame,
        observers: Vec<Orbit>,
        ab_corr: Option<Aberration>,
    ) -> Vec<Occultation> {
        py.allow_threads(|| {
            let mut rslt = observers
                .par_iter()
                .filter_map(|observer| {
                    self.solar_eclipsing(eclipsing_frame, *observer, ab_corr)
                        .map_or_else(
                            |e| {
                                println!("{e}");
                                None
                            },
                            |aer| Some(aer),
                        )
                })
                .collect::<Vec<Occultation>>();
            rslt.sort_by(|aer_a, aer_b| aer_a.epoch.cmp(&aer_b.epoch));
            rslt
        })
    }

    /// Computes the Beta angle (β) for a given orbital state, in degrees. A Beta angle of 0° indicates that the orbit plane is edge-on to the Sun, leading to maximum eclipse time. Conversely, a Beta angle of +90° or -90° means the orbit plane is face-on to the Sun, resulting in continuous sunlight exposure and no eclipses.
    ///
    /// The Beta angle (β) is defined as the angle between the orbit plane of a spacecraft and the vector from the central body (e.g., Earth) to the Sun. In simpler terms, it measures how much of the time a satellite in orbit is exposed to direct sunlight.
    /// The mathematical formula for the Beta angle is: β=arcsin(h⋅usun​)
    /// Where:
    /// - h is the unit vector of the orbital momentum.
    /// - usun​ is the unit vector pointing from the central body to the Sun.
    ///
    /// Original code from GMAT, <https://github.com/ChristopherRabotin/GMAT/blob/GMAT-R2022a/src/gmatutil/util/CalculationUtilities.cpp#L209-L219>
    ///
    /// :type state: Orbit
    /// :type ab_corr: Aberration, optional
    /// :rtype: float
    #[pyo3(name = "beta_angle_deg", signature=(
        state,
        ab_corr=None,
    ))]
    fn py_beta_angle_deg(&self, state: Orbit, ab_corr: Option<Aberration>) -> AlmanacResult<f64> {
        self.beta_angle_deg(state, ab_corr)
    }

    /// Returns the Cartesian state needed to transform the `from_frame` to the `to_frame`.
    ///
    /// # SPICE Compatibility
    /// This function is the SPICE equivalent of spkezr: `spkezr(TARGET_ID, EPOCH_TDB_S, ORIENTATION_ID, ABERRATION, OBSERVER_ID)`
    /// In ANISE, the TARGET_ID and ORIENTATION are provided in the first argument (TARGET_FRAME), as that frame includes BOTH
    /// the target ID and the orientation of that target. The EPOCH_TDB_S is the epoch in the TDB time system, which is computed
    /// in ANISE using Hifitime. THe ABERRATION is computed by providing the optional Aberration flag. Finally, the OBSERVER
    /// argument is replaced by OBSERVER_FRAME: if the OBSERVER_FRAME argument has the same orientation as the TARGET_FRAME, then this call
    /// will return exactly the same data as the spkerz SPICE call.
    ///
    /// # Note
    /// The units will be those of the underlying ephemeris data (typically km and km/s)
    ///
    /// :type target_frame: Orbit
    /// :type observer_frame: Frame
    /// :type epoch: Epoch
    /// :type ab_corr: Aberration, optional
    /// :rtype: Orbit
    #[pyo3(name = "transform", signature=(
        target_frame,
        observer_frame,
        epoch,
        ab_corr=None,
    ))]
    fn py_transform(
        &self,
        target_frame: Frame,
        observer_frame: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<CartesianState> {
        self.transform(target_frame, observer_frame, epoch, ab_corr)
    }

    /// Returns a chronologically sorted list of the Cartesian states that transform the `from_frame` to the `to_frame` for each epoch of the time series, computed in parallel under the hood.
    /// Note: if any transformation fails, the error will be printed to the stderr.
    ///
    /// Refer to [transform] for details.
    ///
    /// :type target_frame: Orbit
    /// :type observer_frame: Frame
    /// :type time_series: TimeSeries
    /// :type ab_corr: Aberration, optional
    /// :rtype: List[Orbit]
    #[pyo3(name = "transform_many", signature=(
        target_frame,
        observer_frame,
        time_series,
        ab_corr=None,
    ))]
    fn py_transform_many<'py>(
        &self,
        py: Python,
        target_frame: Frame,
        observer_frame: Frame,
        time_series: TimeSeries,
        ab_corr: Option<Aberration>,
    ) -> Vec<CartesianState> {
        py.allow_threads(|| {
            let mut states = time_series
                .par_bridge()
                .filter_map(|epoch| {
                    self.transform(target_frame, observer_frame, epoch, ab_corr)
                        .map_or_else(
                            |e| {
                                eprintln!("{e}");
                                None
                            },
                            |state| Some(state),
                        )
                })
                .collect::<Vec<CartesianState>>();
            states.sort_by(|state_a, state_b| state_a.epoch.cmp(&state_b.epoch));
            states
        })
    }

    /// Returns the provided state as seen from the observer frame, given the aberration.
    ///
    /// :type state: Orbit
    /// :type observer_frame: Frame
    /// :type ab_corr: Aberration, optional
    /// :rtype: Orbit
    #[pyo3(name = "transform_to", signature=(
        state,
        observer_frame,
        ab_corr=None,
    ))]
    fn py_transform_to(
        &self,
        state: CartesianState,
        observer_frame: Frame,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<CartesianState> {
        self.transform_to(state, observer_frame, ab_corr)
    }

    /// Returns a chronologically sorted list of the provided states as seen from the observer frame, given the aberration.
    /// Note: if any transformation fails, the error will be printed to the stderr.
    /// Note: the input ordering is lost: the output states will not be in the same order as the input states if these are not chronologically sorted!
    ///
    /// Refer to [transform_to] for details.
    ///
    /// :type states: List[Orbit]
    /// :type observer_frame: Frame
    /// :type ab_corr: Aberration, optional
    /// :rtype: List[Orbit]
    #[pyo3(name = "transform_many_to", signature=(
        states,
        observer_frame,
        ab_corr=None,
    ))]
    fn py_transform_many_to(
        &self,
        py: Python,
        states: Vec<CartesianState>,
        observer_frame: Frame,
        ab_corr: Option<Aberration>,
    ) -> Vec<CartesianState> {
        py.allow_threads(|| {
            let mut rslt = states
                .par_iter()
                .filter_map(|state| {
                    self.transform_to(*state, observer_frame, ab_corr)
                        .map_or_else(
                            |e| {
                                println!("{e}");
                                None
                            },
                            |state| Some(state),
                        )
                })
                .collect::<Vec<CartesianState>>();
            rslt.sort_by(|state_a, state_b| state_a.epoch.cmp(&state_b.epoch));
            rslt
        })
    }

    /// Returns the Cartesian state of the object as seen from the provided observer frame (essentially `spkezr`).
    ///
    /// # Note
    /// The units will be those of the underlying ephemeris data (typically km and km/s)
    ///
    /// :type object_id: int
    /// :type observer: Frame
    /// :type epoch: Epoch
    /// :type ab_corr: Aberration, optional
    /// :rtype: Orbit
    #[pyo3(name = "state_of", signature=(
        object_id,
        observer,
        epoch,
        ab_corr=None,
    ))]
    fn py_state_of(
        &self,
        object_id: NaifId,
        observer: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<CartesianState> {
        self.state_of(object_id, observer, epoch, ab_corr)
    }

    /// Alias fo SPICE's `spkezr` where the inputs must be the NAIF IDs of the objects and frames with the caveat that the aberration is moved to the last positional argument.
    ///
    /// :type target: int
    /// :type epoch: Epoch
    /// :type frame: int
    /// :type observer: int
    /// :type ab_corr: Aberration, optional
    /// :rtype: Orbit
    #[pyo3(name = "spk_ezr", signature=(
        target,
        epoch,
        frame,
        observer,
        ab_corr=None,
    ))]
    fn py_spk_ezr(
        &self,
        target: NaifId,
        epoch: Epoch,
        frame: NaifId,
        observer: NaifId,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<CartesianState> {
        self.spk_ezr(target, epoch, frame, observer, ab_corr)
    }

    /// Returns the Cartesian state of the target frame as seen from the observer frame at the provided epoch, and optionally given the aberration correction.
    ///
    /// # SPICE Compatibility
    /// This function is the SPICE equivalent of spkezr: `spkezr(TARGET_ID, EPOCH_TDB_S, ORIENTATION_ID, ABERRATION, OBSERVER_ID)`
    /// In ANISE, the TARGET_ID and ORIENTATION are provided in the first argument (TARGET_FRAME), as that frame includes BOTH
    /// the target ID and the orientation of that target. The EPOCH_TDB_S is the epoch in the TDB time system, which is computed
    /// in ANISE using Hifitime. THe ABERRATION is computed by providing the optional Aberration flag. Finally, the OBSERVER
    /// argument is replaced by OBSERVER_FRAME: if the OBSERVER_FRAME argument has the same orientation as the TARGET_FRAME, then this call
    /// will return exactly the same data as the spkerz SPICE call.
    ///
    /// # Warning
    /// This function only performs the translation and no rotation whatsoever. Use the `transform` function instead to include rotations.
    ///
    /// # Note
    /// This function performs a recursion of no more than twice the [MAX_TREE_DEPTH].
    ///
    /// :type target_frame: Orbit
    /// :type observer_frame: Frame
    /// :type epoch: Epoch
    /// :type ab_corr: Aberration, optional
    /// :rtype: Orbit
    #[pyo3(name = "translate", signature=(
        target_frame,
        observer_frame,
        epoch,
        ab_corr=None,
    ))]
    fn py_translate(
        &self,
        target_frame: Frame,
        observer_frame: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
    ) -> Result<CartesianState, EphemerisError> {
        self.translate(target_frame, observer_frame, epoch, ab_corr)
    }

    /// Returns the geometric position vector, velocity vector, and acceleration vector needed to translate the `from_frame` to the `to_frame`, where the distance is in km, the velocity in km/s, and the acceleration in km/s^2.
    ///
    /// :type target_frame: Orbit
    /// :type observer_frame: Frame
    /// :type epoch: Epoch
    /// :rtype: Orbit
    #[pyo3(name = "translate_geometric", signature=(
        target_frame,
        observer_frame,
        epoch,
    ))]
    fn py_translate_geometric(
        &self,
        target_frame: Frame,
        observer_frame: Frame,
        epoch: Epoch,
    ) -> Result<CartesianState, EphemerisError> {
        self.translate_geometric(target_frame, observer_frame, epoch)
    }

    /// Translates the provided Cartesian state into the requested observer frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_to` function instead to include rotations.
    ///
    /// :type state: Orbit
    /// :type observer_frame: Frame
    /// :type ab_corr: Aberration, optional
    /// :rtype: Orbit
    #[pyo3(name = "translate_to", signature=(
        state,
        observer_frame,
        ab_corr=None,
    ))]
    pub fn py_translate_to(
        &self,
        state: CartesianState,
        observer_frame: Frame,
        ab_corr: Option<Aberration>,
    ) -> Result<CartesianState, EphemerisError> {
        self.translate_to(state, observer_frame, ab_corr)
    }

    /// Returns the 6x6 DCM needed to rotation the `from_frame` to the `to_frame`.
    ///
    /// # Warning
    /// This function only performs the rotation and no translation whatsoever. Use the `transform_from_to` function instead to include rotations.
    ///
    /// # Note
    /// This function performs a recursion of no more than twice the MAX_TREE_DEPTH.
    ///
    /// :type from_frame: Frame
    /// :type to_frame: Frame
    /// :type epoch: Epoch
    /// :rtype: DCM
    #[pyo3(name = "rotate", signature=(
        from_frame,
        to_frame,
        epoch,
    ))]
    pub fn py_rotate(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
    ) -> Result<DCM, OrientationError> {
        self.rotate(from_frame, to_frame, epoch)
    }

    /// Rotates the provided Cartesian state into the requested observer frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_to` function instead to include rotations.
    ///
    /// :type state: CartesianState
    /// :type observer_frame: Frame
    /// :rtype: CartesianState
    #[pyo3(name = "rotate_to", signature=(
        state,
        observer_frame,
    ))]
    #[allow(clippy::too_many_arguments)]
    pub fn py_rotate_to(
        &self,
        state: CartesianState,
        observer_frame: Frame,
    ) -> Result<CartesianState, OrientationError> {
        self.rotate_to(state, observer_frame)
    }
}
