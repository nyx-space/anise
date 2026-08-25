use anise::prelude::{Almanac, NAIFSummaryRecord};
use egui_extras::{Column, TableBuilder};
use hifitime::{TimeScale, Unit};

pub fn spk_ui(
    ui: &mut egui::Ui,
    almanac: &Almanac,
    show_unix: bool,
    selected_time_scale: TimeScale,
) {
    let Some((_, spk)) = almanac.spk_data.get_index(0) else {
        ui.label(
            egui::RichText::new("No SPK data available in almanac").color(egui::Color32::KHAKI),
        );
        return;
    };

    let summary_size = match spk.file_record() {
        Ok(fr) => fr.summary_size(),
        Err(err) => {
            log::error!("Failed to read SPK file record: {err}");
            ui.label(egui::RichText::new("Corrupt SPK file record").color(egui::Color32::RED));
            return;
        }
    };

    TableBuilder::new(ui)
        .column(Column::auto().at_least(150.0).resizable(true))
        .column(Column::auto().at_least(150.0).resizable(true))
        .column(Column::auto().at_least(250.0).resizable(true))
        .column(Column::auto().at_least(250.0).resizable(true))
        .column(Column::auto().at_least(200.0).resizable(true))
        .column(Column::auto().at_least(150.0).resizable(true))
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("Target");
            });
            header.col(|ui| {
                ui.heading("Name");
            });
            header.col(|ui| {
                ui.heading("Start");
            });
            header.col(|ui| {
                ui.heading("End");
            });
            header.col(|ui| {
                ui.heading("Center");
            });
            header.col(|ui| {
                ui.heading("Domain");
            });
            header.col(|ui| {
                ui.heading("Type");
            });
        })
        .body(|mut body| {
            // NOTE: Using the explicit loop and index here to we can fetch the name record correctly.
            let mut idx = None;
            loop {
                // Fetch segment summaries safely
                let summaries = match spk.data_summaries(idx) {
                    Ok(s) => s,
                    Err(err) => {
                        log::error!("Failed to fetch SPK data summaries for index {idx:?}: {err}");
                        break;
                    }
                };

                let name_rcrd = match spk.name_record(idx) {
                    Ok(r) => Some(r),
                    Err(err) => {
                        log::warn!("Missing name record for index {idx:?}: {err}");
                        None
                    }
                };

                for (sno, summary) in summaries.iter().enumerate() {
                    if summary.is_empty() {
                        continue;
                    }

                    let name = name_rcrd
                        .as_ref()
                        .map(|r| r.nth_name(sno, summary_size))
                        .unwrap_or_else(|| "Unknown");

                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label(format!(
                                "{} ({})",
                                summary.target_frame(),
                                summary.target_id
                            ));
                        });
                        row.col(|ui| {
                            ui.label(name);
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

                        row.col(|ui| {
                            ui.label(format!(
                                "{} ({})",
                                summary.center_frame(),
                                summary.center_id
                            ));
                        });

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
                if let Ok(summary) = spk.daf_summary(idx) {
                    if summary.is_final_record() {
                        break;
                    } else {
                        idx = Some(summary.next_record());
                    }
                }
            }
        });
}
