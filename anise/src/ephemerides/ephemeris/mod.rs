/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */

use super::{EphemerisError, EphemerisPhysicsSnafu, OEMTimeParsingSnafu};
use crate::ephemerides::EphemInterpolationSnafu;
use crate::errors::{AlmanacError, AlmanacPhysicsSnafu, OrientationSnafu};
use crate::frames::Frame;
use crate::math::Vector6;
use crate::math::interpolation::{InterpolationError, hermite_eval, lagrange_eval};
use crate::naif::daf::data_types::DataType;
use crate::prelude::{Almanac, Orbit};
use core::fmt;
use covariance::interpolate_covar_log_euclidean;
use hifitime::{Epoch, TimeSeries};
use snafu::ResultExt;
use std::collections::BTreeMap;

#[cfg(feature = "python")]
use pyo3::prelude::*;

mod almanac;
mod covariance;
mod oem;
#[cfg(feature = "python")]
mod python;
mod record;
mod spk;
mod stk;

pub use covariance::{Covariance, LocalFrame};
pub use record::EphemerisRecord;

/// Initializes a new Ephemeris from the list of Orbit instances and a given object ID.
///
/// In Python if you need to build an ephemeris with covariance, initialize with an empty list of
/// orbit instances and then insert each EphemerisRecord with covariance.
///
/// :type orbit_list: list
/// :type object_id: str
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "python", pyclass(from_py_object))]
#[cfg_attr(feature = "python", pyo3(module = "anise.astro"))]
pub struct Ephemeris {
    pub object_id: String,
    segments: Vec<EphemerisSegment>,
}

/// Block-local interpolation metadata and raw state ownership for one CCSDS OEM block.
#[derive(Clone, Debug, PartialEq)]
struct EphemerisSegment {
    interpolation: DataType,
    degree: usize,
    useable_start: Option<Epoch>,
    useable_end: Option<Epoch>,
    state_data: BTreeMap<Epoch, EphemerisRecord>,
}

#[derive(Clone, Copy)]
struct EphemerisSegmentView<'a> {
    interpolation: DataType,
    degree: usize,
    total_start: Epoch,
    total_end: Epoch,
    useable_start: Epoch,
    useable_end: Epoch,
    state_data: &'a BTreeMap<Epoch, EphemerisRecord>,
}

impl EphemerisSegment {
    fn new(interpolation: DataType, degree: usize) -> Self {
        Self {
            interpolation,
            degree,
            useable_start: None,
            useable_end: None,
            state_data: BTreeMap::new(),
        }
    }

    fn from_state_data(
        interpolation: DataType,
        degree: usize,
        state_data: BTreeMap<Epoch, EphemerisRecord>,
    ) -> Self {
        Self {
            interpolation,
            degree,
            useable_start: None,
            useable_end: None,
            state_data,
        }
    }

    fn view(&self) -> Option<EphemerisSegmentView<'_>> {
        let (&raw_start, _) = self.state_data.first_key_value()?;
        let (&raw_end, _) = self.state_data.last_key_value()?;
        let useable_start = self.useable_start.unwrap_or(raw_start);
        let useable_end = self.useable_end.unwrap_or(raw_end);
        Some(EphemerisSegmentView {
            interpolation: self.interpolation,
            degree: self.degree,
            total_start: raw_start.min(useable_start),
            total_end: raw_end.max(useable_end),
            useable_start,
            useable_end,
            state_data: &self.state_data,
        })
    }
}

impl EphemerisSegmentView<'_> {
    fn nearest_before(&self, epoch: Epoch, almanac: &Almanac) -> Option<EphemerisRecord> {
        self.state_data.range(..=epoch).next_back().map(|entry| {
            let mut record = *entry.1;
            if let Ok(frame) = almanac.frame_info(record.orbit.frame) {
                record.orbit.frame = frame;
            }
            record
        })
    }

    fn nearest_after(&self, epoch: Epoch, almanac: &Almanac) -> Option<EphemerisRecord> {
        self.state_data.range(epoch..).next().map(|entry| {
            let mut record = *entry.1;
            if let Ok(frame) = almanac.frame_info(record.orbit.frame) {
                record.orbit.frame = frame;
            }
            record
        })
    }

    fn covar_at(
        &self,
        epoch: Epoch,
        local_frame: LocalFrame,
        almanac: &Almanac,
    ) -> Result<Option<Covariance>, EphemerisError> {
        if !self
            .state_data
            .values()
            .any(|record| record.covar.is_some())
        {
            return Ok(None);
        }

        let Some(prev_record) = self.nearest_before(epoch, almanac) else {
            return Ok(None);
        };
        let Some(next_record) = self.nearest_after(epoch, almanac) else {
            return Ok(None);
        };

        if prev_record.covar.is_none() || next_record.covar.is_none() {
            return Ok(None);
        }

        let prev_covar = prev_record
            .covar_in_frame(local_frame)
            .context(EphemerisPhysicsSnafu {
                action: "rotating covariance",
            })?
            .expect("prev_record covariance is Some, checked above");
        let next_covar = next_record
            .covar_in_frame(local_frame)
            .context(EphemerisPhysicsSnafu {
                action: "rotating covariance",
            })?
            .expect("next_record covariance is Some, checked above");

        let t0 = prev_record.orbit.epoch;
        let t1 = next_record.orbit.epoch;
        let total_dt = (t1 - t0).to_seconds();
        if total_dt.abs() < 1e-9 {
            return Ok(Some(prev_covar));
        }

        let alpha = (epoch - t0).to_seconds() / total_dt;
        Ok(
            interpolate_covar_log_euclidean(prev_covar.matrix, next_covar.matrix, alpha).map(
                |matrix| Covariance {
                    matrix,
                    local_frame,
                },
            ),
        )
    }

    fn orbit_at(&self, epoch: Epoch, almanac: &Almanac) -> Result<Orbit, EphemerisError> {
        let n = self.degree;
        let prev_states: Vec<EphemerisRecord> = {
            let mut states = self
                .state_data
                .range(..epoch)
                .rev()
                .take(n)
                .map(|entry| *entry.1)
                .collect::<Vec<_>>();
            states.reverse();
            states
        };
        let next_states = self
            .state_data
            .range(epoch..)
            .take(n)
            .map(|entry| *entry.1)
            .collect::<Vec<_>>();
        let states = prev_states
            .into_iter()
            .chain(next_states)
            .collect::<Vec<_>>();
        Self::interpolate_orbit_records(self.interpolation, &states, epoch, almanac)
    }

    fn orbit_at_with_window(
        &self,
        epoch: Epoch,
        window_len: usize,
        almanac: &Almanac,
    ) -> Result<Orbit, EphemerisError> {
        let raw_start = *self
            .state_data
            .first_key_value()
            .expect("empty segment is never constructed")
            .0;
        let states = if epoch < raw_start {
            self.state_data
                .values()
                .take(window_len)
                .copied()
                .collect::<Vec<_>>()
        } else {
            let mut states = self
                .state_data
                .values()
                .rev()
                .take(window_len)
                .copied()
                .collect::<Vec<_>>();
            states.reverse();
            states
        };
        Self::interpolate_orbit_records(self.interpolation, &states, epoch, almanac)
    }

    fn interpolate_orbit_records(
        interpolation: DataType,
        states: &[EphemerisRecord],
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Orbit, EphemerisError> {
        let template = states.first().ok_or(EphemerisError::EphemInterpolation {
            source: InterpolationError::EmptyInterpolationData {},
        })?;
        let xs = states
            .iter()
            .map(|record| record.orbit.epoch.to_tdb_seconds())
            .collect::<Vec<_>>();
        let mut orbit_data = Vector6::zeros();

        match interpolation {
            DataType::Type9LagrangeUnequalStep => {
                for i in 0..6 {
                    let ys = states
                        .iter()
                        .map(|record| record.orbit.to_cartesian_pos_vel()[i])
                        .collect::<Vec<_>>();
                    let (value, _) = lagrange_eval(&xs, &ys, epoch.to_tdb_seconds())
                        .context(EphemInterpolationSnafu)?;
                    orbit_data[i] = value;
                }
            }
            DataType::Type13HermiteUnequalStep | DataType::Type12HermiteEqualStep => {
                for i in 0..3 {
                    let ys = states
                        .iter()
                        .map(|record| record.orbit.to_cartesian_pos_vel()[i])
                        .collect::<Vec<_>>();
                    let ydots = states
                        .iter()
                        .map(|record| record.orbit.to_cartesian_pos_vel()[i + 3])
                        .collect::<Vec<_>>();
                    let (value, derivative) =
                        hermite_eval(&xs, &ys, &ydots, epoch.to_tdb_seconds())
                            .context(EphemInterpolationSnafu)?;
                    orbit_data[i] = value;
                    orbit_data[i + 3] = derivative;
                }
            }
            _ => unreachable!(),
        }

        let mut orbit = template.orbit.with_cartesian_pos_vel(orbit_data);
        orbit.epoch = epoch;
        if let Ok(frame) = almanac.frame_info(orbit.frame) {
            orbit.frame = frame;
        }
        Ok(orbit)
    }

    fn at(&self, epoch: Epoch, almanac: &Almanac) -> Result<EphemerisRecord, EphemerisError> {
        Ok(EphemerisRecord {
            orbit: self.orbit_at(epoch, almanac)?,
            covar: self.covar_at(epoch, LocalFrame::Inertial, almanac)?,
        })
    }
}

