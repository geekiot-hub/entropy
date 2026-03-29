#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod device;
mod firmware;
mod keyboard;
mod keycode;
mod keycode_picker;
mod layouts;
#[cfg(not(target_arch = "wasm32"))]
mod hid;
#[cfg(not(target_arch = "wasm32"))]
mod zmk;
#[cfg(not(target_arch = "wasm32"))]
mod zmk_proto;

use app::EntropyApp;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Entropy — Keyboard Configurator")
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([800.0, 500.0]),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Entropy",
        options,
        Box::new(|cc| {
            // DejaVu Sans as primary font + NotoSansSymbols2 as fallback for ⬅⬆⬇➡⌫⏎⏯⏭⏮▽⌘⌃⌥⇧ etc.
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "dejavu".to_owned(),
                egui::FontData::from_static(
                    include_bytes!("../assets/DejaVuSans.ttf")
                ).into(),
            );
            fonts.font_data.insert(
                "noto_symbols".to_owned(),
                egui::FontData::from_static(
                    include_bytes!("../assets/NotoSansSymbols2-Regular.ttf")
                ).into(),
            );
            fonts.font_data.insert(
                "noto_emoji".to_owned(),
                egui::FontData::from_static(
                    include_bytes!("../assets/NotoEmoji-subset.ttf")
                ).into(),
            );
            let prop = fonts.families
                .entry(egui::FontFamily::Proportional)
                .or_default();
            prop.insert(0, "dejavu".to_owned());
            prop.push("noto_symbols".to_owned());
            prop.push("noto_emoji".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(EntropyApp::new(cc)))
        }),
    )
}
