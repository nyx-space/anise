/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::Unit;
use log::trace;
use snafu::ResultExt;

use super::{EphemerisError, SPKSnafu};
use crate::almanac::Almanac;
use crate::astro::aberration::stellar_aberration;
use crate::astro::Aberration;
use crate::constants::SPEED_OF_LIGHT_KM_S;
use crate::ephemerides::{EphemInterpolationSnafu, EphemerisPhysicsSnafu};
use crate::hifitime::Epoch;
use crate::math::cartesian::CartesianState;
use crate::math::Vector3;
use crate::naif::daf::datatypes::{HermiteSetType13, LagrangeSetType9, Type2ChebyshevSet};
use crate::naif::daf::{DAFError, DafDataType, NAIFDataSet};
use crate::prelude::Frame;

impl Almanac {
    /// Returns the position vector and velocity vector of the `source` with respect to its parent in the ephemeris at the provided epoch,
    /// Units are those used in the SPK, typically distances are in kilometers and velocities in kilometers per second.
    ///
    /// # Errors
    /// + As of now, some interpolation types are not supported, and if that were to happen, this would return an error.
    ///
    /// # Warning
    /// This function only performs the translation and no rotation whatsoever. Use the `transform_to_parent_from` function instead to include rotations.
    pub(crate) fn translation_parts_to_parent(
        &self,
        source: Frame,
        epoch: Epoch,
        ab_corr: Aberration,
    ) -> Result<(Vector3, Vector3, Frame), EphemerisError> {
        // First, let's find the SPK summary for this frame.
        let (summary, spk_no, idx_in_spk) =
            self.spk_summary_at_epoch(source.ephemeris_id, epoch)?;

        let new_frame = source.with_ephem(summary.center_id);

        trace!("translate {source} wrt to {new_frame} @ {epoch:E}");

        // This should not fail because we've fetched the spk_no from above with the spk_summary_at_epoch call.
        let spk_data = self.spk_data[spk_no]
            .as_ref()
            .ok_or(EphemerisError::Unreachable)?;

        // Now let's simply evaluate the data
        if ab_corr == Aberration::NoCorrection {
            let (pos_km, vel_km_s) = match summary.data_type()? {
                DafDataType::Type2ChebyshevTriplet => {
                    let data = spk_data
                        .nth_data::<Type2ChebyshevSet>(idx_in_spk)
                        .with_context(|_| SPKSnafu {
                            action: "fetching data for interpolation",
                        })?;
                    data.evaluate(epoch, summary)
                        .with_context(|_| EphemInterpolationSnafu)?
                }
                DafDataType::Type9LagrangeUnequalStep => {
                    let data = spk_data
                        .nth_data::<LagrangeSetType9>(idx_in_spk)
                        .with_context(|_| SPKSnafu {
                            action: "fetching data for interpolation",
                        })?;
                    data.evaluate(epoch, summary)
                        .with_context(|_| EphemInterpolationSnafu)?
                }
                DafDataType::Type13HermiteUnequalStep => {
                    let data = spk_data
                        .nth_data::<HermiteSetType13>(idx_in_spk)
                        .with_context(|_| SPKSnafu {
                            action: "fetching data for interpolation",
                        })?;
                    data.evaluate(epoch, summary)
                        .with_context(|_| EphemInterpolationSnafu)?
                }
                dtype => {
                    return Err(EphemerisError::SPK {
                        action: "translation to parent",
                        source: DAFError::UnsupportedDatatype {
                            dtype,
                            kind: "SPK computations",
                        },
                    })
                }
            };

            Ok((pos_km, vel_km_s, new_frame))
        } else {
            // This is a rewrite of NAIF SPICE's `spkapo`

            // Find the geometric position of the target body with respect to the solar system barycenter.
            let (tgt_ssb_pos_km, tgt_ssb_vel_km_s, _) =
                self.translation_parts_to_parent(source, epoch, Aberration::NoCorrection)?;

            // Find the geometric position of the observer body with respect to the solar system barycenter.
            let (obs_ssb_pos_km, obs_ssb_vel_km_s, _) =
                self.translation_parts_to_parent(source, epoch, Aberration::NoCorrection)?;

            // Subtract the position of the observer to get the relative position.
            let mut rel_pos_km = tgt_ssb_pos_km - obs_ssb_pos_km;
            // NOTE: We never correct the velocity, so the geometric velocity is what we're seeking.
            let vel_km_s = tgt_ssb_vel_km_s - obs_ssb_vel_km_s;

            // Use this to compute the one-way light time.
            let mut one_way_lt = rel_pos_km.norm() / SPEED_OF_LIGHT_KM_S;

            // To correct for light time, find the position of the target body at the current epoch
            // minus the one-way light time. Note that the observer remains where he is.

            let num_it = if ab_corr.is_converged() { 3 } else { 1 };
            let lt_sign = if ab_corr.is_transmit() { -1.0 } else { 1.0 };

            for _ in 0..num_it {
                let epoch_lt = epoch + lt_sign * one_way_lt * Unit::Second;
                let (tgt_ssb_pos_km, _, _) =
                    self.translation_parts_to_parent(source, epoch_lt, Aberration::NoCorrection)?;

                rel_pos_km = tgt_ssb_pos_km - obs_ssb_pos_km;
                one_way_lt = rel_pos_km.norm() / SPEED_OF_LIGHT_KM_S;
            }

            // If stellar aberration correction is requested, perform it now.
            if ab_corr.has_stellar() {
                // Modifications based on transmission versus reception case is done in the function directly.
                rel_pos_km = stellar_aberration(rel_pos_km, obs_ssb_vel_km_s, ab_corr)
                    .with_context(|_| EphemerisPhysicsSnafu {
                        action: "computing stellar aberration",
                    })?;
            }

            Ok((rel_pos_km, vel_km_s, new_frame))
        }
    }

    pub fn translate_to_parent(
        &self,
        source: Frame,
        epoch: Epoch,
        ab_corr: Aberration,
    ) -> Result<CartesianState, EphemerisError> {
        let (radius_km, velocity_km_s, frame) =
            self.translation_parts_to_parent(source, epoch, ab_corr)?;

        Ok(CartesianState {
            radius_km,
            velocity_km_s,
            epoch,
            frame,
        })
    }
}
