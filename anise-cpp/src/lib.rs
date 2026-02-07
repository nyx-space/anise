use hifitime::{Duration, Epoch, TimeScale, TimeSeries, Unit};
use std::str::FromStr;

#[cxx::bridge(namespace = "anise::time")]
mod ffi {
    #[derive(Debug)]
    enum TimeScale {
        TAI,
        TDB,
        UTC,
        ET,
        TT,
        GPST,
        GST,
        BDT,
    }

    #[derive(Debug)]
    enum Unit {
        Nanosecond,
        Microsecond,
        Millisecond,
        Second,
        Minute,
        Hour,
        Day,
        Week,
        Century,
    }

    extern "Rust" {
        type CxxDuration;
        type CxxEpoch;
        type CxxTimeSeries;

        // Duration methods
        fn duration_from_seconds(seconds: f64) -> Box<CxxDuration>;
        fn duration_from_unit(value: f64, unit: Unit) -> Box<CxxDuration>;
        fn total_seconds(self: &CxxDuration) -> f64;
        fn to_unit(self: &CxxDuration, unit: Unit) -> f64;
        fn duration_add(self: &CxxDuration, other: &CxxDuration) -> Box<CxxDuration>;
        fn duration_sub(self: &CxxDuration, other: &CxxDuration) -> Box<CxxDuration>;
        fn duration_neg(self: &CxxDuration) -> Box<CxxDuration>;
        fn duration_abs(self: &CxxDuration) -> Box<CxxDuration>;

        // Epoch methods
        fn epoch_from_str(s: &str) -> Result<Box<CxxEpoch>>;
        fn epoch_from_tai_seconds(seconds: f64) -> Box<CxxEpoch>;
        fn epoch_from_tai_days(days: f64) -> Box<CxxEpoch>;
        fn epoch_from_seconds(seconds: f64, ts: TimeScale) -> Box<CxxEpoch>;
        fn tai_seconds(self: &CxxEpoch) -> f64;
        fn tai_days(self: &CxxEpoch) -> f64;
        fn to_seconds(self: &CxxEpoch, ts: TimeScale) -> f64;
        fn to_days(self: &CxxEpoch, ts: TimeScale) -> f64;
        fn to_string(self: &CxxEpoch) -> String;
        fn epoch_add_duration(self: &CxxEpoch, duration: &CxxDuration) -> Box<CxxEpoch>;
        fn epoch_sub_duration(self: &CxxEpoch, duration: &CxxDuration) -> Box<CxxEpoch>;
        fn epoch_sub_epoch(self: &CxxEpoch, other: &CxxEpoch) -> Box<CxxDuration>;

        // TimeSeries methods
        fn time_series_new(
            start: &CxxEpoch,
            end: &CxxEpoch,
            step: &CxxDuration,
        ) -> Box<CxxTimeSeries>;
        fn next(self: &mut CxxTimeSeries) -> Result<Box<CxxEpoch>>;
        fn has_next(self: &CxxTimeSeries) -> bool;
    }
}

pub struct CxxDuration(pub Duration);

impl CxxDuration {
    fn total_seconds(&self) -> f64 {
        self.0.to_seconds()
    }
    fn to_unit(&self, unit: ffi::Unit) -> f64 {
        self.0.to_unit(Unit::from(unit))
    }
    fn duration_add(&self, other: &CxxDuration) -> Box<CxxDuration> {
        Box::new(CxxDuration(self.0 + other.0))
    }
    fn duration_sub(&self, other: &CxxDuration) -> Box<CxxDuration> {
        Box::new(CxxDuration(self.0 - other.0))
    }
    fn duration_neg(&self) -> Box<CxxDuration> {
        Box::new(CxxDuration(-self.0))
    }
    fn duration_abs(&self) -> Box<CxxDuration> {
        Box::new(CxxDuration(self.0.abs()))
    }
}

pub struct CxxEpoch(pub Epoch);

impl CxxEpoch {
    fn tai_seconds(&self) -> f64 {
        self.0.to_tai_seconds()
    }
    fn tai_days(&self) -> f64 {
        self.0.to_tai_days()
    }
    fn to_seconds(&self, ts: ffi::TimeScale) -> f64 {
        let ts = TimeScale::from(ts);
        match ts {
            TimeScale::TAI => self.0.to_tai_seconds(),
            TimeScale::TDB => self.0.to_tdb_seconds(),
            TimeScale::UTC => self.0.to_utc_seconds(),
            TimeScale::ET => self.0.to_et_seconds(),
            TimeScale::TT => self.0.to_tt_seconds(),
            TimeScale::GPST => self.0.to_gpst_seconds(),
            TimeScale::GST => self.0.to_gst_seconds(),
            TimeScale::BDT => self.0.to_bdt_seconds(),
            _ => self.0.to_tai_seconds(),
        }
    }
    fn to_days(&self, ts: ffi::TimeScale) -> f64 {
        let ts = TimeScale::from(ts);
        match ts {
            TimeScale::TAI => self.0.to_tai_days(),
            TimeScale::TDB => self.0.to_jde_tdb_days(),
            TimeScale::UTC => self.0.to_utc_days(),
            TimeScale::ET => self.0.to_tt_days(),
            TimeScale::TT => self.0.to_tt_days(),
            TimeScale::GPST => self.0.to_gpst_days(),
            TimeScale::GST => self.0.to_gst_days(),
            TimeScale::BDT => self.0.to_bdt_days(),
            _ => self.0.to_tai_days(),
        }
    }
    fn to_string(&self) -> String {
        format!("{}", self.0)
    }
    fn epoch_add_duration(&self, duration: &CxxDuration) -> Box<CxxEpoch> {
        Box::new(CxxEpoch(self.0 + duration.0))
    }
    fn epoch_sub_duration(&self, duration: &CxxDuration) -> Box<CxxEpoch> {
        Box::new(CxxEpoch(self.0 - duration.0))
    }
    fn epoch_sub_epoch(&self, other: &CxxEpoch) -> Box<CxxDuration> {
        Box::new(CxxDuration(self.0 - other.0))
    }
}

