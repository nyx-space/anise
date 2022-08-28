/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use log::trace;

use crate::asn1::common::InterpolationKind;
use crate::asn1::splinecoeffs::Coefficient;
use crate::hifitime::Epoch;
use crate::math::interpolation::chebyshev::cheby_eval;
use crate::math::Vector3;
use crate::{asn1::context::AniseContext, errors::AniseError, frame::Frame};

impl<'a> AniseContext<'a> {
    /// Returns the position vector and velocity vector of the `source` with respect to its parent at the provided epoch.
    ///
    /// # Errors
    /// + As of now, some interpolation types are not supported, and if that were to happen, this would return an error.
    ///
    /// **WARNING:** This function only performs the translation and no rotation whatsoever. Use the `transform_to_parent_from` function instead to include rotations.
    pub fn translate_to_parent(
        &self,
        source: Frame,
        epoch: Epoch,
    ) -> Result<(Vector3, Vector3), AniseError> {
        // First, let's get a reference to the ephemeris given the frame.

        // Grab the index of the data from the frame's ephemeris hash.
        let idx = self.ephemeris_lut.index_for_hash(&source.ephemeris_hash)?;

        // And the pointer to the data
        let ephem = self.try_ephemeris_data(idx.into())?;

        // Perform a translation with position and velocity;
        let mut pos = Vector3::zeros();
        let mut vel = Vector3::zeros();

        // Grab the pointer to the splines.
        let splines = &ephem.splines;
        match ephem.interpolation_kind {
            InterpolationKind::ChebyshevSeries => {
                trace!("start = {}\tfetch = {}", ephem.start_epoch().epoch, epoch);
                let start_epoch_s = ephem.start_epoch().epoch.as_tdb_seconds();
                let eval_epoch_s = epoch.as_tdb_seconds();

                for (cno, coeff) in [Coefficient::X, Coefficient::Y, Coefficient::Z]
                    .iter()
                    .enumerate()
                {
                    let (val, deriv) = cheby_eval(eval_epoch_s, start_epoch_s, splines, *coeff)?;
                    pos[cno] = val;
                    vel[cno] = deriv;
                }

                if splines.config.num_velocity_coeffs > 0 {
                    // Overwrite the velocity by the direct computation since there are specific coefficients for the velocity
                    for (cno, coeff) in [Coefficient::VX, Coefficient::VY, Coefficient::VZ]
                        .iter()
                        .enumerate()
                    {
                        let (val, _) = cheby_eval(eval_epoch_s, start_epoch_s, splines, *coeff)?;
                        vel[cno] = val;
                    }
                }
            }
            InterpolationKind::HermiteSeries => todo!(),
            InterpolationKind::LagrangeSeries => todo!(),
            InterpolationKind::Polynomial => todo!(),
            InterpolationKind::Trigonometric => todo!(),
        }
        Ok((pos, vel))
    }
}
