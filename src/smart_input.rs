#![allow(non_snake_case)]

use crate::text_expander::{TextExpansionConfig, TextExpansionEngine, TextExpansionRule};
use std::sync::{Mutex, OnceLock, RwLock};

#[derive(Clone, Copy, Debug)]
pub struct SmartSymbol {
    pub trigger_keycode: u16,
    pub symbol: char,
    pub name: &'static str,
}

const KC_F13: u16 = 0x0068;
const MOD_CTRL: u16 = 0x0100;
const MOD_SHIFT: u16 = 0x0200;
const MOD_ALT: u16 = 0x0400;

pub const SMART_SYMBOLS: &[SmartSymbol] = &[
    // F13..F24
    SmartSymbol {
        trigger_keycode: KC_F13,
        symbol: '{',
        name: "Left brace",
    },
    SmartSymbol {
        trigger_keycode: KC_F13 + 1,
        symbol: '}',
        name: "Right brace",
    },
    SmartSymbol {
        trigger_keycode: KC_F13 + 2,
        symbol: '[',
        name: "Left bracket",
    },
    SmartSymbol {
        trigger_keycode: KC_F13 + 3,
        symbol: ']',
        name: "Right bracket",
    },
    SmartSymbol {
        trigger_keycode: KC_F13 + 4,
        symbol: '(',
        name: "Left parenthesis",
    },
    SmartSymbol {
        trigger_keycode: KC_F13 + 5,
        symbol: ')',
        name: "Right parenthesis",
    },
    SmartSymbol {
        trigger_keycode: KC_F13 + 6,
        symbol: '<',
        name: "Less-than",
    },
    SmartSymbol {
        trigger_keycode: KC_F13 + 7,
        symbol: '>',
        name: "Greater-than",
    },
    SmartSymbol {
        trigger_keycode: KC_F13 + 8,
        symbol: '#',
        name: "Number sign",
    },
    SmartSymbol {
        trigger_keycode: KC_F13 + 9,
        symbol: '@',
        name: "At sign",
    },
    SmartSymbol {
        trigger_keycode: KC_F13 + 10,
        symbol: '№',
        name: "Numero sign",
    },
    SmartSymbol {
        trigger_keycode: KC_F13 + 11,
        symbol: '₽',
        name: "Ruble sign",
    },
    // Shift+F13..F24
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | KC_F13,
        symbol: '!',
        name: "Exclamation mark",
    },
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | (KC_F13 + 1),
        symbol: '"',
        name: "Quotation mark",
    },
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | (KC_F13 + 2),
        symbol: '$',
        name: "Dollar sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | (KC_F13 + 3),
        symbol: '%',
        name: "Percent sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | (KC_F13 + 4),
        symbol: '&',
        name: "Ampersand",
    },
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | (KC_F13 + 5),
        symbol: '\'',
        name: "Apostrophe",
    },
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | (KC_F13 + 6),
        symbol: '*',
        name: "Asterisk",
    },
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | (KC_F13 + 7),
        symbol: '+',
        name: "Plus sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | (KC_F13 + 8),
        symbol: '=',
        name: "Equals sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | (KC_F13 + 9),
        symbol: '?',
        name: "Question mark",
    },
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | (KC_F13 + 10),
        symbol: '|',
        name: "Vertical bar",
    },
    SmartSymbol {
        trigger_keycode: MOD_SHIFT | (KC_F13 + 11),
        symbol: '\\',
        name: "Backslash",
    },
    // Ctrl+F13..F24
    SmartSymbol {
        trigger_keycode: MOD_CTRL | KC_F13,
        symbol: '«',
        name: "Left guillemet",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | (KC_F13 + 1),
        symbol: '»',
        name: "Right guillemet",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | (KC_F13 + 2),
        symbol: '€',
        name: "Euro sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | (KC_F13 + 3),
        symbol: '—',
        name: "Em dash",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | (KC_F13 + 4),
        symbol: '–',
        name: "En dash",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | (KC_F13 + 5),
        symbol: '•',
        name: "Bullet",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | (KC_F13 + 6),
        symbol: '×',
        name: "Multiplication sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | (KC_F13 + 7),
        symbol: '±',
        name: "Plus-minus sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | (KC_F13 + 8),
        symbol: '≠',
        name: "Not equal sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | (KC_F13 + 9),
        symbol: '≈',
        name: "Almost equal sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | (KC_F13 + 10),
        symbol: '✓',
        name: "Check mark",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | (KC_F13 + 11),
        symbol: '§',
        name: "Section sign",
    },
    // Alt+F13..F24
    SmartSymbol {
        trigger_keycode: MOD_ALT | KC_F13,
        symbol: '.',
        name: "Full stop",
    },
    SmartSymbol {
        trigger_keycode: MOD_ALT | (KC_F13 + 1),
        symbol: ',',
        name: "Comma",
    },
    SmartSymbol {
        trigger_keycode: MOD_ALT | (KC_F13 + 2),
        symbol: ';',
        name: "Semicolon",
    },
    SmartSymbol {
        trigger_keycode: MOD_ALT | (KC_F13 + 3),
        symbol: ':',
        name: "Colon",
    },
    SmartSymbol {
        trigger_keycode: MOD_ALT | (KC_F13 + 4),
        symbol: '/',
        name: "Slash",
    },
    SmartSymbol {
        trigger_keycode: MOD_ALT | (KC_F13 + 5),
        symbol: '`',
        name: "Grave accent",
    },
    SmartSymbol {
        trigger_keycode: MOD_ALT | (KC_F13 + 6),
        symbol: '^',
        name: "Caret",
    },
    // Ctrl+Alt+F13..F19
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | KC_F13,
        symbol: 'б',
        name: "Cyrillic be",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 1),
        symbol: 'ю',
        name: "Cyrillic yu",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 2),
        symbol: 'ж',
        name: "Cyrillic zhe",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 3),
        symbol: 'э',
        name: "Cyrillic e",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 4),
        symbol: 'х',
        name: "Cyrillic ha",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 5),
        symbol: 'ъ',
        name: "Cyrillic hard sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 6),
        symbol: 'ё',
        name: "Cyrillic yo",
    },
    // Ctrl+Alt+Shift+F13..F19
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | KC_F13,
        symbol: 'Б',
        name: "Cyrillic Be",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 1),
        symbol: 'Ю',
        name: "Cyrillic Yu",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 2),
        symbol: 'Ж',
        name: "Cyrillic Zhe",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 3),
        symbol: 'Э',
        name: "Cyrillic E",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 4),
        symbol: 'Х',
        name: "Cyrillic Ha",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 5),
        symbol: 'Ъ',
        name: "Cyrillic Hard Sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 6),
        symbol: 'Ё',
        name: "Cyrillic Yo",
    },
    // Ctrl+Shift+F13..F24
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | KC_F13,
        symbol: '°',
        name: "Degree sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 1),
        symbol: '‰',
        name: "Per mille sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 2),
        symbol: '′',
        name: "Prime",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 3),
        symbol: '″',
        name: "Double prime",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 4),
        symbol: '‘',
        name: "Left single quotation mark",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 5),
        symbol: '’',
        name: "Right single quotation mark",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 6),
        symbol: '„',
        name: "Double low quotation mark",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 7),
        symbol: '“',
        name: "Left double quotation mark",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 8),
        symbol: '”',
        name: "Right double quotation mark",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 9),
        symbol: '™',
        name: "Trade mark sign",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 10),
        symbol: '~',
        name: "Tilde",
    },
    SmartSymbol {
        trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 11),
        symbol: '_',
        name: "Underscore",
    },
];

