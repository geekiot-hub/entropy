#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
pub(crate) mod app_icon;
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

const APP_TITLE: &str = "Entropy (v1.13.21)";

#[cfg(target_os = "windows")]
struct SingleInstanceGuard(*mut core::ffi::c_void);

#[cfg(target_os = "linux")]
struct SingleInstanceGuard(i32);

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

#[cfg(target_os = "linux")]
impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = flock(self.0, LOCK_UN);
            let _ = <std::fs::File as std::os::fd::FromRawFd>::from_raw_fd(self.0);
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

#[cfg(target_os = "linux")]
fn try_acquire_single_instance() -> bool {
    use std::fs::OpenOptions;
    use std::os::fd::IntoRawFd;

    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    if std::fs::create_dir_all(&dir).is_err() {
        return true;
    }

    let Ok(file) = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(dir.join("single_instance.lock"))
    else {
        return true;
    };
    let fd = file.into_raw_fd();
    let locked = unsafe { flock(fd, LOCK_EX | LOCK_NB) == 0 };
    if locked {
        let _guard = Box::leak(Box::new(SingleInstanceGuard(fd)));
        true
    } else {
        unsafe {
            let _ = <std::fs::File as std::os::fd::FromRawFd>::from_raw_fd(fd);
        }
        false
    }
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

#[cfg(target_os = "linux")]
const LOCK_EX: i32 = 2;
#[cfg(target_os = "linux")]
const LOCK_NB: i32 = 4;
#[cfg(target_os = "linux")]
const LOCK_UN: i32 = 8;

#[cfg(target_os = "linux")]
extern "C" {
    fn flock(fd: i32, operation: i32) -> i32;
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn try_acquire_single_instance() -> bool {
    true
}

fn main() -> eframe::Result<()> {
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    if hid::run_hid_proxy_if_requested() {
        return Ok(());
    }

    env_logger::init();

    #[cfg(target_os = "linux")]
    if !try_acquire_single_instance() {
        notify_existing_instance();
        return Ok(());
    }

    // Temporarily allow multiple Windows instances: a frozen HID session can otherwise keep
    // the global mutex and make a fixed build look like it "does not start".
    #[cfg(not(target_os = "linux"))]
    let _single_instance_available = try_acquire_single_instance();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_TITLE)
            .with_icon(app_icon::egui_icon(64))
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([800.0, 500.0]),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
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
