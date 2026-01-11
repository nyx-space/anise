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
    analysis::report::{ReportScalars, ScalarsRow, ScalarsTable},
    errors::{AlmanacError, MathError, PhysicsError},
    prelude::Orbit,
};
use hifitime::{Epoch, TimeSeries};
use prelude::OrbitalElement;
use rayon::prelude::*;
use snafu::prelude::*;
use std::collections::HashMap;

pub mod dcm_expr;
pub mod elements;
pub mod event;
pub mod event_ops;
pub mod expr;
pub mod report;
pub mod search;
pub mod specs;
pub mod vector_expr;

mod utils;
pub use utils::{adaptive_step_scanner, brent_solver};

pub use dcm_expr::DcmExpr;
use event::Event;
use expr::ScalarExpr;
use specs::{StateSpec, StateSpecTrait};
use vector_expr::VectorExpr;

#[cfg(feature = "python")]
pub mod python;

pub mod prelude {
    pub use super::dcm_expr::DcmExpr;
    pub use super::elements::OrbitalElement;
    pub use super::event::{Condition, Event, EventArc, EventDetails, EventEdge, VisibilityArc};
    pub use super::event_ops::find_arc_intersections;
    pub use super::expr::ScalarExpr;
    pub use super::report::{ReportScalars, ScalarsTable};
    pub use super::specs::{FrameSpec, Plane, StateSpec, StateSpecTrait};
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
    #[snafu(display("computing {expr:?} at {epoch} encountered a physics error {source}"))]
    PhysicsDcmExpr {
        expr: Box<DcmExpr>,
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
    #[snafu(display("all scalars failed for {spec}"))]
    AllScalarsFailed { spec: String },
    #[snafu(display("invalid call to the event evaluator: {err}"))]
    InvalidEventEval { err: String },
    #[snafu(display("{err}"))]
    YetUnimplemented { err: &'static str },
    #[snafu(display("computing AER on {state} encountered an Almanac error {source}"))]
    AlmanacVisibility {
        state: Box<Orbit>,
        #[snafu(source(from(AlmanacError, Box::new)))]
        source: Box<AlmanacError>,
    },
    #[snafu(display("{err}"))]
    GenericAnalysisError { err: String },
}

pub type AnalysisResult<T> = Result<T, AnalysisError>;

impl Almanac {
    /// Report a set of scalar expressions, optionally with aliases, at a fixed time step defined in the TimeSeries.
    pub fn report_scalars<S: StateSpecTrait>(
        &self,
        report: &ReportScalars<S>,
        time_series: TimeSeries,
    ) -> HashMap<Epoch, Result<HashMap<String, AnalysisResult<f64>>, AnalysisError>> {
        time_series
            .par_bridge()
            .map_with((&self, report), |(almanac, report), epoch| {
                match report.state_spec.evaluate(epoch, almanac) {
                    Ok(orbit) => {
                        let mut data = HashMap::new();

                        let ab_corr = report.state_spec.ab_corr();

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

    /// Report a set of scalar expressions, optionally with aliases, at a fixed time step defined in the TimeSeries, as a flat table that can be serialized in columnal form.
    pub fn report_scalars_flat<S: StateSpecTrait>(
        &self,
        report: &ReportScalars<S>,
        time_series: TimeSeries,
    ) -> AnalysisResult<ScalarsTable> {
        let data = self.report_scalars(report, time_series);

        if data.is_empty() {
            return Ok(ScalarsTable {
                headers: Vec::new(),
                rows: Vec::new(),
            });
        }

        let mut headers: Vec<String> = match data.values().find_map(|res| res.as_ref().ok()) {
            Some(map) => map.keys().cloned().collect(),
            None => {
                if data.values().all(|res| res.is_err()) {
                    // All errors, no headers.
                    return Err(AnalysisError::AllScalarsFailed {
                        spec: report.state_spec.to_string(),
                    });
                }

                // This case means data is empty, which we handled.
                return Ok(ScalarsTable {
                    headers: Vec::new(),
                    rows: Vec::new(),
                });
            }
        };
        headers.sort();

        let mut sorted_data: Vec<_> = data.iter().collect();
        sorted_data.sort_by(|(epoch_a, _), (epoch_b, _)| {
            epoch_a
                .partial_cmp(epoch_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut table_rows = Vec::with_capacity(sorted_data.len());
        let num_data_cols = headers.len();

        for (epoch, result) in sorted_data {
            let mut row_values = Vec::with_capacity(num_data_cols);

            match result {
                Ok(inner_map) => {
                    for col_name in &headers {
                        let value = inner_map[col_name].as_ref().copied().unwrap_or(f64::NAN);
                        row_values.push(value);
                    }
                }
                Err(_) => {
                    row_values.resize(num_data_cols, f64::NEG_INFINITY);
                }
            }

            table_rows.push(ScalarsRow {
                epoch: *epoch,
                values: row_values,
            });
        }

        Ok(ScalarsTable {
            headers,
            rows: table_rows,
        })
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
    use crate::ephemerides::ephemeris::Ephemeris;
    use crate::prelude::{Almanac, Frame, Orbit};
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

        almanac = almanac.with_location_data(loc_data);

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
        use crate::analysis::event_ops::find_arc_intersections;

        let lro_frame = Frame::from_ephem_j2000(-85);

        let lro_state_spec = StateSpec {
            target_frame: FrameSpec::Loaded(lro_frame),
            observer_frame: FrameSpec::Loaded(MOON_J2000),
            ab_corr: None,
        };

        // If the Sun has set, then the Sun Angle is less than 90 degrees.
        let sun_has_set_nadir = Event {
            scalar: ScalarExpr::SunAngle { observer_id: -85 },
            condition: Condition::LessThan(90.0),
            epoch_precision: Unit::Second * 0.1,
            ab_corr: None,
        };

        let sun_has_risen_nadir = Event::new(
            ScalarExpr::SunAngle { observer_id: -85 },
            Condition::GreaterThan(90.0),
        );

        let solar_0600 = Event::new(ScalarExpr::LocalSolarTime, Condition::Equals(6.0));

        let max_sun_el = Event {
            scalar: ScalarExpr::SunAngle { observer_id: -85 },
            condition: Condition::Maximum(),
            epoch_precision: Unit::Second * 0.1,
            ab_corr: None,
        };

        let min_sun_el = Event::new(
            ScalarExpr::SunAngle { observer_id: -85 },
            Condition::Minimum(),
        );

        let apolune = Event::apoapsis();
        let perilune = Event::periapsis();

        let eclipse = Event::total_eclipse(MOON_J2000);
        let penumbras = Event::penumbra(MOON_J2000);

        let (start_epoch, mut end_epoch) = almanac.spk_domain(-85).unwrap();
        assert!(
            (end_epoch - Epoch::from_gregorian_utc_at_midnight(2024, 3, 15)).abs()
                < Unit::Second * 1
        );
        // Now set the end epoch to what we had in version 0.8.0 for test consistency.
        end_epoch = Epoch::from_gregorian_str("2024-01-09T00:01:09.184137727 ET").unwrap();

        let start_orbit = almanac
            .transform(lro_frame, MOON_J2000, start_epoch, None)
            .unwrap();
        let period = start_orbit.period().unwrap();

        // End setup

        // For code coverage (and used in debugging), export all of the true anomaly values.
        let true_anom_report = almanac
            .report_scalars_flat(
                &ReportScalars {
                    scalars: vec![
                        (ScalarExpr::Element(OrbitalElement::TrueAnomaly), None),
                        (ScalarExpr::SunAngle { observer_id: -85 }, None),
                        (
                            ScalarExpr::SolarEclipsePercentage {
                                eclipsing_frame: MOON_J2000,
                            },
                            None,
                        ),
                    ],
                    state_spec: lro_state_spec.clone(),
                },
                TimeSeries::inclusive(start_epoch, start_epoch + 3 * period, Unit::Minute * 1),
            )
            .unwrap();
        true_anom_report
            .to_csv("analysis_verif.csv".into())
            .unwrap();

        let apo_events = almanac
            .report_events(&lro_state_spec, &apolune, start_epoch, end_epoch)
            .unwrap();

        println!(
            "Searching for {apolune} yielded {} events (period = {period})",
            apo_events.len()
        );
        println!("\nAPO S-EXPR: {}", apolune.to_s_expr().unwrap());
        let eclipse_s_expr = eclipse.to_s_expr().unwrap();
        let deserd = Event::from_s_expr(&eclipse_s_expr).unwrap();
        assert_eq!(deserd, eclipse);
        println!("\nEclipse S-EXPR: {eclipse_s_expr}");

        let ta_deg_precision = 1e-2;

        for event in &apo_events {
            let ta_deg = event.orbit.ta_deg().unwrap();
            println!("{event} -> true anomaly = {ta_deg:.6} deg");
            assert!((ta_deg - 180.0).abs() < ta_deg_precision);
        }

        let peri_events = almanac
            .report_events(&lro_state_spec, &perilune, start_epoch, end_epoch)
            .unwrap();

        println!(
            "Searching for {perilune} yielded {} events",
            peri_events.len()
        );

        for event in &peri_events {
            let ta_deg = event.orbit.ta_deg().unwrap();
            assert!(ta_deg.abs() < ta_deg_precision || (ta_deg - 360.0).abs() < ta_deg_precision);
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

        let sunset_events = almanac
            .report_event_arcs(&lro_state_spec, &sun_has_set_nadir, start_epoch, end_epoch)
            .unwrap();

        println!(
            "First two sunset of {}:\n\t{}\n\t{}",
            sunset_events.len(),
            sunset_events[0],
            sunset_events[1]
        );
        assert_eq!(sunset_events[1].rise.edge, EventEdge::Falling);
        assert_eq!(sunset_events[1].fall.edge, EventEdge::Rising);
        assert_eq!(sunset_events.len(), 308);

        let sunrise_events = almanac
            .report_event_arcs(
                &lro_state_spec,
                &sun_has_risen_nadir,
                start_epoch,
                end_epoch,
            )
            .unwrap();

        let intersections = find_arc_intersections(vec![sunrise_events, sunset_events]);
        assert!(intersections.is_empty(), "sunrise and sunset events should NOT intersect because one is LessThan the other GreaterThan");

        // Seek the min and max sun angles
        let min_sun_angles = almanac
            .report_events(&lro_state_spec, &min_sun_el, start_epoch, end_epoch)
            .unwrap();
        assert!(!min_sun_angles.is_empty());
        for min_ev in min_sun_angles.iter().take(3) {
            println!("MIN SUN -> {min_ev}");
            assert!(
                (min_ev.value - 50.0).abs() < 1.0,
                "wrong min sun angle found"
            );
        }
        let max_sun_angles = almanac
            .report_events(&lro_state_spec, &max_sun_el, start_epoch, end_epoch)
            .unwrap();
        assert!(!max_sun_angles.is_empty());
        for max_ev in max_sun_angles.iter().take(3) {
            println!("MAX SUN -> {max_ev}");
            assert!(
                (max_ev.value - 129.0).abs() < 1.0,
                "wrong max sun angle found"
            );
        }

        let eclipses = almanac
            .report_event_arcs(
                &lro_state_spec,
                &eclipse,
                start_epoch,
                start_epoch + period * 3,
            )
            .expect("eclipses can be computed...");

        assert_eq!(eclipses.len(), 3, "wrong number of eclipse periods found");

        let eclipse_as_boundary = Event {
            scalar: eclipse.scalar.clone(),
            condition: Condition::Equals(99.0),
            epoch_precision: eclipse.epoch_precision,
            ab_corr: eclipse.ab_corr,
        };

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
                    let this_eclipse = eclipse_as_boundary.eval(orbit, &almanac).unwrap();
                    let in_eclipse = this_eclipse >= 0.0;

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

        let penumbra_events = almanac
            .report_event_arcs(
                &lro_state_spec,
                &penumbras,
                start_epoch,
                start_epoch + period * 3,
            )
            .expect("eclipses can be computed...");

        for event in &penumbra_events {
            println!("{event}");
        }

        let intersect_total_penumra = find_arc_intersections(vec![penumbra_events, eclipses]);

        assert!(
            intersect_total_penumra.is_empty(),
            "penumbras and total eclipses should not intersect"
        );

        let solar6am_events = almanac
            .report_events(&lro_state_spec, &solar_0600, start_epoch, end_epoch)
            .unwrap();
        for (eno, event) in solar6am_events.iter().enumerate() {
            if eno == 0 {
                let solar_time = almanac.local_solar_time(event.orbit, None).unwrap();
                println!("{event} => {solar_time}");
            }
            assert!(event.value.abs() < 1e-2);
        }

        // Test access times
        let mut loc = Location {
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
            terrain_mask_ignored: true,
        };

        let mut loc_data = LocationDataSet::default();
        loc_data.push(loc.clone(), Some(1), None).unwrap();

        // Insert a duplicate of this location but where the terrain mask is applied
        loc.terrain_mask_ignored = false;
        loc_data.push(loc, Some(2), Some("Paris w/ mask")).unwrap();

        almanac = almanac.with_location_data(loc_data);

        let comms_report = almanac
            .report_scalars_flat(
                &ReportScalars {
                    scalars: vec![(
                        ScalarExpr::ElevationFromLocation {
                            location_id: 1,
                            obstructing_body: None,
                        },
                        None,
                    )],
                    state_spec: lro_state_spec.clone(),
                },
                TimeSeries::inclusive(start_epoch, start_epoch + Unit::Day * 3, Unit::Minute * 1),
            )
            .unwrap();
        comms_report.to_csv("comms_verif.csv".into()).unwrap();

        let comm = Event::visible_from_location_id(1, None);
        let mut comm_boundary = comm.clone();
        comm_boundary.condition = Condition::Equals(0.0);

        let comm_arcs = almanac
            .report_event_arcs(
                &lro_state_spec,
                &comm,
                start_epoch,
                start_epoch + Unit::Day * 3,
            )
            .unwrap();
        assert!(comm_arcs.len() == 3);

        // Build another comms report with the mask enabled.
        let comm_mask = Event::visible_from_location_id(2, None);
        let mut comm_boundary_mask = comm_mask.clone();
        comm_boundary_mask.condition = Condition::Equals(0.0);

        let comm_arcs_w_mask = almanac
            .report_event_arcs(
                &lro_state_spec,
                &comm_mask,
                start_epoch,
                start_epoch + Unit::Day * 3,
            )
            .unwrap();
        assert!(comm_arcs_w_mask.len() == 3);
        for pass in &comm_arcs_w_mask {
            println!("w mask: {pass}");
        }

        // Durations no mask
        let exp_durations = [
            Unit::Hour * 8 + Unit::Minute * 57,
            Unit::Hour * 9 + Unit::Minute * 34,
            Unit::Hour * 10 + Unit::Minute * 15,
        ];

        for ((event, duration), event_mask) in
            comm_arcs.iter().zip(exp_durations).zip(comm_arcs_w_mask)
        {
            println!("comms - {event}");
            // Check that this is valid
            for epoch in TimeSeries::inclusive(
                event.start_epoch() - Unit::Minute * 1,
                event.end_epoch() + Unit::Minute * 1,
                Unit::Minute * 0.5,
            ) {
                if let Ok(orbit) = lro_state_spec.evaluate(epoch, &almanac) {
                    let this_eval = comm_boundary.eval(orbit, &almanac).unwrap();
                    // The event is precise to 10 ms, so it may start a few nano degrees below the horizon.
                    let is_accessible = this_eval >= -1e-9;

                    if (event.start_epoch()..event.end_epoch()).contains(&epoch) {
                        // We're in the event, check that it is evaluated to be in the event.
                        assert!(is_accessible, "{this_eval}");
                    } else {
                        assert!(!is_accessible, "{this_eval}");
                    }
                }
            }
            // I only defined the durations to within a minute, so check that these are correct.
            assert!((event.duration() - duration).abs() < Unit::Minute * 1);
            // Check that the event with mask is of shorter duration than without
            assert!(event.duration() > event_mask.duration());
        }

        let vis_arcs = almanac
            .report_visibility_arcs(
                &lro_state_spec,
                2,
                start_epoch,
                start_epoch + Unit::Day * 3,
                Unit::Minute * 5,
                None,
            )
            .unwrap();
        // Duration with mask
        let exp_durations = [
            Unit::Hour * 6 + Unit::Minute * 32,
            Unit::Hour * 7 + Unit::Minute * 16,
            Unit::Hour * 8 + Unit::Minute * 14,
        ];
        for (arc, duration) in vis_arcs.iter().zip(exp_durations) {
            println!("{arc}");
            assert!((arc.duration() - duration).abs() < Unit::Minute * 1);
            // Check the sample rate of 5 min
            let expected_samples = duration.to_unit(Unit::Minute) / 5.0;
            assert!(
                (arc.aer_data.len() as f64 - expected_samples).abs() < 2.0,
                "Expected about {} samples for a duration of {}, but got {}",
                expected_samples.round(),
                duration,
                arc.aer_data.len()
            );
        }

        // Test for a condition that is always met.
        let fpa_always_lt = Event {
            scalar: ScalarExpr::Element(OrbitalElement::FlightPathAngle),
            condition: Condition::LessThan(45.0),
            epoch_precision: Unit::Second * 0.1,
            ab_corr: None,
        };

        let arcs = almanac
            .report_event_arcs(
                &lro_state_spec,
                &fpa_always_lt,
                start_epoch,
                start_epoch + Unit::Day * 1,
            )
            .unwrap();
        assert_eq!(
            arcs.len(),
            1,
            "expected a single arc for a condition that is always met"
        );
        assert_eq!(
            arcs[0].start_epoch(),
            start_epoch,
            "arc should start at the beginning of the search interval"
        );
        assert_eq!(
            arcs[0].end_epoch(),
            start_epoch + Unit::Day * 1,
            "arc should end at the end of the search interval"
        );
    }
    #[rstest]
    fn test_analysis_ric_bsp_diff(mut almanac: Almanac) {
        let eme2k = almanac.frame_info(EME2000).unwrap();
        let epoch = Epoch::from_gregorian_tai_at_midnight(2026, 1, 10);
        // Create ephemeris data from an arbitrary Earth orbit.
        let mut ephem1 = Ephemeris::new("sc1".to_string());
        let mut ephem2 = Ephemeris::new("sc2".to_string());

        let orbit0 =
            Orbit::try_keplerian_altitude(500.0, 1e-3, 32.0, 75.0, 85.0, 95.0, epoch, eme2k)
                .unwrap();
        let orbit1 = orbit0.add_inc_deg(0.5).unwrap();

        // Build the ephems
        let duration = Unit::Day * 1.5;

        let mut min_angle_deg = 180.0_f64;
        let mut min_epoch = epoch;
        let mut min_r_km = 10_000.0_f64;

        for epoch in TimeSeries::exclusive(epoch, epoch + duration, Unit::Minute * 1) {
            let new_orbit0 = orbit0.at_epoch(epoch).unwrap();
            let new_orbit1 = orbit1.at_epoch(epoch).unwrap();
            ephem1.insert_orbit(new_orbit0);
            ephem2.insert_orbit(new_orbit1);

            let new_orbit0_ric0 = new_orbit0
                .dcm3x3_from_ric_to_inertial()
                .unwrap()
                .transpose()
                * new_orbit0.radius_km;

            let new_orbit1_ric0 = new_orbit0
                .dcm3x3_from_ric_to_inertial()
                .unwrap()
                .transpose()
                * new_orbit1.radius_km;
            let angle_deg = new_orbit0_ric0.angle(&new_orbit1_ric0).to_degrees();
            if angle_deg < min_angle_deg {
                min_angle_deg = angle_deg;
                min_epoch = epoch;
            }
            min_r_km = min_r_km.min(new_orbit0.ric_difference(&new_orbit1).unwrap().rmag_km());
        }

        println!("{min_angle_deg} @ {min_epoch}");
        dbg!(min_r_km);

        // Build the BSPs
        almanac = almanac
            .with_spk(ephem1.to_spice_bsp(-10, None).unwrap())
            .with_spk(ephem2.to_spice_bsp(-11, None).unwrap());

        let (start1, end1) = almanac.spk_domain(-10).unwrap();
        let (start2, end2) = almanac.spk_domain(-11).unwrap();

        assert!(((start1 - ephem1.start_epoch().unwrap()).abs()) < Unit::Microsecond * 1);
        assert!(((end1 - ephem1.end_epoch().unwrap()).abs()) < Unit::Microsecond * 1);
        assert!(((start2 - ephem2.start_epoch().unwrap()).abs()) < Unit::Microsecond * 1);
        assert!(((end2 - ephem2.end_epoch().unwrap()).abs()) < Unit::Microsecond * 1);

        let state_spec_ephem1 = StateSpec {
            target_frame: FrameSpec::Loaded(Frame::new(-10, 1)),
            observer_frame: FrameSpec::Loaded(EME2000),
            ab_corr: None,
        };

        // Define the state specs for both trajectories and find when they're close, using an angular separation
        // after rotating both the A and B vectors into the RIC of the A vector.
        let e_ric_angle = Event {
            scalar: ScalarExpr::AngleBetween {
                a: VectorExpr::Rotate {
                    v: Box::new(VectorExpr::Radius(state_spec_ephem1.clone())),
                    dcm: Box::new(DcmExpr::RIC {
                        state: Box::new(state_spec_ephem1.clone()),
                        from: -1,
                        to: 1,
                    }),
                },
                b: VectorExpr::Rotate {
                    v: Box::new(VectorExpr::Radius(StateSpec {
                        target_frame: FrameSpec::Loaded(Frame::new(-11, 1)),
                        observer_frame: FrameSpec::Loaded(EME2000),
                        ab_corr: None,
                    })),
                    dcm: Box::new(DcmExpr::RIC {
                        state: Box::new(state_spec_ephem1.clone()),
                        from: -1,
                        to: 1,
                    }),
                },
            },
            condition: Condition::Maximum(),
            epoch_precision: Unit::Second * 0.1,
            ab_corr: None,
        };

        let events = almanac
            .report_events(&state_spec_ephem1, &e_ric_angle, epoch, end1)
            .unwrap();
        for e in &events {
            assert!(
                (e.value - 0.5).abs() < 1e-3,
                "expect max to be the inclination difference"
            );
            println!("{e}");
        }

        // We can find the minimums of the closest RIC difference with the shortcut.
        let e_ric = Event {
            scalar: ScalarExpr::RicDiff(StateSpec {
                target_frame: FrameSpec::Loaded(Frame::new(-11, 1)),
                observer_frame: FrameSpec::Loaded(EME2000),
                ab_corr: None,
            }),
            condition: Condition::Minimum(),
            epoch_precision: Unit::Second * 0.1,
            ab_corr: None,
        };
        let events = almanac
            .report_events(&state_spec_ephem1, &e_ric, epoch, end1)
            .unwrap();
        for e in &events {
            println!("{e}");
        }

        let dcm = Box::new(DcmExpr::RIC {
            state: Box::new(state_spec_ephem1.clone()),
            from: -1,
            to: 1,
        });

        let e_ric_manual = Event {
            scalar: ScalarExpr::Norm(VectorExpr::Add {
                a: Box::new(VectorExpr::Negate(Box::new(VectorExpr::Rotate {
                    v: Box::new(VectorExpr::Radius(state_spec_ephem1.clone())),
                    dcm: dcm.clone(),
                }))),
                b: Box::new(VectorExpr::Rotate {
                    v: Box::new(VectorExpr::Radius(StateSpec {
                        target_frame: FrameSpec::Loaded(Frame::new(-11, 1)),
                        observer_frame: FrameSpec::Loaded(EME2000),
                        ab_corr: None,
                    })),
                    dcm: dcm.clone(),
                }),
            }),
            condition: Condition::Minimum(),
            epoch_precision: Unit::Second * 0.1,
            ab_corr: None,
        };
        let events_ric2 = almanac
            .report_events(&state_spec_ephem1, &e_ric_manual, epoch, end1)
            .unwrap();

        assert_eq!(events_ric2.len(), events.len());
        for (shortcut, manual) in events.iter().zip(events_ric2.iter()).take(3) {
            assert_eq!(shortcut.orbit.epoch, manual.orbit.epoch);
        }

        // assert_eq!(events_ric2, events);
    }
}