pub fn smart_symbol_for_keycode(keycode: u16) -> Option<SmartSymbol> {
    SMART_SYMBOLS
        .iter()
        .copied()
        .find(|symbol| symbol.trigger_keycode == keycode)
}

static TEXT_EXPANDER_CONFIG: OnceLock<RwLock<TextExpansionConfig>> = OnceLock::new();
static TEXT_EXPANDER_ENGINE: OnceLock<Mutex<TextExpansionEngine>> = OnceLock::new();

fn text_expander_config() -> &'static RwLock<TextExpansionConfig> {
    TEXT_EXPANDER_CONFIG.get_or_init(|| RwLock::new(TextExpansionConfig::default()))
}

fn text_expander_engine() -> &'static Mutex<TextExpansionEngine> {
    TEXT_EXPANDER_ENGINE.get_or_init(|| Mutex::new(TextExpansionEngine::default()))
}

pub fn set_text_expander_config(
    enabled: bool,
    paused: bool,
    rules: Vec<TextExpansionRule>,
    app_blacklist: Vec<String>,
) {
    let config = TextExpansionConfig {
        enabled,
        paused,
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
        .map(|config| {
            config.enabled
                && !config.paused
                && config.rules.iter().any(crate::text_expander::rule_usable)
                && !foreground_app_blacklisted(&config.app_blacklist)
        })
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn foreground_app_blacklisted(app_blacklist: &[String]) -> bool {
    if app_blacklist.is_empty() {
        return false;
    }
    foreground_process_name_lower()
        .map(|name| {
            app_blacklist.iter().any(|blocked| {
                name == *blocked
                    || name.ends_with(blocked)
                    || blocked
                        .strip_suffix(".exe")
                        .is_some_and(|stem| name == stem)
            })
        })
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn foreground_app_blacklisted(_app_blacklist: &[String]) -> bool {
    false
}

#[cfg(target_os = "windows")]
fn foreground_process_name_lower() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let mut process_id = 0u32;
        if GetWindowThreadProcessId(hwnd, &mut process_id as *mut u32) == 0 || process_id == 0 {
            return None;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
        if process.is_null() {
            return None;
        }
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size) != 0;
        CloseHandle(process);
        if !ok || size == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&buffer[..size as usize]);
        std::path::Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_ascii_lowercase())
    }
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
    let session = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        "Wayland"
    } else if std::env::var_os("DISPLAY").is_some() {
        "X11"
    } else {
        "Linux"
    };
    let input_method = linux_input_method_hint();
    match session {
        "Wayland" => format!(
            "Universal output backend: Wayland via IBus/Fcitx5 input method{}",
            input_method
        ),
        "X11" => "Universal output backend: Linux X11 native; Wayland uses IBus/Fcitx5".to_owned(),
        _ => format!(
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
    use std::sync::Once;
    static START: Once = Once::new();
    START.call_once(|| {
        std::thread::spawn(|| unsafe {
            run_windows_keyboard_hook_loop();
        });
    });
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

#[cfg(target_os = "windows")]
fn symbol_for_vk(vk: u32) -> Option<(char, u16)> {
    let base_keycode = match vk {
        0x7C..=0x87 => KC_F13 + (vk - 0x7C) as u16,
        _ => return None,
    };
    smart_symbol_for_transport(
        base_keycode,
        modifier_down(VK_CONTROL),
        modifier_down(VK_SHIFT),
        modifier_down(VK_MENU),
    )
}

#[cfg(target_os = "windows")]
fn modifier_down(vk: i32) -> bool {
    unsafe { GetAsyncKeyState(vk) & 0x8000u16 as i16 != 0 }
}

#[cfg(target_os = "windows")]
unsafe fn run_windows_keyboard_hook_loop() {
    let module = GetModuleHandleW(std::ptr::null());
    let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0);
    if hook.is_null() {
        log::warn!("Smart Input: failed to install keyboard hook");
        return;
    }

    let mut msg = MSG::default();
    while GetMessageW(&mut msg as *mut MSG, std::ptr::null_mut(), 0, 0) > 0 {
        TranslateMessage(&msg as *const MSG);
        DispatchMessageW(&msg as *const MSG);
    }

    UnhookWindowsHookEx(hook);
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    if n_code == HC_ACTION {
        let info = &*(l_param as *const KBDLLHOOKSTRUCT);
        let is_key_down = w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN;
        let is_key_up = w_param == WM_KEYUP || w_param == WM_SYSKEYUP;
        let injected = info.flags & LLKHF_INJECTED != 0;
        if !injected {
            if is_key_down && text_expander_enabled() {
                if info.vkCode == VK_BACK as u32 {
                    if let Ok(mut engine) = text_expander_engine().lock() {
                        engine.backspace();
                    }
                } else if should_reset_text_expander_for_vk(info.vkCode) {
                    if let Ok(mut engine) = text_expander_engine().lock() {
                        engine.reset();
                    }
                } else if let Some(ch) = text_expander_char_for_key(info) {
                    let expansion = text_expander_engine()
                        .lock()
                        .ok()
                        .and_then(|mut engine| engine.push_char(ch));
                    if let Some(expansion) = expansion {
                        schedule_text_expansion(expansion);
                    }
                }
            }

            if let Some((symbol, trigger_keycode)) = symbol_for_vk(info.vkCode) {
                if is_key_down {
                    send_unicode_char(symbol, trigger_keycode);
                }
                if is_key_down || is_key_up {
                    return 1;
                }
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
}

#[cfg(target_os = "windows")]
unsafe fn text_expander_char_for_key(info: &KBDLLHOOKSTRUCT) -> Option<char> {
    if modifier_down(VK_CONTROL)
        || modifier_down(VK_MENU)
        || modifier_down(VK_LWIN)
        || modifier_down(VK_RWIN)
    {
        return None;
    }

    let mut keyboard_state = [0u8; 256];
    if GetKeyboardState(keyboard_state.as_mut_ptr()) == 0 {
        return None;
    }
    set_keyboard_state_down(&mut keyboard_state, VK_SHIFT, modifier_down(VK_SHIFT));
    set_keyboard_state_down(&mut keyboard_state, VK_CONTROL, false);
    set_keyboard_state_down(&mut keyboard_state, VK_MENU, false);
    set_keyboard_state_down(&mut keyboard_state, VK_LWIN, false);
    set_keyboard_state_down(&mut keyboard_state, VK_RWIN, false);
    if GetKeyState(VK_CAPITAL) & 1 != 0 {
        keyboard_state[VK_CAPITAL as usize] |= 1;
    }
    if (info.vkCode as usize) < keyboard_state.len() {
        keyboard_state[info.vkCode as usize] = 0x80;
    }

    let mut buffer = [0u16; 8];
    let len = ToUnicodeEx(
        info.vkCode,
        info.scanCode,
        keyboard_state.as_ptr(),
        buffer.as_mut_ptr(),
        buffer.len() as i32,
        0,
        GetKeyboardLayout(0),
    );
    if len <= 0 {
        return None;
    }

    char::decode_utf16(buffer[..len as usize].iter().copied())
        .next()
        .and_then(Result::ok)
        .filter(|ch| !ch.is_control())
}

#[cfg(target_os = "windows")]
fn set_keyboard_state_down(keyboard_state: &mut [u8; 256], vk: i32, down: bool) {
    if vk >= 0 && (vk as usize) < keyboard_state.len() {
        if down {
            keyboard_state[vk as usize] |= 0x80;
        } else {
            keyboard_state[vk as usize] &= !0x80;
        }
    }
}

#[cfg(target_os = "windows")]
fn should_reset_text_expander_for_vk(vk: u32) -> bool {
    matches!(
        vk as i32,
        VK_RETURN | VK_TAB | VK_ESCAPE | VK_LEFT | VK_UP | VK_RIGHT | VK_DOWN
    )
}

#[cfg(target_os = "windows")]
fn schedule_text_expansion(expansion: crate::text_expander::TextExpansionMatch) {
    std::thread::spawn(move || unsafe {
        std::thread::sleep(std::time::Duration::from_millis(8));
        send_text_expansion(&expansion);
    });
}

#[cfg(target_os = "windows")]
unsafe fn send_text_expansion(expansion: &crate::text_expander::TextExpansionMatch) {
    for _ in 0..expansion.typed_trigger_chars {
        send_vk_tap(VK_BACK as u16);
    }

    let pasted = should_use_clipboard_paste(&expansion.replacement)
        && paste_text_with_clipboard_restore(&expansion.replacement);
    if !pasted {
        send_unicode_text(&expansion.replacement);
    }

    for _ in 0..expansion.cursor_back_chars {
        send_vk_tap(VK_LEFT as u16);
    }
}

#[cfg(target_os = "windows")]
fn should_use_clipboard_paste(text: &str) -> bool {
    text.chars().count() > 64 || text.contains(['\n', '\r', '\t'])
}

#[cfg(target_os = "windows")]
unsafe fn paste_text_with_clipboard_restore(text: &str) -> bool {
    let Some(previous_text) = read_clipboard_text() else {
        return false;
    };
    if !set_clipboard_text(text) {
        return false;
    }
    std::thread::sleep(std::time::Duration::from_millis(12));
    send_ctrl_v();
    std::thread::sleep(std::time::Duration::from_millis(80));
    set_clipboard_text(&previous_text)
}

#[cfg(target_os = "windows")]
unsafe fn read_clipboard_text() -> Option<String> {
    if OpenClipboard(std::ptr::null_mut()) == 0 {
        return None;
    }
    let handle = GetClipboardData(CF_UNICODETEXT);
    if handle.is_null() {
        CloseClipboard();
        return None;
    }
    let locked = GlobalLock(handle) as *const u16;
    if locked.is_null() {
        CloseClipboard();
        return None;
    }
    let mut len = 0usize;
    while *locked.add(len) != 0 {
        len += 1;
    }
    let text = String::from_utf16_lossy(std::slice::from_raw_parts(locked, len));
    GlobalUnlock(handle);
    CloseClipboard();
    Some(text)
}

#[cfg(target_os = "windows")]
unsafe fn set_clipboard_text(text: &str) -> bool {
    if OpenClipboard(std::ptr::null_mut()) == 0 {
        return false;
    }
    if EmptyClipboard() == 0 {
        CloseClipboard();
        return false;
    }
    let utf16 = text
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let bytes = utf16.len() * std::mem::size_of::<u16>();
    let handle = GlobalAlloc(GMEM_MOVEABLE, bytes);
    if handle.is_null() {
        CloseClipboard();
        return false;
    }
    let locked = GlobalLock(handle) as *mut u16;
    if locked.is_null() {
        GlobalFree(handle);
        CloseClipboard();
        return false;
    }
    std::ptr::copy_nonoverlapping(utf16.as_ptr(), locked, utf16.len());
    GlobalUnlock(handle);
    if SetClipboardData(CF_UNICODETEXT, handle).is_null() {
        GlobalFree(handle);
        CloseClipboard();
        return false;
    }
    CloseClipboard();
    true
}

#[cfg(target_os = "windows")]
unsafe fn send_ctrl_v() {
    let inputs = [
        INPUT::keyboard_vk(VK_CONTROL as u16, false),
        INPUT::keyboard_vk(VK_V as u16, false),
        INPUT::keyboard_vk(VK_V as u16, true),
        INPUT::keyboard_vk(VK_CONTROL as u16, true),
    ];
    let sent = SendInput(
        inputs.len() as u32,
        inputs.as_ptr(),
        std::mem::size_of::<INPUT>() as i32,
    );
    if sent != inputs.len() as u32 {
        log::warn!("Smart Input: SendInput failed for Ctrl+V");
    }
}

#[cfg(target_os = "windows")]
unsafe fn send_unicode_text(text: &str) {
    for ch in text.chars() {
        for unit in ch.encode_utf16(&mut [0; 2]) {
            let down = INPUT::keyboard_unicode(*unit, false);
            let up = INPUT::keyboard_unicode(*unit, true);
            let inputs = [down, up];
            let sent = SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
            if sent != inputs.len() as u32 {
                log::warn!(
                    "Smart Input: SendInput failed for text expansion unit U+{:04X}",
                    *unit as u32
                );
            }
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn send_vk_tap(vk: u16) {
    let down = INPUT::keyboard_vk(vk, false);
    let up = INPUT::keyboard_vk(vk, true);
    let inputs = [down, up];
    let sent = SendInput(
        inputs.len() as u32,
        inputs.as_ptr(),
        std::mem::size_of::<INPUT>() as i32,
    );
    if sent != inputs.len() as u32 {
        log::warn!("Smart Input: SendInput failed for VK tap 0x{vk:02X}");
    }
}

#[cfg(target_os = "windows")]
unsafe fn send_unicode_char(symbol: char, trigger_keycode: u16) {
    release_transport_modifiers(trigger_keycode);
    for unit in symbol.encode_utf16(&mut [0; 2]) {
        let down = INPUT::keyboard_unicode(*unit, false);
        let up = INPUT::keyboard_unicode(*unit, true);
        let inputs = [down, up];
        let sent = SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
        if sent != inputs.len() as u32 {
            log::warn!("Smart Input: SendInput failed for U+{:04X}", *unit as u32);
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn release_transport_modifiers(trigger_keycode: u16) {
    if trigger_keycode & MOD_SHIFT != 0 {
        send_vk_keyup(VK_SHIFT as u16);
    }
    if trigger_keycode & MOD_ALT != 0 {
        send_vk_keyup(VK_MENU as u16);
    }
    if trigger_keycode & MOD_CTRL != 0 {
        send_vk_keyup(VK_CONTROL as u16);
    }
}

#[cfg(target_os = "windows")]
unsafe fn send_vk_keyup(vk: u16) {
    let input = INPUT::keyboard_vk(vk, true);
    let sent = SendInput(
        1,
        &input as *const INPUT,
        std::mem::size_of::<INPUT>() as i32,
    );
    if sent != 1 {
        log::warn!("Smart Input: SendInput failed for VK keyup 0x{:02X}", vk);
    }
}

#[cfg(target_os = "windows")]
const WH_KEYBOARD_LL: i32 = 13;
#[cfg(target_os = "windows")]
const HC_ACTION: i32 = 0;
#[cfg(target_os = "windows")]
const WM_KEYDOWN: usize = 0x0100;
#[cfg(target_os = "windows")]
const WM_SYSKEYDOWN: usize = 0x0104;
#[cfg(target_os = "windows")]
const WM_KEYUP: usize = 0x0101;
#[cfg(target_os = "windows")]
const WM_SYSKEYUP: usize = 0x0105;
#[cfg(target_os = "windows")]
const VK_SHIFT: i32 = 0x10;
#[cfg(target_os = "windows")]
const VK_CONTROL: i32 = 0x11;
#[cfg(target_os = "windows")]
const VK_MENU: i32 = 0x12;
#[cfg(target_os = "windows")]
const VK_CAPITAL: i32 = 0x14;
#[cfg(target_os = "windows")]
const VK_BACK: i32 = 0x08;
#[cfg(target_os = "windows")]
const VK_TAB: i32 = 0x09;
#[cfg(target_os = "windows")]
const VK_RETURN: i32 = 0x0D;
#[cfg(target_os = "windows")]
const VK_ESCAPE: i32 = 0x1B;
#[cfg(target_os = "windows")]
const VK_LEFT: i32 = 0x25;
#[cfg(target_os = "windows")]
const VK_UP: i32 = 0x26;
#[cfg(target_os = "windows")]
const VK_RIGHT: i32 = 0x27;
#[cfg(target_os = "windows")]
const VK_DOWN: i32 = 0x28;
#[cfg(target_os = "windows")]
const VK_LWIN: i32 = 0x5B;
#[cfg(target_os = "windows")]
const VK_RWIN: i32 = 0x5C;
#[cfg(target_os = "windows")]
const VK_V: i32 = 0x56;
#[cfg(target_os = "windows")]
const LLKHF_INJECTED: u32 = 0x10;
#[cfg(target_os = "windows")]
const INPUT_KEYBOARD: u32 = 1;
#[cfg(target_os = "windows")]
const KEYEVENTF_KEYUP: u32 = 0x0002;
#[cfg(target_os = "windows")]
const KEYEVENTF_UNICODE: u32 = 0x0004;
#[cfg(target_os = "windows")]
const CF_UNICODETEXT: u32 = 13;
#[cfg(target_os = "windows")]
const GMEM_MOVEABLE: u32 = 0x0002;
#[cfg(target_os = "windows")]
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

#[cfg(target_os = "windows")]
type HHOOK = *mut core::ffi::c_void;
#[cfg(target_os = "windows")]
type HINSTANCE = *mut core::ffi::c_void;
#[cfg(target_os = "windows")]
type HWND = *mut core::ffi::c_void;
#[cfg(target_os = "windows")]
type HANDLE = *mut core::ffi::c_void;

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KBDLLHOOKSTRUCT {
    vkCode: u32,
    scanCode: u32,
    flags: u32,
    time: u32,
    dwExtraInfo: usize,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct MSG {
    hwnd: HWND,
    message: u32,
    wParam: usize,
    lParam: isize,
    time: u32,
    pt_x: i32,
    pt_y: i32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct INPUT {
    input_type: u32,
    u: INPUT_UNION,
}

#[cfg(target_os = "windows")]
impl INPUT {
    fn keyboard_unicode(unit: u16, key_up: bool) -> Self {
        let flags = KEYEVENTF_UNICODE | if key_up { KEYEVENTF_KEYUP } else { 0 };
        Self {
            input_type: INPUT_KEYBOARD,
            u: INPUT_UNION {
                ki: KEYBDINPUT {
                    wVk: 0,
                    wScan: unit,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn keyboard_vk(vk: u16, key_up: bool) -> Self {
        let flags = if key_up { KEYEVENTF_KEYUP } else { 0 };
        Self {
            input_type: INPUT_KEYBOARD,
            u: INPUT_UNION {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
union INPUT_UNION {
    mi: MOUSEINPUT,
    ki: KEYBDINPUT,
}

#[cfg(all(target_os = "windows", target_pointer_width = "64"))]
const _: [(); 40] = [(); std::mem::size_of::<INPUT>()];
#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
const _: [(); 28] = [(); std::mem::size_of::<INPUT>()];

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct MOUSEINPUT {
    dx: i32,
    dy: i32,
    mouseData: u32,
    dwFlags: u32,
    time: u32,
    dwExtraInfo: usize,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct KEYBDINPUT {
    wVk: u16,
    wScan: u16,
    dwFlags: u32,
    time: u32,
    dwExtraInfo: usize,
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn SetWindowsHookExW(
        idHook: i32,
        lpfn: Option<unsafe extern "system" fn(i32, usize, isize) -> isize>,
        hmod: HINSTANCE,
        dwThreadId: u32,
    ) -> HHOOK;
    fn CallNextHookEx(hhk: HHOOK, nCode: i32, wParam: usize, lParam: isize) -> isize;
    fn UnhookWindowsHookEx(hhk: HHOOK) -> i32;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32;
    fn GetAsyncKeyState(vKey: i32) -> i16;
    fn GetKeyboardState(lpKeyState: *mut u8) -> i32;
    fn GetKeyState(nVirtKey: i32) -> i16;
    fn GetKeyboardLayout(idThread: u32) -> *mut core::ffi::c_void;
    fn ToUnicodeEx(
        wVirtKey: u32,
        wScanCode: u32,
        lpKeyState: *const u8,
        pwszBuff: *mut u16,
        cchBuff: i32,
        wFlags: u32,
        dwhkl: *mut core::ffi::c_void,
    ) -> i32;
    fn GetForegroundWindow() -> HWND;
    fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut u32) -> u32;
    fn OpenClipboard(hWndNewOwner: HWND) -> i32;
    fn CloseClipboard() -> i32;
    fn EmptyClipboard() -> i32;
    fn GetClipboardData(uFormat: u32) -> HANDLE;
    fn SetClipboardData(uFormat: u32, hMem: HANDLE) -> HANDLE;
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> HANDLE;
    fn QueryFullProcessImageNameW(
        hProcess: HANDLE,
        dwFlags: u32,
        lpExeName: *mut u16,
        lpdwSize: *mut u32,
    ) -> i32;
    fn CloseHandle(hObject: HANDLE) -> i32;
    fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> HANDLE;
    fn GlobalLock(hMem: HANDLE) -> *mut core::ffi::c_void;
    fn GlobalUnlock(hMem: HANDLE) -> i32;
    fn GlobalFree(hMem: HANDLE) -> HANDLE;
}

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
