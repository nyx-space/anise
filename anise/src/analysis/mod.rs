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
    almanac::Almanac,
    analysis::report::ReportScalars,
    errors::{AlmanacError, MathError, PhysicsError},
    prelude::Orbit,
};
use hifitime::{Epoch, TimeSeries};
use prelude::OrbitalElement;
use rayon::prelude::*;
use snafu::prelude::*;
use std::collections::HashMap;

pub mod elements;
pub mod event;
pub mod expr;
pub mod report;
pub mod search;
pub mod specs;
pub mod vector_expr;

use event::Event;
use expr::ScalarExpr;
use specs::StateSpec;
use vector_expr::VectorExpr;

#[cfg(feature = "python")]
pub mod python;

pub mod prelude {
    pub use super::elements::OrbitalElement;
    pub use super::expr::ScalarExpr;
    pub use super::specs::{FrameSpec, StateSpec};
    pub use super::vector_expr::VectorExpr;
    pub use crate::prelude::Frame;
}

#[derive(Debug, PartialEq, Snafu)]
#[snafu(visibility(pub))]
pub enum AnalysisError {
    #[snafu(display("computing {expr:?} on {state} encountered an Almanac error {source}"))]
    AlmanacExpr {
        expr: Box<ScalarExpr>,
        state: Box<Orbit>,
        #[snafu(source(from(AlmanacError, Box::new)))]
        source: Box<AlmanacError>,
    },
    #[snafu(display("mission data in Almanac to compute {expr:?}"))]
    AlmanacMissingDataExpr { expr: Box<ScalarExpr> },
    #[snafu(display("computing state {spec:?} at {epoch} encountered an Almanac error {source}"))]
    AlmanacStateSpec {
        spec: Box<StateSpec>,
        epoch: Epoch,
        #[snafu(source(from(AlmanacError, Box::new)))]
        source: Box<AlmanacError>,
    },
    #[snafu(display("computing {el:?} on {orbit} encountered a physics error {source}"))]
    PhysicsOrbitEl {
        el: Box<OrbitalElement>,
        orbit: Box<Orbit>,
        #[snafu(source(from(PhysicsError, Box::new)))]
        source: Box<PhysicsError>,
    },
    #[snafu(display("computing {expr:?} at {epoch} encountered a physics error {source}"))]
    PhysicsVecExpr {
        expr: Box<VectorExpr>,
        epoch: Epoch,
        #[snafu(source(from(PhysicsError, Box::new)))]
        source: Box<PhysicsError>,
    },
    #[snafu(display("computing {expr:?} encountered a math error {source}"))]
    MathExpr {
        expr: Box<ScalarExpr>,
        #[snafu(source(from(MathError, Box::new)))]
        source: Box<MathError>,
    },
    #[snafu(display("event {event} not found in [{start}; {end}]"))]
    EventNotFound {
        start: Epoch,
        end: Epoch,
        event: Box<Event>,
    },
}

pub type AnalysisResult<T> = Result<T, AnalysisError>;

