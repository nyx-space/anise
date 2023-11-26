use anise::almanac::Almanac;
use eframe::{egui, Theme};
use egui::Align2;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};

#[derive(Default)]
pub struct UiApp {
    almanac: Almanac,
    path: Option<String>,
    dropped_files: Vec<egui::DroppedFile>,
}

impl UiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        cc.egui_ctx.set_visuals(Theme::Light.egui_visuals());
        Self::default()
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut toasts = Toasts::new()
            .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0)) // 10 units from the bottom right corner
            .direction(egui::Direction::BottomUp);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ANISE -- a modern SPICE rewrite by Nyx Space");
            ui.hyperlink("https://nyxspace.com");

            if ui.button("Open file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    // Try to load this file
                    match self.almanac.load(path.to_str().unwrap()) {
                        Ok(almanac) => {
                            toasts.add(Toast {
                                text: format!("Loaded {path:?}").into(),
                                kind: ToastKind::Success,
                                options: ToastOptions::default()
                                    .duration_in_seconds(5.0)
                                    .show_progress(true),
                            });
                            self.almanac = almanac
                        }
                        Err(e) => {
                            toasts.add(Toast {
                                text: format!("{e}").into(),
                                kind: ToastKind::Error,
                                options: ToastOptions::default()
                                    .duration_in_seconds(5.0)
                                    .show_progress(true),
                            });
                        }
                    }
                }
            }

            // Show and update all toasts
            toasts.show(ctx);
        });
    }
}
