/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{almanac::Almanac, prelude::AniseError, NaifId};

use super::{Frame, FrameTrait};
use core::fmt;

/// Defines a Celestial Frame kind, which is a Frame that also defines a standard gravitational parameter
pub trait CelestialFrameTrait: FrameTrait {
    /// Returns the standard gravitational parameter of this frame (consider switching to UOM for this)
    fn mu_km3_s2(&self) -> f64;
}

/// A CelestialFrame is a frame whose equatorial and semi major radii are defined.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CelestialFrame {
    pub frame: Frame,
    pub mu_km3_s2: f64,
}

impl FrameTrait for CelestialFrame {
    fn ephemeris_id(&self) -> NaifId {
        self.frame.ephemeris_id()
    }

    fn orientation_id(&self) -> NaifId {
        self.frame.orientation_id()
    }
}

impl CelestialFrameTrait for CelestialFrame {
    fn mu_km3_s2(&self) -> f64 {
        self.mu_km3_s2
    }
}

impl fmt::Display for CelestialFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{} (μ = {} km3/s)", self.frame, self.mu_km3_s2())
    }
}

impl<'a> Almanac<'a> {
    /// Tries to find the celestial frame data given the ephemeris center name and the orientation name.
    /// # Note
    /// The ephemeris name MUST match the name of the planetary constant.
    /// To load the planetary constants with another name, use `celestial_frame_from`
    pub fn celestial_frame(
        &self,
        ephemeris_name: &'a str,
        orientation_name: &'a str,
    ) -> Result<CelestialFrame, AniseError> {
        self.celestial_frame_from(ephemeris_name, orientation_name, ephemeris_name)
    }

    /// Tries to find the celestial frame data given the ephemeris center name, the orientation name, and the name of the planetary constants
    pub fn celestial_frame_from(
        &self,
        _ephemeris_name: &'a str,
        _orientation_name: &'a str,
        _planetary_constants_name: &'a str,
    ) -> Result<CelestialFrame, AniseError> {
        todo!()
        // let constants = self.planetary_constants_from_name(planetary_constants_name)?;

        // Ok(CelestialFrame {
        //     frame: Frame::from_ephemeris_orientation_names(ephemeris_name, orientation_name),
        //     mu_km3_s2: constants.mu_km3_s2,
        // })
    }
}
