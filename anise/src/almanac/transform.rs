/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::{Epoch, Unit as TimeUnit};
use snafu::ResultExt;

use crate::{
    constants::orientations::J2000,
    errors::{AlmanacResult, EphemerisSnafu, OrientationSnafu},
    math::{cartesian::CartesianState, units::LengthUnit, Vector3},
    orientations::OrientationPhysicsSnafu,
    prelude::{Aberration, Frame},
    NaifId,
};

use super::Almanac;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python", pymethods)]
impl Almanac {
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
    pub fn transform(
        &self,
        target_frame: Frame,
        observer_frame: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<CartesianState> {
        // Translate
        let state = self
            .translate(target_frame, observer_frame, epoch, ab_corr)
            .context(EphemerisSnafu {
                action: "transform from/to",
            })?;
        // Rotate
        let dcm = self
            .rotate(target_frame, observer_frame, epoch)
            .context(OrientationSnafu {
                action: "transform from/to",
            })?;

        (dcm * state)
            .context(OrientationPhysicsSnafu {})
            .context(OrientationSnafu {
                action: "transform from/to",
            })
    }

    /// Translates a state with its origin (`to_frame`) and given its units (distance_unit, time_unit), returns that state with respect to the requested frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_state_to` function instead to include rotations.
    ///
    /// :type state: Orbit
    /// :type observer_frame: Frame
    /// :type ab_corr: Aberration, optional
    /// :rtype: Orbit
    #[allow(clippy::too_many_arguments)]
    pub fn transform_to(
        &self,
        mut state: CartesianState,
        observer_frame: Frame,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<CartesianState> {
        // If the input and final rotations differ, rotate into J2000 first
        state = if state.frame.orient_origin_match(observer_frame) {
            state
        } else {
            self.rotate_to(state, state.frame.with_orient(J2000))
                .context(OrientationSnafu {
                    action: "transform state dcm",
                })?
        };

        // Transform in the base frame (J2000) or the common frame
        state = self
            .translate_to(state, observer_frame, ab_corr)
            .context(EphemerisSnafu {
                action: "transform state",
            })?;

        // Rotate into the observer frame
        self.rotate_to(state, observer_frame)
            .context(OrientationSnafu {
                action: "transform state",
            })
    }

    /// Returns the Cartesian state of the object as seen from the provided observer frame (essentially `spkezr`).
    ///
    /// # Note
    /// The units will be those of the underlying ephemeris data (typically km and km/s)
    ///
    /// :type object: int
    /// :type observer: Frame
    /// :type epoch: Epoch
    /// :type ab_corr: Aberration, optional
    /// :rtype: Orbit
    pub fn state_of(
        &self,
        object: NaifId,
        observer: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<CartesianState> {
        self.transform(Frame::from_ephem_j2000(object), observer, epoch, ab_corr)
    }

    /// Alias fo SPICE's `spkezr` where the inputs must be the NAIF IDs of the objects and frames with the caveat that the aberration is moved to the last positional argument.
    ///
    /// :type target: int
    /// :type epoch: Epoch
    /// :type frame: int
    /// :type observer: int
    /// :type ab_corr: Aberration, optional
    /// :rtype: Orbit
    pub fn spk_ezr(
        &self,
        target: NaifId,
        epoch: Epoch,
        frame: NaifId,
        observer: NaifId,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<CartesianState> {
        let tgt_j2000 = Frame::from_ephem_j2000(target);
        let obs_j2000 = Frame::from_ephem_j2000(observer);

        // Translate in J2000
        let state = self
            .translate(tgt_j2000, obs_j2000, epoch, ab_corr)
            .context(EphemerisSnafu {
                action: "transform from/to",
            })?;

        // Rotate into the desired frame
        self.rotate_to(state, Frame::new(observer, frame))
            .context(OrientationSnafu {
                action: "spkerz from/to",
            })
    }
}

impl Almanac {
    /// Translates a state with its origin (`to_frame`) and given its units (distance_unit, time_unit), returns that state with respect to the requested frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_state_to` function instead to include rotations.
    #[allow(clippy::too_many_arguments)]
    pub fn transform_state_to(
        &self,
        position: Vector3,
        velocity: Vector3,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
        distance_unit: LengthUnit,
        time_unit: TimeUnit,
    ) -> AlmanacResult<CartesianState> {
        let state = self
            .translate_state_to(
                position,
                velocity,
                from_frame,
                to_frame,
                epoch,
                ab_corr,
                distance_unit,
                time_unit,
            )
            .context(EphemerisSnafu {
                action: "transform provided state",
            })?;

        // Compute the frame rotation
        let dcm = self
            .rotate(from_frame, to_frame, epoch)
            .context(OrientationSnafu {
                action: "transform provided state dcm",
            })?;

        (dcm * state)
            .context(OrientationPhysicsSnafu {})
            .context(OrientationSnafu {
                action: "transform provided state",
            })
    }
}