impl Ephemeris {
    pub fn new(object_id: String) -> Self {
        Self {
            object_id,
            segments: vec![EphemerisSegment::new(DataType::Type13HermiteUnequalStep, 7)],
        }
    }

    fn from_state_data(
        object_id: String,
        interpolation: DataType,
        degree: usize,
        state_data: BTreeMap<Epoch, EphemerisRecord>,
    ) -> Self {
        Self {
            object_id,
            segments: vec![EphemerisSegment::from_state_data(
                interpolation,
                degree,
                state_data,
            )],
        }
    }

    /// Returns the interpolation method when it is uniform across all segments.
    pub fn interpolation(&self) -> Result<DataType, EphemerisError> {
        let first = self
            .segments
            .first()
            .ok_or(EphemerisError::EphemInterpolation {
                source: InterpolationError::EmptyInterpolationData {},
            })?
            .interpolation;
        if self
            .segments
            .iter()
            .all(|segment| segment.interpolation == first)
        {
            Ok(first)
        } else {
            Err(EphemerisError::MixedInterpolation)
        }
    }

    /// Returns the interpolation degree when it is uniform across all segments.
    pub fn degree(&self) -> Result<usize, EphemerisError> {
        let first = self
            .segments
            .first()
            .ok_or(EphemerisError::EphemInterpolation {
                source: InterpolationError::EmptyInterpolationData {},
            })?
            .degree;
        if self.segments.iter().all(|segment| segment.degree == first) {
            Ok(first)
        } else {
            Err(EphemerisError::MixedInterpolationDegree)
        }
    }

    /// Returns the interpolation method for the segment valid at `epoch`.
    pub fn interpolation_at(&self, epoch: Epoch) -> Result<DataType, EphemerisError> {
        Ok(self.segment_at_or_error(epoch)?.interpolation)
    }

    /// Returns the interpolation degree for the segment valid at `epoch`.
    pub fn degree_at(&self, epoch: Epoch) -> Result<usize, EphemerisError> {
        Ok(self.segment_at_or_error(epoch)?.degree)
    }

    /// Applies one interpolation method to every segment.
    pub fn set_interpolation(&mut self, interpolation: DataType) {
        for segment in &mut self.segments {
            segment.interpolation = interpolation;
        }
    }

    /// Applies one strictly positive interpolation degree to every segment.
    pub fn set_degree(&mut self, degree: usize) -> Result<(), EphemerisError> {
        if degree == 0 {
            return Err(EphemerisError::InvalidInterpolationDegree { degree });
        }
        for segment in &mut self.segments {
            segment.degree = degree;
        }
        Ok(())
    }

    fn segment_views(&self) -> Vec<EphemerisSegmentView<'_>> {
        self.segments
            .iter()
            .filter_map(EphemerisSegment::view)
            .collect()
    }

    fn segment_index_at(&self, epoch: Epoch) -> Option<usize> {
        if self.segments.len() == 1 && self.segments[0].view().is_some() {
            return Some(0);
        }
        self.segments.iter().rposition(|segment| {
            segment
                .view()
                .is_some_and(|view| (view.useable_start..=view.useable_end).contains(&epoch))
        })
    }

    fn segment_at(&self, epoch: Epoch) -> Option<EphemerisSegmentView<'_>> {
        self.segments.get(self.segment_index_at(epoch)?)?.view()
    }

    fn interpolation_domain(&self) -> Result<(Epoch, Epoch), EphemerisError> {
        let views = self.segment_views();
        let first = views.first().ok_or(EphemerisError::EphemInterpolation {
            source: InterpolationError::EmptyInterpolationData {},
        })?;
        if views.len() == 1 {
            return Ok((first.total_start, first.total_end));
        }
        let last = views.last().expect("views has a first item");
        Ok((first.useable_start, last.useable_end))
    }

    fn segment_at_or_error(
        &self,
        epoch: Epoch,
    ) -> Result<EphemerisSegmentView<'_>, EphemerisError> {
        let (start, end) = self.interpolation_domain()?;
        if (start..=end).contains(&epoch)
            && let Some(segment) = self.segment_at(epoch)
        {
            return Ok(segment);
        }
        Err(EphemerisError::EphemInterpolation {
            source: InterpolationError::NoInterpolationData {
                req: epoch,
                start,
                end,
            },
        })
    }

    fn insert_preserving_segments(&mut self, record: EphemerisRecord) {
        let epoch = record.orbit.epoch;
        let segment_idx = self
            .segment_index_at(epoch)
            .or_else(|| {
                self.segments.iter().rposition(|segment| {
                    segment
                        .view()
                        .is_some_and(|view| (view.total_start..=view.total_end).contains(&epoch))
                })
            })
            .or_else(|| {
                (self.segments.len() == 1
                    && self.segments[0].useable_start.is_none()
                    && self.segments[0].useable_end.is_none())
                .then_some(0)
            });

        if let Some(segment_idx) = segment_idx {
            self.segments[segment_idx].state_data.insert(epoch, record);
            return;
        }

        if self.segments.len() == 1 && self.segments[0].state_data.is_empty() {
            self.segments[0].state_data.insert(epoch, record);
            return;
        }

        let (interpolation, degree) = self
            .segments
            .last()
            .map(|segment| (segment.interpolation, segment.degree))
            .unwrap_or((DataType::Type13HermiteUnequalStep, 7));
        let mut segment = EphemerisSegment::new(interpolation, degree);
        segment.state_data.insert(epoch, record);
        self.segments.push(segment);
        self.segments
            .sort_by_key(|segment| segment.state_data.first_key_value().map(|entry| *entry.0));
    }
}

#[cfg_attr(feature = "python", pymethods)]
impl Ephemeris {
    /// Returns the time domain of this ephemeris.
    ///
    /// :rtype: tuple
    pub fn domain(&self) -> Result<(Epoch, Epoch), EphemerisError> {
        let start = self
            .segments
            .iter()
            .filter_map(|segment| segment.state_data.first_key_value().map(|entry| *entry.0))
            .min();
        let end = self
            .segments
            .iter()
            .filter_map(|segment| segment.state_data.last_key_value().map(|entry| *entry.0))
            .max();
        match (start, end) {
            (Some(start), Some(end)) => Ok((start, end)),
            _ => Err(EphemerisError::EphemInterpolation {
                source: InterpolationError::EmptyInterpolationData {},
            }),
        }
    }

    /// :rtype: Epoch
    pub fn start_epoch(&self) -> Result<Epoch, EphemerisError> {
        Ok(self.domain()?.0)
    }

    /// :rtype: Epoch
    pub fn end_epoch(&self) -> Result<Epoch, EphemerisError> {
        Ok(self.domain()?.1)
    }

    /// :rtype: str
    pub fn object_id(&self) -> &str {
        &self.object_id
    }

    /// Returns the number of records across all segments.
    pub fn len(&self) -> usize {
        self.segments
            .iter()
            .map(|segment| segment.state_data.len())
            .sum()
    }

    /// Returns true when no segment contains a record.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if all of the data in this ephemeris includes covariance.
    ///
    /// This is a helper function which isn't used in other functions.
    ///
    /// :rtype: bool
    pub fn includes_covariance(&self) -> bool {
        !self.is_empty()
            && self.segments.iter().all(|segment| {
                segment
                    .state_data
                    .values()
                    .all(|entry| entry.covar.is_some())
            })
    }

    /// Inserts a new ephemeris entry to this ephemeris (it is automatically sorted chronologically).
    /// :type record: EphemerisRecord
    /// :rtype: None
    pub fn insert(&mut self, record: EphemerisRecord) {
        self.insert_preserving_segments(record);
    }

    /// Inserts a new orbit (without covariance) to this ephemeris (it is automatically sorted chronologically).
    /// :type orbit: Orbit
    /// :rtype: None
    pub fn insert_orbit(&mut self, orbit: Orbit) {
        self.insert(EphemerisRecord { orbit, covar: None });
    }

    /// Returns the nearest entry before the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: EphemerisRecord
    pub fn nearest_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<EphemerisRecord, EphemerisError> {
        self.segment_at_or_error(epoch)?
            .nearest_before(epoch, almanac)
            .ok_or(EphemerisError::EphemInterpolation {
                source: InterpolationError::EmptyInterpolationData {},
            })
    }

    /// Returns the nearest entry after the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: EphemerisRecord
    pub fn nearest_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<EphemerisRecord, EphemerisError> {
        self.segment_at_or_error(epoch)?
            .nearest_after(epoch, almanac)
            .ok_or(EphemerisError::EphemInterpolation {
                source: InterpolationError::EmptyInterpolationData {},
            })
    }

    /// Returns the nearest orbit before the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: Orbit
    pub fn nearest_orbit_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Orbit, EphemerisError> {
        Ok(self.nearest_before(epoch, almanac)?.orbit)
    }

    /// Returns the nearest orbit after the provided time
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: Orbit
    pub fn nearest_orbit_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Orbit, EphemerisError> {
        Ok(self.nearest_after(epoch, almanac)?.orbit)
    }

