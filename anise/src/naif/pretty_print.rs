use hifitime::{Duration, TimeScale, Unit};
use tabled::{settings::Style, Table, Tabled};

use crate::naif::daf::NAIFSummaryRecord;

use super::{BPC, SPK};

#[derive(Tabled)]
pub struct BpcRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Start epoch")]
    pub start_epoch: String,
    #[tabled(rename = "End epoch")]
    pub end_epoch: String,
    #[tabled(rename = "Duration")]
    pub duration: Duration,
    #[tabled(rename = "Interpolation kind")]
    pub interpolation_kind: String,
    #[tabled(rename = "Frame")]
    pub frame: String,
    #[tabled(rename = "Inertial frame")]
    pub inertial_frame: String,
}

#[derive(Tabled)]
pub struct SpkRow {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Target")]
    pub target: String,
    #[tabled(rename = "Center")]
    pub center: String,
    #[tabled(rename = "Start epoch")]
    pub start_epoch: String,
    #[tabled(rename = "End epoch")]
    pub end_epoch: String,
    #[tabled(rename = "Duration")]
    pub duration: Duration,
    #[tabled(rename = "Interpolation kind")]
    pub interpolation_kind: String,
}

pub trait NAIFPrettyPrint {
    fn describe(&self) -> String {
        self.describe_in(TimeScale::ET, None)
    }

    fn describe_in(&self, time_scale: TimeScale, round: Option<bool>) -> String;
}

impl NAIFPrettyPrint for BPC {
    /// Returns a string of a table representing this BPC where the epochs are printed in the provided time scale
    /// Set `round` to Some(false) to _not_ round the durations. By default, the durations will be rounded to the nearest second.
    fn describe_in(&self, time_scale: TimeScale, round: Option<bool>) -> String {
        // Build the rows of the table
        let mut rows = Vec::new();

        let round_value = if round.unwrap_or(true) {
            Unit::Second * 1.0_f64
        } else {
            Unit::Second * 0
        };

        // NOTE: Using the explicit loop and index here to we can fetch the name record correctly.
        let mut idx = None;
        let file_record = match self.file_record() {
            Ok(fr) => fr,
            Err(e) => return format!("Error reading file record: {e}"),
        };
        loop {
            let summaries = match self.data_summaries(idx) {
                Ok(s) => s,
                Err(e) => return format!("Error reading data summaries: {e}"),
            };
            let name_rcrd = match self.name_record(idx) {
                Ok(nr) => nr,
                Err(e) => return format!("Error reading name record: {e}"),
            };
            for (sno, summary) in summaries.iter().enumerate() {
                let name = name_rcrd.nth_name(sno, file_record.summary_size());
                if summary.is_empty() {
                    continue;
                }
                let interpolation_kind = match summary.data_type() {
                    Ok(dt) => dt.to_string(),
                    Err(e) => format!("Unknown({e})"),
                };
                rows.push(BpcRow {
                    name: name.to_string(),
                    start_epoch: summary
                        .start_epoch()
                        .to_gregorian_str(time_scale)
                        .to_string(),
                    end_epoch: summary.end_epoch().to_gregorian_str(time_scale).to_string(),
                    duration: (summary.end_epoch() - summary.start_epoch()).round(round_value),
                    interpolation_kind,
                    frame: format!("{}", summary.frame_id),
                    inertial_frame: format!("{}", summary.inertial_frame_id),
                });
            }
            if let Ok(summary) = self.daf_summary(idx) {
                if summary.is_final_record() {
                    break;
                } else {
                    idx = Some(summary.next_record());
                }
            }
        }

        let mut tbl = Table::new(rows);
        tbl.with(Style::modern());
        format!("{tbl}")
    }
}

impl NAIFPrettyPrint for SPK {
    /// Returns a string of a table representing this SPK where the epochs are printed in the provided time scale
    /// Set `round` to Some(false) to _not_ round the duration. By default, the durations will be rounded to the nearest second.
    fn describe_in(&self, time_scale: TimeScale, round: Option<bool>) -> String {
        // Build the rows of the table
        let mut rows = Vec::new();

        let round_value = if round.unwrap_or(true) {
            Unit::Second * 1.0_f64
        } else {
            Unit::Second * 0
        };

        // NOTE: Using the explicit loop and index here to we can fetch the name record correctly.
        let mut idx = None;
        let file_record = match self.file_record() {
            Ok(fr) => fr,
            Err(e) => return format!("Error reading file record: {e}"),
        };
        loop {
            let summaries = match self.data_summaries(idx) {
                Ok(s) => s,
                Err(e) => return format!("Error reading data summaries: {e}"),
            };
            let name_rcrd = match self.name_record(idx) {
                Ok(nr) => nr,
                Err(e) => return format!("Error reading name record: {e}"),
            };
            for (sno, summary) in summaries.iter().enumerate() {
                let name = name_rcrd.nth_name(sno, file_record.summary_size());
                if summary.is_empty() {
                    continue;
                }
                let interpolation_kind = match summary.data_type() {
                    Ok(dt) => dt.to_string(),
                    Err(e) => format!("Unknown({e})"),
                };

                rows.push(SpkRow {
                    name: name.to_string(),
                    center: summary.center_frame_uid().to_string(),
                    start_epoch: summary
                        .start_epoch()
                        .to_gregorian_str(time_scale)
                        .to_string(),
                    end_epoch: summary.end_epoch().to_gregorian_str(time_scale).to_string(),
                    duration: (summary.end_epoch() - summary.start_epoch()).round(round_value),
                    interpolation_kind,
                    target: summary.target_frame_uid().to_string(),
                });
            }
            if let Ok(summary) = self.daf_summary(idx) {
                if summary.is_final_record() {
                    break;
                } else {
                    idx = Some(summary.next_record());
                }
            }
        }
        let mut tbl = Table::new(rows);
        tbl.with(Style::sharp());
        format!("{tbl}")
    }
}
