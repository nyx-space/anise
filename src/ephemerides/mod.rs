/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use hifitime::Epoch;

use crate::structure::{ephemeris::Ephemeris, spline::Evenness};

pub mod paths;
pub mod translate_to_parent;
pub mod translations;

impl<'a> Ephemeris<'a> {
    /// Returns the first epoch in the data, which will be the chronological "end" epoch if the ephemeris is generated backward
    fn first_epoch(&self) -> Epoch {
        self.ref_epoch
    }

    /// Returns the last epoch in the data, which will be the chronological "start" epoch if the ephemeris is generated backward
    fn last_epoch(&self) -> Epoch {
        match self.splines.metadata.evenness {
            Evenness::Even { duration_ns } => {
                // Grab the number of splines
                // self.ref_epoch + ((self.splines.len() as f64) * (duration_ns as i64).nanoseconds())
                todo!()
            }
            Evenness::Uneven { indexes: _ } => {
                todo!()
            }
        }
    }

    /// Returns the starting epoch of this ephemeris. It is guaranteed that start_epoch <= end_epoch.
    ///
    /// # Note
    /// + If the ephemeris is stored in chronological order, then the start epoch is the same as the first epoch.
    /// + If the ephemeris is stored in anti-chronological order, then the start epoch is the last epoch.
    pub fn start_epoch(&self) -> Epoch {
        if self.first_epoch() > self.last_epoch() {
            self.last_epoch()
        } else {
            self.first_epoch()
        }
    }

    pub fn end_epoch(&self) -> Epoch {
        if self.first_epoch() > self.last_epoch() {
            self.first_epoch()
        } else {
            self.last_epoch()
        }
    }
}