    /// Returns the nearest covariance before the provided epoch as a tuple (Epoch, Covariance)
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: tuple
    pub fn nearest_covar_before(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Option<(Epoch, Covariance)>, EphemerisError> {
        let record = self.nearest_before(epoch, almanac)?;
        Ok(record.covar.map(|c| (record.orbit.epoch, c)))
    }

    /// Returns the nearest covariance after the provided epoch as a tuple (Epoch, Covariance)
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: tuple
    pub fn nearest_covar_after(
        &self,
        epoch: Epoch,
        almanac: &Almanac,
    ) -> Result<Option<(Epoch, Covariance)>, EphemerisError> {
        let record = self.nearest_after(epoch, almanac)?;
        Ok(record.covar.map(|c| (record.orbit.epoch, c)))
    }

    /// Interpolates the ephemeris state and covariance at the provided epoch.
    ///
    /// # Orbit Interpolation
    /// The orbital state is interpolated using high-fidelity numeric methods consistent
    /// with SPICE standards:
    /// * **Type 9 (Lagrange):** Uses an Nth-order Lagrange polynomial interpolation on
    ///   unequal time steps. It interpolates each of the 6 state components (position
    ///   and velocity) independently.
    /// * **Type 13 (Hermite):** Uses an Nth-order Hermite interpolation. This method
    ///   explicitly uses the velocity data (derivatives) to constrain the interpolation
    ///   of the position, ensuring that the resulting position curve is smooth and
    ///   dynamically consistent with the velocity.
    ///
    /// # Covariance Interpolation (Log-Euclidean)
    /// If covariance data is available, this method performs **Log-Euclidean Riemannian
    /// Interpolation**. Unlike standard linear element-wise interpolation, this approach
    /// respects the geometric manifold of Symmetric Positive Definite (SPD) matrices.
    ///
    /// This guarantees that:
    /// 1. **Positive Definiteness:** The interpolated covariance matrix is always mathematically
    ///    valid (all eigenvalues are strictly positive), preventing numerical crashes in downstream filters.
    /// 2. **Volume Preservation:** It prevents the artificial "swelling" (determinant increase)
    ///    of uncertainty that occurs when linearly interpolating between two valid matrices.
    ///    The interpolation follows the "geodesic" (shortest path) on the curved surface of
    ///    covariance matrices.
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: EphemerisRecord
    pub fn at(&self, epoch: Epoch, almanac: &Almanac) -> Result<EphemerisRecord, EphemerisError> {
        self.segment_at_or_error(epoch)?.at(epoch, almanac)
    }

    /// Interpolate the ephemeris at the provided epoch, returning only the orbit.
    ///
    /// :type epoch: Epoch
    /// :type almanac: Almanac
    /// :rtype: Orbit
    pub fn orbit_at(&self, epoch: Epoch, almanac: &Almanac) -> Result<Orbit, EphemerisError> {
        Ok(self.at(epoch, almanac)?.orbit)
    }

    /// Interpolate the ephemeris covariance at the provided epoch.
    ///
    /// This method implements a "Rotate-Then-Interpolate" strategy to avoid physical
    /// artifacts when interpolating rotating covariances.
    ///
    /// 1. Finds the nearest covariance before and after the requested epoch.
    /// 2. Rotates BOTH endpoints into the requested `local_frame`.
    /// 3. Interpolates between the two stable matrices using Log-Euclidean Riemannian interpolation.
    ///
    /// :type epoch: Epoch
    /// :type local_frame: LocalFrame
    /// :type almanac: Almanac
    /// :rtype: Covariance
    pub fn covar_at(
        &self,
        epoch: Epoch,
        local_frame: LocalFrame,
        almanac: &Almanac,
    ) -> Result<Option<Covariance>, EphemerisError> {
        self.segment_at_or_error(epoch)?
            .covar_at(epoch, local_frame, almanac)
    }

    /// Resample this ephemeris, with covariance, at the provided nonempty time series.
    ///
    /// :type ts: TimeSeries
    /// :type almanac: Almanac
    /// :rtype: Ephemeris
    pub fn resample(&self, ts: TimeSeries, almanac: &Almanac) -> Result<Self, EphemerisError> {
        let epochs = ts.collect::<Vec<_>>();
        if epochs.is_empty() {
            return Err(EphemerisError::EphemInterpolation {
                source: InterpolationError::EmptyInterpolationData {},
            });
        }
        for &epoch in &epochs {
            self.segment_at_or_error(epoch)?;
        }

        let mut me = Self {
            object_id: self.object_id.clone(),
            segments: Vec::new(),
        };
        for source_segment in &self.segments {
            let Some(view) = source_segment.view() else {
                continue;
            };
            let (sampling_start, sampling_end) = if self.segments.len() == 1 {
                (view.total_start, view.total_end)
            } else {
                (view.useable_start, view.useable_end)
            };
            let mut sampled_segment = EphemerisSegment::new(view.interpolation, view.degree);
            for &epoch in &epochs {
                if (sampling_start..=sampling_end).contains(&epoch) {
                    sampled_segment
                        .state_data
                        .insert(epoch, view.at(epoch, almanac)?);
                }
            }
            if !sampled_segment.state_data.is_empty() {
                sampled_segment.useable_start = sampled_segment
                    .state_data
                    .first_key_value()
                    .map(|entry| *entry.0);
                sampled_segment.useable_end = sampled_segment
                    .state_data
                    .last_key_value()
                    .map(|entry| *entry.0);
                me.segments.push(sampled_segment);
            }
        }

        Ok(me)
    }

    /// Transforms this ephemeris into another frame, and rotates the covariance to that frame if the orientations are different.
    /// NOTE: The Nyquist-Shannon theorem is NOT applied here, so the new ephemeris may not be as precise as the original one.
    /// NOTE: If the orientations are different, the covariance will always be in the Inertial frame of the new frame.
    ///
    /// :type new_frame: Frame
    /// :type almanac: Almanac
    /// :rtype: Ephemeris
    pub fn transform(&self, new_frame: Frame, almanac: &Almanac) -> Result<Self, AlmanacError> {
        let transform_record = |epoch: Epoch,
                                orig_record: &EphemerisRecord|
         -> Result<EphemerisRecord, AlmanacError> {
            let orig_frame = orig_record.orbit.frame;
            let mut new_record = EphemerisRecord {
                orbit: almanac.transform_to(orig_record.orbit, new_frame, None)?,
                covar: orig_record.covar,
            };

            if let Some(covar) = &mut new_record.covar
                && orig_frame.orientation_id != new_frame.orientation_id
            {
                // Query the rotation matrix
                let dcm =
                    almanac
                        .rotate(orig_frame, new_frame, epoch)
                        .context(OrientationSnafu {
                            action: "rotating covariance",
                        })?;

                // Unwrap because we know it is set
                covar.matrix = dcm.state_dcm()
                    * orig_record
                        .covar_in_frame(LocalFrame::Inertial)
                        .context(AlmanacPhysicsSnafu {
                            action: "computing covar inertial",
                        })?
                        .expect("covariance is Some, inside if-let guard")
                        .matrix
                    * dcm.state_dcm().transpose();
            }

            Ok(new_record)
        };

        let mut me = self.clone();
        for segment in &mut me.segments {
            for (epoch, record) in &mut segment.state_data {
                *record = transform_record(*epoch, record)?;
            }
        }

        Ok(me)
    }
}

impl fmt::Display for Ephemeris {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "empty ephem for {}", self.object_id)
        } else {
            let (start, stop) = self
                .domain()
                .expect("ephemeris is not empty, checked in if branch above");
            let span = stop - start;
            write!(
                f,
                "{} ephem from {start} to {stop} ({} states, spans {span})",
                self.object_id,
                self.len()
            )
        }
    }
}

impl<'a> IntoIterator for &'a Ephemeris {
    type Item = &'a EphemerisRecord;
    type IntoIter = std::vec::IntoIter<&'a EphemerisRecord>;

    fn into_iter(self) -> Self::IntoIter {
        let mut records = self
            .segments
            .iter()
            .flat_map(|segment| segment.state_data.values())
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.orbit.epoch);
        records.into_iter()
    }
}

impl IntoIterator for Ephemeris {
    type Item = EphemerisRecord;
    type IntoIter = std::vec::IntoIter<EphemerisRecord>;

    fn into_iter(self) -> Self::IntoIter {
        let mut records = self
            .segments
            .into_iter()
            .flat_map(|segment| segment.state_data.into_values())
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.orbit.epoch);
        records.into_iter()
    }
}

#[cfg(test)]
mod ut_oem {
    use super::{Almanac, DataType, Ephemeris, EphemerisError, EphemerisRecord, LocalFrame};
    use crate::analysis::prelude::OrbitalElement;
    use crate::constants::frames::EARTH_J2000;
    use crate::naif::daf::datatypes::LagrangeSetType9;
    use crate::prelude::{Frame, NAIFSummaryRecord, Orbit};
    use hifitime::{Epoch, TimeSeries, Unit};
    use nalgebra::{Matrix6, SymmetricEigen, Vector6};
    use std::collections::BTreeMap;
    use std::{fs::File, io::Write};

    use rstest::*;

    fn riemannian_distance(p1: &Matrix6<f64>, p2: &Matrix6<f64>) -> f64 {
        // 1. Compute M = P1^-1 * P2.
        // Optimization: Cholesky solve is better than explicit inverse.
        let m = p1.cholesky().unwrap().solve(p2);

        // 2. Eigenvalues of M (Generalized Eigenvalues)
        // Since P1, P2 are SPD, eigenvalues of P1^-1 P2 are real and positive.
        let complex_eigenvalues = m.complex_eigenvalues();

        // 3. Sum of log-squares
        complex_eigenvalues
            .iter()
            .map(|c| c.re.ln().powi(2))
            .sum::<f64>()
            .sqrt()
    }

