/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use log::error;

use crate::{
    astro::{Aberration, Occultation},
    constants::{frames::SUN_J2000, orientations::J2000},
    ephemerides::EphemerisPhysicsSnafu,
    errors::{AlmanacError, EphemerisSnafu, OrientationSnafu},
    frames::Frame,
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
    /// :type observer: Orbit
    /// :type observed: Orbit
    /// :type obstructing_body: Frame
    /// :type ab_corr: Aberration, optional
    /// :rtype: bool
    pub fn line_of_sight_obstructed(
        &self,
        observer: Orbit,
        observed: Orbit,
        mut obstructing_body: Frame,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<bool> {
        if observer == observed {
            return Ok(false);
        }

        if obstructing_body.mean_equatorial_radius_km().is_err() {
            obstructing_body =
                self.frame_from_uid(obstructing_body)
                    .map_err(|e| AlmanacError::GenericError {
                        err: format!("{e} when fetching frame data for {obstructing_body}"),
                    })?;
        }

        let ob_mean_eq_radius_km = obstructing_body
            .mean_equatorial_radius_km()
            .context(EphemerisPhysicsSnafu {
                action: "fetching mean equatorial radius of obstructing body",
            })
            .context(EphemerisSnafu {
                action: "computing line of sight",
            })?;

        // Convert the states to the same frame as the obstructing body (ensures we're in the same frame)
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

    /// Computes the occultation percentage of the `back_frame` object by the `front_frame` object as seen from the observer, when according for the provided aberration correction.
    ///
    /// A zero percent occultation means that the back object is fully visible from the observer.
    /// A 100%  percent occultation means that the back object is fully hidden from the observer because of the front frame (i.e. _umbra_ if the back object is the Sun).
    /// A value in between means that the back object is partially hidden from the observser (i.e. _penumbra_ if the back object is the Sun).
    /// Refer to the [MathSpec](https://nyxspace.com/nyxspace/MathSpec/celestial/eclipse/) for modeling details.
    ///
    /// :type back_frame: Frame
    /// :type front_frame: Frame
    /// :type observer: Orbit
    /// :type ab_corr: Aberration, optional
    /// :rtype: Occultation
    pub fn occultation(
        &self,
        mut back_frame: Frame,
        mut front_frame: Frame,
        mut observer: Orbit,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<Occultation> {
        if back_frame.mean_equatorial_radius_km().is_err() {
            back_frame =
                self.frame_from_uid(back_frame)
                    .map_err(|e| AlmanacError::GenericError {
                        err: format!("{e} when fetching {back_frame:e} frame data"),
                    })?;
        }

        if front_frame.mean_equatorial_radius_km().is_err() {
            front_frame =
                self.frame_from_uid(front_frame)
                    .map_err(|e| AlmanacError::GenericError {
                        err: format!("{e} when fetching {front_frame:e} frame data"),
                    })?;
        }

        let bobj_mean_eq_radius_km = back_frame
            .mean_equatorial_radius_km()
            .context(EphemerisPhysicsSnafu {
                action: "fetching mean equatorial radius of back frame",
            })
            .context(EphemerisSnafu {
                action: "computing occultation state",
            })?;

        let epoch = observer.epoch;

        // If the back object's radius is zero, just call the line of sight algorithm
        if bobj_mean_eq_radius_km < f64::EPSILON {
            let observed = -self.transform_to(observer, back_frame, ab_corr)?;
            let percentage =
                if self.line_of_sight_obstructed(observer, observed, front_frame, ab_corr)? {
                    100.0
                } else {
                    0.0
                };
            return Ok(Occultation {
                epoch,
                percentage,
                back_frame,
                front_frame,
            });
        }

        // All of the computations happen with the observer as the center.
        // `eb` stands for front object; `ls` stands for back object.
        // Get the radius vector of the spacecraft to the front object

        // Ensure that the observer is in the J2000 frame.
        observer = self
            .rotate_to(observer, observer.frame.with_orient(J2000))
            .context(OrientationSnafu {
                action: "computing eclipse state",
            })?;
        let r_eb = self
            .transform_to(observer, front_frame.with_orient(J2000), ab_corr)?
            .radius_km;

        // Get the radius vector of the back object to the spacecraft
        let r_ls = -self
            .transform_to(observer, back_frame.with_orient(J2000), ab_corr)?
            .radius_km;

        // Compute the apparent radii of the back object and front object (preventing any NaN)
        let r_ls_prime = if bobj_mean_eq_radius_km >= r_ls.norm() {
            bobj_mean_eq_radius_km
        } else {
            (bobj_mean_eq_radius_km / r_ls.norm()).asin()
        };

        let fobj_mean_eq_radius_km = front_frame
            .mean_equatorial_radius_km()
            .context(EphemerisPhysicsSnafu {
                action: "fetching mean equatorial radius of front object",
            })
            .context(EphemerisSnafu {
                action: "computing eclipse state",
            })?;

        let r_fobj_prime = if fobj_mean_eq_radius_km >= r_eb.norm() {
            fobj_mean_eq_radius_km
        } else {
            (fobj_mean_eq_radius_km / r_eb.norm()).asin()
        };

        // Compute the apparent separation of both circles
        let d_prime = (-(r_ls.dot(&r_eb)) / (r_eb.norm() * r_ls.norm())).acos();

        if d_prime - r_ls_prime > r_fobj_prime {
            // If the closest point where the apparent radius of the back object _starts_ is further
            // away than the furthest point where the front object's shadow can reach, then the light
            // source is totally visible.
            Ok(Occultation {
                epoch,
                percentage: 0.0,
                back_frame,
                front_frame,
            })
        } else if r_fobj_prime > d_prime + r_ls_prime {
            // The back object is fully hidden by the front object, hence we're in total eclipse.
            Ok(Occultation {
                epoch,
                percentage: 100.0,
                back_frame,
                front_frame,
            })
        } else if (r_ls_prime - r_fobj_prime).abs() < d_prime && d_prime < r_ls_prime + r_fobj_prime
        {
            // If we have reached this point, we're in penumbra.
            // Both circles, which represent the back object projected onto the plane and the eclipsing geoid,
            // now overlap creating an asymmetrial lens.
            // The following math comes from http://mathworld.wolfram.com/Circle-CircleIntersection.html
            // and https://stackoverflow.com/questions/3349125/circle-circle-intersection-points .

            // Compute the distances between the center of the eclipsing geoid and the line crossing the intersection
            // points of both circles.
            let d1 =
                (d_prime.powi(2) - r_ls_prime.powi(2) + r_fobj_prime.powi(2)) / (2.0 * d_prime);
            let d2 =
                (d_prime.powi(2) + r_ls_prime.powi(2) - r_fobj_prime.powi(2)) / (2.0 * d_prime);

            let shadow_area = circ_seg_area(r_fobj_prime, d1) + circ_seg_area(r_ls_prime, d2);
            if shadow_area.is_nan() {
                error!(
                "Shadow area is NaN! Please file a bug with initial states, eclipsing bodies, etc."
                );
                return Ok(Occultation {
                    epoch,
                    percentage: 100.0,
                    back_frame,
                    front_frame,
                });
            }
            // Compute the nominal area of the back object
            let nominal_area = core::f64::consts::PI * r_ls_prime.powi(2);
            // And return the percentage (between 0 and 1) of the eclipse.
            let percentage = 100.0 * shadow_area / nominal_area;
            Ok(Occultation {
                epoch,
                percentage,
                back_frame,
                front_frame,
            })
        } else {
            // Annular eclipse.
            // If r_fobj_prime is very small, then the fraction is very small: however, we note a penumbra close to 1.0 as near full back object visibility, so let's subtract one from this.
            let percentage = 100.0 * r_fobj_prime.powi(2) / r_ls_prime.powi(2);
            Ok(Occultation {
                epoch,
                percentage,
                back_frame,
                front_frame,
            })
        }
    }

    /// Computes the solar eclipsing of the observer due to the eclipsing_frame.
    ///
    /// This function calls `occultation` where the back object is the Sun in the J2000 frame, and the front object
    /// is the provided eclipsing frame.
    ///
    /// :type eclipsing_frame: Frame
    /// :type observer: Orbit
    /// :type ab_corr: Aberration, optional
    /// :rtype: Occultation
    pub fn solar_eclipsing(
        &self,
        eclipsing_frame: Frame,
        observer: Orbit,
        ab_corr: Option<Aberration>,
    ) -> AlmanacResult<Occultation> {
        self.occultation(SUN_J2000, eclipsing_frame, observer, ab_corr)
    }
}

/// Compute the area of the circular segment of radius r and chord length d
fn circ_seg_area(r: f64, d: f64) -> f64 {
    r.powi(2) * (d / r).acos() - d * (r.powi(2) - d.powi(2)).sqrt()
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
