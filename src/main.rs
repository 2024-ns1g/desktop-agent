mod models;
mod api;
mod websocket;
mod gui;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use gui::{state::AppState, ui_main};

static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PresenStudio agent")
            .with_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_simple_native("PresenStudio agent", options, move |ctx, _frame| {
        ui_main(ctx);
    })
}