    #[fixture]
    fn almanac() -> Almanac {
        Almanac::default().load("../data/pck11.pca").unwrap()
    }

    #[rstest]
    fn test_parse_oem_leo(almanac: Almanac) {
        let ephem = Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/LEO_10s.oem")
            .expect("could not parse");

        let start = Epoch::from_gregorian_utc_at_noon(2020, 6, 1);

        assert_eq!(ephem.len(), 361);
        assert_eq!(
            ephem.domain().unwrap(),
            (start, Epoch::from_gregorian_utc_hms(2020, 6, 1, 13, 0, 0))
        );
        assert_eq!(
            ephem.interpolation().unwrap(),
            DataType::Type9LagrangeUnequalStep
        );
        assert_eq!(ephem.degree().unwrap(), 7);

        println!("{ephem}");

        assert_eq!((&ephem).into_iter().count(), ephem.len());

        // Check that we can interpolate
        let epoch = start + Unit::Second * 5;
        let halfway_orbit = ephem.orbit_at(epoch, &almanac).unwrap();
        let before = ephem.nearest_orbit_before(epoch, &almanac).unwrap();
        let after = ephem.nearest_orbit_after(epoch, &almanac).unwrap();
        println!("before = {before}\nduring = {halfway_orbit}\nafter = {after}",);
        // Check that the Keplerian data is reasonably constant.
        // Note that the true Hermite test is in the NAIF SPK tests.
        assert!(dbg!(before.sma_km().unwrap() - halfway_orbit.sma_km().unwrap()).abs() < 1e-1);
        assert!(dbg!(after.sma_km().unwrap() - halfway_orbit.sma_km().unwrap()).abs() < 1e-1);
    }

    #[test]
    fn test_parse_oem_meo() {
        let ephem = Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/MEO_60s.oem")
            .expect("could not parse");

        assert_eq!(ephem.len(), 61);
        assert_eq!(
            ephem.domain().unwrap(),
            (
                Epoch::from_gregorian_utc_at_noon(2020, 6, 1),
                Epoch::from_gregorian_utc_hms(2020, 6, 1, 13, 0, 0)
            )
        );
        assert_eq!(
            ephem.interpolation().unwrap(),
            DataType::Type9LagrangeUnequalStep
        );
        assert_eq!(ephem.degree().unwrap(), 5);

        println!("{ephem}");

        // Ensure that we can build an OEM, re-parse it, and it should match
        let outpath = "../data/tests/ccsds/oem/MEO_60s_rebuilt.oem";
        ephem
            .write_ccsds_oem(outpath, Some("My Originator".to_string()), None)
            .unwrap();

        let ephem2 = Ephemeris::from_ccsds_oem_file(outpath).unwrap();
        assert_eq!(ephem2, ephem);

        // Build the SPK/BSP file as Type13 first
        let my_spk = ephem
            .to_spice_bsp(-159, Some(DataType::Type13HermiteUnequalStep))
            .unwrap();

        let mut file = File::create("../data/tests/naif/spk/meo.bsp").unwrap();
        file.write_all(&my_spk.bytes).unwrap();

        let frcrd = my_spk.file_record().unwrap();
        let name_rcrd = my_spk.name_record(None).unwrap();
        let summary_name = name_rcrd.nth_name(0, frcrd.summary_size());
        assert_eq!(summary_name, "0000-000A (converted by Nyx Space ANISE)");
        let summaries = my_spk.data_summaries(None).unwrap();
        assert_eq!(
            summaries[0].data_type().unwrap(),
            DataType::Type13HermiteUnequalStep
        );
        assert!(
            (summaries[0].start_epoch() - ephem.start_epoch().unwrap()).abs()
                < Unit::Microsecond * 0.05
        );
        assert!(
            (summaries[1].end_epoch() - ephem.end_epoch().unwrap()).abs()
                < Unit::Microsecond * 0.05
        );

        // The second checked-in OEM block advertises 12:33 as useable but its
        // first raw state is at 12:34. The BSP must materialize that boundary so
        // its descriptor can actually be evaluated there.
        let boundary = Epoch::from_gregorian_utc_hms(2020, 6, 1, 12, 33, 0);
        let mut output_view = ephem.segment_at(boundary).unwrap();
        output_view.interpolation = DataType::Type13HermiteUnequalStep;
        let expected = output_view
            .orbit_at_with_window(
                boundary,
                output_view.degree.div_ceil(2),
                &Almanac::default(),
            )
            .unwrap();
        let from_bsp = Almanac::from_spk(my_spk.clone())
            .translate_geometric(Frame::from_ephem_j2000(-159), EARTH_J2000, boundary)
            .unwrap();
        assert!((from_bsp.radius_km - expected.radius_km).norm() < 1e-9);
        assert!((from_bsp.velocity_km_s - expected.velocity_km_s).norm() < 1e-12);

        // Build without specifying the data type, which causes the builder to default to using a Lagrange interpolation.
        ephem
            .write_spice_bsp(-159, "../data/tests/naif/spk/meo_lagrange.bsp", None)
            .unwrap();
    }

    #[test]
    fn test_multisegment_oem_to_spk_preserves_segments() {
        let ephem = Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/MEO_60s.oem")
            .expect("could not parse");

        let spk = ephem
            .to_spice_bsp(-159, Some(DataType::Type9LagrangeUnequalStep))
            .expect("could not convert OEM to SPK");

        assert_eq!(spk.daf_summary(None).unwrap().num_summaries(), 2);
    }

