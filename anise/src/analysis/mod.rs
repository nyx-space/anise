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
use hifitime::{Duration, Epoch, TimeSeries, Unit};
use log::{debug, error, warn};
use prelude::OrbitalElement;
use rayon::prelude::*;
use snafu::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::channel;

pub mod elements;
pub mod event;
pub mod expr;
pub mod report;
pub mod specs;
pub mod vector_expr;

use event::{Event, EventArc, EventDetails, EventEdge};
use expr::ScalarExpr;
use specs::StateSpec;
use vector_expr::VectorExpr;

pub mod prelude {
    pub use super::elements::OrbitalElement;
    pub use super::expr::ScalarExpr;
    pub use super::specs::{FrameSpec, StateSpec};
    pub use super::vector_expr::VectorExpr;
    pub use crate::prelude::Frame;
}

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
                                alias
                                    .clone()
                                    .or(Some(expr.to_string()))
                                    .unwrap()
                                    .to_string(),
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

    /// Find the exact state where the request event happens. The event function is expected to be monotone in the provided interval because we find the event using a Brent solver.
    /// This will only return _one_ event within the provided bracket.
    #[allow(clippy::identity_op)]
    pub fn report_event_once(
        &self,
        state_spec: &StateSpec,
        event: &Event,
        start: Epoch,
        end: Epoch,
    ) -> Result<EventDetails, AnalysisError> {
        let max_iter = 50;

        // Helper lambdas, for f64s only
        let has_converged =
            |xa: f64, xb: f64| (xa - xb).abs() <= event.epoch_precision.to_seconds();
        let arrange = |a: f64, ya: f64, b: f64, yb: f64| {
            if ya.abs() > yb.abs() {
                (a, ya, b, yb)
            } else {
                (b, yb, a, ya)
            }
        };

        let xa_e = start;
        let xb_e = end;

        // Search in seconds (convert to epoch just in time)
        let mut xa = 0.0;
        let mut xb = (xb_e - xa_e).to_seconds();
        // Evaluate the event at both bounds

        let ya_state = state_spec.clone().evaluate(xa_e, self)?;
        let yb_state = state_spec.clone().evaluate(xb_e, self)?;
        let mut ya = event.eval(ya_state, self)?;
        let mut yb = event.eval(yb_state, self)?;

        // Check if we're already at the root
        if ya.abs() <= event.value_precision.abs() {
            debug!(
                "{event} -- found with |{ya}| < {} @ {xa_e}",
                event.value_precision.abs()
            );
            let prev_state = state_spec
                .clone()
                .evaluate(xa_e - event.epoch_precision, self)
                .map_or(None, |s| Some(s));
            let next_state = state_spec
                .clone()
                .evaluate(xa_e + event.epoch_precision, self)
                .map_or(None, |s| Some(s));

            return EventDetails::new(ya_state, ya, event, prev_state, next_state, self);
        } else if yb.abs() <= event.value_precision.abs() {
            debug!(
                "{event} -- found with |{yb}| < {} @ {xb_e}",
                event.value_precision.abs()
            );
            let prev_state = state_spec
                .clone()
                .evaluate(xb_e - event.epoch_precision, self)
                .map_or(None, |s| Some(s));
            let next_state = state_spec
                .clone()
                .evaluate(xb_e + event.epoch_precision, self)
                .map_or(None, |s| Some(s));

            return EventDetails::new(ya_state, ya, event, prev_state, next_state, self);
        }

        // The Brent solver, from the roots crate (sadly could not directly integrate it here)
        // Source: https://docs.rs/roots/0.0.5/src/roots/numerical/brent.rs.html#57-131

        let (mut xc, mut yc, mut xd) = (xa, ya, xa);
        let mut flag = true;

        for _ in 0..max_iter {
            if ya.abs() < event.value_precision.abs() {
                // Can't fail, we got it earlier
                let epoch = xa_e + xa * Unit::Second;
                let state = state_spec.clone().evaluate(epoch, self).unwrap();
                debug!(
                    "{event} -- found with |{ya}| < {} @ {}",
                    event.value_precision.abs(),
                    state.epoch,
                );
                let prev_state = state_spec
                    .clone()
                    .evaluate(epoch - event.epoch_precision, self)
                    .map_or(None, |s| Some(s));
                let next_state = state_spec
                    .clone()
                    .evaluate(epoch + event.epoch_precision, self)
                    .map_or(None, |s| Some(s));

                return EventDetails::new(ya_state, ya, event, prev_state, next_state, self);
            }
            if yb.abs() < event.value_precision.abs() {
                // Can't fail, we got it earlier
                let epoch = xa_e + xb * Unit::Second;
                let state = state_spec.clone().evaluate(epoch, self).unwrap();
                debug!(
                    "{event} -- found with |{yb}| < {} @ {}",
                    event.value_precision.abs(),
                    state.epoch
                );
                let prev_state = state_spec
                    .clone()
                    .evaluate(epoch - event.epoch_precision, self)
                    .map_or(None, |s| Some(s));
                let next_state = state_spec
                    .clone()
                    .evaluate(epoch + event.epoch_precision, self)
                    .map_or(None, |s| Some(s));

                return EventDetails::new(ya_state, ya, event, prev_state, next_state, self);
            }
            if has_converged(xa, xb) {
                // The event isn't in the bracket
                return Err(AnalysisError::EventNotFound {
                    start,
                    end,
                    event: event.clone(),
                });
            }
            let mut s = if (ya - yc).abs() > f64::EPSILON && (yb - yc).abs() > f64::EPSILON {
                xa * yb * yc / ((ya - yb) * (ya - yc))
                    + xb * ya * yc / ((yb - ya) * (yb - yc))
                    + xc * ya * yb / ((yc - ya) * (yc - yb))
            } else {
                xb - yb * (xb - xa) / (yb - ya)
            };
            let cond1 = (s - xb) * (s - (3.0 * xa + xb) / 4.0) > 0.0;
            let cond2 = flag && (s - xb).abs() >= (xb - xc).abs() / 2.0;
            let cond3 = !flag && (s - xb).abs() >= (xc - xd).abs() / 2.0;
            let cond4 = flag && has_converged(xb, xc);
            let cond5 = !flag && has_converged(xc, xd);
            if cond1 || cond2 || cond3 || cond4 || cond5 {
                s = (xa + xb) / 2.0;
                flag = true;
            } else {
                flag = false;
            }

            let next_try = state_spec.clone().evaluate(xa_e + s * Unit::Second, self)?;
            let ys = event.eval(next_try, self)?;

            xd = xc;
            xc = xb;
            yc = yb;
            if ya * ys < 0.0 {
                // Root bracketed between a and s
                let next_try = state_spec
                    .clone()
                    .evaluate(xa_e + xa * Unit::Second, self)?;
                let ya_p = event.eval(next_try, self)?;

                let (_a, _ya, _b, _yb) = arrange(xa, ya_p, s, ys);
                {
                    xa = _a;
                    ya = _ya;
                    xb = _b;
                    yb = _yb;
                }
            } else {
                // Root bracketed between s and b
                let next_try = state_spec
                    .clone()
                    .evaluate(xa_e + xb * Unit::Second, self)?;
                let yb_p = event.eval(next_try, self)?;

                let (_a, _ya, _b, _yb) = arrange(s, ys, xb, yb_p);
                {
                    xa = _a;
                    ya = _ya;
                    xb = _b;
                    yb = _yb;
                }
            }
        }
        error!("Brent solver failed after {max_iter} iterations");
        Err(AnalysisError::EventNotFound {
            start,
            end,
            event: event.clone(),
        })
    }

    /// Report all of the states and event details where the provided event occurs.
    ///
    /// # Limitations
    /// This method uses a Brent solver. If the function that defines the event is not unimodal, the event finder may not converge correctly.
    ///
    /// # Heuristic detail
    /// The initial search step is 1% of the duration of the trajectory duration, if the heuristic is set to None.
    /// For example, if the trajectory is 100 days long, then we split the trajectory into 100 chunks of 1 day and see whether
    /// the event is in there. If the event happens twice or more times within 1% of the trajectory duration, only the _one_ of
    /// such events will be found.
    ///
    /// If this heuristic fails to find any such events, then `find_minmax` is called on the event with a time precision of `Unit::Second`.
    /// Then we search only within the min and max bounds of the provided event.
    #[allow(clippy::identity_op)]
    pub fn report_events(
        &self,
        state_spec: &StateSpec,
        event: &Event,
        start_epoch: Epoch,
        end_epoch: Epoch,
        heuristic: Option<Duration>,
    ) -> Result<Vec<EventDetails>, AnalysisError> {
        if start_epoch == end_epoch {
            return Err(AnalysisError::EventNotFound {
                start: start_epoch,
                end: end_epoch,
                event: event.clone(),
            });
        }
        let heuristic = heuristic.unwrap_or((end_epoch - start_epoch) / 100);
        debug!("searching for {event} with initial heuristic of {heuristic}");

        let (sender, receiver) = channel();

        let epochs: Vec<Epoch> = TimeSeries::inclusive(start_epoch, end_epoch, heuristic).collect();
        epochs.into_par_iter().for_each_with(sender, |s, epoch| {
            if let Ok(event_state) =
                self.report_event_once(state_spec, event, epoch, epoch + heuristic)
            {
                s.send(event_state).unwrap()
            };
        });

        let mut states: Vec<_> = receiver.iter().collect();

        if states.is_empty() {
            warn!("Heuristic failed to find any {event} event, using slower approach");
            // Crap, we didn't find the event.
            // Let's find the min and max of this event throughout the trajectory, and search around there.
            match self.report_event_minmax(state_spec, event, Unit::Second, start_epoch, end_epoch)
            {
                Ok((min_event, max_event)) => {
                    let lower_min_epoch = if min_event.epoch - 1 * Unit::Millisecond < start_epoch {
                        start_epoch
                    } else {
                        min_event.epoch - 1 * Unit::Millisecond
                    };

                    let lower_max_epoch = if min_event.epoch + 1 * Unit::Millisecond > end_epoch {
                        end_epoch
                    } else {
                        min_event.epoch + 1 * Unit::Millisecond
                    };

                    let upper_min_epoch = if max_event.epoch - 1 * Unit::Millisecond < start_epoch {
                        start_epoch
                    } else {
                        max_event.epoch - 1 * Unit::Millisecond
                    };

                    let upper_max_epoch = if max_event.epoch + 1 * Unit::Millisecond > end_epoch {
                        end_epoch
                    } else {
                        max_event.epoch + 1 * Unit::Millisecond
                    };

                    // Search around the min event
                    if let Ok(event_state) =
                        self.report_event_once(state_spec, event, lower_min_epoch, lower_max_epoch)
                    {
                        states.push(event_state);
                    };

                    // Search around the max event
                    if let Ok(event_state) =
                        self.report_event_once(state_spec, event, upper_min_epoch, upper_max_epoch)
                    {
                        states.push(event_state);
                    };

                    // If there still isn't any match, report that the event was not found
                    if states.is_empty() {
                        return Err(AnalysisError::EventNotFound {
                            start: start_epoch,
                            end: end_epoch,
                            event: event.clone(),
                        });
                    }
                }
                Err(_) => {
                    return Err(AnalysisError::EventNotFound {
                        start: start_epoch,
                        end: end_epoch,
                        event: event.clone(),
                    });
                }
            };
        }
        // Remove duplicates and reorder
        states.sort_by(|s1, s2| s1.state.epoch.partial_cmp(&s2.state.epoch).unwrap());
        states.dedup();

        match states.len() {
            0 => debug!("Event {event} not found"),
            1 => debug!("Event {event} found once on {}", states[0].state.epoch),
            _ => {
                debug!(
                    "Event {event} found {} times from {} until {}",
                    states.len(),
                    states.first().unwrap().state.epoch,
                    states.last().unwrap().state.epoch
                )
            }
        };

        Ok(states)
    }

    /// Find the minimum and maximum of the provided event through the trajectory
    #[allow(clippy::identity_op)]
    pub fn report_event_minmax(
        &self,
        state_spec: &StateSpec,
        event: &Event,
        precision: Unit,
        start: Epoch,
        end: Epoch,
    ) -> Result<(Orbit, Orbit), AnalysisError> {
        let step: Duration = 1 * precision;
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        let mut min_state = state_spec.evaluate(start, self)?;
        let mut max_state = state_spec.evaluate(end, self)?;

        let (sender, receiver) = channel();

        let epochs: Vec<Epoch> = TimeSeries::inclusive(start, end, step).collect();

        epochs.into_par_iter().for_each_with(sender, |s, epoch| {
            // The `at` call will work because we only query within the start and end times of the trajectory
            let state = state_spec.evaluate(epoch, self).unwrap();
            if let Ok(this_eval) = event.eval(state, self) {
                s.send((this_eval, state)).unwrap();
            }
        });

        let evald_states: Vec<_> = receiver.iter().collect();
        for (this_eval, state) in evald_states {
            if this_eval < min_val {
                min_val = this_eval;
                min_state = state;
            }
            if this_eval > max_val {
                max_val = this_eval;
                max_state = state;
            }
        }

        Ok((min_state, max_state))
    }

    /// Identifies and pairs rising and falling edge events.
    ///
    /// This function processes a sequence of events in a trajectory and pairs each rising edge event with its subsequent falling edge event to form arcs.
    /// Each arc represents a complete cycle of an event rising above and then falling below a specified threshold.
    /// Use this to analyze a trajectory's behavior when understanding the complete cycle of an event (from rising to falling) is essential, e.g. ground station passes.
    ///
    /// # Logic
    /// - Sorts the events by their epoch to ensure chronological processing.
    /// - Iterates through the sorted events, identifying transitions from falling to rising edges and vice versa.
    /// - Pairs a rising edge with the subsequent falling edge to form an arc.
    /// - Handles edge cases where the trajectory starts or ends with a rising or falling edge.
    /// - Prints debug information for each event and arc.
    ///
    /// ## Note
    /// If no zero crossing happens in the trajectory, i.e. there are no "event is true" _and_ "event is false",
    /// then this function checks whether the event is true at the start and end of the trajectory. If so, it means
    /// that there is a single arc that spans the whole trajectory.
    ///
    pub fn report_event_arcs<E>(
        &self,
        state_spec: &StateSpec,
        event: &Event,
        start: Epoch,
        end: Epoch,
        heuristic: Option<Duration>,
    ) -> Result<Vec<EventArc>, AnalysisError> {
        let mut events = match self.report_events(state_spec, event, start, end, heuristic) {
            Ok(events) => events,
            Err(_) => {
                // We haven't found the start or end of an arc, i.e. no zero crossing on the event.
                // However, if the trajectory start and end are above the event value, then we found an arc.
                let start_orbit = state_spec.evaluate(start, self)?;
                let end_orbit = state_spec.evaluate(end, self)?;
                let first_eval = event.eval(start_orbit, self)?;
                let last_eval = event.eval(end_orbit, self)?;
                if first_eval > 0.0 && last_eval > 0.0 {
                    // No event crossing found, but from the start until the end of the trajectory, we're in the same arc
                    // because the evaluation of the event is above the zero crossing.
                    // Hence, there's a single arc, and it's from start until the end of the trajectory.
                    vec![
                        EventDetails::new(
                            start_orbit,
                            first_eval,
                            event,
                            None,
                            state_spec
                                .evaluate(start + event.epoch_precision, self)
                                .map(|s| Some(s))?,
                            self,
                        )?,
                        EventDetails::new(
                            end_orbit,
                            last_eval,
                            event,
                            state_spec
                                .evaluate(start + event.epoch_precision, self)
                                .map(|s| Some(s))?,
                            None,
                            self,
                        )?,
                    ]
                } else {
                    return Err(AnalysisError::EventNotFound {
                        start,
                        end,
                        event: event.clone(),
                    });
                }
            }
        };
        events.sort_by_key(|event| event.state.epoch);

        // Now, let's pair the events.
        let mut arcs = Vec::new();

        if events.is_empty() {
            return Ok(arcs);
        }

        // If the first event isn't a rising edge, then we mark the start of the trajectory as a rising edge
        let mut prev_rise = if events[0].edge != EventEdge::Rising {
            let first_orbit = state_spec.evaluate(start, self)?;
            let next_orbit = state_spec.evaluate(start + event.epoch_precision, self)?;
            let value = event.eval(first_orbit, self)?;
            Some(EventDetails::new(
                first_orbit,
                value,
                event,
                None,
                Some(next_orbit),
                self,
            )?)
        } else {
            Some(events[0].clone())
        };

        let mut prev_fall = if events[0].edge == EventEdge::Falling {
            Some(events[0].clone())
        } else {
            None
        };

        for event in events {
            if event.edge == EventEdge::Rising {
                if prev_rise.is_none() && prev_fall.is_none() {
                    // This is a new rising edge
                    prev_rise = Some(event.clone());
                } else if prev_fall.is_some() {
                    // We've found a transition from a fall to a rise, so we can close this arc out.
                    if prev_rise.is_some() {
                        let arc = EventArc {
                            rise: prev_rise.clone().unwrap(),
                            fall: prev_fall.clone().unwrap(),
                        };
                        arcs.push(arc);
                    } else {
                        let arc = EventArc {
                            rise: event.clone(),
                            fall: prev_fall.clone().unwrap(),
                        };
                        arcs.push(arc);
                    }
                    prev_fall = None;
                    // We have a new rising edge since this is how we ended up here.
                    prev_rise = Some(event.clone());
                }
            } else if event.edge == EventEdge::Falling {
                prev_fall = Some(event.clone());
            }
        }

        // Add the final pass
        if prev_rise.is_some() {
            if prev_fall.is_some() {
                let arc = EventArc {
                    rise: prev_rise.clone().unwrap(),
                    fall: prev_fall.clone().unwrap(),
                };
                arcs.push(arc);
            } else {
                // Use the last trajectory as the end of the arc
                let penult_orbit = state_spec.evaluate(end - event.epoch_precision, self)?;
                let last_orbit = state_spec.evaluate(end, self)?;
                let value = event.eval(last_orbit, self)?;
                let fall =
                    EventDetails::new(last_orbit, value, event, Some(penult_orbit), None, self)?;
                let arc = EventArc {
                    rise: prev_rise.clone().unwrap(),
                    fall,
                };
                arcs.push(arc);
            }
        }

        Ok(arcs)
    }
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
        event: Event,
    },
}

