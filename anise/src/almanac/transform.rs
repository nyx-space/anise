/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::{Epoch, Unit as TimeUnit};
use snafu::ResultExt;

use crate::{
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
            .with_context(|_| EphemerisSnafu {
                action: "transform from/to",
            })?;
        // Rotate
        let dcm = self
            .rotate_from_to(target_frame, observer_frame, epoch)
            .with_context(|_| OrientationSnafu {
                action: "transform from/to",
            })?;

        (dcm * state)
            .with_context(|_| OrientationPhysicsSnafu {})
            .with_context(|_| OrientationSnafu {
                action: "transform from/to",
            })
    }

    /// Translates a state with its origin (`to_frame`) and given its units (distance_unit, time_unit), returns that state with respect to the requested frame
    ///
    /// **WARNING:** This function only performs the translation and no rotation _whatsoever_. Use the `transform_state_to` function instead to include rotations.
    #[allow(clippy::too_many_arguments)]
    pub fn transform_to(
        &self,
        state: CartesianState,
        observer_frame: Frame,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<CartesianState> {
        let state = self
            .translate_to(state, observer_frame, ab_corr)
            .with_context(|_| EphemerisSnafu {
                action: "transform state",
            })?;

        // Compute the frame rotation
        let dcm = self
            .rotate_from_to(state.frame, observer_frame, state.epoch)
            .with_context(|_| OrientationSnafu {
                action: "transform state dcm",
            })?;

        (dcm * state)
            .with_context(|_| OrientationPhysicsSnafu {})
            .with_context(|_| OrientationSnafu {
                action: "transform state",
            })
    }

    /// Returns the Cartesian state of the object as seen from the provided observer frame (essentially `spkezr`).
    ///
    /// # Note
    /// The units will be those of the underlying ephemeris data (typically km and km/s)
    pub fn state_of(
        &self,
        object: NaifId,
        observer: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<CartesianState> {
        self.transform(Frame::from_ephem_j2000(object), observer, epoch, ab_corr)
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
            .with_context(|_| EphemerisSnafu {
                action: "transform provided state",
            })?;

        // Compute the frame rotation
        let dcm = self
            .rotate_from_to(from_frame, to_frame, epoch)
            .with_context(|_| OrientationSnafu {
                action: "transform provided state dcm",
            })?;

        (dcm * state)
            .with_context(|_| OrientationPhysicsSnafu {})
            .with_context(|_| OrientationSnafu {
                action: "transform provided state",
            })
    }
}
