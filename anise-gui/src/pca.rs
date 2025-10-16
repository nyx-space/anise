use anise::prelude::Almanac;
use egui_extras::{Column, TableBuilder};

pub fn pca_ui(ui: &mut egui::Ui, almanac: &Almanac) {
    TableBuilder::new(ui)
        .column(Column::auto().at_least(100.0).resizable(true))
        .column(Column::auto().at_least(50.0).resizable(true))
        .column(Column::auto().at_least(75.0).resizable(true))
        .column(Column::auto().at_least(75.0).resizable(true))
        .column(Column::auto().at_least(75.0).resizable(true))
        .column(Column::auto().at_least(125.0).resizable(true))
        .column(Column::auto().at_least(125.0).resizable(true))
        .column(Column::auto().at_least(125.0).resizable(true))
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("Name");
            });
            header.col(|ui| {
                ui.heading("ID");
            });
            header.col(|ui| {
                ui.heading("Gravity param (km^3/s^2)");
            });

            header.col(|ui| {
                ui.heading("Major axis (km)");
            });
            header.col(|ui| {
                ui.heading("Minor axis (km)");
            });
            header.col(|ui| {
                ui.heading("Polar axis (km)");
            });

            header.col(|ui| {
                ui.heading("Pole right asc.");
            });
            header.col(|ui| {
                ui.heading("Pole declination");
            });
            header.col(|ui| {
                ui.heading("Prime meridian");
            });
        })
        .body(|mut body| {
            let pck = &almanac.planetary_data;

            let binding = pck.lut.entries();
            let mut values = binding.values().collect::<Vec<_>>().to_vec();
            values.sort_by_key(|(opt_id, _)| match opt_id {
                Some(id) => *id,
                None => 0,
            });

            for (opt_id, opt_name) in values {
                let data = if let Some(id) = opt_id {
                    pck.get_by_id(*id).unwrap()
                } else {
                    pck.get_by_name(&opt_name.clone().unwrap()).unwrap()
                };

                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.label(match opt_name {
                            Some(name) => name.to_string(),
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
                        ui.text_edit_singleline(&mut format!("{}", data.mu_km3_s2));
                    });

                    match data.shape {
                        None => {
                            // Three empty columns
                            row.col(|ui| {
                                ui.label("Unset");
                            });
                            row.col(|ui| {
                                ui.label("Unset");
                            });
                            row.col(|ui| {
                                ui.label("Unset");
                            });
                        }
                        Some(shape) => {
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut format!(
                                    "{}",
                                    shape.semi_major_equatorial_radius_km
                                ));
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut format!(
                                    "{}",
                                    shape.semi_minor_equatorial_radius_km
                                ));
                            });
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut format!("{}", shape.polar_radius_km));
                            });
                        }
                    }

                    match data.pole_right_ascension {
                        None => row.col(|ui| {
                            ui.label("Unset");
                        }),
                        Some(pole_ra) => row.col(|ui| {
                            ui.label(format!("{pole_ra}"));
                        }),
                    };

                    match data.pole_declination {
                        None => row.col(|ui| {
                            ui.label("Unset");
                        }),
                        Some(pole_dec) => row.col(|ui| {
                            ui.label(format!("{pole_dec}"));
                        }),
                    };

                    match data.prime_meridian {
                        None => row.col(|ui| {
                            ui.label("Unset");
                        }),
                        Some(pm) => row.col(|ui| {
                            ui.label(format!("{pm}"));
                        }),
                    };
                });
            }
        });
}
