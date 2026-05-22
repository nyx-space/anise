use anise::prelude::Almanac;
use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, SurfaceIndex};
use hifitime::TimeScale;

#[cfg(target_arch = "wasm32")]
use poll_promise::Promise;

use crate::{bpc::bpc_ui, epa::epa_ui, pca::pca_ui, spk::spk_ui};

#[cfg(target_arch = "wasm32")]
type AlmanacFile = Option<(String, Vec<u8>)>;

#[derive(Copy, Clone)]
pub enum Tab {
    Info,
    Analysis,
}

struct TabViewer<'a> {
    app: &'a mut UiApp,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::Info => "Info".into(),
            Tab::Analysis => "Analysis".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Info => self.app.info_tab_ui(ui),
            Tab::Analysis => self.app.analysis_tab_ui(ui),
        }
    }
}

pub struct UiApp {
    pub selected_time_scale: TimeScale,
    pub show_unix: bool,
    pub almanac: Almanac,
    /// List of loaded file paths and their associated aliases in the Almanac
    pub loaded_files: Vec<String>,
    /// The currently selected file for the Info tab
    pub selected_file: Option<String>,
    #[cfg(target_arch = "wasm32")]
    promise: Option<Promise<AlmanacFile>>,
    dock_state: DockState<Tab>,
}

impl Default for UiApp {
    fn default() -> Self {
        let mut dock_state = DockState::new(vec![Tab::Info, Tab::Analysis]);
        Self {
            selected_time_scale: TimeScale::UTC,
            show_unix: false,
            almanac: Default::default(),
            loaded_files: Vec::new(),
            selected_file: None,
            #[cfg(target_arch = "wasm32")]
            promise: Default::default(),
            dock_state,
        }
    }
}

enum FileLoadResult {
    NoFileSelectedYet,
    Ok((String, Box<Almanac>)),
    Error(String),
}

impl UiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        Self::default()
    }

    fn info_tab_ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::both().show(ui, |ui| {
            if let Some(alias) = &self.selected_file {
                ui.heading(format!("File: {alias}"));
                // Generic data
                let (label, crc) = if let Some(spk) = self.almanac.spk_data.get(alias) {
                    ("DAF/SPK", spk.crc32())
                } else if let Some(bpc) = self.almanac.bpc_data.get(alias) {
                    ("DAF/PCK", bpc.crc32())
                } else if let Some(pca) = self.almanac.planetary_data.get(alias) {
                    ("ANISE/PCA", pca.crc32())
                } else if let Some(epa) = self.almanac.euler_param_data.get(alias) {
                    ("ANISE/EPA", epa.crc32())
                } else {
                    ("UNKNOWN", 0)
                };

                ui.horizontal(|ui| {
                    ui.label("File type");
                    ui.label(label);
                    ui.label("CRC32");
                    ui.text_edit_singleline(&mut format!("{crc}"));
                });

                if label.starts_with("DAF/") {
                    ui.horizontal(|ui| {
                        ui.label("Time scale");
                        egui::ComboBox::new("ts_combo", "")
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
                            });
                        ui.checkbox(&mut self.show_unix, "UNIX timestamps");
                    });
                }

                ui.separator();

                if label == "DAF/PCK" {
                    bpc_ui(ui, &self.almanac, self.show_unix, self.selected_time_scale);
                } else if label == "DAF/SPK" {
                    spk_ui(ui, &self.almanac, self.show_unix, self.selected_time_scale);
                } else if label == "ANISE/PCA" {
                    pca_ui(ui, &self.almanac);
                } else if label == "ANISE/EPA" {
                    epa_ui(ui, &self.almanac);
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.label("Select a file from the sidebar to see its details.");
                });
            }
        });
    }

    fn analysis_tab_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Analysis tools coming soon...");
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_almanac(&mut self) -> FileLoadResult {
        if let Some(path_buf) = rfd::FileDialog::new().pick_file() {
            let path = path_buf.to_str().unwrap().to_string();
            match self.almanac.clone().load(&path) {
                Ok(almanac) => FileLoadResult::Ok((path, Box::new(almanac))),
                Err(e) => FileLoadResult::Error(e.to_string()),
            }
        } else {
            FileLoadResult::NoFileSelectedYet
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn load_almanac(&mut self) -> FileLoadResult {
        if let Some(promise) = self.promise.as_ref() {
            if let Some(result) = promise.ready() {
                let (file_name, data) = result.as_ref().map(|x| x.clone()).unwrap();
                self.promise = None;
                match self
                    .almanac
                    .clone()
                    .load_from_bytes(bytes::Bytes::from(data).into())
                {
                    Ok(almanac) => FileLoadResult::Ok((file_name, Box::new(almanac))),
                    Err(e) => FileLoadResult::Error(e.to_string()),
                }
            } else {
                FileLoadResult::NoFileSelectedYet
            }
        } else {
            self.promise = Some(Promise::spawn_local(async move {
                let fh = rfd::AsyncFileDialog::new().pick_file().await?;
                Some((fh.file_name(), fh.read().await))
            }));
            FileLoadResult::NoFileSelectedYet
        }
    }
}

impl eframe::App for UiApp {
    fn ui(&mut self, ctx: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.25);

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("ANISE v0.10");
                    ui.label("A modern rewrite of NASA's SPICE toolkit");
                    ui.hyperlink_to(
                        "https://www.nyxspace.com",
                        "https://www.nyxspace.com?utm_source=gui",
                    );
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(100.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Run log");
                });

                egui_logger::LoggerUi::default()
                    .enable_ctx_menu(false)
                    .enable_regex(false)
                    .show(ui);
            });

        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Loaded Files");
                if ui.button("Load file...").clicked() {
                    match self.load_almanac() {
                        FileLoadResult::Ok((path, almanac)) => {
                            self.almanac = *almanac;
                            self.loaded_files.push(path.clone());
                            self.selected_file = Some(path);
                        }
                        FileLoadResult::Error(e) => {
                            log::error!("{e}");
                        }
                        FileLoadResult::NoFileSelectedYet => {}
                    }
                }

                #[cfg(target_arch = "wasm32")]
                if self.promise.is_some() {
                    match self.load_almanac() {
                        FileLoadResult::Ok((path, almanac)) => {
                            self.almanac = *almanac;
                            self.loaded_files.push(path.clone());
                            self.selected_file = Some(path);
                        }
                        FileLoadResult::Error(e) => {
                            log::error!("{e}");
                        }
                        FileLoadResult::NoFileSelectedYet => {}
                    }
                }

                ui.separator();

                let mut to_unload = None;
                for file in &self.loaded_files {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.selected_file.as_ref() == Some(file), file)
                            .clicked()
                        {
                            self.selected_file = Some(file.clone());
                        }
                        if ui.button("x").clicked() {
                            to_unload = Some(file.clone());
                        }
                    });
                }

                if let Some(file) = to_unload {
                    self.almanac.unload(&file);
                    self.loaded_files.retain(|f| f != &file);
                    if self.selected_file.as_ref() == Some(&file) {
                        self.selected_file = self.loaded_files.last().cloned();
                    }
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            DockArea::new(&mut self.dock_state.clone())
                .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
                .show_inside(ui, &mut TabViewer { app: self });
        });
    }
}
