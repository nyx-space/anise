/*
 * ANISE Toolkit
 * Copyright (C) 2021-2022 Christopher Rabotin <christop&her.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

pub mod celestial_objects {
    use crate::NaifId;

    // TODO: Merge with id_to_human_name

    pub const SOLAR_SYSTEM_BARYCENTER: NaifId = 0;
    pub const MERCURY: NaifId = 1;
    pub const VENUS: NaifId = 2;
    pub const EARTH_MOON_BARYCENTER: NaifId = 3;
    pub const MARS_BARYCENTER: NaifId = 4;
    pub const JUPITER_BARYCENTER: NaifId = 5;
    pub const SATURN_BARYCENTER: NaifId = 6;
    pub const URANUS_BARYCENTER: NaifId = 7;
    pub const NEPTUNE_BARYCENTER: NaifId = 8;
    pub const PLUTO_BARYCENTER: NaifId = 9;
    pub const SUN: NaifId = 10;
    pub const LUNA: NaifId = 301;
    pub const EARTH: NaifId = 399;

    pub const fn hash_celestial_name<'a>(hash: NaifId) -> Option<&'a str> {
        match hash {
            SOLAR_SYSTEM_BARYCENTER => Some("Solar System Barycenter"),
            MERCURY => Some("Mercury"),
            VENUS => Some("Venus"),
            EARTH_MOON_BARYCENTER => Some("Earth-Moon Barycenter"),
            MARS_BARYCENTER => Some("Mars Barycenter"),
            JUPITER_BARYCENTER => Some("Jupiter Barycenter"),
            SATURN_BARYCENTER => Some("Saturn Barycenter"),
            URANUS_BARYCENTER => Some("Uranus Barycenter"),
            NEPTUNE_BARYCENTER => Some("Neptune Barycenter"),
            PLUTO_BARYCENTER => Some("Pluto Barycenter"),
            SUN => Some("Sun"),
            LUNA => Some("Luna"),
            EARTH => Some("Earth"),
            _ => None,
        }
    }
}

pub mod orientations {
    use crate::NaifId;
    pub const J2000: NaifId = 0;

    pub const fn hash_orientation_name<'a>(hash: NaifId) -> Option<&'a str> {
        match hash {
            J2000 => Some("J2000"),
            _ => None,
        }
    }
}

pub mod frames {
    use crate::prelude::Frame;

    use super::{celestial_objects::*, orientations::J2000};

    pub const SSB_J2000: Frame = Frame::from_ephem_orient(SOLAR_SYSTEM_BARYCENTER, J2000);
    pub const MERCURY_J2000: Frame = Frame::from_ephem_orient(MERCURY, J2000);
    pub const VENUS_J2000: Frame = Frame::from_ephem_orient(VENUS, J2000);
    pub const EARTH_MOON_BARYCENTER_J2000: Frame =
        Frame::from_ephem_orient(EARTH_MOON_BARYCENTER, J2000);
    pub const MARS_BARYCENTER_J2000: Frame = Frame::from_ephem_orient(MARS_BARYCENTER, J2000);
    pub const JUPITER_BARYCENTER_J2000: Frame = Frame::from_ephem_orient(JUPITER_BARYCENTER, J2000);
    pub const SATURN_BARYCENTER_J2000: Frame = Frame::from_ephem_orient(SATURN_BARYCENTER, J2000);
    pub const URANUS_BARYCENTER_J2000: Frame = Frame::from_ephem_orient(URANUS_BARYCENTER, J2000);
    pub const NEPTUNE_BARYCENTER_J2000: Frame = Frame::from_ephem_orient(NEPTUNE_BARYCENTER, J2000);
    pub const PLUTO_BARYCENTER_J2000: Frame = Frame::from_ephem_orient(PLUTO_BARYCENTER, J2000);
    pub const SUN_J2000: Frame = Frame::from_ephem_orient(SUN, J2000);
    pub const LUNA_J2000: Frame = Frame::from_ephem_orient(LUNA, J2000);
    pub const EARTH_J2000: Frame = Frame::from_ephem_orient(EARTH, J2000);
}
