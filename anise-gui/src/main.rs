#[allow(dead_code)]
const LOG_VAR: &str = "ANISE_LOG";

mod aesthetix;
mod ui;
use ui::UiApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use std::env::{set_var, var};

    if var(LOG_VAR).is_err() {
        set_var(LOG_VAR, "INFO");
    }

    let _ = pretty_env_logger::try_init_custom_env(LOG_VAR).is_err();

    let _ = eframe::run_native(
        "ANISE by Nyx Space",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(UiApp::new(cc))),
    );
}

// Entrypoint for WebAssembly
#[cfg(target_arch = "wasm32")]
fn main() {
    use log::info;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();

    info!("Starting ANISE in WebAssembly mode");
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "anise_canvas",
                web_options,
                Box::new(|cc| Box::new(UiApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
