#![allow(non_snake_case)]

use crate::text_expander::{TextExpansionConfig, TextExpansionEngine, TextExpansionRule};
use std::sync::{Mutex, OnceLock, RwLock};

#[path = "smart_input_symbols.rs"]
mod smart_input_symbols;
#[cfg(target_os = "windows")]
#[path = "smart_input_windows.rs"]
mod smart_input_windows;
pub use smart_input_symbols::{smart_symbol_for_keycode, SmartSymbol, SMART_SYMBOLS};
use smart_input_symbols::{KC_F13, MOD_ALT, MOD_CTRL, MOD_SHIFT};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextExpanderAppCandidate {
    pub exe: String,
    pub title: String,
}

static TEXT_EXPANDER_CONFIG: OnceLock<RwLock<TextExpansionConfig>> = OnceLock::new();
static TEXT_EXPANDER_ENGINE: OnceLock<Mutex<TextExpansionEngine>> = OnceLock::new();
static RECENT_FOREGROUND_APPS: OnceLock<Mutex<Vec<TextExpanderAppCandidate>>> = OnceLock::new();

fn text_expander_config() -> &'static RwLock<TextExpansionConfig> {
    TEXT_EXPANDER_CONFIG.get_or_init(|| RwLock::new(TextExpansionConfig::default()))
}

fn text_expander_engine() -> &'static Mutex<TextExpansionEngine> {
    TEXT_EXPANDER_ENGINE.get_or_init(|| Mutex::new(TextExpansionEngine::default()))
}

fn recent_foreground_apps() -> &'static Mutex<Vec<TextExpanderAppCandidate>> {
    RECENT_FOREGROUND_APPS.get_or_init(|| Mutex::new(Vec::new()))
}

fn current_process_name_lower() -> Option<String> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.file_name().map(|name| name.to_owned()))
        .and_then(|name| name.to_str().map(|name| name.to_ascii_lowercase()))
}

fn remember_foreground_app(candidate: TextExpanderAppCandidate) {
    let exe = candidate.exe.trim().to_ascii_lowercase();
    if exe.is_empty() {
        return;
    }
    if current_process_name_lower().as_deref() == Some(exe.as_str()) {
        return;
    }
    let title = candidate.title.trim().to_owned();
    if let Ok(mut apps) = recent_foreground_apps().lock() {
        apps.retain(|app| app.exe != exe);
        apps.insert(0, TextExpanderAppCandidate { exe, title });
        apps.truncate(12);
    }
}

pub fn text_expander_app_candidates() -> Vec<TextExpanderAppCandidate> {
    let mut apps = platform_open_window_candidates();
    if let Ok(recent) = recent_foreground_apps().lock() {
        for candidate in recent.iter().rev() {
            if !apps.iter().any(|app| app.exe == candidate.exe) {
                apps.insert(0, candidate.clone());
            }
        }
    }
    apps.truncate(16);
    apps
}

pub fn set_text_expander_config(
    enabled: bool,
    rules: Vec<TextExpansionRule>,
    app_blacklist: Vec<String>,
) {
    let config = TextExpansionConfig {
        enabled,
        rules: rules.clone(),
        app_blacklist: app_blacklist
            .into_iter()
            .map(|name| name.trim().to_ascii_lowercase())
            .filter(|name| !name.is_empty())
            .collect(),
    };
    if let Ok(mut guard) = text_expander_config().write() {
        *guard = config;
    }
    if let Ok(mut engine) = text_expander_engine().lock() {
        engine.set_rules(rules);
    }
}

fn text_expander_enabled() -> bool {
    text_expander_config()
        .read()
        .map(|config| config.enabled && config.rules.iter().any(|rule| rule.enabled))
        .unwrap_or(false)
}