impl Almanac {
    /// Report a set of scalar expressions, optionally with aliases, at a fixed time step defined in the TimeSeries.
    pub fn report_scalars(
        &self,
        report: &ReportScalars,
        time_series: TimeSeries,
    ) -> HashMap<Epoch, Result<HashMap<String, AnalysisResult<f64>>, AnalysisError>> {
        time_series
            .par_bridge()
            .map_with((&self, report), |(almanac, report), epoch| {
                match report.state_spec.evaluate(epoch, almanac) {
                    Ok(orbit) => {
                        let mut data = HashMap::new();

                        let ab_corr = report.state_spec.ab_corr;

                        for (expr, alias) in report.scalars.iter() {
                            data.insert(
                                alias.clone().unwrap_or_else(|| expr.to_string()),
                                expr.evaluate(orbit, ab_corr, almanac),
                            );
                        }
                        (epoch, Ok(data))
                    }
                    Err(e) => (epoch, Err(e)),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod ut_analysis {

    use crate::analysis::event::{Event, EventEdge};
    use crate::analysis::prelude::*;
    use crate::analysis::report::ReportScalars;
    use crate::analysis::specs::{OrthogonalFrame, Plane};
    use crate::astro::{Aberration, Location, TerrainMask};
    use crate::constants::frames::{EME2000, IAU_EARTH_FRAME, MOON_J2000, SUN_J2000, VENUS_J2000};
    use crate::prelude::{Almanac, Frame};
    use crate::structure::LocationDataSet;
    use hifitime::{Duration, Epoch, TimeSeries, Unit};
    use rstest::*;

    #[fixture]
    fn almanac() -> Almanac {
        use std::path::PathBuf;

        // Build the new location
        let dsn_madrid = Location {
            latitude_deg: 40.427_222,
            longitude_deg: 4.250_556,
            height_km: 0.834_939,
            frame: IAU_EARTH_FRAME.into(),
            // Create a fake elevation mask to check that functionality
            terrain_mask: vec![
                TerrainMask {
                    azimuth_deg: 0.0,
                    elevation_mask_deg: 0.0,
                },
                TerrainMask {
                    azimuth_deg: 130.0,
                    elevation_mask_deg: 8.0,
                },
                TerrainMask {
                    azimuth_deg: 140.0,
                    elevation_mask_deg: 0.0,
                },
            ],
            // Ignore terrain mask for the test
            terrain_mask_ignored: true,
        };

        // Build a dataset with this single location
        let mut loc_data = LocationDataSet::default();
        loc_data
            .push(dsn_madrid, Some(123), Some("DSN Madrid"))
            .unwrap();

        let manifest_dir =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_string()));

        let mut almanac = Almanac::new(
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
        .load(
            &manifest_dir
                .clone()
                .join("../data/lro.bsp")
                .to_string_lossy(),
        )
        .unwrap();

        almanac.location_data = loc_data;

        almanac
    }

    #[test]
    fn test_displays() {
        let from_frame = FrameSpec::Loaded(EME2000);
        let to_frame = FrameSpec::Loaded(SUN_J2000);

        let state = StateSpec {
            target_frame: from_frame,
            observer_frame: to_frame,
            ab_corr: Aberration::NONE,
        };

        assert_eq!(format!("{state}"), "Earth J2000 -> Sun J2000");

        let r = VectorExpr::Radius(state.clone());
        let v = VectorExpr::Velocity(state.clone());
        let h = VectorExpr::CrossProduct {
            a: Box::new(r.clone()),
            b: Box::new(v.clone()),
        };
        println!("{r}\n{v}\n{h}");
    }

    #[rstest]
    fn test_analysis_gen_report(almanac: Almanac) {
        // Try to compute the SMA of the Earth with respect to the Sun.
        let target_frame = FrameSpec::Loaded(EME2000);
        let observer_frame = FrameSpec::Loaded(MOON_J2000);

        let state = StateSpec {
            target_frame: target_frame.clone(),
            observer_frame: observer_frame.clone(),
            ab_corr: Aberration::NONE,
        };

        // Build the orthogonal VNC frame of the Earth ... isn't useful per-se
        // just a proof of concept, ensuring we normalize these vectors.
        let vnc = OrthogonalFrame::XY {
            x: VectorExpr::Unit(Box::new(VectorExpr::Velocity(state.clone()))),
            y: VectorExpr::Unit(Box::new(VectorExpr::OrbitalMomentum(state.clone()))),
        };

        let sun_state = StateSpec {
            target_frame,
            observer_frame: FrameSpec::Loaded(SUN_J2000),
            ab_corr: Aberration::LT,
        };

        // Project the Earth->Sun vector onto the VNC frame
        let proj = VectorExpr::Project {
            v: Box::new(VectorExpr::Negate(Box::new(VectorExpr::Unit(Box::new(
                VectorExpr::Radius(sun_state),
            ))))),
            frame: Box::new(vnc),
            plane: Some(Plane::XY),
        };

        println!("{proj}");

        // Rebuild the Local Solar Time calculation
        let sun_frame = FrameSpec::Loaded(SUN_J2000);
        let earth_sun = StateSpec {
            target_frame: sun_frame,
            observer_frame: observer_frame.clone(),
            ab_corr: Aberration::LT,
        };

        let u = VectorExpr::Unit(Box::new(VectorExpr::CrossProduct {
            a: Box::new(VectorExpr::Unit(Box::new(VectorExpr::Radius(earth_sun)))),
            b: Box::new(VectorExpr::Unit(Box::new(VectorExpr::OrbitalMomentum(
                state.clone(),
            )))),
        }));

        let v = VectorExpr::CrossProduct {
            a: Box::new(VectorExpr::Unit(Box::new(VectorExpr::OrbitalMomentum(
                state.clone(),
            )))),
            b: Box::new(u.clone()),
        };

        let r = VectorExpr::Radius(state.clone());

        let sin_theta = ScalarExpr::DotProduct {
            a: v.clone(),
            b: r.clone(),
        };
        let cos_theta = ScalarExpr::DotProduct {
            a: u.clone(),
            b: r.clone(),
        };

        let theta = ScalarExpr::Atan2 {
            y: Box::new(sin_theta),
            x: Box::new(cos_theta),
        };

        let lst_prod = ScalarExpr::Mul {
            a: Box::new(ScalarExpr::Mul {
                a: Box::new(theta),
                b: Box::new(ScalarExpr::Constant(1.0 / 180.0)),
            }),
            b: Box::new(ScalarExpr::Constant(12.0)),
        };

        let lst_add = ScalarExpr::Add {
            a: Box::new(lst_prod),
            b: Box::new(ScalarExpr::Constant(6.0)),
        };

        let lst = ScalarExpr::Modulo {
            v: Box::new(lst_add),
            m: Box::new(ScalarExpr::Constant(24.0)),
        };

        let scalars = [
            ScalarExpr::Element(OrbitalElement::SemiMajorAxis),
            ScalarExpr::Element(OrbitalElement::Eccentricity),
            ScalarExpr::Element(OrbitalElement::Rmag),
            ScalarExpr::BetaAngle,
            ScalarExpr::SolarEclipsePercentage {
                eclipsing_frame: VENUS_J2000,
            },
            ScalarExpr::Norm(VectorExpr::Radius(state.clone())),
            ScalarExpr::DotProduct {
                a: VectorExpr::EccentricityVector(state.clone()),
                b: VectorExpr::Fixed {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            ScalarExpr::VectorX(VectorExpr::EccentricityVector(state.clone())),
            ScalarExpr::VectorY(VectorExpr::EccentricityVector(state.clone())),
            ScalarExpr::VectorZ(VectorExpr::EccentricityVector(state.clone())),
            // Test orbital momentum magnitude
            ScalarExpr::Norm(VectorExpr::CrossProduct {
                a: Box::new(VectorExpr::Radius(state.clone())),
                b: Box::new(VectorExpr::Velocity(state.clone())),
            }),
            ScalarExpr::Element(OrbitalElement::Hmag),
            ScalarExpr::AngleBetween {
                a: VectorExpr::Radius(state.clone()),
                b: VectorExpr::Velocity(state.clone()),
            },
            ScalarExpr::AzimuthFromLocation {
                location_id: 123,
                obstructing_body: None,
            },
            ScalarExpr::ElevationFromLocation {
                location_id: 123,
                obstructing_body: None,
            },
            ScalarExpr::RangeFromLocation {
                location_id: 123,
                obstructing_body: None,
            },
            ScalarExpr::RangeRateFromLocation {
                location_id: 123,
                obstructing_body: None,
            },
            ScalarExpr::LocalTimeAscNode,
            ScalarExpr::LocalTimeDescNode,
            ScalarExpr::VectorX(proj.clone()),
            ScalarExpr::VectorY(proj.clone()),
            ScalarExpr::VectorZ(proj.clone()),
            ScalarExpr::LocalSolarTime,
            lst,
        ];

        // Demo of an S-Expression export
        let proj = scalars.last().unwrap();
        let proj_s = proj.to_s_expr().unwrap();
        let proj_reload = ScalarExpr::from_s_expr(&proj_s).unwrap();
        assert_eq!(&proj_reload, proj);

        let cnt = scalars.len();

        let mut scalars_with_aliases = scalars.map(|s| (s, None));
        // Set an alias for the last three.
        scalars_with_aliases[cnt - 5].1 = Some("proj VNC X".to_string());
        scalars_with_aliases[cnt - 4].1 = Some("proj VNC Y".to_string());
        scalars_with_aliases[cnt - 3].1 = Some("proj VNC Z".to_string());
        scalars_with_aliases[cnt - 1].1 = Some("LST (h)".to_string());

        // Build the report, ensure we can serialize it and deserialize it.
        let report = ReportScalars {
            scalars: scalars_with_aliases.to_vec(),
            state_spec: state,
        };

        let report_s_expr = report.to_s_expr().unwrap();

        println!("REPORT S-EXPR\n{report_s_expr}\n");

        let report_reloaded = ReportScalars::from_s_expr(&report_s_expr).unwrap();

        assert_eq!(report_reloaded, report);

        let data = almanac.report_scalars(
            &report,
            TimeSeries::inclusive(
                Epoch::from_gregorian_utc_at_midnight(2025, 1, 1),
                Epoch::from_gregorian_utc_at_noon(2025, 1, 2),
                Unit::Day * 0.5,
            ),
        );

        assert_eq!(data.len(), 4);

        let last_row = data.values().last().unwrap().as_ref().unwrap();

        println!("{last_row:?}");
        assert_eq!(last_row.len(), scalars_with_aliases.len());

        // Test that we correctly computed the norm of the cross product
        assert_eq!(
            last_row["Hmag (km)"],
            last_row["|Radius(Earth J2000 -> Moon J2000) ⨯ Velocity(Earth J2000 -> Moon J2000)|"]
        );

        assert!(
            (last_row["LST (h)"].as_ref().unwrap()
                - last_row["local solar time (h)"].as_ref().unwrap())
            .abs()
                * Unit::Hour
                < Unit::Second * 1
        );

        for (k, v) in last_row.iter() {
            if k.contains("proj") {
                // Check that we have correctly defined the projections onto an othogonal frame
                assert!(v.as_ref().unwrap().abs() <= 1.0);
            }
        }
    }

    #[rstest]
    fn test_analysis_event(mut almanac: Almanac) {
        let lro_frame = Frame::from_ephem_j2000(-85);

        let lro_state_spec = StateSpec {
            target_frame: FrameSpec::Loaded(lro_frame),
            observer_frame: FrameSpec::Loaded(MOON_J2000),
            ab_corr: None,
        };

        let sunset_nadir = Event {
            scalar: ScalarExpr::SunAngle { observer_id: -85 },
            desired_value: 90.0,
            epoch_precision: Unit::Second * 0.5,
            value_precision: 0.1,
            ab_corr: None,
        };

        let apolune = Event::apoapsis();
        let perilune = Event::periapsis();

        let eclipse = Event::eclipse(MOON_J2000);

        let (start_epoch, end_epoch) = almanac.spk_domain(-85).unwrap();

        let start_orbit = almanac
            .transform(lro_frame, MOON_J2000, start_epoch, None)
            .unwrap();
        let period = start_orbit.period().unwrap();

        // End setup

        let apo_events = almanac
            .report_events(
                &lro_state_spec,
                &apolune,
                start_epoch,
                end_epoch,
                Some(period * 0.5),
            )
            .unwrap();

        println!("Searching for {apolune}");
        println!("\nAPO S-EXPR: {}", apolune.to_s_expr().unwrap());
        let eclipse_s_expr = eclipse.to_s_expr().unwrap();
        let deserd = Event::from_s_expr(&eclipse_s_expr).unwrap();
        assert_eq!(deserd, eclipse);
        println!("\nEclipse S-EXPR: {eclipse_s_expr}");

        for event in &apo_events {
            let ta_deg = event.orbit.ta_deg().unwrap();
            println!("{event} -> true anomaly = {ta_deg:.6} deg",);
            assert!((ta_deg - 180.0).abs() < apolune.value_precision);
        }

        let peri_events = almanac
            .report_events(
                &lro_state_spec,
                &perilune,
                start_epoch,
                end_epoch,
                Some(period * 0.5),
            )
            .unwrap();

        println!("Searching for {perilune}");

        for event in &peri_events {
            let ta_deg = event.orbit.ta_deg().unwrap();
            println!("{event} -> true anomaly = {ta_deg:.6} deg",);
            assert!(
                ta_deg.abs() < perilune.value_precision
                    || (ta_deg - 360.0).abs() < perilune.value_precision
            );
        }

        println!(
            "Found {} apos and {} peris over {}",
            apo_events.len(),
            peri_events.len(),
            end_epoch - start_epoch
        );

        let mut missed_events = 0;
        // Check the time difference between two subsequent apoapses
        let dt_bw_apos = apo_events
            .iter()
            .take(apo_events.len() - 1)
            .zip(apo_events.iter().skip(1))
            .map(|(first, second)| second.orbit.epoch - first.orbit.epoch)
            .collect::<Vec<Duration>>();

        for dt in dt_bw_apos {
            let err = period - dt;
            // We expect one apo per orbit
            if err.abs() > Unit::Minute * 5 {
                missed_events += 1;
            }
        }

        // Check the time difference between two subsequent apoapses
        let dt_bw_peris = peri_events
            .iter()
            .take(peri_events.len() - 1)
            .zip(peri_events.iter().skip(1))
            .map(|(first, second)| second.orbit.epoch - first.orbit.epoch)
            .collect::<Vec<Duration>>();

        for dt in dt_bw_peris {
            let err = period - dt;
            // We expect one apo per orbit
            if err.abs() > Unit::Minute * 5 {
                missed_events += 1;
            }
        }
        assert_eq!(missed_events, 0);

        let events = almanac
            .report_event_arcs(&lro_state_spec, &sunset_nadir, start_epoch, end_epoch)
            .unwrap();

        println!("First sunset of {}: {}", events.len(), events[1]);
        assert_eq!(events[1].rise.edge, EventEdge::Rising);
        assert_eq!(events[1].fall.edge, EventEdge::Falling);
        assert_eq!(events.len(), 309);

        let eclipses = almanac
            .report_event_arcs(
                &lro_state_spec,
                &eclipse,
                start_epoch,
                start_epoch + Unit::Hour * 3,
            )
            .unwrap();

        assert_eq!(eclipses.len(), 2, "wrong number of eclipse periods found");

        for event in &eclipses {
            println!("{event}\n{event:?}");
            assert!(event.duration() > Unit::Minute * 24 && event.duration() < Unit::Minute * 40);
            // Check that this is valid
            for epoch in TimeSeries::inclusive(
                event.start_epoch() - eclipse.epoch_precision,
                event.end_epoch() + eclipse.epoch_precision,
                Unit::Minute * 0.5,
            ) {
                if let Ok(orbit) = lro_state_spec.evaluate(epoch, &almanac) {
                    let this_eclipse = eclipse.eval(orbit, &almanac).unwrap();
                    let in_eclipse = this_eclipse.abs() <= eclipse.value_precision.abs();

                    if (event.start_epoch()..event.end_epoch()).contains(&epoch) {
                        // We're in the event, check that it is evaluated to be in the event.
                        assert!(in_eclipse);
                    } else {
                        assert!(!in_eclipse || this_eclipse < 0.0);
                    }
                }
            }
            println!("\n");
        }

        // https://github.com/nyx-space/anise/issues/537
        /* // Test access times
        let loc = Location {
            latitude_deg: 40.427_222,
            longitude_deg: 4.250_556,
            height_km: 0.834_939,
            frame: IAU_EARTH_FRAME.into(),
            terrain_mask: vec![
                TerrainMask {
                    azimuth_deg: 0.0,
                    elevation_mask_deg: 5.0,
                },
                TerrainMask {
                    azimuth_deg: 35.0,
                    elevation_mask_deg: 10.0,
                },
                TerrainMask {
                    azimuth_deg: 270.0,
                    elevation_mask_deg: 3.0,
                },
            ],
            terrain_mask_ignored: false,
        };

        almanac.location_data.push(loc, Some(1), None).unwrap();

        let comm = Event::above_horizon_from_location_id(1, None);

        let comm_arcs = almanac
            .report_event_arcs(
                &lro_state_spec,
                &comm,
                start_epoch,
                start_epoch + Unit::Day * 3,
            )
            .unwrap();
        assert!(comm_arcs.len() > 1);
        for event in &comm_arcs {
            println!("{event}\n{event:?}");
            // Check that this is valid
            for epoch in TimeSeries::inclusive(
                event.start_epoch() - Unit::Minute * 1,
                event.end_epoch() + Unit::Minute * 1,
                Unit::Minute * 0.5,
            ) {
                if let Ok(orbit) = lro_state_spec.evaluate(epoch, &almanac) {
                    let this_eval = comm.eval(orbit, &almanac).unwrap();
                    let is_accessible = this_eval.abs() <= comm.value_precision.abs();

                    if (event.start_epoch()..event.end_epoch()).contains(&epoch) {
                        // We're in the event, check that it is evaluated to be in the event.
                        assert!(is_accessible);
                    } else {
                        assert!(!is_accessible || this_eval < 0.0);
                    }
                }
            }
            println!("\n");
        } */
    }
}
