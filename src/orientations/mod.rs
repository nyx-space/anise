use hifitime::{Duration, Epoch, TimeUnits};

use crate::{
    prelude::AniseError,
    structure::{
        orientation::{orient_data::OrientationData, Orientation},
        spline::Evenness,
    },
};

impl<'a> Orientation<'a> {
    pub const fn is_high_precision(&self) -> bool {
        matches!(self.orientation_data, OrientationData::HighPrecision { .. })
    }

    pub fn check_integrity(&self) -> Result<(), AniseError> {
        match &self.orientation_data {
            OrientationData::PlanetaryConstant { .. } => {
                // Planetary constant information won't be decodable unless its integrity is valid.
                Ok(())
            }
            OrientationData::HighPrecision {
                ref_epoch: _,
                backward: _,
                interpolation_kind: _,
                splines,
            } => splines.check_integrity(),
        }
    }

    /// Returns the starting epoch of this ephemeris. It is guaranteed that start_epoch <= end_epoch.
    ///
    /// # Note
    /// + If the ephemeris is stored in chronological order, then the start epoch is the same as the first epoch.
    /// + If the ephemeris is stored in anti-chronological order, then the start epoch is the last epoch.
    pub fn start_epoch(&self) -> Epoch {
        if !self.is_high_precision() {
            Epoch::from_tdb_duration(Duration::MIN)
        } else {
            if self.first_epoch() > self.last_epoch() {
                self.last_epoch().unwrap()
            } else {
                self.first_epoch().unwrap()
            }
        }
    }

    pub fn end_epoch(&self) -> Epoch {
        if !self.is_high_precision() {
            Epoch::from_tdb_duration(Duration::MAX)
        } else {
            if self.first_epoch() > self.last_epoch() {
                self.first_epoch().unwrap()
            } else {
                self.last_epoch().unwrap()
            }
        }
    }

    /// Returns the first epoch in the data, which will be the chronological "end" epoch if the ephemeris is generated backward
    fn first_epoch(&self) -> Option<Epoch> {
        match self.orientation_data {
            OrientationData::PlanetaryConstant { .. } => None,
            OrientationData::HighPrecision {
                ref_epoch,
                backward: _,
                interpolation_kind: _,
                splines: _,
            } => Some(ref_epoch),
        }
    }

    /// Returns the last epoch in the data, which will be the chronological "start" epoch if the ephemeris is generated backward
    fn last_epoch(&self) -> Option<Epoch> {
        match self.orientation_data {
            OrientationData::PlanetaryConstant { .. } => None,
            OrientationData::HighPrecision {
                ref_epoch,
                backward: _,
                interpolation_kind: _,
                splines,
            } => match splines.metadata.evenness {
                Evenness::Even { duration_ns } => {
                    // Grab the number of splines
                    Some(ref_epoch + ((splines.len() as f64) * (duration_ns as i64).nanoseconds()))
                }
                Evenness::Uneven { indexes: _ } => {
                    todo!()
                }
            },
        }
    }
}