fn text_expander_suppressed_for_context() -> bool {
    text_expander_config()
        .read()
        .map(|config| foreground_app_blacklisted(&config.app_blacklist))
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn foreground_app_blacklisted(app_blacklist: &[String]) -> bool {
    smart_input_windows::foreground_app_blacklisted(app_blacklist)
}

#[cfg(target_os = "windows")]
pub fn platform_open_window_candidates() -> Vec<TextExpanderAppCandidate> {
    smart_input_windows::platform_open_window_candidates()
}

#[cfg(not(target_os = "windows"))]
fn foreground_app_blacklisted(_app_blacklist: &[String]) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn platform_open_window_candidates() -> Vec<TextExpanderAppCandidate> {
    Vec::new()
}

#[cfg(target_os = "windows")]
pub fn universal_output_status() -> String {
    "Universal output backend: Windows native".to_owned()
}

#[cfg(target_os = "macos")]
pub fn universal_output_status() -> String {
    "Universal output backend: macOS native — requires Accessibility/Input Monitoring permission"
        .to_owned()
}

#[cfg(target_os = "linux")]
pub fn universal_output_status() -> String {
    let session = linux_session_kind();
    let input_method = linux_input_method_hint();
    match session {
        LinuxSessionKind::Wayland => format!(
            "Universal output backend: Wayland via IBus/Fcitx5 input method{}",
            input_method
        ),
        LinuxSessionKind::X11 => {
            "Universal output backend: Linux X11 native; Wayland uses IBus/Fcitx5".to_owned()
        }
        LinuxSessionKind::Unknown => format!(
            "Universal output backend: Linux; use IBus/Fcitx5 for Wayland{}",
            input_method
        ),
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn universal_output_status() -> String {
    "Universal output backend: unsupported on this OS".to_owned()
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinuxSessionKind {
    Wayland,
    X11,
    Unknown,
}

#[cfg(target_os = "linux")]
fn linux_session_kind() -> LinuxSessionKind {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        LinuxSessionKind::Wayland
    } else if std::env::var_os("DISPLAY").is_some() {
        LinuxSessionKind::X11
    } else {
        LinuxSessionKind::Unknown
    }
}

#[cfg(target_os = "linux")]
pub fn text_expander_runs_outside_entropy_process() -> bool {
    matches!(linux_session_kind(), LinuxSessionKind::Wayland)
}

#[cfg(not(target_os = "linux"))]
pub fn text_expander_runs_outside_entropy_process() -> bool {
    false
}

#[cfg(target_os = "linux")]
fn linux_input_method_hint() -> &'static str {
    let im_vars = ["GTK_IM_MODULE", "QT_IM_MODULE", "XMODIFIERS"];
    let combined = im_vars
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    if combined.contains("fcitx") {
        " — Fcitx detected"
    } else if combined.contains("ibus") {
        " — IBus detected"
    } else {
        ""
    }
}

#[cfg(target_os = "macos")]
pub fn universal_output_setup_hint() -> Option<&'static str> {
    Some("Open Config → Universal Symbols to finish permissions setup")
}

#[cfg(target_os = "linux")]
pub fn universal_output_setup_hint() -> Option<&'static str> {
    Some("Open Config → Universal Symbols to finish Linux setup")
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn universal_output_setup_hint() -> Option<&'static str> {
    None
}

fn smart_symbol_for_transport(
    base_keycode: u16,
    ctrl: bool,
    shift: bool,
    alt: bool,
) -> Option<(char, u16)> {
    let mut trigger_keycode = base_keycode;
    if ctrl {
        trigger_keycode |= MOD_CTRL;
    }
    if shift {
        trigger_keycode |= MOD_SHIFT;
    }
    if alt {
        trigger_keycode |= MOD_ALT;
    }
    smart_symbol_for_keycode(trigger_keycode).map(|symbol| (symbol.symbol, trigger_keycode))
}

#[cfg(target_os = "windows")]
pub fn start() {
    smart_input_windows::start();
}

#[cfg(target_os = "macos")]
pub fn start() {
    use std::sync::Once;
    static START: Once = Once::new();
    START.call_once(|| {
        std::thread::spawn(|| unsafe {
            macos::run_event_tap();
        });
    });
}

