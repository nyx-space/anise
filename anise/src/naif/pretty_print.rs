use hifitime::{Duration, TimeScale};
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

impl BPC {
    /// Returns a string of a table representing this BPC where the epochs are printed in the TDB time scale
    pub fn describe(&self) -> String {
        self.describe_in(TimeScale::TDB)
    }

    /// Returns a string of a table representing this BPC where the epochs are printed in the provided time scale
    pub fn describe_in(&self, time_scale: TimeScale) -> String {
        // Build the rows of the table
        let mut rows = Vec::new();

        for (sno, summary) in self.data_summaries().unwrap().iter().enumerate() {
            let name_rcrd = self.name_record().unwrap();
            let name = name_rcrd.nth_name(sno, self.file_record().unwrap().summary_size());
            if summary.is_empty() {
                continue;
            }
            rows.push(BpcRow {
                name: name.to_string(),
                start_epoch: summary
                    .start_epoch()
                    .to_gregorian_str(time_scale)
                    .to_string(),
                end_epoch: summary.end_epoch().to_gregorian_str(time_scale).to_string(),
                duration: summary.end_epoch() - summary.start_epoch(),
                interpolation_kind: summary.data_type().unwrap().to_string(),
                frame: format!("{}", summary.frame_id),
                inertial_frame: format!("{}", summary.inertial_frame_id),
            });
        }

        let mut tbl = Table::new(rows);
        tbl.with(Style::modern());
        format!("{tbl}")
    }
}

impl SPK {
    /// Returns a string of a table representing this SPK where the epochs are printed in the TDB time scale
    pub fn describe(&self) -> String {
        self.describe_in(TimeScale::TDB)
    }

    /// Returns a string of a table representing this SPK where the epochs are printed in the provided time scale
    pub fn describe_in(&self, time_scale: TimeScale) -> String {
        // Build the rows of the table
        let mut rows = Vec::new();

        for (sno, summary) in self.data_summaries().unwrap().iter().enumerate() {
            let name_rcrd = self.name_record().unwrap();
            let name = name_rcrd.nth_name(sno, self.file_record().unwrap().summary_size());
            if summary.is_empty() {
                continue;
            }

            rows.push(SpkRow {
                name: name.to_string(),
                center: summary.center_frame_uid().to_string(),
                start_epoch: summary
                    .start_epoch()
                    .to_gregorian_str(time_scale)
                    .to_string(),
                end_epoch: summary.end_epoch().to_gregorian_str(time_scale).to_string(),
                duration: summary.end_epoch() - summary.start_epoch(),
                interpolation_kind: summary.data_type().unwrap().to_string(),
                target: summary.target_frame_uid().to_string(),
            });
        }

        let mut tbl = Table::new(rows);
        tbl.with(Style::modern());
        format!("{tbl}")
    }
}
