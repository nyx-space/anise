/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::asn1::common::InterpolationKind;
use crate::constants::orientations::J2000;
use crate::errors::InternalErrorKind;
use crate::hifitime::Epoch;
use crate::math::Vector3;
use crate::HashType;
use crate::{
    asn1::{context::AniseContext, ephemeris::Ephemeris},
    errors::{AniseError, IntegrityErrorKind},
    frame::Frame,
};

impl<'a> AniseContext<'a> {
    /// Returns the position vector and velocity vector of the `from_frame` with respect to its parent.
    ///
    /// # Errors
    /// + As of now, some interpolation types are not supported, and if that were to happen, this would return an error.
    ///
    /// **WARNING:** This function only performs the translation and no rotation whatsoever. Use the `transform_to_parent_from` function instead to include rotations.
    pub fn translate_to_parent_from(
        &self,
        source: Frame,
        epoch: Epoch,
    ) -> Result<(Vector3, Vector3), AniseError> {
        // First, let's get a reference to the ephemeris given the frame.

        // Grab the index of the data from the frame's ephemeris hash.
        let idx = self.ephemeris_lut.index_for_hash(&source.ephemeris_hash)?;

        // And the pointer to the data
        let ephem = self.try_ephemeris_data(idx.into())?;

        // Grab the pointer to the splines.
        let splines = &ephem.splines;
        match ephem.interpolation_kind {
            InterpolationKind::ChebyshevSeries => todo!(),
            InterpolationKind::HermiteSeries => todo!(),
            InterpolationKind::LagrangeSeries => todo!(),
            InterpolationKind::Polynomial => todo!(),
            InterpolationKind::Trigonometric => todo!(),
        }
        todo!();
    }
}