#[cfg(target_os = "linux")]
pub fn start() {
    use std::sync::Once;
    static START: Once = Once::new();
    START.call_once(|| {
        std::thread::spawn(linux_x11::run_x11_loop);
    });
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn start() {}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::ffi::c_void;
    use std::ptr::null_mut;

    type CGEventTapProxy = *mut c_void;
    type CGEventRef = *mut c_void;
    type CFMachPortRef = *mut c_void;
    type CFRunLoopSourceRef = *mut c_void;
    type CFRunLoopRef = *mut c_void;
    type CFStringRef = *const c_void;

    const K_CG_HID_EVENT_TAP: u32 = 0;
    const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
    const K_CG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;
    const K_CG_EVENT_KEY_DOWN: u32 = 10;
    const K_CG_EVENT_KEY_UP: u32 = 11;
    const K_CG_KEYBOARD_EVENT_KEYCODE: i32 = 9;
    const K_CG_EVENT_FLAG_MASK_SHIFT: u64 = 1 << 17;
    const K_CG_EVENT_FLAG_MASK_CONTROL: u64 = 1 << 18;
    const K_CG_EVENT_FLAG_MASK_ALTERNATE: u64 = 1 << 19;
    const K_CG_ANNOTATED_SESSION_EVENT_TAP: u32 = 1;

    pub unsafe fn run_event_tap() {
        let mask = (1u64 << K_CG_EVENT_KEY_DOWN) | (1u64 << K_CG_EVENT_KEY_UP);
        let tap = CGEventTapCreate(
            K_CG_HID_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_DEFAULT,
            mask,
            Some(event_tap_callback),
            null_mut(),
        );
        if tap.is_null() {
            log::warn!("Smart Input: macOS event tap failed; Accessibility/Input Monitoring permission may be required");
            return;
        }

        let source = CFMachPortCreateRunLoopSource(null_mut(), tap, 0);
        if source.is_null() {
            log::warn!("Smart Input: macOS run-loop source creation failed");
            CFRelease(tap as *const c_void);
            return;
        }

        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
        CFRunLoopRun();
    }

    unsafe extern "C" fn event_tap_callback(
        _proxy: CGEventTapProxy,
        event_type: u32,
        event: CGEventRef,
        _user_info: *mut c_void,
    ) -> CGEventRef {
        if event_type != K_CG_EVENT_KEY_DOWN && event_type != K_CG_EVENT_KEY_UP {
            return event;
        }
        let keycode = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u16;
        let Some(base_keycode) = mac_keycode_to_qmk_f_key(keycode) else {
            return event;
        };
        let flags = CGEventGetFlags(event);
        let ctrl = flags & K_CG_EVENT_FLAG_MASK_CONTROL != 0;
        let shift = flags & K_CG_EVENT_FLAG_MASK_SHIFT != 0;
        let alt = flags & K_CG_EVENT_FLAG_MASK_ALTERNATE != 0;
        if let Some((symbol, _trigger_keycode)) =
            smart_symbol_for_transport(base_keycode, ctrl, shift, alt)
        {
            if event_type == K_CG_EVENT_KEY_DOWN {
                send_unicode_char(symbol);
            }
            return null_mut();
        }
        event
    }

    fn mac_keycode_to_qmk_f_key(keycode: u16) -> Option<u16> {
        let offset = match keycode {
            0x69 => 0, // F13
            0x6B => 1, // F14
            0x71 => 2, // F15
            0x6A => 3, // F16
            0x40 => 4, // F17
            0x4F => 5, // F18
            0x50 => 6, // F19
            0x5A => 7, // F20
            // F21..F24 are not declared by HIToolbox, but external keyboards
            // may still surface them through CGEvent with these adjacent codes.
            0x5B => 8,
            0x5C => 9,
            0x5D => 10,
            0x5E => 11,
            _ => return None,
        };
        Some(KC_F13 + offset)
    }

    unsafe fn send_unicode_char(symbol: char) {
        let mut buffer = [0u16; 2];
        let units = symbol.encode_utf16(&mut buffer);
        let event = CGEventCreateKeyboardEvent(null_mut(), 0, true);
        if event.is_null() {
            return;
        }
        CGEventSetFlags(event, 0);
        CGEventKeyboardSetUnicodeString(event, units.len(), units.as_ptr());
        CGEventPost(K_CG_ANNOTATED_SESSION_EVENT_TAP, event);
        CFRelease(event as *const c_void);
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventTapCreate(
            tap: u32,
            place: u32,
            options: u32,
            eventsOfInterest: u64,
            callback: Option<
                unsafe extern "C" fn(CGEventTapProxy, u32, CGEventRef, *mut c_void) -> CGEventRef,
            >,
            userInfo: *mut c_void,
        ) -> CFMachPortRef;
        fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
        fn CGEventGetIntegerValueField(event: CGEventRef, field: i32) -> i64;
        fn CGEventGetFlags(event: CGEventRef) -> u64;
        fn CGEventSetFlags(event: CGEventRef, flags: u64);
        fn CGEventCreateKeyboardEvent(
            source: *mut c_void,
            virtualKey: u16,
            keyDown: bool,
        ) -> CGEventRef;
        fn CGEventKeyboardSetUnicodeString(
            event: CGEventRef,
            stringLength: usize,
            unicodeString: *const u16,
        );
        fn CGEventPost(tap: u32, event: CGEventRef);
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        static kCFRunLoopCommonModes: CFStringRef;
        fn CFMachPortCreateRunLoopSource(
            allocator: *mut c_void,
            port: CFMachPortRef,
            order: isize,
        ) -> CFRunLoopSourceRef;
        fn CFRunLoopGetCurrent() -> CFRunLoopRef;
        fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
        fn CFRunLoopRun();
        fn CFRelease(cf: *const c_void);
    }
}

#[cfg(target_os = "linux")]
mod linux_x11 {
    use super::*;
    use std::process::Command;

    const XK_F13: u64 = 0xffca;
    const SHIFT_MASK: u32 = 1;
    const LOCK_MASK: u32 = 2;
    const CONTROL_MASK: u32 = 4;
    const MOD1_MASK: u32 = 8;
    const KEY_PRESS: i32 = 2;
    const KEY_RELEASE: i32 = 3;

