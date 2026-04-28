#![allow(non_snake_case)]

#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};

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
    SmartSymbol { trigger_keycode: KC_F13,      symbol: '{', name: "Left brace" },
    SmartSymbol { trigger_keycode: KC_F13 + 1,  symbol: '}', name: "Right brace" },
    SmartSymbol { trigger_keycode: KC_F13 + 2,  symbol: '[', name: "Left bracket" },
    SmartSymbol { trigger_keycode: KC_F13 + 3,  symbol: ']', name: "Right bracket" },
    SmartSymbol { trigger_keycode: KC_F13 + 4,  symbol: '(', name: "Left parenthesis" },
    SmartSymbol { trigger_keycode: KC_F13 + 5,  symbol: ')', name: "Right parenthesis" },
    SmartSymbol { trigger_keycode: KC_F13 + 6,  symbol: '<', name: "Less-than" },
    SmartSymbol { trigger_keycode: KC_F13 + 7,  symbol: '>', name: "Greater-than" },
    SmartSymbol { trigger_keycode: KC_F13 + 8,  symbol: '#', name: "Number sign" },
    SmartSymbol { trigger_keycode: KC_F13 + 9,  symbol: '@', name: "At sign" },
    SmartSymbol { trigger_keycode: KC_F13 + 10, symbol: '№', name: "Numero sign" },
    SmartSymbol { trigger_keycode: KC_F13 + 11, symbol: '₽', name: "Ruble sign" },

    // Shift+F13..F24
    SmartSymbol { trigger_keycode: MOD_SHIFT | KC_F13,      symbol: '!', name: "Exclamation mark" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 1),  symbol: '"', name: "Quotation mark" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 2),  symbol: '$', name: "Dollar sign" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 3),  symbol: '%', name: "Percent sign" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 4),  symbol: '&', name: "Ampersand" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 5),  symbol: '\'', name: "Apostrophe" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 6),  symbol: '*', name: "Asterisk" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 7),  symbol: '+', name: "Plus sign" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 8),  symbol: '=', name: "Equals sign" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 9),  symbol: '?', name: "Question mark" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 10), symbol: '|', name: "Vertical bar" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 11), symbol: '\\', name: "Backslash" },

    // Ctrl+F13..F24
    SmartSymbol { trigger_keycode: MOD_CTRL | KC_F13,      symbol: '«', name: "Left guillemet" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 1),  symbol: '»', name: "Right guillemet" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 2),  symbol: '€', name: "Euro sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 3),  symbol: '—', name: "Em dash" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 4),  symbol: '–', name: "En dash" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 5),  symbol: '•', name: "Bullet" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 6),  symbol: '×', name: "Multiplication sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 7),  symbol: '±', name: "Plus-minus sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 8),  symbol: '≠', name: "Not equal sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 9),  symbol: '≈', name: "Almost equal sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 10), symbol: '✓', name: "Check mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 11), symbol: '§', name: "Section sign" },

    // Alt+F13..F24
    SmartSymbol { trigger_keycode: MOD_ALT | KC_F13,      symbol: '.', name: "Full stop" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 1),  symbol: ',', name: "Comma" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 2),  symbol: ';', name: "Semicolon" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 3),  symbol: ':', name: "Colon" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 4),  symbol: '/', name: "Slash" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 5),  symbol: '`', name: "Grave accent" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 6),  symbol: '^', name: "Caret" },

    // Ctrl+Alt+F13..F19
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | KC_F13,      symbol: 'б', name: "Cyrillic be" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 1),  symbol: 'ю', name: "Cyrillic yu" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 2),  symbol: 'ж', name: "Cyrillic zhe" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 3),  symbol: 'э', name: "Cyrillic e" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 4),  symbol: 'х', name: "Cyrillic ha" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 5),  symbol: 'ъ', name: "Cyrillic hard sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 6),  symbol: 'ё', name: "Cyrillic yo" },

    // Ctrl+Alt+Shift+F13..F19
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | KC_F13,      symbol: 'Б', name: "Cyrillic Be" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 1),  symbol: 'Ю', name: "Cyrillic Yu" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 2),  symbol: 'Ж', name: "Cyrillic Zhe" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 3),  symbol: 'Э', name: "Cyrillic E" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 4),  symbol: 'Х', name: "Cyrillic Ha" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 5),  symbol: 'Ъ', name: "Cyrillic Hard Sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | MOD_SHIFT | (KC_F13 + 6),  symbol: 'Ё', name: "Cyrillic Yo" },

    // Ctrl+Shift+F13..F24
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | KC_F13,      symbol: '°', name: "Degree sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 1),  symbol: '‰', name: "Per mille sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 2),  symbol: '′', name: "Prime" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 3),  symbol: '″', name: "Double prime" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 4),  symbol: '‘', name: "Left single quotation mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 5),  symbol: '’', name: "Right single quotation mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 6),  symbol: '„', name: "Double low quotation mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 7),  symbol: '“', name: "Left double quotation mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 8),  symbol: '”', name: "Right double quotation mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 9),  symbol: '™', name: "Trade mark sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 10), symbol: '~', name: "Tilde" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 11), symbol: '_', name: "Underscore" },
];

