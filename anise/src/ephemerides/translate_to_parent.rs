/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
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
use crate::ephemerides::EphemInterpolationSnafu;
use crate::hifitime::Epoch;
use crate::math::cartesian::CartesianState;
use crate::math::Vector3;
use crate::naif::daf::datatypes::{
    HermiteSetType13, LagrangeSetType9, Type2ChebyshevSet, Type3ChebyshevSet,
};
use crate::naif::daf::{DAFError, DafDataType, NAIFDataSet, NAIFSummaryRecord};
use crate::prelude::Frame;

#[cfg(feature = "python")]
use pyo3::prelude::*;

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

        let (pos_km, vel_km_s) = match summary.data_type()? {
            DafDataType::Type2ChebyshevTriplet => {
                let data =
                    spk_data
                        .nth_data::<Type2ChebyshevSet>(idx_in_spk)
                        .context(SPKSnafu {
                            action: "fetching data for interpolation",
                        })?;
                data.evaluate(epoch, summary)
                    .context(EphemInterpolationSnafu)?
            }
            DafDataType::Type3ChebyshevSextuplet => {
                let data =
                    spk_data
                        .nth_data::<Type3ChebyshevSet>(idx_in_spk)
                        .context(SPKSnafu {
                            action: "fetching data for interpolation",
                        })?;
                data.evaluate(epoch, summary)
                    .context(EphemInterpolationSnafu)?
            }
            DafDataType::Type9LagrangeUnequalStep => {
                let data = spk_data
                    .nth_data::<LagrangeSetType9>(idx_in_spk)
                    .context(SPKSnafu {
                        action: "fetching data for interpolation",
                    })?;
                data.evaluate(epoch, summary)
                    .context(EphemInterpolationSnafu)?
            }
            DafDataType::Type13HermiteUnequalStep => {
                let data = spk_data
                    .nth_data::<HermiteSetType13>(idx_in_spk)
                    .context(SPKSnafu {
                        action: "fetching data for interpolation",
                    })?;
                data.evaluate(epoch, summary)
                    .context(EphemInterpolationSnafu)?
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
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Almanac {
    /// Performs the GEOMETRIC translation to the parent. Use translate_from_to for aberration.
    ///
    /// :type source: Frame
    /// :type epoch: Epoch
    /// :rtype: Orbit
    pub fn translate_to_parent(
        &self,
        source: Frame,
        epoch: Epoch,
    ) -> Result<CartesianState, EphemerisError> {
        let (radius_km, velocity_km_s, frame) = self.translation_parts_to_parent(source, epoch)?;

        Ok(CartesianState {
            radius_km,
            velocity_km_s,
            epoch,
            frame,
        })
    }
}
