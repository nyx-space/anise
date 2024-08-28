/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use crate::{
    astro::Aberration, ephemerides::EphemerisPhysicsSnafu, errors::EphemerisSnafu, frames::Frame,
    prelude::Orbit,
};

use super::Almanac;
use crate::errors::AlmanacResult;

use snafu::ResultExt;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python", pymethods)]
impl Almanac {
    /// Computes whether the line of sight between an observer and an observed Cartesian state is obstructed by the obstructing body.
    /// Returns true if the obstructing body is in the way, false otherwise.
    ///
    /// For example, if the Moon is in between a Lunar orbiter (observed) and a ground station (observer), then this function returns `true`
    /// because the Moon (obstructing body) is indeed obstructing the line of sight.
    ///
    /// ```text
    /// Observed
    ///   o  -
    ///    +    -
    ///     +      -
    ///      + ***   -
    ///     * +    *   -
    ///     *  + + * + + o
    ///     *     *     Observer
    ///       ****
    ///```
    ///
    /// Key Elements:
    /// - `o` represents the positions of the observer and observed objects.
    /// - The dashed line connecting the observer and observed is the line of sight.
    ///
    /// Algorithm (source: Algorithm 35 of Vallado, 4th edition, page 308.):
    /// - `r1` and `r2` are the transformed radii of the observed and observer objects, respectively.
    /// - `r1sq` and `r2sq` are the squared magnitudes of these vectors.
    /// - `r1dotr2` is the dot product of `r1` and `r2`.
    /// - `tau` is a parameter that determines the intersection point along the line of sight.
    /// - The condition `(1.0 - tau) * r1sq + r1dotr2 * tau <= ob_mean_eq_radius_km^2` checks if the line of sight is within the obstructing body's radius, indicating an obstruction.
    ///
    pub fn line_of_sight_obstructed(
        &self,
        observer: Orbit,
        observed: Orbit,
        obstructing_body: Frame,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<bool> {
        if observer == observed {
            return Ok(false);
        }

        let ob_mean_eq_radius_km = obstructing_body
            .mean_equatorial_radius_km()
            .context(EphemerisPhysicsSnafu {
                action: "fetching mean equatorial radius of eclipsing body",
            })
            .context(EphemerisSnafu {
                action: "computing eclipse state",
            })?;

        // Convert the states to the same frame as the eclipsing body (ensures we're in the same frame)
        let r1 = self
            .transform_to(observed, obstructing_body, ab_corr)?
            .radius_km;
        let r2 = self
            .transform_to(observer, obstructing_body, ab_corr)?
            .radius_km;

        let r1sq = r1.dot(&r1);
        let r2sq = r2.dot(&r2);
        let r1dotr2 = r1.dot(&r2);

        let tau = (r1sq - r1dotr2) / (r1sq + r2sq - 2.0 * r1dotr2);
        if !(0.0..=1.0).contains(&tau)
            || (1.0 - tau) * r1sq + r1dotr2 * tau > ob_mean_eq_radius_km.powi(2)
        {
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

#[cfg(test)]
mod ut_los {
    use crate::constants::frames::{EARTH_J2000, MOON_J2000};

    use super::*;
    use hifitime::Epoch;
    use rstest::*;

    #[fixture]
    pub fn almanac() -> Almanac {
        use std::path::PathBuf;

        let manifest_dir =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string()));

        Almanac::new(
            &manifest_dir
                .clone()
                .join("../data/de440s.bsp")
                .to_string_lossy(),
        )
        .unwrap()
        .load(
            &manifest_dir
                .clone()
                .join("../data/pck08.pca")
                .to_string_lossy(),
        )
        .unwrap()
    }

    #[rstest]
    fn los_edge_case(almanac: Almanac) {
        let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();
        let luna = almanac.frame_from_uid(MOON_J2000).unwrap();

        let dt1 = Epoch::from_gregorian_tai_hms(2020, 1, 1, 6, 7, 40);
        let dt2 = Epoch::from_gregorian_tai_hms(2020, 1, 1, 6, 7, 50);
        let dt3 = Epoch::from_gregorian_tai_hms(2020, 1, 1, 6, 8, 0);

        let xmtr1 = Orbit::new(
            397_477.494_485,
            -57_258.902_156,
            -62_857.909_437,
            0.230_482,
            2.331_362,
            0.615_501,
            dt1,
            eme2k,
        );
        let rcvr1 = Orbit::new(
            338_335.467_589,
            -55_439.526_977,
            -13_327.354_273,
            0.197_141,
            0.944_261,
            0.337_407,
            dt1,
            eme2k,
        );
        let xmtr2 = Orbit::new(
            397_479.756_900,
            -57_235.586_465,
            -62_851.758_851,
            0.222_000,
            2.331_768,
            0.614_614,
            dt2,
            eme2k,
        );
        let rcvr2 = Orbit::new(
            338_337.438_860,
            -55_430.084_340,
            -13_323.980_229,
            0.197_113,
            0.944_266,
            0.337_402,
            dt2,
            eme2k,
        );
        let xmtr3 = Orbit::new(
            397_481.934_480,
            -57_212.266_970,
            -62_845.617_185,
            0.213_516,
            2.332_122,
            0.613_717,
            dt3,
            eme2k,
        );
        let rcvr3 = Orbit::new(
            338_339.409_858,
            -55_420.641_651,
            -13_320.606_228,
            0.197_086,
            0.944_272,
            0.337_398,
            dt3,
            eme2k,
        );

        assert_eq!(
            almanac.line_of_sight_obstructed(xmtr1, rcvr1, luna, None),
            Ok(true)
        );
        assert_eq!(
            almanac.line_of_sight_obstructed(xmtr2, rcvr2, luna, None),
            Ok(true)
        );
        assert_eq!(
            almanac.line_of_sight_obstructed(xmtr3, rcvr3, luna, None),
            Ok(true)
        );

        // Test converse
        assert_eq!(
            almanac.line_of_sight_obstructed(rcvr1, xmtr1, luna, None),
            Ok(true)
        );
        assert_eq!(
            almanac.line_of_sight_obstructed(rcvr2, xmtr2, luna, None),
            Ok(true)
        );
        assert_eq!(
            almanac.line_of_sight_obstructed(rcvr3, xmtr3, luna, None),
            Ok(true)
        );
    }

    #[rstest]
    fn los_earth_eclipse(almanac: Almanac) {
        let eme2k = almanac.frame_from_uid(EARTH_J2000).unwrap();

        let dt = Epoch::from_gregorian_tai_at_midnight(2020, 1, 1);

        let sma = eme2k.mean_equatorial_radius_km().unwrap() + 300.0;

        let sc1 = Orbit::keplerian(sma, 0.001, 0.1, 90.0, 75.0, 0.0, dt, eme2k);
        let sc2 = Orbit::keplerian(sma + 1.0, 0.001, 0.1, 90.0, 75.0, 0.0, dt, eme2k);
        let sc3 = Orbit::keplerian(sma, 0.001, 0.1, 90.0, 75.0, 180.0, dt, eme2k);

        // Out of phase by pi.
        assert_eq!(
            almanac.line_of_sight_obstructed(sc1, sc3, eme2k, None),
            Ok(true)
        );

        assert_eq!(
            almanac.line_of_sight_obstructed(sc2, sc1, eme2k, None),
            Ok(false)
        );

        // Nearly identical orbits in the same phasing
        assert_eq!(
            almanac.line_of_sight_obstructed(sc1, sc2, eme2k, None),
            Ok(false)
        );
    }
}
