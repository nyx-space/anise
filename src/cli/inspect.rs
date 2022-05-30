use crc32fast::hash;
use hifitime::Epoch;
use std::fmt::{Display, Error as FmtError, Formatter};
use tabled::{Style, Table, Tabled};

use crate::{asn1::common::InterpolationKind, prelude::AniseContext};

/// A row is used only to display a context
#[derive(Tabled)]
struct Row<'a> {
    name: &'a str,
    data_kind: &'a str,
    hash: u32,
    start_epoch: Epoch,
    end_epoch: Epoch,
    interpolation_kind: InterpolationKind,
}

impl<'a> Display for AniseContext<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        writeln!(
            f,
            "ANISE version {}.{}.{}",
            self.metadata.anise_version.major,
            self.metadata.anise_version.minor,
            self.metadata.anise_version.patch
        )?;
        writeln!(
            f,
            "Originator: {}",
            if self.metadata.originator.is_empty() {
                "(not set)"
            } else {
                self.metadata.originator
            }
        )?;
        writeln!(f, "Creation date: {}", self.metadata.creation_date)?;
        writeln!(
            f,
            "Metadata URI: {}",
            if self.metadata.metadata_uri.is_empty() {
                "(not set)"
            } else {
                self.metadata.metadata_uri
            }
        )?;
        // Build the rows of the table
        let mut rows = Vec::new();
        // Add the ephemeris data
        for ephem in self.ephemeris_data.iter() {
            rows.push(Row {
                name: ephem.name,
                data_kind: "Ephemeris",
                hash: hash(ephem.name.as_bytes()),
                start_epoch: ephem.start_epoch().epoch,
                end_epoch: ephem.end_epoch().epoch,
                interpolation_kind: ephem.interpolation_kind,
            });
        }
        // Add the orientation data
        for orientation in self.orientation_data.iter() {
            rows.push(Row {
                name: orientation.name,
                data_kind: "Orientation",
                hash: hash(orientation.name.as_bytes()),
                start_epoch: orientation.start_epoch().epoch,
                end_epoch: orientation.end_epoch().epoch,
                interpolation_kind: orientation.interpolation_kind,
            });
        }
        let tbl = Table::new(rows).with(Style::rounded());
        write!(f, "{}", tbl)
    }
}
