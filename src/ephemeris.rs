/*
 * ANISE Toolkit
 * Copyright (C) 2021 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::TimeUnits;

use crate::{
    asn1::{common::InterpolationKind, ephemeris::Ephemeris, splinekind::SplineKind, time::Epoch},
    errors::{AniseError, InternalErrorKind},
};

impl<'a> Ephemeris<'a> {
    /// Returns the first epoch in the data, which will be the chronological "end" epoch if the ephemeris is generated backward
    fn first_epoch(&self) -> Epoch {
        self.ref_epoch
    }

    /// Returns the last epoch in the data, which will be the chronological "start" epoch if the ephemeris is generated backward
    fn last_epoch(&self) -> Epoch {
        match self.splines.kind {
            SplineKind::FixedWindow { window_duration_s } => {
                // Grab the number of splines
                (self.ref_epoch.epoch + ((self.splines.len() as f64) * window_duration_s).seconds())
                    .into()
            }
            SplineKind::SlidingWindow { indexes: _ } => {
                todo!()
            }
        }
    }

    pub fn start_epoch(&self) -> Epoch {
        if self.first_epoch().epoch > self.last_epoch().epoch {
            self.last_epoch()
        } else {
            self.first_epoch()
        }
    }

    pub fn end_epoch(&self) -> Epoch {
        if self.first_epoch().epoch > self.last_epoch().epoch {
            self.first_epoch()
        } else {
            self.last_epoch()
        }
    }

    /// Evaluate this ephemeris at the requested epoch and returns the position only.
    pub fn pos(&self, req_epoch: Epoch) -> Result<[f64; 3], AniseError> {
        let orbit = self.posvel(req_epoch)?;
        Ok([orbit[0], orbit[1], orbit[2]])
    }

    /// Evaluate this ephemeris at the requested epoch and returns the velocity only.
    pub fn vel(&self, req_epoch: Epoch) -> Result<[f64; 3], AniseError> {
        let orbit = self.posvel(req_epoch)?;
        Ok([orbit[3], orbit[4], orbit[5]])
    }

    /// Evaluate this ephemeris at the requested epoch and returns an orbit structure.
    pub fn posvel(&self, req_epoch: Epoch) -> Result<[f64; 6], AniseError> {
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
                self.splines.posvel_at(
                    index as usize,
                    offset_s,
                    window_duration_s,
                    self.interpolation_kind,
                )
            }
        }
    }
}