    pub fn run_x11_loop() {
        unsafe {
            let Ok(xlib) = x11_dl::xlib::Xlib::open() else {
                log::warn!("Smart Input: Xlib is unavailable");
                return;
            };
            let display = (xlib.XOpenDisplay)(std::ptr::null());
            if display.is_null() {
                log::warn!("Smart Input: X11 display is unavailable; Wayland is not supported yet");
                return;
            }
            let root = (xlib.XDefaultRootWindow)(display);
            let mut keycodes = Vec::new();
            for idx in 0..12u16 {
                let keycode = (xlib.XKeysymToKeycode)(display, XK_F13 + idx as u64);
                if keycode == 0 {
                    continue;
                }
                keycodes.push((keycode, KC_F13 + idx));
                for modifiers in transport_modifier_masks() {
                    (xlib.XGrabKey)(
                        display,
                        keycode as i32,
                        modifiers,
                        root,
                        1,
                        x11_dl::xlib::GrabModeAsync,
                        x11_dl::xlib::GrabModeAsync,
                    );
                }
            }
            (xlib.XFlush)(display);

            loop {
                let mut event: x11_dl::xlib::XEvent = std::mem::zeroed();
                (xlib.XNextEvent)(display, &mut event);
                let event_type = event.get_type();
                if event_type != KEY_PRESS && event_type != KEY_RELEASE {
                    continue;
                }
                let xkey = event.key;
                let Some((_, base_keycode)) = keycodes
                    .iter()
                    .find(|(keycode, _)| *keycode == xkey.keycode as u8)
                else {
                    continue;
                };
                let ctrl = xkey.state & CONTROL_MASK != 0;
                let shift = xkey.state & SHIFT_MASK != 0;
                let alt = xkey.state & MOD1_MASK != 0;
                if let Some((symbol, _trigger_keycode)) =
                    smart_symbol_for_transport(*base_keycode, ctrl, shift, alt)
                {
                    if event_type == KEY_PRESS {
                        type_unicode(symbol);
                    }
                }
            }
        }
    }

    fn transport_modifier_masks() -> Vec<u32> {
        let base_masks = [
            0,
            SHIFT_MASK,
            CONTROL_MASK,
            MOD1_MASK,
            CONTROL_MASK | SHIFT_MASK,
            CONTROL_MASK | MOD1_MASK,
            SHIFT_MASK | MOD1_MASK,
            CONTROL_MASK | SHIFT_MASK | MOD1_MASK,
        ];
        let lock_masks = [0, LOCK_MASK];
        let mut masks = Vec::with_capacity(base_masks.len() * lock_masks.len());
        for base in base_masks {
            for lock in lock_masks {
                masks.push(base | lock);
            }
        }
        masks
    }

    fn type_unicode(symbol: char) {
        let status = Command::new("xdotool")
            .arg("type")
            .arg("--clearmodifiers")
            .arg(symbol.to_string())
            .status();
        if !matches!(status, Ok(status) if status.success()) {
            log::warn!("Smart Input: xdotool failed or is not installed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEXT_EXPANDER_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn rule(trigger: &str, replacement: &str) -> TextExpansionRule {
        TextExpansionRule {
            enabled: true,
            trigger: trigger.to_owned(),
            replacement: replacement.to_owned(),
        }
    }

    fn push_text(text: &str) -> Option<crate::text_expander::TextExpansionMatch> {
        let mut matched = None;
        let mut engine = text_expander_engine().lock().unwrap();
        engine.reset();
        for ch in text.chars() {
            matched = engine.push_char(ch);
        }
        matched
    }

    #[test]
    fn text_expander_runtime_config_enables_loaded_rules() {
        let _guard = TEXT_EXPANDER_TEST_LOCK.lock().unwrap();
        set_text_expander_config(
            true,
            vec![rule(":hello", "Привет")],
            vec![" Notepad.EXE ".to_owned()],
        );

        assert!(text_expander_enabled());
        assert_eq!(
            text_expander_config().read().unwrap().app_blacklist,
            vec!["notepad.exe".to_owned()]
        );
        assert_eq!(push_text(":hello").unwrap().replacement, "Привет");
    }

    #[test]
    fn text_expander_runtime_config_replaces_previous_rules() {
        let _guard = TEXT_EXPANDER_TEST_LOCK.lock().unwrap();
        set_text_expander_config(true, vec![rule(":old", "Old")], Vec::new());
        assert_eq!(push_text(":old").unwrap().replacement, "Old");

        set_text_expander_config(true, vec![rule(":new", "New")], Vec::new());

        assert!(push_text(":old").is_none());
        assert_eq!(push_text(":new").unwrap().replacement, "New");
    }

    #[test]
    fn text_expander_runtime_disabled_config_does_not_report_enabled() {
        let _guard = TEXT_EXPANDER_TEST_LOCK.lock().unwrap();
        set_text_expander_config(false, vec![rule(":hello", "Привет")], Vec::new());

        assert!(!text_expander_enabled());
    }
}
