use tabled::{settings::Style, Table, Tabled};

use crate::structure::EulerParameterDataSet;

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
                    Some(name) => format!("{name}"),
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
