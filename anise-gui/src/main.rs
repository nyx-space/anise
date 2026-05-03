#![windows_subsystem = "windows"]
#[allow(dead_code)]
const LOG_VAR: &str = "ANISE_LOG";

mod ui;
use ui::UiApp;

mod bpc;
mod epa;
mod pca;
mod spk;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use std::env::{set_var, var};

    if var(LOG_VAR).is_err() {
        unsafe {
            set_var(LOG_VAR, "INFO");
        }
    }

    // Initialize the logger
    egui_logger::builder()
        .init()
        .expect("Error initializing logger");

    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 640.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../icon-256.png")[..]).unwrap(),
            ),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "ANISE by Nyx Space",
        opts,
        Box::new(|cc| Ok(Box::new(UiApp::new(cc)))),
    );
}

// Entrypoint for WebAssembly
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast;
    use eframe::web_sys;
    use log::info;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();

    info!("Starting ANISE in WebAssembly mode");
    wasm_bindgen_futures::spawn_local(async {
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("anise_canvas"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("failed to find canvas element");
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(UiApp::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });
}
