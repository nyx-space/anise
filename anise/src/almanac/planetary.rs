/*
 * ANISE Toolkit
 * Copyright (C) 2021-onward Christopher Rabotin <christopher.rabotin@gmail.com> et al. (cf. AUTHORS.md)
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Documentation: https://nyxspace.com/
 */
use super::Almanac;
use snafu::prelude::*;
use tabled::{settings::Style, Table, Tabled};

use crate::{
    prelude::{Frame, FrameUid},
    structure::{dataset::DataSetError, PlanetaryDataSet},
};

#[derive(Debug, Snafu, PartialEq)]
#[snafu(visibility(pub(crate)))]
pub enum PlanetaryDataError {
    #[snafu(display("when {action}, {source}"))]
    PlanetaryDataSet {
        action: &'static str,
        source: DataSetError,
    },
}

impl Almanac {
    /// Given the frame UID (or something that can be transformed into it), attempt to retrieve the full frame information, if that frame is loaded
    pub fn frame_from_uid<U: Into<FrameUid>>(&self, uid: U) -> Result<Frame, PlanetaryDataError> {
        let uid = uid.into();
        Ok(self
            .planetary_data
            .get_by_id(uid.ephemeris_id)
            .context(PlanetaryDataSetSnafu {
                action: "fetching frame by its UID via ephemeris_id",
            })?
            .to_frame(uid))
    }

    /// Loads the provided planetary data into a clone of this original Almanac.
    pub fn with_planetary_data(&self, planetary_data: PlanetaryDataSet) -> Self {
        let mut me = self.clone();
        me.planetary_data = planetary_data;
        me
    }
}

#[derive(Tabled, Default)]
struct PlanetaryRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Gravity param (km^3/s^2)")]
    gm: String,
    #[tabled(rename = "Major axis (km)")]
    major_axis: String,
    #[tabled(rename = "Minor axis (km)")]
    minor_axis: String,
    #[tabled(rename = "Polar axis (km)")]
    polar_axis: String,
    #[tabled(rename = "Pole right asc.")]
    pole_ra: String,
    #[tabled(rename = "Pole declination")]
    pole_decl: String,
    #[tabled(rename = "Prime meridian")]
    pm: String,
}

impl PlanetaryDataSet {
    /// Returns a table describing this planetary data set
    pub fn describe(&self) -> String {
        let binding = self.lut.entries();
        let mut values = binding.values().collect::<Vec<_>>().to_vec();
        values.sort_by_key(|(opt_id, _)| match opt_id {
            Some(id) => *id,
            None => 0,
        });

        let mut rows = Vec::new();

        for (opt_id, opt_name) in values {
            let data = if let Some(id) = opt_id {
                self.get_by_id(*id).unwrap()
            } else {
                self.get_by_name(&opt_name.clone().unwrap()).unwrap()
            };

            let mut row = PlanetaryRow {
                name: match opt_name {
                    Some(name) => format!("{name}"),
                    None => "Unset".to_string(),
                },
                id: match opt_id {
                    Some(id) => format!("{id}"),
                    None => "Unset".to_string(),
                },
                gm: format!("{}", data.mu_km3_s2),
                pole_ra: match data.pole_right_ascension {
                    None => "Unset".to_string(),
                    Some(pole_ra) => format!("{pole_ra}"),
                },
                pole_decl: match data.pole_declination {
                    None => "Unset".to_string(),
                    Some(pole_dec) => format!("{pole_dec}"),
                },
                pm: match data.prime_meridian {
                    None => "Unset".to_string(),
                    Some(pm) => format!("{pm}"),
                },
                major_axis: "Unset".to_string(),
                minor_axis: "Unset".to_string(),
                polar_axis: "Unset".to_string(),
            };

            match data.shape {
                None => {
                    // Three empty columns -- don't change the data
                }
                Some(shape) => {
                    row.major_axis = format!("{}", shape.semi_major_equatorial_radius_km);
                    row.minor_axis = format!("{}", shape.semi_minor_equatorial_radius_km);
                    row.polar_axis = format!("{}", shape.polar_radius_km);
                }
            }

            rows.push(row);
        }

        let mut tbl = Table::new(rows);
        tbl.with(Style::modern());
        format!("{tbl}")
    }
}
