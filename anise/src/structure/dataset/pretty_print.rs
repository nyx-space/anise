use tabled::{settings::Style, Table, Tabled};

use crate::structure::{EulerParameterDataSet, LocationDataSet};

use super::NaifId;

#[derive(Tabled, Default)]
struct EulerParamRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Quat w")]
    qw: f64,
    #[tabled(rename = "Quat x")]
    qx: f64,
    #[tabled(rename = "Quat y")]
    qy: f64,
    #[tabled(rename = "Quat z")]
    qz: f64,
    #[tabled(rename = "To ID")]
    to: NaifId,
    #[tabled(rename = "From ID")]
    from: NaifId,
}

impl EulerParameterDataSet {
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

            let row = EulerParamRow {
                name: match opt_name {
                    Some(name) => name.clone(),
                    None => "Unset".to_string(),
                },
                id: match opt_id {
                    Some(id) => format!("{id}"),
                    None => "Unset".to_string(),
                },
                qw: data.w,
                qx: data.x,
                qy: data.y,
                qz: data.z,
                to: data.to,
                from: data.from,
            };

            rows.push(row);
        }

        let mut tbl = Table::new(rows);
        tbl.with(Style::modern());
        format!("{tbl}")
    }
}

#[derive(Tabled, Default)]
struct LocationRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Latitude (deg)")]
    latitude_deg: f64,
    #[tabled(rename = "Longitude (deg)")]
    longitude_deg: f64,
    #[tabled(rename = "Height (km)")]
    height_km: f64,
    #[tabled(rename = "Terrain Mask ?")]
    has_terrain_mask: bool,
    #[tabled(rename = "Terrain Mask Ignored")]
    terrain_mask_ignored: bool,
}

impl LocationDataSet {
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

            let row = LocationRow {
                name: match opt_name {
                    Some(name) => name.clone(),
                    None => "Unset".to_string(),
                },
                id: match opt_id {
                    Some(id) => format!("{id}"),
                    None => "Unset".to_string(),
                },
                latitude_deg: data.latitude_deg,
                longitude_deg: data.longitude_deg,
                height_km: data.height_km,
                has_terrain_mask: !data.terrain_mask.is_empty(),
                terrain_mask_ignored: data.terrain_mask_ignored,
            };

            rows.push(row);
        }

        let mut tbl = Table::new(rows);
        tbl.with(Style::modern());
        format!("{tbl}")
    }
}