pub fn smart_symbol_for_keycode(keycode: u16) -> Option<SmartSymbol> {
    SMART_SYMBOLS
        .iter()
        .copied()
        .find(|symbol| symbol.trigger_keycode == keycode)
}


#[cfg(target_os = "windows")]
#[derive(Clone, Copy, PartialEq, Eq)]
enum SmartHrmMode {
    Off,
    Safe,
    Balanced,
    Fast,
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
struct SmartHrmRuntimeConfig {
    mode: SmartHrmMode,
    prior_idle_ms: u64,
    positional: bool,
    same_hand_tap_bias: bool,
}

#[cfg(target_os = "windows")]
impl Default for SmartHrmRuntimeConfig {
    fn default() -> Self {
        Self {
            mode: SmartHrmMode::Off,
            prior_idle_ms: 120,
            positional: true,
            same_hand_tap_bias: true,
        }
    }
}

#[cfg(target_os = "windows")]
impl SmartHrmRuntimeConfig {
    fn enabled(self) -> bool {
        self.mode != SmartHrmMode::Off
    }
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
struct SmartHrmPending {
    carrier_vk: u32,
    tap_vk: u16,
    hold_vk: u16,
    prior_idle_ok: bool,
    hold_active: bool,
}

#[cfg(target_os = "windows")]
#[derive(Default)]
struct SmartHrmState {
    pending: Option<SmartHrmPending>,
    swallow_keyups: Vec<u32>,
    last_real_key: Option<Instant>,
    last_config_load: Option<Instant>,
    config: SmartHrmRuntimeConfig,
}

#[cfg(target_os = "windows")]
fn smart_hrm_state() -> &'static std::sync::Mutex<SmartHrmState> {
    static STATE: std::sync::OnceLock<std::sync::Mutex<SmartHrmState>> = std::sync::OnceLock::new();
    STATE.get_or_init(|| std::sync::Mutex::new(SmartHrmState::default()))
}

#[cfg(target_os = "windows")]
#[derive(serde::Deserialize)]
struct SmartHrmSettingsFile {
    #[serde(default)]
    smart_hrm_mode: Option<String>,
    #[serde(default)]
    smart_hrm_prior_idle_ms: Option<u64>,
    #[serde(default)]
    smart_hrm_positional: Option<bool>,
    #[serde(default)]
    smart_hrm_same_hand_tap_bias: Option<bool>,
}

#[cfg(target_os = "windows")]
fn smart_hrm_settings_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy")
        .join("app_settings.json")
}

#[cfg(target_os = "windows")]
fn load_smart_hrm_config() -> SmartHrmRuntimeConfig {
    let Ok(raw) = std::fs::read_to_string(smart_hrm_settings_path()) else {
        return SmartHrmRuntimeConfig::default();
    };
    let Ok(file) = serde_json::from_str::<SmartHrmSettingsFile>(&raw) else {
        return SmartHrmRuntimeConfig::default();
    };
    let mode = match file.smart_hrm_mode.as_deref() {
        Some("Safe") => SmartHrmMode::Safe,
        Some("Balanced") => SmartHrmMode::Balanced,
        Some("Fast") => SmartHrmMode::Fast,
        _ => SmartHrmMode::Off,
    };
    SmartHrmRuntimeConfig {
        mode,
        prior_idle_ms: file.smart_hrm_prior_idle_ms.unwrap_or(120),
        positional: file.smart_hrm_positional.unwrap_or(true),
        same_hand_tap_bias: file.smart_hrm_same_hand_tap_bias.unwrap_or(true),
    }
}

