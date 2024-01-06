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
    errors::{AlmanacError, EphemerisSnafu, OrientationSnafu},
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
    /// # Note
    /// The units will be those of the underlying ephemeris data (typically km and km/s)
    pub fn transform_from_to(
        &self,
        from_frame: Frame,
        to_frame: Frame,
        epoch: Epoch,
        ab_corr: Option<Aberration>,
    ) -> Result<CartesianState, AlmanacError> {
        // Translate
        let state = self
            .translate(from_frame, to_frame, epoch, ab_corr)
            .with_context(|_| EphemerisSnafu {
                action: "transform from/to",
            })?;
        // Rotate
        let dcm = self
            .rotate_from_to(from_frame, to_frame, epoch)
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
        to_frame: Frame,
        ab_corr: Option<Aberration>,
    ) -> Result<CartesianState, AlmanacError> {
        let state = self
            .translate_to(state, to_frame, ab_corr)
            .with_context(|_| EphemerisSnafu {
                action: "transform state",
            })?;

        // Compute the frame rotation
        let dcm = self
            .rotate_from_to(state.frame, to_frame, state.epoch)
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
    ) -> Result<CartesianState, AlmanacError> {
        self.transform_from_to(Frame::from_ephem_j2000(object), observer, epoch, ab_corr)
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
    ) -> Result<CartesianState, AlmanacError> {
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