pub struct CxxTimeSeries {
    pub iter: TimeSeries,
    pub current: Option<Epoch>,
}

impl CxxTimeSeries {
    fn next(&mut self) -> Result<Box<CxxEpoch>, String> {
        if let Some(val) = self.current {
            self.current = self.iter.next();
            Ok(Box::new(CxxEpoch(val)))
        } else {
            Err("TimeSeries exhausted".to_string())
        }
    }

    fn has_next(&self) -> bool {
        self.current.is_some()
    }
}

impl From<ffi::Unit> for Unit {
    fn from(u: ffi::Unit) -> Self {
        match u {
            ffi::Unit::Nanosecond => Unit::Nanosecond,
            ffi::Unit::Microsecond => Unit::Microsecond,
            ffi::Unit::Millisecond => Unit::Millisecond,
            ffi::Unit::Second => Unit::Second,
            ffi::Unit::Minute => Unit::Minute,
            ffi::Unit::Hour => Unit::Hour,
            ffi::Unit::Day => Unit::Day,
            ffi::Unit::Week => Unit::Week,
            ffi::Unit::Century => Unit::Century,
            _ => Unit::Second,
        }
    }
}

impl From<ffi::TimeScale> for TimeScale {
    fn from(ts: ffi::TimeScale) -> Self {
        match ts {
            ffi::TimeScale::TAI => TimeScale::TAI,
            ffi::TimeScale::TDB => TimeScale::TDB,
            ffi::TimeScale::UTC => TimeScale::UTC,
            ffi::TimeScale::ET => TimeScale::ET,
            ffi::TimeScale::TT => TimeScale::TT,
            ffi::TimeScale::GPST => TimeScale::GPST,
            ffi::TimeScale::GST => TimeScale::GST,
            ffi::TimeScale::BDT => TimeScale::BDT,
            _ => TimeScale::TAI,
        }
    }
}

// Duration implementations
fn duration_from_seconds(seconds: f64) -> Box<CxxDuration> {
    Box::new(CxxDuration(Duration::from_seconds(seconds)))
}

fn duration_from_unit(value: f64, unit: ffi::Unit) -> Box<CxxDuration> {
    Box::new(CxxDuration(value * Unit::from(unit)))
}

// Epoch implementations
fn epoch_from_str(s: &str) -> Result<Box<CxxEpoch>, String> {
    Epoch::from_str(s)
        .map(|e| Box::new(CxxEpoch(e)))
        .map_err(|e| e.to_string())
}

fn epoch_from_tai_seconds(seconds: f64) -> Box<CxxEpoch> {
    Box::new(CxxEpoch(Epoch::from_tai_seconds(seconds)))
}

fn epoch_from_tai_days(days: f64) -> Box<CxxEpoch> {
    Box::new(CxxEpoch(Epoch::from_tai_days(days)))
}

fn epoch_from_seconds(seconds: f64, ts: ffi::TimeScale) -> Box<CxxEpoch> {
    let ts = TimeScale::from(ts);
    Box::new(CxxEpoch(match ts {
        TimeScale::TAI => Epoch::from_tai_seconds(seconds),
        TimeScale::TDB => Epoch::from_tdb_seconds(seconds),
        TimeScale::UTC => Epoch::from_utc_seconds(seconds),
        TimeScale::ET => Epoch::from_et_seconds(seconds),
        TimeScale::TT => Epoch::from_tt_seconds(seconds),
        TimeScale::GPST => Epoch::from_gpst_seconds(seconds),
        TimeScale::GST => Epoch::from_gst_seconds(seconds),
        TimeScale::BDT => Epoch::from_bdt_seconds(seconds),
        _ => Epoch::from_tai_seconds(seconds),
    }))
}

// TimeSeries implementations
fn time_series_new(start: &CxxEpoch, end: &CxxEpoch, step: &CxxDuration) -> Box<CxxTimeSeries> {
    let mut iter = TimeSeries::inclusive(start.0, end.0, step.0);
    let current = iter.next();
    Box::new(CxxTimeSeries { iter, current })
}