#[cfg(target_os = "windows")]
fn refresh_smart_hrm_config(state: &mut SmartHrmState) -> SmartHrmRuntimeConfig {
    let now = Instant::now();
    if state
        .last_config_load
        .map(|last| now.duration_since(last) > Duration::from_millis(350))
        .unwrap_or(true)
    {
        state.config = load_smart_hrm_config();
        state.last_config_load = Some(now);
    }
    state.config
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
fn smart_hrm_carrier_for_vk(vk: u32) -> Option<SmartHrmPending> {
    let ctrl = modifier_down(VK_CONTROL);
    let alt = modifier_down(VK_MENU);
    let shift = modifier_down(VK_SHIFT);
    if !(ctrl && alt) {
        return None;
    }
    let prior_idle_ok = true;
    match (vk, shift) {
        (VK_F20, false) => Some(SmartHrmPending { carrier_vk: vk, tap_vk: VK_A as u16, hold_vk: VK_LCONTROL as u16, prior_idle_ok, hold_active: false }),
        (VK_F21, false) => Some(SmartHrmPending { carrier_vk: vk, tap_vk: VK_S as u16, hold_vk: VK_LMENU as u16, prior_idle_ok, hold_active: false }),
        (VK_F22, false) => Some(SmartHrmPending { carrier_vk: vk, tap_vk: VK_D as u16, hold_vk: VK_LSHIFT as u16, prior_idle_ok, hold_active: false }),
        (VK_F23, false) => Some(SmartHrmPending { carrier_vk: vk, tap_vk: VK_F as u16, hold_vk: VK_LWIN as u16, prior_idle_ok, hold_active: false }),
        (VK_F24, false) => Some(SmartHrmPending { carrier_vk: vk, tap_vk: VK_J as u16, hold_vk: VK_RWIN as u16, prior_idle_ok, hold_active: false }),
        (VK_F20, true) => Some(SmartHrmPending { carrier_vk: vk, tap_vk: VK_K as u16, hold_vk: VK_RSHIFT as u16, prior_idle_ok, hold_active: false }),
        (VK_F21, true) => Some(SmartHrmPending { carrier_vk: vk, tap_vk: VK_L as u16, hold_vk: VK_RMENU as u16, prior_idle_ok, hold_active: false }),
        (VK_F22, true) => Some(SmartHrmPending { carrier_vk: vk, tap_vk: VK_OEM_1 as u16, hold_vk: VK_RCONTROL as u16, prior_idle_ok, hold_active: false }),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn smart_hrm_hand_for_vk(vk: u32) -> Option<i8> {
    match vk {
        VK_Q | VK_W | VK_E | VK_R | VK_T | VK_A | VK_S | VK_D | VK_F | VK_G | VK_Z | VK_X | VK_C | VK_V | VK_B => Some(-1),
        VK_Y | VK_U | VK_I | VK_O | VK_P | VK_H | VK_J | VK_K | VK_L | VK_OEM_1 | VK_N | VK_M | VK_OEM_COMMA | VK_OEM_PERIOD | VK_OEM_2 => Some(1),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn smart_hrm_swallow_keyup(state: &mut SmartHrmState, vk: u32) -> bool {
    if let Some(pos) = state.swallow_keyups.iter().position(|queued| *queued == vk) {
        state.swallow_keyups.remove(pos);
        true
    } else {
        false
    }
}

#[cfg(target_os = "windows")]
fn smart_hrm_should_hold(pending: SmartHrmPending, next_vk: u32, config: SmartHrmRuntimeConfig) -> bool {
    if !pending.prior_idle_ok {
        return false;
    }
    if !config.positional {
        return true;
    }
    let Some(pending_hand) = smart_hrm_hand_for_vk(pending.tap_vk as u32) else {
        return true;
    };
    let Some(next_hand) = smart_hrm_hand_for_vk(next_vk) else {
        return true;
    };
    if config.same_hand_tap_bias && pending_hand == next_hand {
        return false;
    }
    pending_hand != next_hand
}

#[cfg(target_os = "windows")]
unsafe fn handle_smart_hrm_event(vk: u32, is_key_down: bool, is_key_up: bool) -> bool {
    let now = Instant::now();
    let Ok(mut state) = smart_hrm_state().lock() else {
        return false;
    };
    let config = refresh_smart_hrm_config(&mut state);
    if !config.enabled() {
        state.pending = None;
        state.swallow_keyups.clear();
        if is_key_down {
            state.last_real_key = Some(now);
        }
        return false;
    }

    if let Some(mut pending) = state.pending {
        if vk == pending.carrier_vk {
            if is_key_up {
                if pending.hold_active {
                    send_vk_keyup(pending.hold_vk);
                } else {
                    send_vk_tap(pending.tap_vk);
                }
                state.pending = None;
                smart_hrm_swallow_keyup(&mut state, vk);
                state.last_real_key = Some(now);
                return true;
            }
            if is_key_down {
                return true;
            }
        } else if is_key_down {
            if smart_hrm_carrier_for_vk(vk).is_some() {
                // Home-row-to-home-row rolls should stay typing-first. Emit the
                // previous tap before arming the next carrier instead of letting
                // the previous transport leak or become a modifier.
                if pending.hold_active {
                    send_vk_keyup(pending.hold_vk);
                } else {
                    send_vk_tap(pending.tap_vk);
                }
                state.pending = None;
            } else if !pending.hold_active {
                if smart_hrm_should_hold(pending, vk, config) {
                    send_vk_keydown(pending.hold_vk);
                    pending.hold_active = true;
                    state.pending = Some(pending);
                } else {
                    send_vk_tap(pending.tap_vk);
                    state.pending = None;
                }
            }
        }
    }

    if is_key_up && smart_hrm_swallow_keyup(&mut state, vk) {
        return true;
    }

    if is_key_down {
        if let Some(mut carrier) = smart_hrm_carrier_for_vk(vk) {
            carrier.prior_idle_ok = state
                .last_real_key
                .map(|last| now.duration_since(last).as_millis() >= config.prior_idle_ms as u128)
                .unwrap_or(true);
            release_smart_hrm_transport_modifiers(carrier.carrier_vk);
            state.swallow_keyups.push(carrier.carrier_vk);
            state.pending = Some(carrier);
            return true;
        }
    }

    if is_key_down {
        state.last_real_key = Some(now);
    }
    false
}

#[cfg(target_os = "windows")]
unsafe fn release_smart_hrm_transport_modifiers(carrier_vk: u32) {
    send_vk_keyup(VK_CONTROL as u16);
    send_vk_keyup(VK_MENU as u16);
    if matches!(carrier_vk, VK_F20 | VK_F21 | VK_F22) && modifier_down(VK_SHIFT) {
        send_vk_keyup(VK_SHIFT as u16);
    }
}

#[cfg(target_os = "windows")]
unsafe fn send_vk_keydown(vk: u16) {
    let input = INPUT::keyboard_vk(vk, false);
    let sent = SendInput(
        1,
        &input as *const INPUT,
        std::mem::size_of::<INPUT>() as i32,
    );
    if sent != 1 {
        log::warn!("Smart HRM: SendInput failed for VK keydown 0x{:02X}", vk);
    }
}

#[cfg(target_os = "windows")]
unsafe fn send_vk_tap(vk: u16) {
    send_vk_keydown(vk);
    send_vk_keyup(vk);
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    if n_code == HC_ACTION {
        let info = &*(l_param as *const KBDLLHOOKSTRUCT);
        let is_key_down = w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN;
        let is_key_up = w_param == WM_KEYUP || w_param == WM_SYSKEYUP;
        let injected = info.flags & LLKHF_INJECTED != 0;
        if !injected {
            if handle_smart_hrm_event(info.vkCode, is_key_down, is_key_up) {
                return 1;
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
const VK_A: u32 = 0x41;
#[cfg(target_os = "windows")]
const VK_B: u32 = 0x42;
#[cfg(target_os = "windows")]
const VK_C: u32 = 0x43;
#[cfg(target_os = "windows")]
const VK_D: u32 = 0x44;
#[cfg(target_os = "windows")]
const VK_E: u32 = 0x45;
#[cfg(target_os = "windows")]
const VK_F: u32 = 0x46;
#[cfg(target_os = "windows")]
const VK_G: u32 = 0x47;
#[cfg(target_os = "windows")]
const VK_H: u32 = 0x48;
#[cfg(target_os = "windows")]
const VK_I: u32 = 0x49;
#[cfg(target_os = "windows")]
const VK_J: u32 = 0x4A;
#[cfg(target_os = "windows")]
const VK_K: u32 = 0x4B;
#[cfg(target_os = "windows")]
const VK_L: u32 = 0x4C;
#[cfg(target_os = "windows")]
const VK_M: u32 = 0x4D;
#[cfg(target_os = "windows")]
const VK_N: u32 = 0x4E;
#[cfg(target_os = "windows")]
const VK_O: u32 = 0x4F;
#[cfg(target_os = "windows")]
const VK_P: u32 = 0x50;
#[cfg(target_os = "windows")]
const VK_Q: u32 = 0x51;
#[cfg(target_os = "windows")]
const VK_R: u32 = 0x52;
#[cfg(target_os = "windows")]
const VK_S: u32 = 0x53;
#[cfg(target_os = "windows")]
const VK_T: u32 = 0x54;
#[cfg(target_os = "windows")]
const VK_U: u32 = 0x55;
#[cfg(target_os = "windows")]
const VK_V: u32 = 0x56;
#[cfg(target_os = "windows")]
const VK_W: u32 = 0x57;
#[cfg(target_os = "windows")]
const VK_X: u32 = 0x58;
#[cfg(target_os = "windows")]
const VK_Y: u32 = 0x59;
#[cfg(target_os = "windows")]
const VK_Z: u32 = 0x5A;
#[cfg(target_os = "windows")]
const VK_LWIN: u32 = 0x5B;
#[cfg(target_os = "windows")]
const VK_RWIN: u32 = 0x5C;
#[cfg(target_os = "windows")]
const VK_F20: u32 = 0x83;
#[cfg(target_os = "windows")]
const VK_F21: u32 = 0x84;
#[cfg(target_os = "windows")]
const VK_F22: u32 = 0x85;
#[cfg(target_os = "windows")]
const VK_F23: u32 = 0x86;
#[cfg(target_os = "windows")]
const VK_F24: u32 = 0x87;
#[cfg(target_os = "windows")]
const VK_LSHIFT: u32 = 0xA0;
#[cfg(target_os = "windows")]
const VK_RSHIFT: u32 = 0xA1;
#[cfg(target_os = "windows")]
const VK_LCONTROL: u32 = 0xA2;
#[cfg(target_os = "windows")]
const VK_RCONTROL: u32 = 0xA3;
#[cfg(target_os = "windows")]
const VK_LMENU: u32 = 0xA4;
#[cfg(target_os = "windows")]
const VK_RMENU: u32 = 0xA5;
#[cfg(target_os = "windows")]
const VK_OEM_1: u32 = 0xBA;
#[cfg(target_os = "windows")]
const VK_OEM_COMMA: u32 = 0xBC;
#[cfg(target_os = "windows")]
const VK_OEM_PERIOD: u32 = 0xBE;
#[cfg(target_os = "windows")]
const VK_OEM_2: u32 = 0xBF;
#[cfg(target_os = "windows")]
const LLKHF_INJECTED: u32 = 0x10;
#[cfg(target_os = "windows")]
const INPUT_KEYBOARD: u32 = 1;
#[cfg(target_os = "windows")]
const KEYEVENTF_KEYUP: u32 = 0x0002;
#[cfg(target_os = "windows")]
const KEYEVENTF_UNICODE: u32 = 0x0004;

#[cfg(target_os = "windows")]
type HHOOK = *mut core::ffi::c_void;
#[cfg(target_os = "windows")]
type HINSTANCE = *mut core::ffi::c_void;
#[cfg(target_os = "windows")]
type HWND = *mut core::ffi::c_void;

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
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
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
