use anise::prelude::Almanac;
use egui_extras::{Column, TableBuilder};

pub fn epa_ui(ui: &mut egui::Ui, almanac: &Almanac) {
    TableBuilder::new(ui)
        .column(Column::auto().at_least(100.0).resizable(true))
        .column(Column::auto().at_least(75.0).resizable(true))
        .column(Column::auto().at_least(75.0).resizable(true))
        .column(Column::auto().at_least(75.0).resizable(true))
        .column(Column::auto().at_least(75.0).resizable(true))
        .column(Column::auto().at_least(75.0).resizable(true))
        .column(Column::auto().at_least(75.0).resizable(true))
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("Name");
            });
            header.col(|ui| {
                ui.heading("ID");
            });
            header.col(|ui| {
                ui.heading("Quat w");
            });

            header.col(|ui| {
                ui.heading("Quat x");
            });
            header.col(|ui| {
                ui.heading("Quat y");
            });
            header.col(|ui| {
                ui.heading("Quat z");
            });

            header.col(|ui| {
                ui.heading("From ID");
            });
            header.col(|ui| {
                ui.heading("To ID");
            });
        })
        .body(|mut body| {
            let epa = &almanac.euler_param_data;

            let binding = epa.lut.entries();
            let mut values = binding.values().collect::<Vec<_>>().to_vec();
            values.sort_by_key(|(opt_id, _)| match opt_id {
                Some(id) => *id,
                None => 0,
            });

            for (opt_id, opt_name) in values {
                let data = if let Some(id) = opt_id {
                    epa.get_by_id(*id).unwrap()
                } else {
                    epa.get_by_name(&opt_name.clone().unwrap()).unwrap()
                };

                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.label(match opt_name {
                            Some(name) => format!("{name}"),
                            None => "Unset".to_string(),
                        });
                    });

                    row.col(|ui| {
                        ui.label(match opt_id {
                            Some(id) => format!("{id}"),
                            None => "Unset".to_string(),
                        });
                    });

                    row.col(|ui| {
                        ui.text_edit_singleline(&mut format!("{}", data.w));
                    });

                    row.col(|ui| {
                        ui.text_edit_singleline(&mut format!("{}", data.x));
                    });

                    row.col(|ui| {
                        ui.text_edit_singleline(&mut format!("{}", data.y));
                    });

                    row.col(|ui| {
                        ui.text_edit_singleline(&mut format!("{}", data.z));
                    });

                    row.col(|ui| {
                        ui.text_edit_singleline(&mut format!("{}", data.from));
                    });

                    row.col(|ui| {
                        ui.text_edit_singleline(&mut format!("{}", data.to));
                    });
                })
            }
        });
}