    #[test]
    fn multisegment_oem_preserves_domains_metadata_and_covariance() {
        let input_path = std::env::temp_dir().join("anise_segmented_oem_input.oem");
        let rebuilt_path = std::env::temp_dir().join("anise_segmented_oem_rebuilt.oem");
        let mut file = File::create(&input_path).unwrap();
        file.write_all(
            br#"CCSDS_OEM_VERS = 3.0
CREATION_DATE = 2020-01-01T00:00:00
ORIGINATOR = ANISE TEST

META_START
OBJECT_NAME = TEST
OBJECT_ID = TEST
CENTER_NAME = EARTH
REF_FRAME = EME2000
TIME_SYSTEM = UTC
START_TIME = 2020-01-01T00:00:00.000000
USEABLE_START_TIME = 2020-01-01T00:00:30.000000
USEABLE_STOP_TIME = 2020-01-01T00:01:00.000000
STOP_TIME = 2020-01-01T00:02:00.000000
INTERPOLATION = LAGRANGE
INTERPOLATION_DEGREE = 1
META_STOP
2020-01-01T00:00:00.000000 0 0 0 0 0 0
2020-01-01T00:01:00.000000 1 0 0 0 0 0
2020-01-01T00:02:00.000000 2 0 0 0 0 0
COVARIANCE_START
EPOCH = 2020-01-01T00:01:00.000000
COV_REF_FRAME = EME2000
1
0 1
0 0 1
0 0 0 1
0 0 0 0 1
0 0 0 0 0 1
COVARIANCE_STOP

META_START
OBJECT_NAME = TEST
OBJECT_ID = TEST
CENTER_NAME = EARTH
REF_FRAME = EME2000
TIME_SYSTEM = UTC
START_TIME = 2020-01-01T00:01:00.000000
USEABLE_START_TIME = 2020-01-01T00:01:00.000000
USEABLE_STOP_TIME = 2020-01-01T00:02:30.000000
STOP_TIME = 2020-01-01T00:03:00.000000
INTERPOLATION = HERMITE
INTERPOLATION_DEGREE = 3
META_STOP
2020-01-01T00:01:00.000000 1000 0 0 0 0 0
2020-01-01T00:02:30.000000 1001 0 0 0 0 0
2020-01-01T00:03:00.000000 1002 0 0 0 0 0
COVARIANCE_START
EPOCH = 2020-01-01T00:01:00.000000
COV_REF_FRAME = EME2000
2
0 2
0 0 2
0 0 0 2
0 0 0 0 2
0 0 0 0 0 2
COVARIANCE_STOP
"#,
        )
        .unwrap();
        drop(file);

        let ephem = Ephemeris::from_ccsds_oem_file(&input_path).unwrap();
        // Segment-owned iteration preserves both records at a shared boundary.
        assert_eq!((&ephem).into_iter().count(), 6);
        assert_eq!(ephem.len(), 6);
        let iter_epochs = (&ephem)
            .into_iter()
            .map(|record| record.orbit.epoch)
            .collect::<Vec<_>>();
        assert!(iter_epochs.windows(2).all(|window| window[0] <= window[1]));
        assert_eq!(
            ephem
                .segments
                .iter()
                .map(|segment| segment.state_data.len())
                .sum::<usize>(),
            6
        );

        let boundary = Epoch::from_gregorian_utc_hms(2020, 1, 1, 0, 1, 0);
        assert_eq!(
            ephem.interpolation(),
            Err(EphemerisError::MixedInterpolation)
        );
        assert_eq!(
            ephem.degree(),
            Err(EphemerisError::MixedInterpolationDegree)
        );
        assert_eq!(
            ephem.interpolation_at(boundary - Unit::Second).unwrap(),
            DataType::Type9LagrangeUnequalStep
        );
        assert_eq!(ephem.degree_at(boundary - Unit::Second).unwrap(), 1);
        assert_eq!(
            ephem.interpolation_at(boundary).unwrap(),
            DataType::Type13HermiteUnequalStep
        );
        assert_eq!(ephem.degree_at(boundary).unwrap(), 3);
        let before = ephem
            .orbit_at(boundary - Unit::Second * 15, &Almanac::default())
            .unwrap();
        let after = ephem
            .orbit_at(boundary + Unit::Second * 30, &Almanac::default())
            .unwrap();
        assert!(before.radius_km.x < 10.0);
        assert!(after.radius_km.x > 900.0);
        assert_eq!(
            ephem
                .covar_at(boundary, LocalFrame::Inertial, &Almanac::default())
                .unwrap()
                .unwrap()
                .matrix[(0, 0)],
            2.0
        );
        assert_eq!(
            ephem.segments[0].state_data[&boundary]
                .covar
                .unwrap()
                .matrix[(0, 0)],
            1.0
        );
        assert!(
            ephem
                .orbit_at(boundary, &Almanac::default())
                .unwrap()
                .radius_km
                .x
                > 900.0
        );
        assert!(
            ephem
                .at(
                    Epoch::from_gregorian_utc_hms(2020, 1, 1, 0, 2, 45),
                    &Almanac::default()
                )
                .is_err()
        );

        // Existing post-processing APIs must not flatten the newly preserved
        // boundary and recreate cross-segment interpolation.
        let resampled = ephem
            .resample(
                TimeSeries::inclusive(
                    Epoch::from_gregorian_utc_hms(2020, 1, 1, 0, 0, 30),
                    Epoch::from_gregorian_utc_hms(2020, 1, 1, 0, 2, 30),
                    Unit::Second * 15,
                ),
                &Almanac::default(),
            )
            .unwrap();
        assert_eq!(resampled.segments.len(), 2);
        assert!(
            resampled
                .orbit_at(boundary - Unit::Second * 15, &Almanac::default())
                .unwrap()
                .radius_km
                .x
                < 10.0
        );
        assert!(
            resampled
                .orbit_at(boundary + Unit::Second * 30, &Almanac::default())
                .unwrap()
                .radius_km
                .x
                > 900.0
        );
        assert!(
            resampled
                .orbit_at(boundary - Unit::Second * 7.5, &Almanac::default())
                .unwrap()
                .radius_km
                .x
                < 10.0
        );

        let transformed = ephem.transform(EARTH_J2000, &Almanac::default()).unwrap();
        assert_eq!(transformed.segments.len(), 2);
        assert_eq!(
            transformed
                .segments
                .iter()
                .map(|segment| segment.state_data.len())
                .sum::<usize>(),
            6
        );
        assert!(
            transformed
                .orbit_at(boundary + Unit::Second * 30, &Almanac::default())
                .unwrap()
                .radius_km
                .x
                > 900.0
        );

        // Explicit setters apply a deliberate global override to every segment.
        let mut overridden = ephem.clone();
        overridden.set_interpolation(DataType::Type9LagrangeUnequalStep);
        overridden.set_degree(1).unwrap();
        assert!(overridden.segment_views().iter().all(|segment| {
            segment.interpolation == DataType::Type9LagrangeUnequalStep && segment.degree == 1
        }));

        let mut inserted = ephem.clone();
        let inserted_epoch = boundary - Unit::Second * 10;
        inserted.insert_orbit(Orbit::from_cartesian_pos_vel(
            Vector6::new(7.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            inserted_epoch,
            EARTH_J2000,
        ));
        assert_eq!(inserted.segments.len(), 2);
        assert_eq!(
            inserted
                .orbit_at(inserted_epoch, &Almanac::default())
                .unwrap()
                .radius_km
                .x,
            7.0
        );
        assert_eq!(
            inserted
                .to_spice_bsp(-159, None)
                .unwrap()
                .daf_summary(None)
                .unwrap()
                .num_summaries(),
            2
        );

        let spk = ephem.to_spice_bsp(-159, None).unwrap();
        let parsed_spk = crate::naif::SPK::parse(spk.bytes.clone()).unwrap();
        let summaries = parsed_spk.data_summaries(None).unwrap();
        assert_eq!(parsed_spk.daf_summary(None).unwrap().num_summaries(), 2);
        assert_eq!(
            summaries[0].data_type().unwrap(),
            DataType::Type9LagrangeUnequalStep
        );
        assert_eq!(
            summaries[1].data_type().unwrap(),
            DataType::Type13HermiteUnequalStep
        );
        assert!(summaries[0].end_index() < summaries[1].start_index());
        assert!((summaries[0].end_epoch_et_s() - boundary.to_et_seconds()).abs() < 1e-9);
        assert!((summaries[1].start_epoch_et_s() - boundary.to_et_seconds()).abs() < 1e-9);
        let selected = parsed_spk
            .summary_from_id_at_epoch(-159, boundary.to_et_seconds())
            .unwrap()
            .0;
        assert_eq!(
            selected.data_type().unwrap(),
            DataType::Type13HermiteUnequalStep
        );
        // Empty physical summary slots have ID zero. A reloaded BSP with a real
        // ID zero must still keep its chronological index and later-segment
        // precedence at a shared descriptor endpoint.
        let zero_id_spk = ephem.to_spice_bsp(0, None).unwrap();
        let zero_id_spk = crate::naif::SPK::parse(zero_id_spk.bytes.clone()).unwrap();
        let selected = zero_id_spk
            .summary_from_id_at_epoch(0, boundary.to_et_seconds())
            .unwrap()
            .0;
        assert_eq!(
            selected.data_type().unwrap(),
            DataType::Type13HermiteUnequalStep
        );

        ephem
            .write_ccsds_oem(
                &rebuilt_path,
                Some("ANISE TEST".to_string()),
                Some("TEST".to_string()),
            )
            .unwrap();
        let reparsed = Ephemeris::from_ccsds_oem_file(&rebuilt_path).unwrap();
        let rebuilt_spk = reparsed.to_spice_bsp(-159, None).unwrap();
        assert_eq!(rebuilt_spk.daf_summary(None).unwrap().num_summaries(), 2);
        assert_eq!(
            reparsed
                .covar_at(boundary, LocalFrame::Inertial, &Almanac::default())
                .unwrap()
                .unwrap()
                .matrix[(0, 0)],
            2.0
        );
        assert_eq!(
            reparsed.segments[0].state_data[&boundary]
                .covar
                .unwrap()
                .matrix[(0, 0)],
            1.0
        );

        let _ = std::fs::remove_file(input_path);
        let _ = std::fs::remove_file(rebuilt_path);
    }

    #[test]
    fn bsp_writer_chains_more_than_twenty_five_oem_segments() {
        let input_path = std::env::temp_dir().join("anise_26_segment_oem.oem");
        let mut file = File::create(&input_path).unwrap();
        writeln!(
            file,
            "CCSDS_OEM_VERS = 3.0\nCREATION_DATE = 2020-01-01T00:00:00\nORIGINATOR = ANISE TEST\n"
        )
        .unwrap();

        for segment_idx in 0..26 {
            let start_minute = segment_idx * 2;
            let stop_minute = start_minute + 1;
            writeln!(
                file,
                "META_START\nOBJECT_NAME = TEST\nOBJECT_ID = TEST\nCENTER_NAME = EARTH\nREF_FRAME = EME2000\nTIME_SYSTEM = UTC\nSTART_TIME = 2020-01-01T00:{start_minute:02}:00.000000\nUSEABLE_START_TIME = 2020-01-01T00:{start_minute:02}:00.000000\nUSEABLE_STOP_TIME = 2020-01-01T00:{stop_minute:02}:00.000000\nSTOP_TIME = 2020-01-01T00:{stop_minute:02}:00.000000\nINTERPOLATION = LAGRANGE\nINTERPOLATION_DEGREE = 1\nMETA_STOP\n2020-01-01T00:{start_minute:02}:00.000000 {segment_idx} 0 0 0 0 0\n2020-01-01T00:{stop_minute:02}:00.000000 {segment_idx} 0 0 0 0 0\n"
            )
            .unwrap();
        }
        drop(file);

        let ephem = Ephemeris::from_ccsds_oem_file(&input_path).unwrap();
        let spk = ephem.to_spice_bsp(-159, None).unwrap();
        let spk = crate::naif::SPK::parse(spk.bytes.clone()).unwrap();

        let first = spk.daf_summary(None).unwrap();
        assert_eq!(first.num_summaries(), 25);
        assert_eq!(first.prev_record(), 0);
        assert_eq!(first.next_record(), 4);
        let second = spk.daf_summary(Some(4)).unwrap();
        assert_eq!(second.num_summaries(), 1);
        assert_eq!(second.prev_record(), 2);
        assert_eq!(second.next_record(), 0);
        assert_eq!(spk.file_record().unwrap().backward, 4);

        let summaries = spk
            .iter_summary_blocks()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .flat_map(|block| block.iter())
            .filter(|summary| summary.id() == -159)
            .collect::<Vec<_>>();
        assert_eq!(summaries.len(), 26);
        assert!(
            summaries
                .windows(2)
                .all(|pair| pair[0].end_index() < pair[1].start_index())
        );
        // File record plus two summary/name pairs occupy five records, so data
        // starts at the first word of record six.
        assert_eq!(summaries[0].start_index(), 641);
        assert_eq!(
            spk.name_record(Some(4))
                .unwrap()
                .nth_name(0, spk.file_record().unwrap().summary_size()),
            "TEST (converted by Nyx Space ANISE)"
        );

        // The 26th segment lives in the second summary record. Verify that the
        // parsed index follows the record chain and returns its physical location.
        let final_epoch = Epoch::from_gregorian_utc_hms(2020, 1, 1, 0, 50, 30);
        let (final_summary, final_block, final_index) = spk
            .summary_from_id_at_epoch(-159, final_epoch.to_et_seconds())
            .unwrap();
        assert_eq!(final_block, Some(4));
        assert_eq!(final_index, 0);
        assert_eq!(final_summary, summaries[25]);
        let final_state = Almanac::from_spk(spk.clone())
            .translate_geometric(Frame::from_ephem_j2000(-159), EARTH_J2000, final_epoch)
            .unwrap();
        assert!((final_state.radius_km.x - 25.0).abs() < 1e-12);

        let _ = std::fs::remove_file(input_path);
    }

    #[test]
    fn multisegment_oem_rejects_internal_useable_gap_queries() {
        let input_path = std::env::temp_dir().join("anise_segment_gap.oem");
        let mut file = File::create(&input_path).unwrap();
        file.write_all(
            br#"CCSDS_OEM_VERS = 3.0
CREATION_DATE = 2020-01-01T00:00:00
ORIGINATOR = ANISE TEST
META_START
OBJECT_NAME = TEST
OBJECT_ID = TEST
CENTER_NAME = EARTH
REF_FRAME = EME2000
TIME_SYSTEM = UTC
USEABLE_START_TIME = 2020-01-01T00:00:00
USEABLE_STOP_TIME = 2020-01-01T00:01:00
INTERPOLATION = LAGRANGE
INTERPOLATION_DEGREE = 1
META_STOP
2020-01-01T00:00:00 0 0 0 0 0 0
2020-01-01T00:01:00 1 0 0 0 0 0
META_START
OBJECT_NAME = TEST
OBJECT_ID = TEST
CENTER_NAME = EARTH
REF_FRAME = EME2000
TIME_SYSTEM = UTC
USEABLE_START_TIME = 2020-01-01T00:01:30
USEABLE_STOP_TIME = 2020-01-01T00:03:00
INTERPOLATION = LAGRANGE
INTERPOLATION_DEGREE = 1
META_STOP
2020-01-01T00:01:00 1000 0 0 0 0 0
2020-01-01T00:02:00 1001 0 0 0 0 0
2020-01-01T00:03:00 1002 0 0 0 0 0
"#,
        )
        .unwrap();
        drop(file);

        let ephem = Ephemeris::from_ccsds_oem_file(&input_path).unwrap();
        let first_end = Epoch::from_gregorian_utc_hms(2020, 1, 1, 0, 1, 0);
        let gap = first_end + Unit::Second * 15;
        assert_eq!(
            ephem
                .orbit_at(first_end, &Almanac::default())
                .unwrap()
                .radius_km
                .x,
            1.0
        );
        assert!(ephem.at(gap, &Almanac::default()).is_err());
        assert!(
            ephem
                .resample(
                    TimeSeries::inclusive(first_end, gap, Unit::Second * 15),
                    &Almanac::default(),
                )
                .is_err()
        );
        let spk = ephem.to_spice_bsp(-159, None).unwrap();
        assert!(
            spk.summary_from_id_at_epoch(-159, gap.to_et_seconds())
                .is_err()
        );

        let _ = std::fs::remove_file(input_path);
    }

    #[test]
    fn resample_single_segment_uses_its_raw_support_domain() {
        let start = Epoch::from_gregorian_utc_at_midnight(2020, 1, 1);
        let middle = start + Unit::Minute;
        let end = middle + Unit::Minute;
        let state_data = [start, middle, end]
            .into_iter()
            .enumerate()
            .map(|(idx, epoch)| {
                let orbit = Orbit::from_cartesian_pos_vel(
                    Vector6::new(idx as f64, 0.0, 0.0, 0.0, 0.0, 0.0),
                    epoch,
                    EARTH_J2000,
                );
                (epoch, EphemerisRecord { orbit, covar: None })
            })
            .collect();
        let ephem = Ephemeris {
            object_id: "TEST".to_string(),
            segments: vec![super::EphemerisSegment {
                interpolation: DataType::Type9LagrangeUnequalStep,
                degree: 1,
                useable_start: Some(middle),
                useable_end: Some(end),
                state_data,
            }],
        };

        let resampled = ephem
            .resample(
                TimeSeries::inclusive(start, middle, Unit::Minute * 1),
                &Almanac::default(),
            )
            .unwrap();

        assert_eq!(resampled.len(), 2);
        assert_eq!(resampled.start_epoch().unwrap(), start);
        assert_eq!(
            resampled
                .orbit_at(start, &Almanac::default())
                .unwrap()
                .radius_km
                .x,
            0.0
        );
        assert!(
            ephem
                .resample(
                    TimeSeries::exclusive(start, start, Unit::Minute * 1),
                    &Almanac::default(),
                )
                .is_err()
        );
    }

    #[test]
    fn bsp_writer_uses_spice_epoch_registry_boundaries() {
        let epoch = Epoch::from_gregorian_utc_at_midnight(2020, 1, 1);
        let mut ephem = Ephemeris::new("TEST".to_string());
        ephem.set_interpolation(DataType::Type9LagrangeUnequalStep);
        ephem.set_degree(1).unwrap();

        for idx in 0..100 {
            ephem.insert_orbit(Orbit::from_cartesian_pos_vel(
                Vector6::new(idx as f64, 0.0, 0.0, 0.0, 0.0, 0.0),
                epoch + Unit::Second * idx,
                EARTH_J2000,
            ));
        }
        let spk = ephem.to_spice_bsp(-159, None).unwrap();
        let data: LagrangeSetType9<'_> = spk.nth_data(None, 0).unwrap();
        assert_eq!(data.epoch_registry.len(), 0);

        for idx in 100..200 {
            ephem.insert_orbit(Orbit::from_cartesian_pos_vel(
                Vector6::new(idx as f64, 0.0, 0.0, 0.0, 0.0, 0.0),
                epoch + Unit::Second * idx,
                EARTH_J2000,
            ));
        }
        let spk = ephem.to_spice_bsp(-159, None).unwrap();
        let data: LagrangeSetType9<'_> = spk.nth_data(None, 0).unwrap();
        assert_eq!(data.epoch_registry.len(), 1);

        ephem.insert_orbit(Orbit::from_cartesian_pos_vel(
            Vector6::new(200.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            epoch + Unit::Second * 200,
            EARTH_J2000,
        ));
        let spk = ephem.to_spice_bsp(-159, None).unwrap();
        let data: LagrangeSetType9<'_> = spk.nth_data(None, 0).unwrap();
        assert_eq!(data.epoch_registry.len(), 2);

        ephem.set_degree(16).unwrap();
        assert!(ephem.to_spice_bsp(-159, None).is_err());
        ephem.set_interpolation(DataType::Type13HermiteUnequalStep);
        ephem.set_degree(2).unwrap();
        assert!(ephem.to_spice_bsp(-159, None).is_err());
        ephem.set_degree(3).unwrap();
        ephem.set_interpolation(DataType::Type12HermiteEqualStep);
        assert!(ephem.to_spice_bsp(-159, None).is_err());
        assert!(
            ephem
                .to_spice_bsp(-159, Some(DataType::Type12HermiteEqualStep))
                .is_err()
        );
    }

    #[test]
    fn test_parse_oem_meo_bad() {
        assert!(Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/MEO_60s_bad.oem").is_err());
    }

    #[test]
    fn test_parse_oem_covar_extra_row() {
        // A covariance block with a seventh data row must be rejected rather than
        // indexing past the 6x6 covariance matrix.
        assert!(
            Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/JPL_MGS_cov_extra_row.oem")
                .is_err()
        );
    }

    #[test]
    fn test_parse_oem_covar_no_epoch() {
        // A covariance data row before any EPOCH line must be rejected rather than
        // unwrapping the still-empty covariance matrix.
        assert!(
            Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/JPL_MGS_cov_no_epoch.oem")
                .is_err()
        );
    }

    #[test]
    fn test_parse_oem_covar_stray_after_block() {
        // Each covariance block must start from a clean slate: a stray data row in a
        // later block before its own EPOCH line must be rejected rather than folded
        // into a previous block's matrix.
        assert!(
            Ephemeris::from_ccsds_oem_file(
                "../data/tests/ccsds/oem/JPL_MGS_cov_stray_after_block.oem"
            )
            .is_err()
        );
    }

    #[test]
    fn oem_rejects_unclosed_covariance_sections() {
        let prefix = br#"CCSDS_OEM_VERS = 3.0
CREATION_DATE = 2020-01-01T00:00:00
ORIGINATOR = ANISE TEST
META_START
OBJECT_NAME = TEST
OBJECT_ID = TEST
CENTER_NAME = EARTH
REF_FRAME = EME2000
TIME_SYSTEM = UTC
INTERPOLATION = LAGRANGE
INTERPOLATION_DEGREE = 1
META_STOP
2020-01-01T00:00:00 0 0 0 0 0 0
2020-01-01T00:01:00 1 0 0 0 0 0
COVARIANCE_START
EPOCH = 2020-01-01T00:00:00
COV_REF_FRAME = EME2000
"#;
        let incomplete = "1\n";
        let complete_without_stop = "1\n0 1\n0 0 1\n0 0 0 1\n0 0 0 0 1\n0 0 0 0 0 1\n";

        for (name, suffix) in [
            ("incomplete", incomplete),
            ("complete_without_stop", complete_without_stop),
        ] {
            let path = std::env::temp_dir().join(format!("anise_{name}_covariance.oem"));
            let mut file = File::create(&path).unwrap();
            file.write_all(prefix).unwrap();
            file.write_all(suffix.as_bytes()).unwrap();
            drop(file);
            assert!(Ephemeris::from_ccsds_oem_file(&path).is_err());
            let _ = std::fs::remove_file(path);
        }
    }

    #[rstest]
    fn test_parse_oem_covar(almanac: Almanac) {
        let ephem = Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/JPL_MGS_cov.oem")
            .expect("could not parse");

        let (start, end) = (
            Epoch::from_gregorian(
                1996,
                12,
                28,
                21,
                29,
                7,
                267_000_000,
                hifitime::TimeScale::TDB,
            ),
            Epoch::from_gregorian(
                1996,
                12,
                30,
                1,
                28,
                2,
                267_000_000,
                hifitime::TimeScale::TDB,
            ),
        );
        assert_eq!(ephem.len(), 4);
        assert_eq!(ephem.domain().unwrap(), (start, end));
        assert_eq!(
            ephem.interpolation().unwrap(),
            DataType::Type13HermiteUnequalStep
        );
        assert_eq!(ephem.degree().unwrap(), 7);

        println!("{ephem}");

        // Check that we can interpolate the covariance
        let epoch = start + Unit::Minute * 15;
        let halfway = ephem
            .covar_at(
                epoch,
                crate::ephemerides::ephemeris::LocalFrame::Inertial,
                &almanac,
            )
            .unwrap()
            .unwrap()
            .matrix;
        let before = ephem
            .nearest_covar_before(epoch, &almanac)
            .unwrap()
            .unwrap()
            .1
            .matrix;
        let after = ephem
            .nearest_covar_after(epoch, &almanac)
            .unwrap()
            .unwrap()
            .1
            .matrix;
        println!("before = {before}\nduring = {halfway}\nafter = {after}");
        assert!((halfway - before).norm() < 1e-14);
        assert!((halfway - after).norm() < 1e-14);

        // Check that we can interpolate throughout the ephemeris
        for epoch in TimeSeries::inclusive(
            ephem.start_epoch().unwrap(),
            ephem.end_epoch().unwrap(),
            Unit::Minute * 1.337,
        ) {
            assert!(ephem.at(epoch, &almanac).is_ok());
        }

        // Re-export with covariance
        let rebuilt_path = "../data/tests/ccsds/oem/JPL_MGS_cov_rebuilt.oem";
        ephem.write_ccsds_oem(rebuilt_path, None, None).unwrap();
        let ephem2 =
            Ephemeris::from_ccsds_oem_file(rebuilt_path).expect("could not parse rebuilt OEM");

        assert!(ephem2.nearest_covar_after(epoch, &almanac).is_ok());
    }

    #[rstest]
    fn test_oem_interp_covar_truth(almanac: Almanac) {
        let ephem = Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/LRO_Nyx.oem")
            .expect("could not parse");

        let start = Epoch::from_gregorian_utc_at_midnight(2024, 1, 1);
        let end = start + Unit::Minute * 3;

        assert_eq!(ephem.len(), 4);
        assert_eq!(ephem.domain().unwrap(), (start, end));
        assert_eq!(
            ephem.interpolation().unwrap(),
            DataType::Type13HermiteUnequalStep
        );
        assert_eq!(ephem.degree().unwrap(), 7);

        // We have data from Nyx showing the proper covariance in between the data in the OEM.
        // So we'll check that the interpolator somewhat matches that data.
        let offset = Unit::Minute * 1 + Unit::Second * 24.696597;

        let epoch = start + offset;

        // Check that we can interpolate the covariance and that it correctly rotates.
        let bw_1_2 = ephem
            .covar_at(epoch, LocalFrame::Inertial, &almanac)
            .unwrap()
            .unwrap();
        assert_eq!(bw_1_2.local_frame, LocalFrame::Inertial);
        let bw_1_2_truth = Matrix6::new(
            0.209575, 0.4048630, 0.2455520, 0.001016, 0.0019710, 0.0011840, // X column
            0.404863, 1.1089200, -0.066758, 0.001961, 0.0055080, -0.000494, // Y column
            0.245552, -0.066758, 1.1863670, 0.001197, -0.000509, 0.0060070, // Z column
            0.001016, 0.0019610, 0.0011970, 0.000012, 0.0000240, 0.0000140, // Vx column
            0.001971, 0.0055080, -0.000509, 0.000024, 0.0000660, 0.0000060, // Vy column
            0.001184, -0.000494, 0.0060070, 0.000014, 0.0000060, 0.0000720, // Vz column
        );

        // Compute the Riemann distance since we interpolate in Reimann space
        let rdist = riemannian_distance(&bw_1_2.matrix, &bw_1_2_truth);
        assert!(rdist < 0.4, "arbitrary max distance failed");

        let covar_prev = ephem
            .nearest_covar_before(epoch, &almanac)
            .unwrap()
            .unwrap()
            .1
            .matrix;
        let covar_next = ephem
            .nearest_covar_after(epoch, &almanac)
            .unwrap()
            .unwrap()
            .1
            .matrix;

        let det_prev = covar_prev.determinant();
        let det_next = covar_next.determinant();
        let det_interp = bw_1_2.matrix.determinant();
        let det_truth = bw_1_2_truth.determinant();

        // Log-Euclidean guarantees the log-determinant is linearly interpolated!
        // This is a MUCH stricter mathematical check than comparing to external truth.
        let log_det_prev = det_prev.ln();
        let log_det_next = det_next.ln();
        let log_det_interp = det_interp.ln();
        let log_det_truth = det_truth.ln();

        // VALIDATION: Check Log-Euclidean Property (Volume Monotonicity)
        // The log-determinant must be linearly interpolated.
        // This validates the math of the implementation.
        let alpha = (offset - Unit::Minute).to_seconds() / 60.0;
        let expected_log_det = log_det_prev * (1.0 - alpha) + log_det_next * alpha;

        // Tolerance can be very tight because this is purely algebraic
        assert!(
            (log_det_interp - expected_log_det).abs() < 1e-12,
            "Log-Euclidean implementation failed linearity check"
        );

        // OBSERVATION: Compare with Truth
        // We expect a deviation here because interpolation != propagation.
        // This print confirms the "swelling/shrinking" discrepancy.
        let vol_ratio = (log_det_interp - log_det_truth).exp();
        println!("Volume Ratio (Interp/Truth): {vol_ratio:.4}");
        // Truth covar has a larger volume (less negative) because dynamics are not the shortest path in Log-Euclidian space
        assert!((vol_ratio - 0.8).abs() < 0.05);
        // Expect ~0.80 (Interp is smaller)

        dbg!(log_det_prev, log_det_next, log_det_interp, log_det_truth);

        // Ensure that this is a symmetric matrix
        assert!((bw_1_2.matrix.transpose() - bw_1_2.matrix).norm() < 1e-15);
        // Ensure that it's PSD
        let decomp = SymmetricEigen::new(bw_1_2.matrix);
        assert!(decomp.eigenvalues.iter().all(|&e| e >= 0.0));

        // Ensure that we're close to the RIC frame uncertainties

        let bw_1_2_ric = ephem
            .covar_at(epoch, LocalFrame::RIC, &almanac)
            .unwrap()
            .unwrap();

        assert_eq!(bw_1_2_ric.local_frame, LocalFrame::RIC);

        let diag = bw_1_2_ric.matrix.diagonal();
        let diag_sqrt = Vector6::from_iterator(diag.iter().map(|x| x.sqrt()));

        // Nyx reports these as Sigmas, so we apply the square root of the covariance here.
        let ric_diag_sigmas =
            Vector6::new(1.104494, 0.335512, 1.082771, 0.008646, 0.002558, 0.008262);
        let ric_err = diag_sqrt - ric_diag_sigmas;
        println!("{diag_sqrt:.6e}\n{ric_diag_sigmas:.6e}\n{ric_err:0.6e}",);
        let ric_pos_km_err = ric_err.fixed_rows::<3>(0);
        let ric_vel_km_s_err = ric_err.fixed_rows::<3>(3);
        assert!(dbg!(ric_pos_km_err.norm()) < 0.2);
        assert!(dbg!(ric_vel_km_s_err.norm()) < 2.5e-3);
    }

    #[rstest]
    fn test_oem_covar_orbital_element_uncertainty(almanac: Almanac) {
        let ephem = Ephemeris::from_ccsds_oem_file("../data/tests/ccsds/oem/LRO_Nyx.oem")
            .expect("could not parse");

        let start = Epoch::from_gregorian_utc_at_midnight(2024, 1, 1);
        let end = start + Unit::Minute * 3;

        assert_eq!(ephem.len(), 4);
        assert_eq!(ephem.domain().unwrap(), (start, end));
        assert_eq!(
            ephem.interpolation().unwrap(),
            DataType::Type13HermiteUnequalStep
        );
        assert_eq!(ephem.degree().unwrap(), 7);

        // We have data from Nyx showing the proper covariance in between the data in the OEM.
        // So we'll check that the interpolator somewhat matches that data.
        let offset = Unit::Minute * 1 + Unit::Second * 24.696597;

        let epoch = start + offset;

        let rcrd1 = ephem.at(start, &almanac).unwrap();
        let rcrd2 = ephem.at(epoch, &almanac).unwrap();
        let rcrd3 = ephem.at(end, &almanac).unwrap();

        for oe in [
            OrbitalElement::Rmag,
            OrbitalElement::SemiMajorAxis,
            OrbitalElement::Hmag,
        ] {
            // Test that the covariance interpolation follows the manifold and does not swell.
            let sigma1 = rcrd1.sigma_for(oe).unwrap();
            let sigma2 = rcrd2.sigma_for(oe).unwrap();
            let sigma3 = rcrd3.sigma_for(oe).unwrap();

            dbg!(oe, sigma1, sigma2, sigma3);

            // The range is empty if start > end, so we check both options.
            assert!(
                (sigma1..=sigma3).contains(&sigma2) || (sigma3..=sigma1).contains(&sigma2),
                "failed on {oe:?}"
            );
        }
    }

    #[rstest]
    fn test_parse_stk_e_v12(almanac: Almanac) {
        let path = "../data/tests/ansys-stk/test_v12.e";
        let ephem = Ephemeris::from_stk_e_file(path).expect("Could not parse STK file");

        // Check metadata
        assert_eq!(
            format!("{:?}", ephem.interpolation().unwrap()),
            "Type9LagrangeUnequalStep"
        );
        assert_eq!(ephem.object_id(), "test_v12");

        // Check domain
        let (start, end) = ephem.domain().expect("Could not get domain");

        // ScenarioEpoch: 1 Jun 2020 12:00:00.000000
        let scenario_epoch = Epoch::from_gregorian_utc_at_noon(2020, 6, 1);

        // First point at 0.0s offset
        let expected_start = scenario_epoch;
        assert!(
            (start - expected_start).to_seconds().abs() < 1e-6,
            "Start epoch mismatch: {start} vs {expected_start}"
        );

        // Last point at 1980.0s offset
        let expected_end = scenario_epoch + Unit::Second * 1980.0;
        assert!(
            (end - expected_end).to_seconds().abs() < 1e-6,
            "End epoch mismatch: {end} vs {expected_end}",
        );

        assert_eq!(ephem.len(), 34);

        let record = ephem.at(expected_end, &almanac).unwrap();
        assert!(record.covar.is_none());
        assert_eq!(record.orbit.epoch, expected_end);
    }

    #[rstest]
    fn test_parse_stk_e_with_covariance(almanac: Almanac) {
        let path = "../data/tests/ansys-stk/stk_cov.e";
        let ephem = Ephemeris::from_stk_e_file(path).expect("Could not parse STK file");

        // Check metadata
        assert_eq!(
            format!("{:?}", ephem.interpolation().unwrap()),
            "Type9LagrangeUnequalStep"
        );

        assert!(
            ephem.includes_covariance(),
            "Ephemeris should have covariance"
        );

        let (start, _) = ephem.domain().expect("Could not get domain");

        // Check first point (Time 0.0) -> Sequence 1.0 to 21.0
        // LowerTriangular Order:
        // C[0][0] = 1
        // C[1][0] = 2, C[1][1] = 3
        // C[2][0] = 4, C[2][1] = 5, C[2][2] = 6
        // ...
        // C[5][5] = 21
        let rec0 = ephem.at(start, &almanac).unwrap();
        assert!(rec0.covar.is_some());
        let mat0 = rec0.covar.unwrap().matrix;

        assert!((mat0[(0, 0)] - 1.0).abs() < 1e-9);
        assert!((mat0[(1, 0)] - 2.0).abs() < 1e-9);
        assert!((mat0[(1, 1)] - 3.0).abs() < 1e-9);
        assert!((mat0[(2, 0)] - 4.0).abs() < 1e-9);
        assert!((mat0[(2, 2)] - 6.0).abs() < 1e-9);

        // Check last element C[5][5]
        // Row 3: 7, 8, 9, 10
        // Row 4: 11, 12, 13, 14, 15
        // Row 5: 16, 17, 18, 19, 20, 21
        assert!((mat0[(5, 0)] - 16.0).abs() < 1e-9);
        assert!((mat0[(5, 5)] - 21.0).abs() < 1e-9);

        // Verify Symmetry
        assert!((mat0[(0, 1)] - 2.0).abs() < 1e-9);
        assert!((mat0[(5, 2)] - 18.0).abs() < 1e-9); // C[2][5] symmetric to C[5][2] (value 18)
    }

    #[test]
    fn oem_rejects_zero_interpolation_degree() {
        // A degree of zero leaves no samples to interpolate with, and it used to underflow the
        // window size computation when the parsed ephemeris was written back out as a BSP.
        let path = std::env::temp_dir().join("anise_oem_zero_degree.oem");
        let mut file = File::create(&path).unwrap();
        file.write_all(
            b"CCSDS_OEM_VERS = 2.0\n\
              META_START\n\
              OBJECT_ID = TEST\n\
              CENTER_NAME = EARTH\n\
              REF_FRAME = EME2000\n\
              TIME_SYSTEM = UTC\n\
              INTERPOLATION = HERMITE\n\
              INTERPOLATION_DEGREE = 0\n\
              META_STOP\n\
              2020-06-01T12:00:00 7000.0 0.0 0.0 0.0 7.5 0.0\n",
        )
        .unwrap();
        drop(file);

        assert!(Ephemeris::from_ccsds_oem_file(path.to_str().unwrap()).is_err());
    }

    #[test]
    fn oem_rejects_invalid_metadata_transitions_and_zero_length_domains() {
        let dangling_path = std::env::temp_dir().join("anise_oem_dangling_metadata.oem");
        let missing_start_path = std::env::temp_dir().join("anise_oem_missing_metadata_start.oem");
        let zero_domain_path = std::env::temp_dir().join("anise_oem_zero_useable_domain.oem");
        let valid_block = "META_START\nOBJECT_ID = TEST\nCENTER_NAME = EARTH\nREF_FRAME = EME2000\nTIME_SYSTEM = UTC\nINTERPOLATION = LAGRANGE\nINTERPOLATION_DEGREE = 1\nMETA_STOP\n2020-06-01T12:00:00 7000 0 0 0 7.5 0\n2020-06-01T12:01:00 7001 0 0 0 7.5 0\n";

        std::fs::write(
            &dangling_path,
            format!("CCSDS_OEM_VERS = 3.0\n{valid_block}META_START\n"),
        )
        .unwrap();
        std::fs::write(
            &missing_start_path,
            format!("CCSDS_OEM_VERS = 3.0\n{valid_block}TIME_SYSTEM = UTC\n"),
        )
        .unwrap();
        std::fs::write(
            &zero_domain_path,
            "CCSDS_OEM_VERS = 3.0\nMETA_START\nOBJECT_ID = TEST\nCENTER_NAME = EARTH\nREF_FRAME = EME2000\nTIME_SYSTEM = UTC\nUSEABLE_START_TIME = 2020-06-01T12:00:30\nUSEABLE_STOP_TIME = 2020-06-01T12:00:30\nINTERPOLATION = LAGRANGE\nINTERPOLATION_DEGREE = 1\nMETA_STOP\n2020-06-01T12:00:00 7000 0 0 0 7.5 0\n2020-06-01T12:01:00 7001 0 0 0 7.5 0\n",
        )
        .unwrap();

        for path in [&dangling_path, &missing_start_path, &zero_domain_path] {
            assert!(Ephemeris::from_ccsds_oem_file(path).is_err());
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn bsp_writer_rejects_zero_degree() {
        let epoch = Epoch::from_gregorian_utc_at_noon(2020, 6, 1);
        let orbit = Orbit::from_cartesian_pos_vel(
            Vector6::new(7000.0, 0.0, 0.0, 0.0, 7.5, 0.0),
            epoch,
            EARTH_J2000,
        );
        let mut state_data = BTreeMap::new();
        state_data.insert(epoch, EphemerisRecord { orbit, covar: None });
        let ephem = Ephemeris {
            object_id: "TEST".to_string(),
            segments: vec![super::EphemerisSegment {
                interpolation: DataType::Type13HermiteUnequalStep,
                degree: 0,
                useable_start: None,
                useable_end: None,
                state_data,
            }],
        };
        assert!(ephem.to_spice_bsp(-999, None).is_err());

        let next_epoch = epoch + Unit::Second;
        let next_orbit = Orbit::from_cartesian_pos_vel(
            Vector6::new(7001.0, 0.0, 0.0, 0.0, 7.5, 0.0),
            next_epoch,
            EARTH_J2000,
        );
        let state_data = BTreeMap::from([
            (epoch, EphemerisRecord { orbit, covar: None }),
            (
                next_epoch,
                EphemerisRecord {
                    orbit: next_orbit,
                    covar: None,
                },
            ),
        ]);
        let ephem = Ephemeris {
            object_id: "TEST".to_string(),
            segments: vec![super::EphemerisSegment {
                interpolation: DataType::Type9LagrangeUnequalStep,
                degree: 1,
                useable_start: Some(epoch),
                useable_end: Some(epoch),
                state_data,
            }],
        };
        assert!(ephem.to_spice_bsp(-999, None).is_err());
    }
}
