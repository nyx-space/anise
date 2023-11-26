use anise::{
    almanac::Almanac, constants::orientations::orientation_name_from_id,
    naif::daf::NAIFSummaryRecord,
};
use eframe::egui;
use egui::Align2;
use egui_extras::{Column, TableBuilder};
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use hifitime::TimeScale;

#[derive(Default)]
pub struct UiApp {
    pub selected_time_scale: TimeScale,
    almanac: Almanac,
    path: Option<String>,
}

impl UiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut toasts = Toasts::new()
            .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0)) // 10 units from the bottom right corner
            .direction(egui::Direction::BottomUp);

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("ANISE");
                    ui.label("A modern rewrite of NAIF SPICE");
                    ui.hyperlink_to(
                        "Take the user survey!",
                        "https://7ug5imdtt8v.typeform.com/to/qYDB14Hj",
                    );
                    ui.hyperlink("https://www.nyxspace.com");
                    ui.label("ANISE is open-sourced under the Mozilla Public License");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.vertical_centered(|ui| {
                        match &self.path {
                            None => {
                                // Show the open file dialog
                                if ui.button("Open file to inspect...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                                        // Try to load this file
                                        match self.almanac.load(path.to_str().unwrap()) {
                                            Ok(almanac) => {
                                                toasts.add(Toast {
                                                    text: format!("Loaded {path:?}").into(),
                                                    kind: ToastKind::Success,
                                                    options: ToastOptions::default()
                                                        .duration_in_seconds(15.0)
                                                        .show_progress(true),
                                                });
                                                self.almanac = almanac;
                                                self.path =
                                                    Some(path.to_str().unwrap().to_string());
                                            }
                                            Err(e) => {
                                                toasts.add(Toast {
                                                    text: format!("{e}").into(),
                                                    kind: ToastKind::Error,
                                                    options: ToastOptions::default()
                                                        .duration_in_seconds(15.0)
                                                        .show_progress(true),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            Some(path) => {
                                // Grab generic data
                                let (label, crc) = if self.almanac.num_loaded_spk() == 1 {
                                    (
                                        "DAF/SPK",
                                        self.almanac.spk_data[0].as_ref().unwrap().crc32(),
                                    )
                                } else if self.almanac.num_loaded_bpc() == 1 {
                                    (
                                        "DAF/PCK",
                                        self.almanac.bpc_data[0].as_ref().unwrap().crc32(),
                                    )
                                } else if !self.almanac.planetary_data.is_empty() {
                                    ("ANISE/PCK", self.almanac.planetary_data.crc32())
                                } else if !self.almanac.spacecraft_data.is_empty() {
                                    ("ANISE/SC", self.almanac.spacecraft_data.crc32())
                                } else if !self.almanac.euler_param_data.is_empty() {
                                    ("ANISE/EP", self.almanac.euler_param_data.crc32())
                                } else {
                                    ("UNKNOWN", 0)
                                };

                                ui.vertical(|ui| {
                                    ui.label(format!("Inspecting {path}"));
                                    ui.horizontal(|ui| {
                                        ui.label("File type");
                                        ui.label(label);
                                    });
                                    ui.spacing();
                                    ui.horizontal(|ui| {
                                        ui.label("CRC32");
                                        ui.text_edit_singleline(&mut format!("{crc}"));
                                    });
                                    ui.spacing();
                                    ui.horizontal(|ui| {
                                        ui.label("Time scale");
                                        egui::ComboBox::new("attention", "")
                                            .selected_text(format!("{}", self.selected_time_scale))
                                            .show_ui(ui, |ui| {
                                                for ts in [
                                                    TimeScale::UTC,
                                                    TimeScale::ET,
                                                    TimeScale::TDB,
                                                    TimeScale::TAI,
                                                    TimeScale::TT,
                                                ] {
                                                    ui.selectable_value(
                                                        &mut self.selected_time_scale,
                                                        ts,
                                                        format!("{ts}"),
                                                    );
                                                }
                                            })
                                    });
                                    ui.spacing();

                                    // Now diplay the data
                                    if label == "DAF/PCK" {
                                        // We can use the summary
                                        TableBuilder::new(ui)
                                            .column(Column::auto().at_least(125.0).resizable(true))
                                            .column(Column::auto().at_least(225.0).resizable(true))
                                            .column(Column::auto().at_least(225.0).resizable(true))
                                            .column(Column::auto().at_least(150.0).resizable(true))
                                            .column(Column::auto().at_least(50.0).resizable(true))
                                            .column(Column::auto().at_least(50.0).resizable(true))
                                            .column(Column::remainder())
                                            .header(20.0, |mut header| {
                                                header.col(|ui| {
                                                    ui.heading("Segment name");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("Start epoch");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("End epoch");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("Validity");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("Kind");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("Frame");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("Inertial frame");
                                                });
                                            })
                                            .body(|mut body| {
                                                let pck =
                                                    self.almanac.bpc_data[0].as_ref().unwrap();

                                                for (sno, summary) in
                                                    pck.data_summaries().unwrap().iter().enumerate()
                                                {
                                                    let name_rcrd = pck.name_record().unwrap();
                                                    let name = name_rcrd.nth_name(
                                                        sno,
                                                        pck.file_record().unwrap().summary_size(),
                                                    );
                                                    if summary.is_empty() {
                                                        continue;
                                                    }

                                                    body.row(30.0, |mut row| {
                                                        row.col(|ui| {
                                                            ui.label(name);
                                                        });

                                                        row.col(|ui| {
                                                            ui.label(
                                                                summary
                                                                    .start_epoch()
                                                                    .to_gregorian_str(
                                                                        self.selected_time_scale,
                                                                    ),
                                                            );
                                                        });

                                                        row.col(|ui| {
                                                            ui.label(
                                                                summary
                                                                    .end_epoch()
                                                                    .to_gregorian_str(
                                                                        self.selected_time_scale,
                                                                    ),
                                                            );
                                                        });

                                                        row.col(|ui| {
                                                            ui.label(format!(
                                                                "{}",
                                                                summary.end_epoch()
                                                                    - summary.start_epoch()
                                                            ));
                                                        });

                                                        row.col(|ui| {
                                                            ui.label(format!(
                                                                "{}",
                                                                summary.data_type().unwrap()
                                                            ));
                                                        });

                                                        row.col(
                                                            |ui| match orientation_name_from_id(
                                                                summary.frame_id,
                                                            ) {
                                                                Some(name) => {
                                                                    ui.label(format!(
                                                                        "{name} ({})",
                                                                        summary.frame_id
                                                                    ));
                                                                }
                                                                None => {
                                                                    ui.label(format!(
                                                                        "{}",
                                                                        summary.frame_id
                                                                    ));
                                                                }
                                                            },
                                                        );

                                                        row.col(
                                                            |ui| match orientation_name_from_id(
                                                                summary.inertial_frame_id,
                                                            ) {
                                                                Some(name) => {
                                                                    ui.label(format!(
                                                                        "{name} ({})",
                                                                        summary.inertial_frame_id
                                                                    ));
                                                                }
                                                                None => {
                                                                    ui.label(format!(
                                                                        "{}",
                                                                        summary.inertial_frame_id
                                                                    ));
                                                                }
                                                            },
                                                        );
                                                    });
                                                }
                                            });
                                    } else if label == "DAF/SPK" {
                                        // We can use the summary
                                        TableBuilder::new(ui)
                                            .column(Column::auto().at_least(125.0).resizable(true))
                                            .column(Column::auto().at_least(225.0).resizable(true))
                                            .column(Column::auto().at_least(225.0).resizable(true))
                                            .column(Column::auto().at_least(150.0).resizable(true))
                                            .column(Column::auto().at_least(50.0).resizable(true))
                                            .column(Column::auto().at_least(50.0).resizable(true))
                                            .column(Column::remainder())
                                            .header(20.0, |mut header| {
                                                header.col(|ui| {
                                                    ui.heading("Segment name");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("Start epoch");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("End epoch");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("Validity");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("Kind");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("Target");
                                                });
                                                header.col(|ui| {
                                                    ui.heading("Center");
                                                });
                                            })
                                            .body(|mut body| {
                                                let spk =
                                                    self.almanac.spk_data[0].as_ref().unwrap();

                                                for (sno, summary) in
                                                    spk.data_summaries().unwrap().iter().enumerate()
                                                {
                                                    let name_rcrd = spk.name_record().unwrap();
                                                    let name = name_rcrd.nth_name(
                                                        sno,
                                                        spk.file_record().unwrap().summary_size(),
                                                    );
                                                    if summary.is_empty() {
                                                        continue;
                                                    }

                                                    body.row(30.0, |mut row| {
                                                        row.col(|ui| {
                                                            ui.label(name);
                                                        });

                                                        row.col(|ui| {
                                                            ui.label(
                                                                summary
                                                                    .start_epoch()
                                                                    .to_gregorian_str(
                                                                        self.selected_time_scale,
                                                                    ),
                                                            );
                                                        });

                                                        row.col(|ui| {
                                                            ui.label(
                                                                summary
                                                                    .end_epoch()
                                                                    .to_gregorian_str(
                                                                        self.selected_time_scale,
                                                                    ),
                                                            );
                                                        });

                                                        row.col(|ui| {
                                                            ui.label(format!(
                                                                "{}",
                                                                summary.end_epoch()
                                                                    - summary.start_epoch()
                                                            ));
                                                        });

                                                        row.col(|ui| {
                                                            ui.label(format!(
                                                                "{}",
                                                                summary.data_type().unwrap()
                                                            ));
                                                        });

                                                        row.col(|ui| {
                                                            ui.label(format!(
                                                                "{} ({})",
                                                                summary.target_frame(),
                                                                summary.target_id
                                                            ));
                                                        });

                                                        row.col(|ui| {
                                                            ui.label(format!(
                                                                "{} ({})",
                                                                summary.center_frame(),
                                                                summary.center_id
                                                            ));
                                                        });
                                                    });
                                                }
                                            });
                                    }
                                });
                            }
                        };
                    });
                });

                // Show and update all toasts
            });
            toasts.show(ctx);
        });
    }
}
