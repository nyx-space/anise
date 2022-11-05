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

use crate::astro::Aberration;
use crate::hifitime::Epoch;
use crate::math::interpolation::chebyshev::cheby_eval;
use crate::math::Vector3;
use crate::structure::common::InterpolationKind;
use crate::structure::spline::Field;
use crate::structure::units::*;
use crate::{errors::AniseError, prelude::Frame, structure::context::AniseContext};

impl<'a> AniseContext<'a> {
    /// Returns the position vector and velocity vector of the `source` with respect to its parent in the ephemeris at the provided epoch,
    /// and in the provided distance and time units.
    ///
    /// # Example
    /// If the ephemeris stores position interpolation coefficients in kilometer but this function is called with millimeters as a distance unit,
    /// the output vectors will be in mm, mm/s, mm/s^2 respectively.
    ///
    /// # Errors
    /// + As of now, some interpolation types are not supported, and if that were to happen, this would return an error.
    ///
    /// **WARNING:** This function only performs the translation and no rotation whatsoever. Use the `transform_to_parent_from` function instead to include rotations.
    pub fn translate_to_parent(
        &self,
        source: Frame,
        epoch: Epoch,
        _ab_corr: Aberration,
        distance_unit: LengthUnit,
        time_unit: TimeUnit,
    ) -> Result<(Vector3, Vector3, Vector3, Frame), AniseError> {
        // TODO: Create a CartesianState struct which can be "upgraded" to an Orbit if the frame is of the correct type?
        // I guess this is what the `Orbit` struct in Nyx does.
        // First, let's get a reference to the ephemeris given the frame.

        // Grab the index of the data from the frame's ephemeris hash.
        let idx = self.ephemeris_lut.index_for_hash(&source.ephemeris_hash)?;

        // And the pointer to the data
        let ephem = self.try_ephemeris_data(idx.into())?;

        let new_frame = source.with_ephem(ephem.parent_ephemeris_hash);

        trace!("query {source} wrt to {new_frame} @ {epoch:E}");

        // Perform a translation with position and velocity;
        let mut pos = Vector3::zeros();
        let mut vel = Vector3::zeros();
        let mut acc = Vector3::zeros();

        // Grab the pointer to the splines.
        let splines = &ephem.splines;
        match ephem.interpolation_kind {
            InterpolationKind::ChebyshevSeries => {
                let start_epoch = ephem.start_epoch();
                let end_epoch = ephem.end_epoch();

                if epoch < start_epoch || epoch > end_epoch {
                    return Err(AniseError::MissingInterpolationData(epoch));
                }

                if !splines.metadata.state_kind.includes_position() {
                    return Err(AniseError::NoInterpolationData);
                }

                // Compute the position and its derivative
                for (cno, field) in [Field::X, Field::Y, Field::Z].iter().enumerate() {
                    let (val, deriv) = cheby_eval(epoch, start_epoch, splines, *field)?;
                    pos[cno] = val;
                    vel[cno] = deriv;
                }

                // If relevant, compute the velocity from the coefficients directly by overwriting the derivative we just computed.
                if splines.metadata.state_kind.includes_velocity() {
                    for (cno, field) in [Field::Vx, Field::Vy, Field::Vz].iter().enumerate() {
                        let (val, deriv) = cheby_eval(epoch, start_epoch, splines, *field)?;
                        vel[cno] = val;
                        acc[cno] = deriv;
                    }

                    // Similarly, if there is acceleration, we should compute that too.
                    if splines.metadata.state_kind.includes_acceleration() {
                        for (cno, field) in [Field::Ax, Field::Ay, Field::Az].iter().enumerate() {
                            let (val, _) = cheby_eval(epoch, start_epoch, splines, *field)?;
                            acc[cno] = val;
                        }
                    }
                }
            }
            InterpolationKind::HermiteSeries => todo!(),
            InterpolationKind::LagrangeSeries => todo!(),
            InterpolationKind::Polynomial => todo!(),
            InterpolationKind::Trigonometric => todo!(),
        }

        // Convert the units based on the storage units.
        let dist_unit_factor = ephem.length_unit.from_meters() * distance_unit.to_meters();
        let time_unit_factor = ephem.time_unit.from_seconds() * time_unit.in_seconds();

        Ok((
            pos * dist_unit_factor,
            vel * dist_unit_factor / time_unit_factor,
            acc * dist_unit_factor / time_unit_factor.powi(2),
            new_frame,
        ))
    }
}
