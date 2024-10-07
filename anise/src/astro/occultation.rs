/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use core::cmp::Ordering;
use core::fmt;

use crate::{constants::celestial_objects::SUN, frames::Frame};

use hifitime::Epoch;
#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Stores the result of an occultation computation with the occulation percentage
/// Refer to the [MathSpec](https://nyxspace.com/nyxspace/MathSpec/celestial/eclipse/) for modeling details.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "python", pyo3(module = "anise"))]
#[cfg_attr(feature = "python", pyo3(get_all, set_all))]
pub struct Occultation {
    pub epoch: Epoch,
    pub percentage: f64,
    pub back_frame: Frame,
    pub front_frame: Frame,
}

#[cfg_attr(feature = "python", pymethods)]
impl Occultation {
    /// Returns the percentage as a factor between 0 and 1
    pub fn factor(&self) -> f64 {
        self.percentage / 100.0
    }

    /// Returns true if the back object is the Sun, false otherwise
    pub const fn is_eclipse_computation(&self) -> bool {
        self.back_frame.ephem_origin_id_match(SUN)
    }

    /// Returns true if the occultation percentage is less than or equal 0.001%
    pub fn is_visible(&self) -> bool {
        self.percentage < 1e-3
    }

    /// Returns true if the occultation percentage is greater than or equal 99.999%
    pub fn is_obstructed(&self) -> bool {
        self.percentage > 99.999
    }

    /// Returns true if neither occulted nor visible (i.e. penumbra for solar eclipsing)
    pub fn is_partial(&self) -> bool {
        !self.is_visible() && !self.is_obstructed()
    }
}

impl fmt::Display for Occultation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_eclipse_computation() {
            // This is an eclipse computation
            if self.is_visible() {
                write!(
                    f,
                    "{}: no eclipse due to {:e}",
                    self.epoch, self.front_frame
                )
            } else if self.is_obstructed() {
                write!(f, "{}: umbra due to {:e}", self.epoch, self.front_frame)
            } else {
                write!(
                    f,
                    "{}: penumbra of {:.3}% due to {:e}",
                    self.epoch, self.percentage, self.front_frame
                )
            }
        } else {
            write!(
                f,
                "{}: {:.3}% occultation of {:e} due to {:e}",
                self.epoch, self.percentage, self.front_frame, self.back_frame
            )
        }
    }
}

impl PartialOrd for Occultation {
    /// Provides an ordering of the occultation by percentage, if the back and front objects match
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.front_frame != other.front_frame || self.back_frame != other.back_frame {
            None
        } else {
            self.percentage.partial_cmp(&other.percentage)
        }
    }
}