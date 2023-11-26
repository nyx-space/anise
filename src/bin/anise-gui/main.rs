use pretty_env_logger;
use std::env::{set_var, var};

const LOG_VAR: &str = "ANISE_LOG";

mod ui;

use ui::UiApp;

fn main() {
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
