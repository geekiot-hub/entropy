mod app;
mod device;
mod keyboard;
mod keycode;
mod keycode_picker;
#[cfg(not(target_arch = "wasm32"))]
mod vial;

use app::EntropyApp;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Entropy — Keyboard Configurator")
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Entropy",
        options,
        Box::new(|cc| Ok(Box::new(EntropyApp::new(cc)))),
    )
}
