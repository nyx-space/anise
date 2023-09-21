/*
 * ANISE Toolkit
 * Copyright (C) 2021-2023 Christopher Rabotin <christop&her.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

pub mod celestial_objects {
    use crate::NaifId;

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

    pub const fn celestial_name_from_id(id: NaifId) -> Option<&'static str> {
        match id {
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

/// Defines the orientations known to ANISE and SPICE.
/// References used in the constants.
/// \[1\] Jay Lieske, ``Precession Matrix Based on IAU (1976)
/// System of Astronomical Constants,'' Astron. Astrophys.
/// 73, 282-284 (1979).
///
/// \[2\] E.M. Standish, Jr., ``Orientation of the JPL Ephemerides,
/// DE 200/LE 200, to the Dynamical Equinox of J2000,''
/// Astron. Astrophys. 114, 297-302 (1982).
///
/// \[3\] E.M. Standish, Jr., ``Conversion of Ephemeris Coordinates
/// from the B1950 System to the J2000 System,'' JPL IOM
/// 314.6-581, 24 June 1985.
///
/// \[4\] E.M. Standish, Jr., ``The Equinox Offsets of the JPL
/// Ephemeris,'' JPL IOM 314.6-929, 26 February 1988.
///
/// \[5\] Jay Lieske, ``Expressions for the Precession  Quantities
/// Based upon the IAU (1976) System of Astronomical
/// Constants'' Astron. Astrophys. 58, 1-16 (1977).
///
/// \[6\] Laura Bass and Robert Cesarone "Mars Observer Planetary
/// Constants and Models" JPL D-3444 November 1990.
///
/// \[7\] "Explanatory Supplement to the Astronomical Almanac"
///  edited by P. Kenneth Seidelmann. University Science
///  Books, 20 Edgehill Road, Mill Valley, CA 94941 (1992)
pub mod orientations {
    use crate::NaifId;
    /// Earth mean equator, dynamical equinox of J2000. The root reference frame for SPICE.
    pub const J2000: NaifId = 1;
    /// Earth mean equator, dynamical equinox of B1950.
    /// The B1950 reference frame is obtained by precessing the J2000 frame backwards from Julian year 2000 to Besselian year 1950, using the 1976 IAU precession model.
    /// The rotation from B1950 to J2000 is
    /// \[ -z \]  \[ theta \]  \[ -zeta \]
    ///         3            2            3
    /// The values for z, theta, and zeta are computed from the formulas given in table 5 of [5].
    /// z     =  1153.04066200330"
    /// theta =  1002.26108439117"
    /// zeta  =  1152.84248596724"
    pub const B1950: NaifId = 2;
    /// Fundamental Catalog (4). The FK4 reference frame is derived from the B1950 frame by applying the equinox offset determined by Fricke.
    /// \[ 0.525" \]
    ///             3
    pub const FK4: NaifId = 3;

    /// JPL Developmental Ephemeris (118). The DE-118 reference frame is nearly identical to the FK4 frame. It is also derived from the B1950 frame.
    /// Only the offset is different
    ///
    ///  \[ 0.53155" \]
    ///                3
    ///
    /// In [2], Standish uses two separate rotations,
    ///
    ///   \[ 0.00073" \]  P \[ 0.5316" \]
    ///                 3                3
    ///
    /// (where P is the precession matrix used above to define the B1950 frame). The major effect of the second rotation is to correct for truncating the magnitude of the first rotation.
    /// At his suggestion, we will use the untruncated value, and stick to a single rotation.
    ///
    ///
    /// Most of the other DE historical reference frames are defined relative to either the DE-118 or B1950 frame.
    /// The values below are taken from \[4\].
    ///```text
    ///    DE number  Offset from DE-118  Offset from B1950
    ///    ---------  ------------------  -----------------
    ///           96            +0.1209"           +0.4107"
    ///          102            +0.3956"           +0.1359"
    ///          108            +0.0541"           +0.4775"
    ///          111            -0.0564"           +0.5880"
    ///          114            -0.0213"           +0.5529"
    ///          122            +0.0000"           +0.5316"
    ///          125            -0.0438"           +0.5754"
    ///          130            +0.0069"           +0.5247"
    ///```
    pub const DE118: NaifId = 4;
    pub const DE096: NaifId = 5;
    pub const DE102: NaifId = 6;
    pub const DE108: NaifId = 7;
    pub const DE111: NaifId = 8;
    pub const DE114: NaifId = 9;
    pub const DE122: NaifId = 10;
    pub const DE125: NaifId = 11;
    pub const DE130: NaifId = 12;
    /// Galactic System II. The Galactic System II reference frame is defined by the following rotations:
    ///       o            o              o
    /// \[ 327  \]  \[ 62.6  \]  \[ 282.25  \]
    ///           3            1             3
    /// In the absence of better information, we assume the rotations are relative to the FK4 frame.
    pub const GALACTIC: NaifId = 13;
    pub const DE200: NaifId = 14;
    pub const DE202: NaifId = 15;
    /// Mars Mean Equator and IAU vector of J2000. The IAU-vector at Mars is the point on the mean equator of Mars where the equator ascends through the earth mean equator.
    /// This vector is the cross product of Earth mean north with Mars mean north.
    pub const MARSIAU: NaifId = 16;
    /// Ecliptic coordinates based upon the J2000 frame.
    /// The value for the obliquity of the ecliptic at J2000 is taken from page 114  of \[7\] equation 3.222-1.
    /// This agrees with the expression given in \[5\].
    pub const ECLIPJ2000: NaifId = 17;
    /// Ecliptic coordinates based upon the B1950 frame.
    /// The value for the obliquity of the ecliptic at B1950 is taken from page 171 of \[7\].
    pub const ECLIPB1950: NaifId = 18;
    /// JPL Developmental Ephemeris. (140)
    /// The DE-140 frame is the DE-400 frame rotated:
    ///
    ///   0.9999256765384668  0.0111817701197967  0.0048589521583895
    ///  -0.0111817701797229  0.9999374816848701 -0.0000271545195858
    ///  -0.0048589520204830 -0.0000271791849815  0.9999881948535965
    ///
    /// The DE-400 frame is treated as equivalent to the J2000 frame.
    pub const DE140: NaifId = 19;

    /// JPL Developmental Ephemeris. (142)
    /// The DE-142 frame is the DE-402 frame rotated:
    ///
    ///    0.9999256765402605  0.0111817697320531  0.0048589526815484
    ///   -0.0111817697907755  0.9999374816892126 -0.0000271547693170
    ///   -0.0048589525464121 -0.0000271789392288  0.9999881948510477
    ///
    /// The DE-402 frame is treated as equivalent to the J2000 frame.
    pub const DE142: NaifId = 20;

    /// JPL Developmental Ephemeris. (143)
    /// The DE-143 frame is the DE-403 frame rotated:
    ///
    ///    0.9999256765435852  0.0111817743077255  0.0048589414674762
    ///   -0.0111817743300355  0.9999374816382505 -0.0000271622115251
    ///   -0.0048589414161348 -0.0000271713942366  0.9999881949053349
    ///
    /// The DE-403 frame is treated as equivalent to the J2000 frame.
    pub const DE143: NaifId = 21;

    /// Given the frame ID, try to return a human name
    /// Source: // https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/frames.html#Appendix.%20%60%60Built%20in''%20Inertial%20Reference%20Frames
    pub const fn orientation_name_from_id(hash: NaifId) -> Option<&'static str> {
        match hash {
            J2000 => Some("J2000"),
            B1950 => Some("B1950"),
            FK4 => Some("FK4"),
            GALACTIC => Some("Galactic"),
            MARSIAU => Some("Mars IAU"),
            ECLIPJ2000 => Some("ECLIPJ2000"),
            ECLIPB1950 => Some("ECLIPB1950"),
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
    pub const EME2000: Frame = Frame::from_ephem_orient(EARTH, J2000);
}

#[cfg(test)]
mod constants_ut {
    use crate::constants::orientations::{
        orientation_name_from_id, B1950, ECLIPB1950, ECLIPJ2000, FK4, J2000, MARSIAU,
    };

    use crate::constants::celestial_objects::*;

    #[test]
    fn orient_name_from_id() {
        assert_eq!(orientation_name_from_id(J2000).unwrap(), "J2000");
        assert_eq!(orientation_name_from_id(B1950).unwrap(), "B1950");
        assert_eq!(orientation_name_from_id(ECLIPB1950).unwrap(), "ECLIPB1950");
        assert_eq!(orientation_name_from_id(ECLIPJ2000).unwrap(), "ECLIPJ2000");
        assert_eq!(orientation_name_from_id(FK4).unwrap(), "FK4");
        assert_eq!(orientation_name_from_id(MARSIAU).unwrap(), "Mars IAU");
        assert!(orientation_name_from_id(-1).is_none());
    }

    #[test]
    fn object_name_from_id() {
        assert_eq!(
            celestial_name_from_id(SOLAR_SYSTEM_BARYCENTER).unwrap(),
            "Solar System Barycenter"
        );
        assert_eq!(celestial_name_from_id(MERCURY).unwrap(), "Mercury");
        assert_eq!(celestial_name_from_id(VENUS).unwrap(), "Venus");
        assert_eq!(
            celestial_name_from_id(EARTH_MOON_BARYCENTER).unwrap(),
            "Earth-Moon Barycenter"
        );
        assert_eq!(
            celestial_name_from_id(MARS_BARYCENTER).unwrap(),
            "Mars Barycenter"
        );
        assert_eq!(
            celestial_name_from_id(JUPITER_BARYCENTER).unwrap(),
            "Jupiter Barycenter"
        );
        assert_eq!(
            celestial_name_from_id(SATURN_BARYCENTER).unwrap(),
            "Saturn Barycenter"
        );
        assert_eq!(
            celestial_name_from_id(URANUS_BARYCENTER).unwrap(),
            "Uranus Barycenter"
        );
        assert_eq!(
            celestial_name_from_id(NEPTUNE_BARYCENTER).unwrap(),
            "Neptune Barycenter"
        );
        assert_eq!(
            celestial_name_from_id(PLUTO_BARYCENTER).unwrap(),
            "Pluto Barycenter"
        );
        assert_eq!(celestial_name_from_id(SUN).unwrap(), "Sun");
        assert_eq!(celestial_name_from_id(LUNA).unwrap(), "Luna");
        assert_eq!(celestial_name_from_id(EARTH).unwrap(), "Earth");
        assert!(celestial_name_from_id(-1).is_none());
    }
}
