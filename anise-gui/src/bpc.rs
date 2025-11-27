use anise::{
    constants::orientations::orientation_name_from_id,
    prelude::{Almanac, NAIFSummaryRecord},
};
use egui_extras::{Column, TableBuilder};
use hifitime::{TimeScale, Unit};

pub fn bpc_ui(
    ui: &mut egui::Ui,
    almanac: &Almanac,
    show_unix: bool,
    selected_time_scale: TimeScale,
) {
    // We can use the summary
    TableBuilder::new(ui)
        .column(Column::auto().at_least(125.0).resizable(true))
        .column(Column::auto().at_least(125.0).resizable(true))
        .column(Column::auto().at_least(250.0).resizable(true))
        .column(Column::auto().at_least(250.0).resizable(true))
        .column(Column::auto().at_least(200.0).resizable(true))
        .column(Column::auto().at_least(150.0).resizable(true))
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("Frame");
            });
            header.col(|ui| {
                ui.heading("Segment name");
            });
            header.col(|ui| {
                ui.heading("Start");
            });
            header.col(|ui| {
                ui.heading("End");
            });
            header.col(|ui| {
                ui.heading("Inertial frame");
            });
            header.col(|ui| {
                ui.heading("Domain");
            });
            header.col(|ui| {
                ui.heading("Type");
            });
        })
        .body(|mut body| {
            let pck = almanac.bpc_data.get_index(0).unwrap().1;

            // NOTE: Using the explicit loop and index here to we can fetch the name record correctly.
            let mut idx = None;
            loop {
                for (sno, summary) in pck.data_summaries(idx).unwrap().iter().enumerate() {
                    let name_rcrd = pck.name_record(idx).unwrap();
                    let name = name_rcrd.nth_name(sno, pck.file_record().unwrap().summary_size());
                    if summary.is_empty() {
                        continue;
                    }

                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label(name);
                        });

                        row.col(|ui| match orientation_name_from_id(summary.frame_id) {
                            Some(name) => {
                                ui.label(format!("{name} ({})", summary.frame_id));
                            }
                            None => {
                                ui.label(format!("{}", summary.frame_id));
                            }
                        });

                        row.col(|ui| {
                            if show_unix {
                                ui.text_edit_singleline(&mut format!(
                                    "{}",
                                    summary.start_epoch().to_unix_seconds()
                                ));
                            } else {
                                ui.label(
                                    summary.start_epoch().to_gregorian_str(selected_time_scale),
                                );
                            };
                        });

                        row.col(|ui| {
                            if show_unix {
                                ui.text_edit_singleline(&mut format!(
                                    "{}",
                                    summary.end_epoch().to_unix_seconds()
                                ));
                            } else {
                                ui.label(summary.end_epoch().to_gregorian_str(selected_time_scale));
                            };
                        });

                        row.col(
                            |ui| match orientation_name_from_id(summary.inertial_frame_id) {
                                Some(name) => {
                                    ui.label(format!("{name} ({})", summary.inertial_frame_id));
                                }
                                None => {
                                    ui.label(format!("{}", summary.inertial_frame_id));
                                }
                            },
                        );

                        row.col(|ui| {
                            ui.label(format!(
                                "{}",
                                (summary.end_epoch() - summary.start_epoch())
                                    .round(Unit::Second * 1)
                            ));
                        });

                        row.col(|ui| {
                            ui.label(format!("{}", summary.data_type().unwrap()));
                        });
                    });
                }
                if let Ok(summary) = pck.daf_summary(idx) {
                    if summary.is_final_record() {
                        break;
                    } else {
                        idx = Some(summary.next_record());
                    }
                }
            }
        });
}
