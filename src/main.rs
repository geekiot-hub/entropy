#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod device;
mod firmware;
#[cfg(not(target_arch = "wasm32"))]
mod hid;
mod i18n;
mod keyboard;
mod keycode;
mod keycode_picker;
mod layouts;
mod popup_state;
#[cfg(not(target_arch = "wasm32"))]
mod qmk_hid_host;
mod smart_input;
mod text_expander;
mod ui_style;

use app::EntropyApp;

#[cfg(target_os = "windows")]
struct SingleInstanceGuard(*mut core::ffi::c_void);

#[cfg(target_os = "windows")]
impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                CloseHandle(self.0);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn try_acquire_single_instance() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    const ERROR_ALREADY_EXISTS: u32 = 183;
    let name: Vec<u16> = OsStr::new("Global\\EntropySingleInstance")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let handle = CreateMutexW(null_mut(), 1, name.as_ptr());
        if handle.is_null() {
            return true;
        }
        let already_exists = GetLastError() == ERROR_ALREADY_EXISTS;
        if already_exists {
            CloseHandle(handle);
            false
        } else {
            let _guard = Box::leak(Box::new(SingleInstanceGuard(handle)));
            true
        }
    }
}

fn notify_existing_instance() {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    let _ = std::fs::create_dir_all(&dir);
    let signal_path = dir.join("single_instance_signal");
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string());
    let _ = std::fs::write(signal_path, now_ms);
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn CreateMutexW(
        lpMutexAttributes: *mut core::ffi::c_void,
        bInitialOwner: i32,
        lpName: *const u16,
    ) -> *mut core::ffi::c_void;
    fn GetLastError() -> u32;
    fn CloseHandle(hObject: *mut core::ffi::c_void) -> i32;
}

#[cfg(not(target_os = "windows"))]
fn try_acquire_single_instance() -> bool {
    true
}

fn main() -> eframe::Result<()> {
    env_logger::init();

    // Temporarily allow multiple instances: a frozen HID session can otherwise keep
    // the global mutex and make a fixed build look like it "does not start".
    let _single_instance_available = try_acquire_single_instance();

    smart_input::start();

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
            // Roboto as primary UI font, with Unicode/symbol fallbacks.
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "roboto".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/Roboto-Regular.ttf")).into(),
            );
            fonts.font_data.insert(
                "dejavu".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/DejaVuSans.ttf")).into(),
            );
            fonts.font_data.insert(
                "noto_symbols".to_owned(),
                egui::FontData::from_static(include_bytes!(
                    "../assets/NotoSansSymbols2-Regular.ttf"
                ))
                .into(),
            );
            fonts.font_data.insert(
                "noto_emoji".to_owned(),
                egui::FontData::from_static(include_bytes!("../assets/NotoEmoji-subset.ttf"))
                    .into(),
            );
            let prop = fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default();
            prop.insert(0, "roboto".to_owned());
            prop.push("dejavu".to_owned());
            prop.push("noto_symbols".to_owned());
            prop.push("noto_emoji".to_owned());
            let mono = fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default();
            mono.push("dejavu".to_owned());
            mono.push("noto_symbols".to_owned());
            mono.push("noto_emoji".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(EntropyApp::new(cc)))
        }),
    )
}
