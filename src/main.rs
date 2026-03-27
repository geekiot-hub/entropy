#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod device;
mod keyboard;
mod keycode;
mod keycode_picker;
#[cfg(not(target_arch = "wasm32"))]
mod hid;

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
        Box::new(|cc| {
            // Load DejaVu Sans for Unicode symbol support (▽ ✕ etc)
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "dejavu".to_owned(),
                egui::FontData::from_static(
                    include_bytes!("../assets/DejaVuSans.ttf")
                ).into(),
            );
            fonts.families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("dejavu".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(EntropyApp::new(cc)))
        }),
    )
}