pub type AnalysisResult<T> = Result<T, AnalysisError>;

#[cfg(test)]
mod ut_analysis {

    use crate::analysis::prelude::*;
    use crate::analysis::report::ReportScalars;
    use crate::analysis::specs::{OrthogonalFrame, Plane};
    use crate::astro::{Aberration, Location, TerrainMask};
    use crate::constants::frames::{EME2000, IAU_EARTH_FRAME, MOON_J2000, SUN_J2000, VENUS_J2000};
    use crate::prelude::Almanac;
    use crate::structure::LocationDataSet;
    use hifitime::{Epoch, TimeSeries, Unit};
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
            observer_frame,
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
            ScalarExpr::VectorX(proj.clone()),
            ScalarExpr::VectorY(proj.clone()),
            ScalarExpr::VectorZ(proj.clone()),
        ];

        // Demo of an S-Expression export
        let sexpr_str = serde_lexpr::to_value(&scalars).unwrap();
        let proj = scalars.last().unwrap();
        let proj_s = proj.to_s_expr();
        let proj_reload = ScalarExpr::from_s_expr(&proj_s).unwrap();
        assert_eq!(&proj_reload, proj);
        println!("{sexpr_str}\n\nPROJ ONLY\n{proj_s}\n");

        let cnt = scalars.len();

        let mut scalars_with_aliases = scalars.map(|s| (s, None));
        // Set an alias for the last three.
        scalars_with_aliases[cnt - 3].1 = Some("proj VNC X".to_string());
        scalars_with_aliases[cnt - 2].1 = Some("proj VNC Y".to_string());
        scalars_with_aliases[cnt - 1].1 = Some("proj VNC Z".to_string());

        // Build the report, ensure we can serialize it and deserialize it.
        let report = ReportScalars {
            scalars: scalars_with_aliases.to_vec(),
            state_spec: state,
        };

        let report_s_expr = report.to_s_expr();

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

        for (k, v) in last_row.iter() {
            if k.contains("proj") {
                // Check that we have correctly defined the projections onto an othogonal frame
                assert!(v.as_ref().unwrap().abs() <= 1.0);
            }
        }
    }
}
