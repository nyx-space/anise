use crc32fast::hash;
use std::fmt::{Display, Error as FmtError, Formatter};
use tabled::{Style, Table, Tabled};

use crate::prelude::DataSet;
use crate::structure::orientation::orient_data::OrientationData;
use crate::structure::records::Record;
use crate::NaifId;

/// A row is used only to display a context
#[derive(Tabled)]
struct Row<'a> {
    name: &'a str,
    data_kind: &'a str,
    hash: NaifId,
    start_epoch: String,
    end_epoch: String,
    interpolation_kind: String,
}

impl<'a, R: Record<'a>> Display for DataSet<'a, R> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        writeln!(f, "{}", self.metadata)

        // // Build the rows of the table
        // let mut rows = Vec::new();
        // // Add the ephemeris data
        // for ephem in self.ephemeris_data.iter() {
        //     rows.push(Row {
        //         name: ephem.name,
        //         data_kind: "Ephemeris",
        //         hash: hash(ephem.name.as_bytes()),
        //         start_epoch: format!("{:?}", ephem.start_epoch()),
        //         end_epoch: format!("{:?}", ephem.end_epoch()),
        //         interpolation_kind: format!("{}", ephem.interpolation_kind),
        //     });
        // }
        // // Add the orientation data
        // for orientation in self.orientation_data.iter() {
        //     rows.push(Row {
        //         name: orientation.name,
        //         data_kind: "Orientation",
        //         hash: hash(orientation.name.as_bytes()),
        //         start_epoch: format!("{:?}", orientation.start_epoch()),
        //         end_epoch: format!("{:?}", orientation.end_epoch()),
        //         interpolation_kind: match orientation.orientation_data {
        //             OrientationData::PlanetaryConstant { .. } => {
        //                 "N/A (planetary constant)".to_string()
        //             }
        //             OrientationData::HighPrecision {
        //                 ref_epoch: _,
        //                 backward: _,
        //                 interpolation_kind,
        //                 splines: _,
        //             } => format!("{interpolation_kind}"),
        //         },
        //     });
        // }
        // let mut tbl = Table::new(rows);
        // tbl.with(Style::rounded());
        // write!(f, "{}", tbl)
    }
}
