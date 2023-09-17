/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use log::trace;
use snafu::ResultExt;

use super::{EphemerisError, SPKSnafu};
use crate::almanac::Almanac;
use crate::astro::Aberration;
use crate::ephemerides::EphemInterpolationSnafu;
use crate::hifitime::Epoch;
use crate::math::units::*;
use crate::math::Vector3;
use crate::naif::daf::NAIFDataSet;
use crate::naif::spk::datatypes::{HermiteSetType13, LagrangeSetType9, Type2ChebyshevSet};
use crate::prelude::Frame;

impl<'a> Almanac<'a> {
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
    ) -> Result<(Vector3, Vector3, Frame), EphemerisError> {
        // TODO: Create a CartesianState struct which can be "upgraded" to an Orbit if the frame is of the correct type?
        // I guess this is what the `Orbit` struct in Nyx does.

        // First, let's find the SPK summary for this frame.
        let (summary, spk_no, idx_in_spk) =
            self.spk_summary_at_epoch(source.ephemeris_id, epoch)?;

        let new_frame = source.with_ephem(summary.center_id);

        trace!("query {source} wrt to {new_frame} @ {epoch:E}");

        // This should not fail because we've fetched the spk_no from above with the spk_summary_at_epoch call.
        let spk_data = self.spk_data[spk_no].ok_or(EphemerisError::Unreachable)?;

        // Now let's simply evaluate the data
        let (pos_km, vel_km_s) = match summary.data_type_i {
            // TODO : match against enumeration instead of direct integer match for clarity ?
            2 => {
                // Type 2 Chebyshev
                let data = spk_data
                    .nth_data::<Type2ChebyshevSet>(idx_in_spk)
                    .with_context(|_| SPKSnafu {
                        action: "fetching data for interpolation",
                    })?;
                data.evaluate(epoch, summary)
                    .with_context(|_| EphemInterpolationSnafu)?
            }
            9 => {
                // Type 9: Lagrange Interpolation --- Unequal Time Steps
                let data = spk_data
                    .nth_data::<LagrangeSetType9>(idx_in_spk)
                    .with_context(|_| SPKSnafu {
                        action: "fetching data for interpolation",
                    })?;
                data.evaluate(epoch, summary)
                    .with_context(|_| EphemInterpolationSnafu)?
            }
            13 => {
                // Type 13: Hermite Interpolation --- Unequal Time Steps
                let data = spk_data
                    .nth_data::<HermiteSetType13>(idx_in_spk)
                    .with_context(|_| SPKSnafu {
                        action: "fetching data for interpolation",
                    })?;
                data.evaluate(epoch, summary)
                    .with_context(|_| EphemInterpolationSnafu)?
            }
            _ => todo!("{} is not yet supported", summary.data_type_i),
        };

        // Convert the units based on the storage units.
        let dist_unit_factor = LengthUnit::Kilometer.from_meters() * distance_unit.to_meters();
        let time_unit_factor = TimeUnit::Second.from_seconds() * time_unit.in_seconds();

        Ok((
            pos_km * dist_unit_factor,
            vel_km_s * dist_unit_factor / time_unit_factor,
            new_frame,
        ))
    }
}
