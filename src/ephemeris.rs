/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{
    asn1::{common::InterpolationKind, ephemeris::Ephemeris, splinekind::SplineKind, time::Epoch},
    errors::{AniseError, InternalErrorKind},
};

impl<'a> Ephemeris<'a> {
    /// Evaluate this ephemeris at the requested epoch and returns the position only.
    pub fn position_at(&self, req_epoch: Epoch) -> Result<[f64; 3], AniseError> {
        todo!()
    }

    /// Evaluate this ephemeris at the requested epoch and returns the velocity only.
    pub fn velocity_at(&self, req_epoch: Epoch) -> Result<[f64; 3], AniseError> {
        todo!()
    }

    /// Evaluate this ephemeris at the requested epoch and returns an orbit structure.
    pub fn orbit_at(&self, req_epoch: Epoch) -> Result<[f64; 6], AniseError> {
        if self.interpolation_kind != InterpolationKind::ChebyshevSeries {
            return Err(InternalErrorKind::InterpolationNotSupported.into());
        }
        match self.splines.kind {
            SplineKind::SlidingWindow { .. } => {
                Err(InternalErrorKind::InterpolationNotSupported.into())
            }
            SplineKind::FixedWindow { window_duration_s } => {
                // Compute the offset compared to the reference epoch of this ephemeris.
                let offset_s = if self.backward {
                    (req_epoch.epoch - self.ref_epoch.epoch).in_seconds()
                } else {
                    (self.ref_epoch.epoch - req_epoch.epoch).in_seconds()
                };

                // The index for a fixed window is simply the rounded division.
                let index = if self.backward {
                    (offset_s / window_duration_s).ceil()
                } else {
                    (offset_s / window_duration_s).floor()
                };

                // Then let the spline compute the state.
                self.splines.orbit_at(
                    index as usize,
                    offset_s,
                    window_duration_s,
                    self.interpolation_kind,
                )
            }
        }
    }
}
