#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sortie::app::SortieApp;

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Starting Sortie Phase 1...");

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([700.0, 550.0])
            .with_title("Sortie — Grid Launcher & Project Bundler"),
        ..Default::default()
    };

    eframe::run_native(
        "Sortie",
        native_options,
        Box::new(|cc| Ok(Box::new(SortieApp::new(cc)))),
    )
}
