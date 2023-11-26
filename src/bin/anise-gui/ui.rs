use anise::almanac::Almanac;
use eframe::egui;
use egui::Align2;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

#[derive(Default)]
pub struct UiApp {
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
                                    self.path = Some(path.to_str().unwrap().to_string());
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
                            "DAF/BPC",
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
                        ui.horizontal(|ui| {
                            ui.label("CRC32");
                            ui.text_edit_singleline(&mut format!("{crc}"));
                        });
                    });
                }
            };

            // Show and update all toasts
            toasts.show(ctx);
        });
    }
}
