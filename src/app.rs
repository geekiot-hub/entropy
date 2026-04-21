use crate::device::DeviceManager;
use crate::firmware::FirmwareProtocol;
use crate::zmk::{zmk_binding_label, zmk_binding_tooltip, ZmkBinding};

/// Sanitize a device name into a filesystem-safe slug.
fn device_id_slug(device_name: &str) -> String {
    device_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn layer_names_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_id_slug(device_name);
    dir.join(format!("layer_names_{}.json", slug))
}

fn single_instance_signal_path() -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    dir.join("single_instance_signal")
}

fn read_single_instance_signal() -> String {
    std::fs::read_to_string(single_instance_signal_path())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn load_saved_layer_names(device_name: &str) -> Option<Vec<String>> {
    let path = layer_names_path(device_name);
    let data = std::fs::read_to_string(&path).ok()?;
    let mut v = serde_json::from_str::<Vec<String>>(&data).ok()?;
    if v.is_empty() {
        return None;
    }
    while v.len() < 16 {
        let n = v.len();
        v.push(n.to_string());
    }
    Some(v)
}

fn load_layer_names(device_name: &str) -> Vec<String> {
    if let Some(v) = load_saved_layer_names(device_name) {
        return v;
    }
    let mut v: Vec<String> = (0..16).map(|i| i.to_string()).collect();
    v[0] = "Main".to_string();
    v
}

fn encoder_visibility_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    dir.join(format!("encoder_visibility_{}.json", slug))
}

fn load_encoder_visibility(device_name: &str, count: usize) -> Vec<bool> {
    if count == 0 {
        return vec![];
    }
    let path = encoder_visibility_path(device_name);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(mut v) = serde_json::from_str::<Vec<bool>>(&data) {
            v.truncate(count);
            while v.len() < count {
                v.push(true);
            }
            return v;
        }
    }
    vec![true; count]
}

fn save_encoder_visibility(visibility: &[bool], device_name: &str) {
    let path = encoder_visibility_path(device_name);
    match serde_json::to_string_pretty(visibility) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                log::warn!("save_encoder_visibility failed: {}", e);
            }
        }
        Err(e) => log::warn!("save_encoder_visibility serialize failed: {}", e),
    }
}

fn save_layer_names(names: &[String], device_name: &str) {
    // Always save at least 16 slots so load_layer_names can detect a valid file
    let mut full = names.to_vec();
    while full.len() < 16 {
        let n = full.len();
        full.push(n.to_string());
    }
    if let Ok(data) = serde_json::to_string(&full) {
        let path = layer_names_path(device_name);
        if let Err(e) = std::fs::write(&path, &data) {
            log::warn!("save_layer_names failed at {:?}: {e}", path);
        } else {
            log::info!("save_layer_names ok → {:?}", path);
        }
    }
}

fn tap_dance_names_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_id_slug(device_name);
    dir.join(format!("tap_dance_names_{}.json", slug))
}

fn load_tap_dance_names(device_name: &str) -> Vec<String> {
    let path = tap_dance_names_path(device_name);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Vec<String>>(&data) {
            return v;
        }
    }
    vec![]
}

fn save_tap_dance_names(names: &[String], device_name: &str) {
    if let Ok(data) = serde_json::to_string(names) {
        let path = tap_dance_names_path(device_name);
        if let Err(e) = std::fs::write(&path, &data) {
            log::warn!("save_tap_dance_names failed at {:?}: {e}", path);
        } else {
            log::info!("save_tap_dance_names ok → {:?}", path);
        }
    }
}

fn combo_names_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_id_slug(device_name);
    dir.join(format!("combo_names_{}.json", slug))
}

fn load_combo_names(device_name: &str) -> Vec<String> {
    let path = combo_names_path(device_name);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Vec<String>>(&data) {
            return v;
        }
    }
    vec![]
}

fn save_combo_names(names: &[String], device_name: &str) {
    if let Ok(data) = serde_json::to_string(names) {
        let path = combo_names_path(device_name);
        if let Err(e) = std::fs::write(&path, &data) {
            log::warn!("save_combo_names failed at {:?}: {e}", path);
        } else {
            log::info!("save_combo_names ok → {:?}", path);
        }
    }
}

fn combo_display_name(combo_names: &[String], idx: usize) -> String {
    match combo_names.get(idx) {
        Some(name) if !name.trim().is_empty() => name.clone(),
        _ => format!("C{}", idx),
    }
}

fn key_override_names_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_id_slug(device_name);
    dir.join(format!("key_override_names_{}.json", slug))
}

fn load_key_override_names(device_name: &str) -> Vec<String> {
    let path = key_override_names_path(device_name);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Vec<String>>(&data) {
            return v;
        }
    }
    vec![]
}

fn save_key_override_names(names: &[String], device_name: &str) {
    if let Ok(data) = serde_json::to_string(names) {
        let path = key_override_names_path(device_name);
        if let Err(e) = std::fs::write(&path, &data) {
            log::warn!("save_key_override_names failed at {:?}: {e}", path);
        } else {
            log::info!("save_key_override_names ok → {:?}", path);
        }
    }
}

fn key_override_display_name(key_override_names: &[String], idx: usize) -> String {
    match key_override_names.get(idx) {
        Some(name) if !name.trim().is_empty() => name.clone(),
        _ => format!("KO{}", idx),
    }
}

fn macro_custom_name(macro_names: &[String], idx: usize) -> Option<String> {
    macro_names
        .get(idx)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn macro_display_name(macro_names: &[String], idx: usize) -> String {
    macro_custom_name(macro_names, idx).unwrap_or_else(|| format!("M{}", idx))
}

fn tap_dance_custom_name(tap_dance_names: &[String], idx: usize) -> Option<String> {
    tap_dance_names
        .get(idx)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn tap_dance_display_name(tap_dance_names: &[String], idx: usize) -> String {
    tap_dance_custom_name(tap_dance_names, idx).unwrap_or_else(|| format!("TD{}", idx))
}

fn app_accent() -> Color32 {
    crate::ui_style::accent()
}
fn app_panel_fill(dark: bool) -> Color32 {
    crate::ui_style::panel_fill(dark)
}
fn app_window_fill(dark: bool) -> Color32 {
    crate::ui_style::window_fill(dark)
}
fn app_surface_fill(dark: bool) -> Color32 {
    crate::ui_style::surface_fill(dark)
}
fn app_hover_fill(dark: bool) -> Color32 {
    crate::ui_style::hover_fill(dark)
}
fn app_border_color(dark: bool) -> Color32 {
    crate::ui_style::border_color(dark)
}
fn app_muted_text(dark: bool) -> Color32 {
    crate::ui_style::muted_text(dark)
}

fn keycode_label_with_macro_names(
    value: u16,
    custom: &[crate::keyboard::CustomKeycode],
    layer_names: &[String],
    macro_names: &[String],
    tap_dance_names: &[String],
) -> String {
    if (0x7700..=0x77FF).contains(&value) {
        let idx = (value - 0x7700) as usize;
        if let Some(name) = macro_custom_name(macro_names, idx) {
            return format!("M{}\n{}", idx, name);
        }
        return format!("M{}", idx);
    }
    if (0x5700..=0x57FF).contains(&value) {
        let idx = (value - 0x5700) as usize;
        if let Some(name) = tap_dance_custom_name(tap_dance_names, idx) {
            return format!("TD{}\n{}", idx, name);
        }
        return format!("TD{}", idx);
    }
    keycode_label_with_names(value, custom, layer_names)
}

fn keycode_tooltip_with_macro_names(
    value: u16,
    custom: &[crate::keyboard::CustomKeycode],
    layer_names: &[String],
    macro_names: &[String],
    tap_dance_names: &[String],
) -> String {
    if (0x7700..=0x77FF).contains(&value) {
        let idx = (value - 0x7700) as usize;
        let name = macro_display_name(macro_names, idx);
        return format!("{} — macro {}", name, idx);
    }
    if (0x5700..=0x57FF).contains(&value) {
        let idx = (value - 0x5700) as usize;
        let name = tap_dance_display_name(tap_dance_names, idx);
        return format!("{} — tap dance {}", name, idx);
    }
    keycode_tooltip(value, custom, layer_names)
}
use crate::keyboard::KeyboardLayout;
use crate::keycode::{key_label_font_sizes, keycode_label_with_names, keycode_tooltip};
use crate::keycode_picker::{egui_key_to_qmk, KeycodePicker, KeycodeTab};
use egui::{Color32, FontId, RichText, Sense, Stroke, Vec2};

#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc;

/// Result sent back from the background connect thread.
#[cfg(not(target_arch = "wasm32"))]
struct ConnectResult {
    device_name: String,
    layout: KeyboardLayout,
    layer_count: usize,
    /// Persistent ZMK connection (if ZMK device)
    zmk_conn: Option<crate::zmk::ZmkConnection>,
    /// ZMK lock state at connect time
    zmk_lock_state: i32,
    /// Macro texts read from device
    macro_texts: Vec<String>,
    /// Tap dance entries
    tap_dance_entries: Vec<crate::keycode_picker::TapDanceEntry>,
    /// Combo entries
    combo_entries: Vec<ComboEntry>,
    /// Global combo timeout/term from QMK settings, if supported
    combo_term: Option<u16>,
    /// Auto Shift flags from QMK settings, if supported
    auto_shift_options: AutoShiftOptionsState,
    /// Auto Shift timeout from QMK settings, if supported
    auto_shift_timeout: Option<u16>,
    /// Mouse Keys settings from QMK settings, if supported (qsid 9..=17)
    mouse_keys_settings: MouseKeysSettingsState,
    /// Runtime RGB settings, if supported by the current Vial/QMK lighting backend
    rgb_settings: RgbSettingsState,
    /// Key Override entries
    key_override_entries: Vec<KeyOverrideEntry>,
    /// Alt Repeat entries
    alt_repeat_entries: Vec<AltRepeatKeyEntry>,
}

#[cfg(not(target_arch = "wasm32"))]
enum ConnectState {
    Idle,
    Loading(mpsc::Receiver<Result<ConnectResult, String>>),
}

/// Message from background ZMK operation thread
#[cfg(not(target_arch = "wasm32"))]
enum ZmkOpResult {
    AddLayerOk { layer_idx: u32, layer_name: String },
    AddLayerFail(String),
    RemoveLayerOk,
    RemoveLayerFail(String),
    SaveOk,
    SaveFail(String),
    DiscardOk,
    DiscardFail(String),
    LockStateChanged(i32),
}

fn toggle_handed_modifier(value: u16) -> Option<u16> {
    match value {
        0x00E0 => Some(0x00E4),
        0x00E4 => Some(0x00E0),
        0x00E1 => Some(0x00E5),
        0x00E5 => Some(0x00E1),
        0x00E2 => Some(0x00E6),
        0x00E6 => Some(0x00E2),
        0x00E3 => Some(0x00E7),
        0x00E7 => Some(0x00E3),
        0x52A1 => Some(0x52B1),
        0x52B1 => Some(0x52A1),
        0x52A2 => Some(0x52B2),
        0x52B2 => Some(0x52A2),
        0x52A4 => Some(0x52B4),
        0x52B4 => Some(0x52A4),
        0x52A8 => Some(0x52B8),
        0x52B8 => Some(0x52A8),
        _ => {
            let base = value & 0xFF00;
            let low = value & 0x00FF;
            match base {
                0x2100 => Some(0x3100 | low),
                0x3100 => Some(0x2100 | low),
                0x2200 => Some(0x3200 | low),
                0x3200 => Some(0x2200 | low),
                0x2400 => Some(0x3400 | low),
                0x3400 => Some(0x2400 | low),
                0x2800 => Some(0x3800 | low),
                0x3800 => Some(0x2800 | low),
                _ => None,
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct ComboEntry {
    keys: [u16; 4],
    output: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct KeyOverrideOptionsState {
    activation_trigger_down: bool,
    activation_required_mod_down: bool,
    activation_negative_mod_up: bool,
    one_mod: bool,
    no_reregister_trigger: bool,
    no_unregister_on_other_key_down: bool,
    enabled: bool,
}

impl KeyOverrideOptionsState {
    fn from_bits(bits: u8) -> Self {
        Self {
            activation_trigger_down: bits & (1 << 0) != 0,
            activation_required_mod_down: bits & (1 << 1) != 0,
            activation_negative_mod_up: bits & (1 << 2) != 0,
            one_mod: bits & (1 << 3) != 0,
            no_reregister_trigger: bits & (1 << 4) != 0,
            no_unregister_on_other_key_down: bits & (1 << 5) != 0,
            enabled: bits & (1 << 7) != 0,
        }
    }

    fn bits(&self) -> u8 {
        (self.activation_trigger_down as u8) << 0
            | (self.activation_required_mod_down as u8) << 1
            | (self.activation_negative_mod_up as u8) << 2
            | (self.one_mod as u8) << 3
            | (self.no_reregister_trigger as u8) << 4
            | (self.no_unregister_on_other_key_down as u8) << 5
            | (self.enabled as u8) << 7
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct KeyOverrideEntry {
    trigger: u16,
    replacement: u16,
    layers: u16,
    trigger_mods: u8,
    negative_mod_mask: u8,
    suppressed_mods: u8,
    options: KeyOverrideOptionsState,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct AltRepeatKeyOptionsState {
    default_to_this_alt_key: bool,
    bidirectional: bool,
    ignore_mod_handedness: bool,
    enabled: bool,
}

impl AltRepeatKeyOptionsState {
    fn from_bits(bits: u8) -> Self {
        Self {
            default_to_this_alt_key: bits & (1 << 0) != 0,
            bidirectional: bits & (1 << 1) != 0,
            ignore_mod_handedness: bits & (1 << 2) != 0,
            enabled: bits & (1 << 3) != 0,
        }
    }

    fn bits(self) -> u8 {
        (self.default_to_this_alt_key as u8)
            | ((self.bidirectional as u8) << 1)
            | ((self.ignore_mod_handedness as u8) << 2)
            | ((self.enabled as u8) << 3)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AltRepeatKeyEntry {
    keycode: u16,
    alt_keycode: u16,
    allowed_mods: u8,
    options: AltRepeatKeyOptionsState,
}

#[derive(Clone, Copy, Debug, Default)]
struct AutoShiftOptionsState {
    enabled: bool,
    enable_for_modifiers: bool,
    no_special: bool,
    no_numeric: bool,
    no_alpha: bool,
    enable_keyrepeat: bool,
    disable_keyrepeat_timeout: bool,
}

impl AutoShiftOptionsState {
    fn from_bits(bits: u8) -> Self {
        Self {
            enabled: bits & (1 << 0) != 0,
            enable_for_modifiers: bits & (1 << 1) != 0,
            no_special: bits & (1 << 2) != 0,
            no_numeric: bits & (1 << 3) != 0,
            no_alpha: bits & (1 << 4) != 0,
            enable_keyrepeat: bits & (1 << 5) != 0,
            disable_keyrepeat_timeout: bits & (1 << 6) != 0,
        }
    }

    fn bits(self) -> u8 {
        (self.enabled as u8)
            | ((self.enable_for_modifiers as u8) << 1)
            | ((self.no_special as u8) << 2)
            | ((self.no_numeric as u8) << 3)
            | ((self.no_alpha as u8) << 4)
            | ((self.enable_keyrepeat as u8) << 5)
            | ((self.disable_keyrepeat_timeout as u8) << 6)
    }
}

/// Mirrors Vial GUI Mouse keys settings (qsid 9..=17). All values are u16.
#[derive(Clone, Copy, Debug, Default)]
struct MouseKeysSettingsState {
    /// qsid 9: Delay between pressing a movement key and cursor movement
    delay: u16,
    /// qsid 10: Time between cursor movements in milliseconds
    interval: u16,
    /// qsid 11: Step size
    max_speed: u16,
    /// qsid 12: Maximum cursor speed at which acceleration stops
    time_to_max: u16,
    /// qsid 13: Time until maximum cursor speed is reached
    move_delta: u16,
    /// qsid 14: Delay between pressing a wheel key and wheel movement
    wheel_delay: u16,
    /// qsid 15: Time between wheel movements
    wheel_interval: u16,
    /// qsid 16: Maximum number of scroll steps per scroll action
    wheel_max_speed: u16,
    /// qsid 17: Time until maximum scroll speed is reached
    wheel_time_to_max: u16,
    /// Whether any of the qsids were readable (firmware support flag)
    supported: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum RgbSupportKind {
    #[default]
    None,
    QmkRgblight,
    VialRgb,
}

#[derive(Clone, Debug, Default)]
struct RgbSettingsState {
    supported: bool,
    kind: RgbSupportKind,
    effect: u16,
    brightness: u8,
    speed: u8,
    hue: u8,
    saturation: u8,
    max_brightness: u8,
    supported_effects: Vec<u16>,
    last_enabled_effect: u16,
}

impl RgbSettingsState {
    fn is_enabled(&self) -> bool {
        self.supported && self.effect != 0
    }

    fn fallback_effect(&self) -> u16 {
        match self.kind {
            RgbSupportKind::QmkRgblight => 1,
            RgbSupportKind::VialRgb => 2,
            RgbSupportKind::None => 0,
        }
    }

    fn effect_or_default(&self) -> u16 {
        let candidate = if self.last_enabled_effect != 0 {
            self.last_enabled_effect
        } else {
            self.fallback_effect()
        };
        match self.kind {
            RgbSupportKind::VialRgb => {
                if self.supported_effects.is_empty() || self.supported_effects.contains(&candidate)
                {
                    candidate
                } else {
                    self.supported_effects.first().copied().unwrap_or(candidate)
                }
            }
            _ => candidate,
        }
    }
}

const QMK_RGBLIGHT_EFFECTS: &[(u16, &str)] = &[
    (0, "All Off"),
    (1, "Solid Color"),
    (2, "Breathing 1"),
    (3, "Breathing 2"),
    (4, "Breathing 3"),
    (5, "Breathing 4"),
    (6, "Rainbow Mood 1"),
    (7, "Rainbow Mood 2"),
    (8, "Rainbow Mood 3"),
    (9, "Rainbow Swirl 1"),
    (10, "Rainbow Swirl 2"),
    (11, "Rainbow Swirl 3"),
    (12, "Rainbow Swirl 4"),
    (13, "Rainbow Swirl 5"),
    (14, "Rainbow Swirl 6"),
    (15, "Snake 1"),
    (16, "Snake 2"),
    (17, "Snake 3"),
    (18, "Snake 4"),
    (19, "Snake 5"),
    (20, "Snake 6"),
    (21, "Knight 1"),
    (22, "Knight 2"),
    (23, "Knight 3"),
    (24, "Christmas"),
    (25, "Gradient 1"),
    (26, "Gradient 2"),
    (27, "Gradient 3"),
    (28, "Gradient 4"),
    (29, "Gradient 5"),
    (30, "Gradient 6"),
    (31, "Gradient 7"),
    (32, "Gradient 8"),
    (33, "Gradient 9"),
    (34, "Gradient 10"),
    (35, "RGB Test"),
    (36, "Alternating"),
];

const VIALRGB_EFFECTS: &[(u16, &str)] = &[
    (0, "Disable"),
    (1, "Direct Control"),
    (2, "Solid Color"),
    (3, "Alphas Mods"),
    (4, "Gradient Up Down"),
    (5, "Gradient Left Right"),
    (6, "Breathing"),
    (7, "Band Sat"),
    (8, "Band Val"),
    (9, "Band Pinwheel Sat"),
    (10, "Band Pinwheel Val"),
    (11, "Band Spiral Sat"),
    (12, "Band Spiral Val"),
    (13, "Cycle All"),
    (14, "Cycle Left Right"),
    (15, "Cycle Up Down"),
    (16, "Rainbow Moving Chevron"),
    (17, "Cycle Out In"),
    (18, "Cycle Out In Dual"),
    (19, "Cycle Pinwheel"),
    (20, "Cycle Spiral"),
    (21, "Dual Beacon"),
    (22, "Rainbow Beacon"),
    (23, "Rainbow Pinwheels"),
    (24, "Raindrops"),
    (25, "Jellybean Raindrops"),
    (26, "Hue Breathing"),
    (27, "Hue Pendulum"),
    (28, "Hue Wave"),
    (29, "Typing Heatmap"),
    (30, "Digital Rain"),
    (31, "Solid Reactive Simple"),
    (32, "Solid Reactive"),
    (33, "Solid Reactive Wide"),
    (34, "Solid Reactive Multiwide"),
    (35, "Solid Reactive Cross"),
    (36, "Solid Reactive Multicross"),
    (37, "Solid Reactive Nexus"),
    (38, "Solid Reactive Multinexus"),
    (39, "Splash"),
    (40, "Multisplash"),
    (41, "Solid Splash"),
    (42, "Solid Multisplash"),
    (43, "Pixel Rain"),
    (44, "Pixel Fractal"),
];

fn rgb_effect_options(state: &RgbSettingsState) -> Vec<(u16, &'static str)> {
    match state.kind {
        RgbSupportKind::QmkRgblight => QMK_RGBLIGHT_EFFECTS.to_vec(),
        RgbSupportKind::VialRgb => VIALRGB_EFFECTS
            .iter()
            .copied()
            .filter(|(id, _)| {
                *id == 0
                    || state.supported_effects.is_empty()
                    || state.supported_effects.contains(id)
            })
            .collect(),
        RgbSupportKind::None => vec![],
    }
}

fn rgb_effect_supports_color(kind: RgbSupportKind, effect: u16) -> bool {
    match kind {
        RgbSupportKind::QmkRgblight => matches!(effect, 1..=5 | 15..=36),
        RgbSupportKind::VialRgb => effect != 0,
        RgbSupportKind::None => false,
    }
}

fn rgb_effect_supports_speed(kind: RgbSupportKind, effect: u16) -> bool {
    match kind {
        RgbSupportKind::QmkRgblight => matches!(effect, 2..=36),
        RgbSupportKind::VialRgb => !matches!(effect, 0..=5),
        RgbSupportKind::None => false,
    }
}

fn rgb_picker_contrast(color: impl Into<egui::Rgba>) -> Color32 {
    if color.into().intensity() < 0.5 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

fn compact_rgb_slider_1d(
    ui: &mut egui::Ui,
    value: &mut f32,
    color_at: impl Fn(f32) -> Color32,
) -> bool {
    let desired_size = Vec2::new(ui.spacing().slider_width, 18.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
    let mut changed = false;

    if let Some(pos) = response.interact_pointer_pos() {
        let new_value = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        if (*value - new_value).abs() > f32::EPSILON {
            *value = new_value;
            changed = true;
        }
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);
        let mut mesh = egui::epaint::Mesh::default();
        const N: u32 = 36;
        for i in 0..N {
            let t = i as f32 / (N - 1) as f32;
            let x = egui::lerp(rect.x_range(), t);
            mesh.colored_vertex(egui::pos2(x, rect.top()), color_at(t));
            mesh.colored_vertex(egui::pos2(x, rect.bottom()), color_at(t));
            if i + 1 < N {
                let idx = i * 2;
                mesh.add_triangle(idx, idx + 1, idx + 2);
                mesh.add_triangle(idx + 1, idx + 2, idx + 3);
            }
        }
        ui.painter().add(egui::Shape::mesh(mesh));

        let x = egui::lerp(rect.x_range(), *value);
        let picked_color = color_at(*value);
        let stroke = Stroke::new(1.2, rgb_picker_contrast(picked_color));
        let handle_rect = egui::Rect::from_center_size(
            egui::pos2(x, rect.center().y),
            Vec2::new(10.0, rect.height() + 6.0),
        );
        ui.painter().rect(
            handle_rect,
            3.0,
            picked_color,
            stroke,
            egui::StrokeKind::Inside,
        );
        ui.painter().rect_stroke(
            rect,
            2.0,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
    }

    changed
}

fn compact_rgb_slider_2d(
    ui: &mut egui::Ui,
    x_value: &mut f32,
    y_value: &mut f32,
    color_at: impl Fn(f32, f32) -> Color32,
) -> bool {
    let desired_size = Vec2::splat(ui.spacing().slider_width);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
    let mut changed = false;

    if let Some(pos) = response.interact_pointer_pos() {
        let new_x_value = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let new_y_value = ((rect.bottom() - pos.y) / rect.height()).clamp(0.0, 1.0);
        if (*x_value - new_x_value).abs() > f32::EPSILON {
            *x_value = new_x_value;
            changed = true;
        }
        if (*y_value - new_y_value).abs() > f32::EPSILON {
            *y_value = new_y_value;
            changed = true;
        }
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);
        let mut mesh = egui::epaint::Mesh::default();
        const N: u32 = 36;
        for xi in 0..N {
            let xt = xi as f32 / (N - 1) as f32;
            let x = egui::lerp(rect.x_range(), xt);
            for yi in 0..N {
                let yt = yi as f32 / (N - 1) as f32;
                let y = egui::lerp(rect.y_range(), 1.0 - yt);
                mesh.colored_vertex(egui::pos2(x, y), color_at(xt, yt));
                if xi + 1 < N && yi + 1 < N {
                    let tl = yi + xi * N;
                    mesh.add_triangle(tl, tl + 1, tl + N);
                    mesh.add_triangle(tl + 1, tl + N, tl + N + 1);
                }
            }
        }
        ui.painter().add(egui::Shape::mesh(mesh));

        let x = egui::lerp(rect.x_range(), *x_value);
        let y = egui::lerp(rect.y_range(), 1.0 - *y_value);
        let picked_color = color_at(*x_value, *y_value);
        let stroke = Stroke::new(1.6, rgb_picker_contrast(picked_color));
        ui.painter().circle_stroke(egui::pos2(x, y), 10.0, stroke);
        ui.painter().rect_stroke(
            rect,
            2.0,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
    }

    changed
}

fn compact_rgb_color_picker(ui: &mut egui::Ui, hsva: &mut egui::ecolor::Hsva) -> bool {
    let mut changed = false;
    let mut h = hsva.h.rem_euclid(1.0);
    let mut s = hsva.s.clamp(0.0, 1.0);
    let mut v = hsva.v.clamp(0.0, 1.0);

    changed |= compact_rgb_slider_2d(ui, &mut s, &mut v, |s, v| {
        egui::ecolor::Hsva { h, s, v, a: 1.0 }.into()
    });
    ui.add_space(6.0);
    changed |= compact_rgb_slider_1d(ui, &mut h, |h| {
        egui::ecolor::Hsva {
            h,
            s: 1.0,
            v: 1.0,
            a: 1.0,
        }
        .into()
    });

    if changed {
        hsva.h = h;
        hsva.s = s;
        hsva.v = v;
    }
    changed
}

#[cfg(not(target_arch = "wasm32"))]
fn load_rgb_settings(
    dev_conn: &crate::hid::HidDevice,
    layout: &KeyboardLayout,
) -> RgbSettingsState {
    let mut candidates = Vec::new();
    match layout.lighting_mode.as_deref() {
        Some("vialrgb") => {
            candidates.extend([RgbSupportKind::VialRgb, RgbSupportKind::QmkRgblight])
        }
        Some("qmk_rgblight") | Some("qmk_backlight_rgblight") => {
            candidates.extend([RgbSupportKind::QmkRgblight, RgbSupportKind::VialRgb]);
        }
        _ if layout.supports_rgb => {
            candidates.extend([RgbSupportKind::VialRgb, RgbSupportKind::QmkRgblight])
        }
        _ => return RgbSettingsState::default(),
    }

    for kind in candidates {
        match kind {
            RgbSupportKind::VialRgb => {
                let Ok((version, max_brightness)) = dev_conn.get_vialrgb_info() else {
                    continue;
                };
                if version != 1 {
                    continue;
                }
                let Ok((effect, speed, hue, saturation, brightness)) = dev_conn.get_vialrgb_mode()
                else {
                    continue;
                };
                let mut supported_effects =
                    dev_conn.get_vialrgb_supported_effects().unwrap_or_default();
                if !supported_effects.contains(&0) {
                    supported_effects.insert(0, 0);
                }
                let mut state = RgbSettingsState {
                    supported: true,
                    kind,
                    effect,
                    brightness,
                    speed,
                    hue,
                    saturation,
                    max_brightness,
                    supported_effects,
                    last_enabled_effect: effect,
                };
                if state.last_enabled_effect == 0 {
                    state.last_enabled_effect = state.fallback_effect();
                }
                return state;
            }
            RgbSupportKind::QmkRgblight => {
                let Ok(brightness) = dev_conn.get_qmk_rgblight_brightness() else {
                    continue;
                };
                let Ok(effect) = dev_conn.get_qmk_rgblight_effect() else {
                    continue;
                };
                let speed = dev_conn.get_qmk_rgblight_effect_speed().unwrap_or(0);
                let (hue, saturation) = dev_conn.get_qmk_rgblight_color().unwrap_or((0, 0));
                let mut state = RgbSettingsState {
                    supported: true,
                    kind,
                    effect: effect as u16,
                    brightness,
                    speed,
                    hue,
                    saturation,
                    max_brightness: u8::MAX,
                    supported_effects: vec![],
                    last_enabled_effect: effect as u16,
                };
                if state.last_enabled_effect == 0 {
                    state.last_enabled_effect = state.fallback_effect();
                }
                return state;
            }
            RgbSupportKind::None => {}
        }
    }

    RgbSettingsState::default()
}

/// Returns true if the given Vial keycode is a QMK mouse key (0x00CD..=0x00DF).
fn is_mouse_keycode(kc: u16) -> bool {
    (0x00CD..=0x00DF).contains(&kc)
}

fn is_alt_repeat_keycode(kc: u16) -> bool {
    kc == 0x7C7A
}

#[derive(Clone, Debug)]
enum UndoAction {
    Key {
        layer: usize,
        key_idx: usize,
        old_kc: u16,
        old_binding: crate::zmk::ZmkBinding,
    },
    Encoder {
        layer: usize,
        encoder_visual_idx: usize,
        old_kc: u16,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyOverridePickField {
    Trigger,
    Replacement,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AltRepeatPickField {
    LastKey,
    AltKey,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MainMenuTab {
    Keyboard,
    Advanced,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ComboPickField {
    Trigger(usize),
    Output,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    MatrixTester,
}

pub struct EntropyApp {
    device_manager: DeviceManager,
    selected_device: Option<usize>,
    selected_layer: usize,
    selected_key: Option<(usize, usize)>,
    selected_encoder: Option<(usize, usize)>,
    layout: Option<KeyboardLayout>,
    layer_count: usize,
    keycode_picker: KeycodePicker,
    status_msg: String,
    #[cfg(not(target_arch = "wasm32"))]
    connect_state: ConnectState,
    /// Persistent open HID device for real-time writes (Vial)
    #[cfg(not(target_arch = "wasm32"))]
    hid_device: Option<crate::hid::HidDevice>,
    /// Persistent ZMK connection passed from the connect thread
    #[cfg(not(target_arch = "wasm32"))]
    zmk_conn: Option<crate::zmk::ZmkConnection>,
    /// Current firmware type (mirrors layout.firmware)
    firmware: FirmwareProtocol,
    zmk_base_layer_count: usize,
    zmk_no_extra_layers: bool,
    /// Undo stack for key and encoder assignments
    undo_stack: Vec<UndoAction>,
    /// Frame counter for periodic device scan
    scan_frame: u32,
    /// Layer to preview on hover (None = show selected_layer)
    hover_layer: Option<usize>,
    /// Key index hovered in previous frame (for hint display)
    prev_hovered_key: Option<usize>,
    prev_hovered_encoder: bool,
    prev_hovered_encoder_keycode: Option<u16>,
    /// Set when secondary click was handled by a key (prevents global jump-back)
    secondary_click_handled: bool,
    /// Deferred left/right modifier swap, applied after Ctrl is released
    pending_handed_swap: Option<(usize, usize, u16)>,
    /// Animation progress for hover layer preview (0.0 = hidden, 1.0 = fully shown)
    hover_layer_progress: f32,
    /// Stack of layers to return to on right-click (last = most recent)
    jump_back_stack: Vec<usize>,
    dark_mode: bool,
    main_menu_tab: MainMenuTab,
    combo_entries: Vec<ComboEntry>,
    combo_names: Vec<String>,
    selected_combo: usize,
    combo_dirty: bool,
    combo_names_dirty: bool,
    combo_term: Option<u16>,
    auto_shift_options: AutoShiftOptionsState,
    auto_shift_timeout: Option<u16>,
    mouse_keys_settings: MouseKeysSettingsState,
    mouse_keys_window_open: bool,
    alt_repeat_entries: Vec<AltRepeatKeyEntry>,
    alt_repeat_window_open: bool,
    selected_alt_repeat: usize,
    alt_repeat_visible_count: usize,
    alt_repeat_pick_target: Option<AltRepeatPickField>,
    alt_repeat_reopen_after_pick: bool,
    modal_focus_pending: bool,
    prev_any_floating_window_open: bool,
    last_single_instance_signal: String,
    rgb_settings: RgbSettingsState,
    rgb_window_open: bool,
    encoder_visibility: Vec<bool>,
    encoder_visibility_window_open: bool,
    combo_term_dirty: bool,
    combo_window_open: bool,
    combo_reopen_after_pick: bool,
    combo_visible_count: usize,
    combo_capture_open: bool,
    combo_capture_keys: Vec<u16>,
    combo_undo_stack: Vec<(Vec<ComboEntry>, Vec<String>, Option<u16>, usize, usize)>,
    combo_pick_target: Option<(usize, ComboPickField)>,
    auto_shift_window_open: bool,
    key_override_entries: Vec<KeyOverrideEntry>,
    key_override_names: Vec<String>,
    key_override_window_open: bool,
    key_override_visible_count: usize,
    key_override_undo_stack: Vec<(Vec<KeyOverrideEntry>, Vec<String>, usize, usize)>,
    selected_key_override: usize,
    key_override_pick_target: Option<KeyOverridePickField>,
    key_override_reopen_after_pick: bool,
    matrix_tester_pressed: Vec<bool>,
    matrix_tester_ever_pressed: Vec<bool>,
    matrix_tester_last_poll: std::time::Instant,
    matrix_tester_unlock_prompted: bool,
    macro_auto_unlock_cancelled: bool,
    settings_tab: SettingsTab,
    layer_names: Vec<String>,
    editing_layer: Option<usize>, // layer being renamed
    editing_layer_text: String,
    editing_layer_focus_requested: bool,
    /// Current connected device name (for per-device layer names)
    current_device_name: String,
    /// ZMK lock state: 0=Locked, 1=Unlocked
    zmk_lock_state: i32,
    /// Whether ZMK has unsaved changes
    zmk_has_unsaved: bool,
    /// Vial unlock dialog open
    unlock_open: bool,
    vial_unlock_keys: Vec<(u8, u8)>,
    vial_unlock_polling: bool,
    vial_unlock_counter: u8,
    vial_unlock_best: u8,
    vial_unlock_total: u8,
    /// Channel for ZMK background operation results
    #[cfg(not(target_arch = "wasm32"))]
    zmk_op_rx: Option<mpsc::Receiver<ZmkOpResult>>,
}

impl EntropyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            #[cfg(not(target_arch = "wasm32"))]
            hid_device: None,
            #[cfg(not(target_arch = "wasm32"))]
            zmk_conn: None,
            firmware: FirmwareProtocol::Vial,
            zmk_base_layer_count: 0,
            zmk_no_extra_layers: false,
            undo_stack: Vec::new(),
            scan_frame: 0,
            hover_layer: None,
            prev_hovered_key: None,
            prev_hovered_encoder: false,
            prev_hovered_encoder_keycode: None,
            secondary_click_handled: false,
            pending_handed_swap: None,
            hover_layer_progress: 0.0,
            jump_back_stack: Vec::new(),
            device_manager: DeviceManager::new(),
            selected_device: None,
            selected_layer: 0,
            selected_key: None,
            selected_encoder: None,
            layout: None,
            layer_count: 4,
            keycode_picker: KeycodePicker::default(),
            status_msg: String::new(),
            dark_mode: false,
            main_menu_tab: MainMenuTab::Keyboard,
            combo_entries: vec![],
            combo_names: vec![],
            selected_combo: 0,
            combo_dirty: false,
            combo_names_dirty: false,
            combo_term: None,
            auto_shift_options: AutoShiftOptionsState::default(),
            auto_shift_timeout: None,
            mouse_keys_settings: MouseKeysSettingsState::default(),
            mouse_keys_window_open: false,
            alt_repeat_entries: vec![],
            alt_repeat_window_open: false,
            selected_alt_repeat: 0,
            alt_repeat_visible_count: 1,
            alt_repeat_pick_target: None,
            alt_repeat_reopen_after_pick: false,
            modal_focus_pending: false,
            prev_any_floating_window_open: false,
            last_single_instance_signal: read_single_instance_signal(),
            rgb_settings: RgbSettingsState::default(),
            rgb_window_open: false,
            encoder_visibility: vec![],
            encoder_visibility_window_open: false,
            combo_term_dirty: false,
            combo_window_open: false,
            combo_reopen_after_pick: false,
            combo_visible_count: 1,
            combo_capture_open: false,
            combo_capture_keys: Vec::new(),
            combo_undo_stack: Vec::new(),
            combo_pick_target: None,
            auto_shift_window_open: false,
            key_override_entries: Vec::new(),
            key_override_names: vec![],
            key_override_window_open: false,
            key_override_visible_count: 1,
            key_override_undo_stack: Vec::new(),
            selected_key_override: 0,
            key_override_pick_target: None,
            key_override_reopen_after_pick: false,
            matrix_tester_pressed: Vec::new(),
            matrix_tester_ever_pressed: Vec::new(),
            matrix_tester_last_poll: std::time::Instant::now(),
            matrix_tester_unlock_prompted: false,
            macro_auto_unlock_cancelled: false,
            settings_tab: SettingsTab::MatrixTester,
            layer_names: load_layer_names("default"),
            editing_layer: None,
            editing_layer_text: String::new(),
            editing_layer_focus_requested: false,
            current_device_name: String::new(),
            zmk_lock_state: 1, // Unlocked by default
            zmk_has_unsaved: false,
            unlock_open: false,
            vial_unlock_keys: vec![],
            vial_unlock_polling: false,
            vial_unlock_counter: 0,
            vial_unlock_best: 50,
            vial_unlock_total: 50,
            #[cfg(not(target_arch = "wasm32"))]
            zmk_op_rx: None,
            #[cfg(not(target_arch = "wasm32"))]
            connect_state: ConnectState::Idle,
        };
        // Auto-connect to first device if available
        #[cfg(not(target_arch = "wasm32"))]
        if !app.device_manager.devices().is_empty() {
            app.selected_device = Some(0);
            app.start_connect(0);
        }
        app
    }

    /// Spawn background thread to connect + load layout/keycodes.
    #[cfg(not(target_arch = "wasm32"))]
    fn start_connect(&mut self, device_idx: usize) {
        let dev = match self.device_manager.devices().get(device_idx) {
            Some(d) => d.clone(),
            None => {
                self.status_msg = "Device not found".into();
                return;
            }
        };

        self.status_msg = format!("Connecting to {}…", dev.name);
        self.layout = None;
        self.selected_key = None;
        self.selected_encoder = None;
        self.selected_layer = 0;
        self.hid_device = None;
        self.zmk_conn = None;
        self.zmk_has_unsaved = false;
        self.zmk_lock_state = 1; // assume unlocked until we know
        self.zmk_no_extra_layers = false;
        self.combo_window_open = false;
        self.combo_reopen_after_pick = false;
        self.combo_visible_count = 1;
        self.combo_capture_open = false;
        self.combo_capture_keys.clear();
        self.combo_undo_stack.clear();
        self.combo_pick_target = None;
        self.combo_dirty = false;
        self.combo_names_dirty = false;
        self.combo_term_dirty = false;
        self.auto_shift_options = AutoShiftOptionsState::default();
        self.auto_shift_timeout = None;
        self.auto_shift_window_open = false;
        self.mouse_keys_settings = MouseKeysSettingsState::default();
        self.mouse_keys_window_open = false;
        self.alt_repeat_entries.clear();
        self.alt_repeat_window_open = false;
        self.selected_alt_repeat = 0;
        self.alt_repeat_visible_count = 1;
        self.alt_repeat_pick_target = None;
        self.alt_repeat_reopen_after_pick = false;
        self.rgb_settings = RgbSettingsState::default();
        self.rgb_window_open = false;
        self.encoder_visibility.clear();
        self.encoder_visibility_window_open = false;
        self.key_override_entries.clear();
        self.key_override_names.clear();
        self.key_override_window_open = false;
        self.key_override_visible_count = 1;
        self.key_override_undo_stack.clear();
        self.selected_key_override = 0;
        self.key_override_pick_target = None;
        self.key_override_reopen_after_pick = false;
        self.reset_matrix_tester_state();

        let (tx, rx) = mpsc::channel();
        self.connect_state = ConnectState::Loading(rx);

        std::thread::spawn(move || {
            let result = (|| -> Result<ConnectResult, String> {
                match dev.firmware {
                    FirmwareProtocol::Vial => {
                        use crate::hid::HidDevice;

                        log::info!("Opening HID device: {}", dev.path);
                        let dev_conn =
                            HidDevice::open(&dev.path).map_err(|e| format!("Open failed: {e}"))?;

                        log::info!("Getting protocol version…");
                        match dev_conn.get_protocol_version() {
                            Ok(v) => log::info!("VIA protocol version: {v}"),
                            Err(e) => log::warn!("get_protocol_version failed: {e}"),
                        }

                        log::info!("Getting layer count…");
                        let layer_count = dev_conn
                            .get_layer_count()
                            .map(|c| c as usize)
                            .unwrap_or_else(|e| {
                                log::warn!("get_layer_count failed: {e}, defaulting to 4");
                                4
                            });
                        log::info!("Layer count: {layer_count}");

                        log::info!("Getting layout JSON…");
                        let json = dev_conn
                            .get_layout_json()
                            .map_err(|e| format!("Layout read failed: {e}"))?;

                        let mut layout = KeyboardLayout::from_vial_json(&json)
                            .map_err(|e| format!("Layout parse failed: {e}"))?;

                        // Override coords from embedded layout
                        log::info!("Vial: looking up embedded layout for '{}'", dev.name);
                        if let Some((emb, ref_keys)) = crate::layouts::lookup_layout(&dev.name) {
                            log::info!(
                                "Vial: found embedded layout '{}' with {} keys",
                                emb.name,
                                ref_keys.len()
                            );
                            use std::collections::HashMap;
                            let ref_map: HashMap<(u8, u8), &crate::keyboard::PhysicalKey> =
                                ref_keys.iter().map(|k| ((k.row, k.col), k)).collect();
                            let mut patched = 0usize;
                            for key in &mut layout.keys {
                                if let Some(rk) = ref_map.get(&(key.row, key.col)) {
                                    key.x = rk.x;
                                    key.y = rk.y;
                                    key.rotation = 0.0;
                                    key.rotation_x = 0.0;
                                    key.rotation_y = 0.0;
                                    patched += 1;
                                }
                            }
                            log::info!("Vial: patched {} keys", patched);
                        } else {
                            log::info!("Vial: no embedded layout found for '{}'", dev.name);
                        }

                        let num_keys = layout.keys.len();
                        layout.layers = vec![vec![0u16; num_keys]; layer_count];

                        match dev_conn.get_keymap_buffer(layer_count, layout.rows, layout.cols) {
                            Ok(buf) => {
                                for layer in 0..layer_count {
                                    for (ki, key) in layout.keys.iter().enumerate() {
                                        let idx = layer * layout.rows * layout.cols
                                            + key.row as usize * layout.cols
                                            + key.col as usize;
                                        if let Some(&kc) = buf.get(idx) {
                                            layout.layers[layer][ki] = kc;
                                        }
                                    }
                                }
                                log::info!("Keymap loaded from buffer");
                            }
                            Err(e) => {
                                log::warn!("get_keymap_buffer failed: {e}");
                            }
                        }

                        if !layout.encoders.is_empty() {
                            layout.encoder_layers =
                                vec![vec![0u16; layout.encoders.len()]; layer_count];
                            let encoder_count = layout.encoder_count();
                            for layer in 0..layer_count {
                                let mut per_encoder = vec![(0u16, 0u16); encoder_count];
                                for encoder_idx in 0..encoder_count {
                                    match dev_conn.get_encoder(layer as u8, encoder_idx as u8) {
                                        Ok((ccw, cw)) => per_encoder[encoder_idx] = (ccw, cw),
                                        Err(e) => log::warn!(
                                            "get_encoder(layer={}, idx={}): {}",
                                            layer,
                                            encoder_idx,
                                            e
                                        ),
                                    }
                                }
                                for (visual_idx, encoder) in layout.encoders.iter().enumerate() {
                                    if let Some((ccw, cw)) =
                                        per_encoder.get(encoder.encoder_idx as usize)
                                    {
                                        layout.encoder_layers[layer][visual_idx] =
                                            if encoder.direction == 0 { *ccw } else { *cw };
                                    }
                                }
                            }
                        }

                        let mut firmware_layer_names = Vec::new();
                        for layer in 0..layer_count.min(16) {
                            match dev_conn.get_qmk_setting_string(200 + layer as u16) {
                                Ok(name) if !name.is_empty() => firmware_layer_names.push(name),
                                Ok(_) => firmware_layer_names.push(layer.to_string()),
                                Err(_) => {
                                    firmware_layer_names.clear();
                                    break;
                                }
                            }
                        }
                        if !firmware_layer_names.is_empty() {
                            layout.layer_names = firmware_layer_names;
                        }

                        // Read macros
                        let macro_texts = match dev_conn.get_macro_count() {
                            Ok(count) => {
                                log::info!("Macro count: {count}");
                                match dev_conn.get_macro_buffer_size() {
                                    Ok(size) => {
                                        log::info!("Macro buffer size: {size}");
                                        match dev_conn.get_macro_buffer(size) {
                                            Ok(buf) => {
                                                crate::hid::HidDevice::parse_macros(&buf, count)
                                            }
                                            Err(e) => {
                                                log::warn!("get_macro_buffer: {e}");
                                                vec![String::new(); count as usize]
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("get_macro_buffer_size: {e}");
                                        vec![String::new(); count as usize]
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("get_macro_count: {e}");
                                vec![String::new(); 16]
                            }
                        };

                        let combo_entries = match dev_conn.get_combo_count() {
                            Ok(count) => {
                                log::info!("Combo count: {count}");
                                let mut entries = Vec::new();
                                for i in 0..count {
                                    match dev_conn.get_combo(i) {
                                        Ok((keys, output)) => {
                                            entries.push(ComboEntry { keys, output })
                                        }
                                        Err(e) => {
                                            log::warn!("get_combo({i}): {e}");
                                            entries.push(Default::default());
                                        }
                                    }
                                }
                                entries
                            }
                            Err(e) => {
                                log::warn!("get_combo_count: {e}");
                                vec![]
                            }
                        };

                        let combo_term = match dev_conn.get_qmk_setting_u16(2) {
                            Ok(value) => Some(value),
                            Err(e) => {
                                log::warn!("get_qmk_setting_u16(combo_term): {e}");
                                None
                            }
                        };
                        let auto_shift_options = match dev_conn.get_qmk_setting_u8(3) {
                            Ok(value) => Some(AutoShiftOptionsState::from_bits(value)),
                            Err(e) => {
                                log::warn!("get_qmk_setting_u8(auto_shift_flags): {e}");
                                None
                            }
                        };
                        let auto_shift_timeout = match dev_conn.get_qmk_setting_u16(4) {
                            Ok(value) => Some(value),
                            Err(e) => {
                                log::warn!("get_qmk_setting_u16(auto_shift_timeout): {e}");
                                None
                            }
                        };

                        let rgb_settings = load_rgb_settings(&dev_conn, &layout);

                        // Mouse keys settings (qsid 9..=17, all u16). If qsid 9 is unsupported,
                        // we assume the whole group is unavailable.
                        let mouse_keys_settings = {
                            let mut mk = MouseKeysSettingsState::default();
                            match dev_conn.get_qmk_setting_u8(9) {
                                Ok(v) => {
                                    mk.delay = v as u16;
                                    mk.supported = true;
                                    let read = |qsid: u16| -> u16 {
                                        match dev_conn.get_qmk_setting_u8(qsid) {
                                            Ok(val) => val as u16,
                                            Err(e) => {
                                                log::warn!("get_qmk_setting_u8(mouse_keys qsid {qsid}): {e}");
                                                0
                                            }
                                        }
                                    };
                                    mk.interval = read(10);
                                    mk.move_delta = read(11);
                                    mk.max_speed = read(12);
                                    mk.time_to_max = read(13);
                                    mk.wheel_delay = read(14);
                                    mk.wheel_interval = read(15);
                                    mk.wheel_max_speed = read(16);
                                    mk.wheel_time_to_max = read(17);
                                }
                                Err(e) => {
                                    log::warn!("get_qmk_setting_u16(mouse_keys delay): {e}");
                                }
                            }
                            mk
                        };

                        // Read tap dance entries
                        let tap_dance_entries = match dev_conn.get_tap_dance_count() {
                            Ok(count) => {
                                log::info!("Tap dance count: {count}");
                                let mut entries = Vec::new();
                                for i in 0..count {
                                    match dev_conn.get_tap_dance(i) {
                                        Ok((tap, hold, dtap, taphold, term)) => {
                                            entries.push(crate::keycode_picker::TapDanceEntry {
                                                on_tap: tap,
                                                on_hold: hold,
                                                on_double_tap: dtap,
                                                on_tap_hold: taphold,
                                                tapping_term: term,
                                            });
                                        }
                                        Err(e) => {
                                            log::warn!("get_tap_dance({i}): {e}");
                                            entries.push(Default::default());
                                        }
                                    }
                                }
                                entries
                            }
                            Err(e) => {
                                log::warn!("get_tap_dance_count: {e}");
                                vec![]
                            }
                        };
                        let key_override_entries = match dev_conn.get_key_override_count() {
                            Ok(count) => {
                                log::info!("Key Override count: {count}");
                                let mut entries = Vec::new();
                                for i in 0..count {
                                    match dev_conn.get_key_override(i) {
                                        Ok((
                                            trigger,
                                            replacement,
                                            layers,
                                            trigger_mods,
                                            negative_mod_mask,
                                            suppressed_mods,
                                            options,
                                        )) => {
                                            entries.push(KeyOverrideEntry {
                                                trigger,
                                                replacement,
                                                layers,
                                                trigger_mods,
                                                negative_mod_mask,
                                                suppressed_mods,
                                                options: KeyOverrideOptionsState::from_bits(
                                                    options,
                                                ),
                                            });
                                        }
                                        Err(e) => {
                                            log::warn!("get_key_override({i}): {e}");
                                            entries.push(Default::default());
                                        }
                                    }
                                }
                                entries
                            }
                            Err(e) => {
                                log::warn!("get_key_override_count: {e}");
                                vec![]
                            }
                        };

                        let alt_repeat_entries = match dev_conn.get_alt_repeat_key_count() {
                            Ok(count) => {
                                log::info!("Alt Repeat count: {count}");
                                let mut entries = Vec::new();
                                for i in 0..count {
                                    match dev_conn.get_alt_repeat_key(i) {
                                        Ok((keycode, alt_keycode, allowed_mods, options)) => {
                                            entries.push(AltRepeatKeyEntry {
                                                keycode,
                                                alt_keycode,
                                                allowed_mods,
                                                options: AltRepeatKeyOptionsState::from_bits(options),
                                            });
                                        }
                                        Err(e) => {
                                            log::warn!("get_alt_repeat_key({i}): {e}");
                                            entries.push(Default::default());
                                        }
                                    }
                                }
                                entries
                            }
                            Err(e) => {
                                log::warn!("get_alt_repeat_key_count: {e}");
                                vec![]
                            }
                        };

                        Ok(ConnectResult {
                            device_name: dev.name.clone(),
                            macro_texts,
                            tap_dance_entries,
                            combo_entries,
                            combo_term,
                            auto_shift_options: auto_shift_options.unwrap_or_default(),
                            auto_shift_timeout,
                            mouse_keys_settings,
                            rgb_settings,
                            key_override_entries,
                            alt_repeat_entries,
                            layout,
                            layer_count,
                            zmk_conn: None,
                            zmk_lock_state: 1,
                        })
                    }

                    FirmwareProtocol::Zmk => {
                        use crate::zmk::ZmkConnection;

                        log::info!("Opening ZMK serial device: {}", dev.path);
                        let mut conn = ZmkConnection::open(&dev.path)
                            .map_err(|e| format!("ZMK open failed: {e}"))?;

                        // Check lock state (don't wait — just get it)
                        let lock_state = conn
                            .get_lock_state()
                            .unwrap_or(crate::zmk_proto::core::LockState::Unlocked as i32);

                        // If locked, return early with the lock state so UI can show unlock modal
                        if lock_state == crate::zmk_proto::core::LockState::Locked as i32 {
                            log::warn!("ZMK keyboard is locked — returning for unlock UI");
                            // Return minimal result with lock state set
                            let layout = KeyboardLayout {
                                name: dev.name.clone(),
                                rows: 0,
                                cols: 0,
                                keys: vec![],
                                encoders: vec![],
                                layers: vec![],
                                encoder_layers: vec![],
                                layer_names: vec![],
                                custom_keycodes: vec![],
                                supports_rgb: false,
                                lighting_mode: None,
                                firmware: FirmwareProtocol::Zmk,
                                zmk_bindings: vec![],
                                zmk_behaviors: vec![],
                                zmk_layer_ids: vec![],
                                zmk_layer_names: vec![],
                            };
                            return Ok(ConnectResult {
                                device_name: dev.name.clone(),
                                macro_texts: vec![],
                                tap_dance_entries: vec![],
                                combo_entries: vec![],
                                combo_term: None,
                                auto_shift_options: AutoShiftOptionsState::default(),
                                auto_shift_timeout: None,
                                mouse_keys_settings: MouseKeysSettingsState::default(),
                                rgb_settings: RgbSettingsState::default(),
                                key_override_entries: vec![],
                                alt_repeat_entries: vec![],
                                layout,
                                layer_count: 0,
                                zmk_conn: Some(conn),
                                zmk_lock_state: lock_state,
                            });
                        }

                        log::info!("Fetching ZMK behaviors…");
                        conn.fetch_all_behaviors()
                            .map_err(|e| format!("ZMK behaviors failed: {e}"))?;

                        log::info!("Fetching ZMK keymap…");
                        let keymap = conn
                            .get_keymap()
                            .map_err(|e| format!("ZMK keymap failed: {e}"))?;

                        log::info!("Fetching ZMK physical layouts…");
                        let phys = conn
                            .get_physical_layouts()
                            .map_err(|e| format!("ZMK layouts failed: {e}"))?;

                        let layer_count = keymap.layers.len();
                        log::info!("ZMK: {} layers", layer_count);

                        // Build layout from device physical layout
                        let mut layout = crate::layouts::build_layout_from_zmk(&phys, &keymap);
                        layout.firmware = FirmwareProtocol::Zmk;
                        layout.zmk_behaviors = conn.behaviors.clone();

                        // Extract bindings
                        layout.zmk_layer_ids = keymap.layers.iter().map(|l| l.id).collect();
                        layout.zmk_layer_names =
                            keymap.layers.iter().map(|l| l.name.clone()).collect();

                        let num_keys = layout.keys.len();
                        layout.zmk_bindings = keymap
                            .layers
                            .iter()
                            .map(|layer| {
                                let mut bindings = vec![crate::zmk::ZmkBinding::none(); num_keys];
                                for (i, b) in layer.bindings.iter().enumerate() {
                                    if i < num_keys {
                                        bindings[i] = crate::zmk::ZmkBinding::from_proto(b);
                                    }
                                }
                                bindings
                            })
                            .collect();

                        Ok(ConnectResult {
                            device_name: dev.name.clone(),
                            macro_texts: vec![],
                            tap_dance_entries: vec![],
                            combo_entries: vec![],
                            combo_term: None,
                            auto_shift_options: AutoShiftOptionsState::default(),
                            auto_shift_timeout: None,
                            mouse_keys_settings: MouseKeysSettingsState::default(),
                            rgb_settings: RgbSettingsState::default(),
                            key_override_entries: vec![],
                            alt_repeat_entries: vec![],
                            layout,
                            layer_count,
                            zmk_conn: Some(conn),
                            zmk_lock_state: lock_state,
                        })
                    }
                }
            })();

            let _ = tx.send(result);
        });
    }

    /// Poll background thread for connect result.
    #[cfg(not(target_arch = "wasm32"))]
    fn poll_connect(&mut self, ctx: &egui::Context) {
        let result = match &self.connect_state {
            ConnectState::Loading(rx) => match rx.try_recv() {
                Ok(r) => Some(r),
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint(); // keep polling
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status_msg = "Connect thread died".into();
                    self.connect_state = ConnectState::Idle;
                    return;
                }
            },
            ConnectState::Idle => return,
        };

        self.connect_state = ConnectState::Idle;

        match result.unwrap() {
            Ok(r) => {
                self.layer_count = r.layer_count;
                self.firmware = r.layout.firmware;
                if r.layout.firmware == FirmwareProtocol::Zmk {
                    self.zmk_base_layer_count = r.layer_count;
                }
                self.current_device_name = r.device_name.clone();
                self.zmk_lock_state = r.zmk_lock_state;
                self.zmk_has_unsaved = false;
                self.keycode_picker.tap_dance_entries = r.tap_dance_entries.clone();
                self.combo_entries = r.combo_entries.clone();
                self.key_override_entries = r.key_override_entries.clone();
                self.alt_repeat_entries = r.alt_repeat_entries.clone();
                self.selected_alt_repeat = 0;
                self.alt_repeat_visible_count = if self.alt_repeat_entries.is_empty() { 1 } else { 1.min(self.alt_repeat_entries.len()) };
                self.key_override_names = load_key_override_names(&self.current_device_name);
                self.key_override_names
                    .resize(self.key_override_entries.len(), String::new());
                self.key_override_visible_count = 1;
                self.key_override_undo_stack.clear();
                self.selected_key_override = 0;
                self.combo_names = load_combo_names(&self.current_device_name);
                self.combo_names
                    .resize(self.combo_entries.len(), String::new());
                self.combo_term = r.combo_term.or(Some(50));
                self.auto_shift_options = r.auto_shift_options;
                self.auto_shift_timeout = r.auto_shift_timeout;
                self.mouse_keys_settings = r.mouse_keys_settings;
                self.rgb_settings = r.rgb_settings;
                let highest_used_combo = self
                    .combo_entries
                    .iter()
                    .enumerate()
                    .filter(|(i, combo)| {
                        combo.output != 0
                            || combo.keys.iter().any(|&k| k != 0)
                            || self
                                .combo_names
                                .get(*i)
                                .map(|n| !n.trim().is_empty())
                                .unwrap_or(false)
                    })
                    .map(|(i, _)| i + 1)
                    .max()
                    .unwrap_or(1);
                self.combo_visible_count = highest_used_combo.min(self.combo_entries.len().max(1));
                self.selected_combo = self
                    .selected_combo
                    .min(self.combo_visible_count.saturating_sub(1));
                if !r.macro_texts.is_empty() {
                    self.keycode_picker.macro_count = r.macro_texts.len();
                    self.keycode_picker.macro_texts = r.macro_texts.clone();
                    self.keycode_picker.macro_names = vec![String::new(); r.macro_texts.len()];
                    // Parse macro texts into actions
                    // Parse macro texts → actions (Vial protocol v2: prefix 0x01 before actions)
                    self.keycode_picker.macro_actions = r
                        .macro_texts
                        .iter()
                        .map(|text| {
                            let bytes = text.as_bytes();
                            let mut actions = Vec::new();
                            let mut i = 0;
                            while i < bytes.len() {
                                if bytes[i] == 1 && i + 1 < bytes.len() {
                                    // SS_QMK_PREFIX
                                    match bytes[i + 1] {
                                        1 if i + 2 < bytes.len() => {
                                            // SS_TAP
                                            actions.push(crate::keycode_picker::MacroAction::Tap(
                                                bytes[i + 2],
                                            ));
                                            i += 3;
                                        }
                                        2 if i + 2 < bytes.len() => {
                                            // SS_DOWN
                                            actions.push(crate::keycode_picker::MacroAction::Down(
                                                bytes[i + 2],
                                            ));
                                            i += 3;
                                        }
                                        3 if i + 2 < bytes.len() => {
                                            // SS_UP
                                            actions.push(crate::keycode_picker::MacroAction::Up(
                                                bytes[i + 2],
                                            ));
                                            i += 3;
                                        }
                                        4 if i + 3 < bytes.len() => {
                                            // SS_DELAY
                                            let ms = (bytes[i + 2] as u16 - 1)
                                                + (bytes[i + 3] as u16 - 1) * 255;
                                            actions.push(
                                                crate::keycode_picker::MacroAction::Delay(ms),
                                            );
                                            i += 4;
                                        }
                                        _ => {
                                            i += 2;
                                        } // skip unknown
                                    }
                                } else {
                                    // Text character
                                    let start = i;
                                    while i < bytes.len() && bytes[i] != 1 {
                                        i += 1;
                                    }
                                    if let Ok(s) = std::str::from_utf8(&bytes[start..i]) {
                                        actions.push(crate::keycode_picker::MacroAction::Text(
                                            s.to_string(),
                                        ));
                                    }
                                }
                            }
                            actions
                        })
                        .collect();
                }

                // If ZMK keyboard is locked, show the unlock modal
                if r.zmk_lock_state == crate::zmk_proto::core::LockState::Locked as i32 {
                    self.status_msg = "Keyboard locked".into();
                    if let Some(conn) = r.zmk_conn {
                        self.zmk_conn = Some(conn);
                    }
                    // Start polling for unlock in background
                    self.start_zmk_lock_poll(ctx);
                    return;
                }

                self.status_msg = format!("Connected: {}", r.device_name);

                // ZMK: store connection
                if let Some(conn) = r.zmk_conn {
                    self.zmk_conn = Some(conn);
                }

                // Load per-device layer names
                let device_name = r.device_name.clone();
                if !r.layout.zmk_layer_names.is_empty() {
                    // ZMK: device-reported names
                    self.layer_names = r.layout.zmk_layer_names.clone();
                    while self.layer_names.len() < 16 {
                        let n = self.layer_names.len();
                        self.layer_names.push(n.to_string());
                    }
                } else {
                    // Vial: prefer names from descriptor/firmware, then overlay local overrides only if a real saved file exists
                    let mut layer_names = r.layout.layer_names.clone();
                    if let Some(local_layer_names) = load_saved_layer_names(&device_name) {
                        layer_names = local_layer_names;
                    }
                    if layer_names.is_empty() {
                        layer_names = load_layer_names(&device_name);
                    }
                    self.layer_names = layer_names;
                }

                let encoder_count = r.layout.encoder_count();
                self.encoder_visibility = load_encoder_visibility(&device_name, encoder_count);

                // Populate picker based on firmware
                self.keycode_picker.firmware = self.firmware;
                self.keycode_picker.supports_rgb = r.layout.supports_rgb;
                self.keycode_picker.layer_count = r.layout.layers.len().max(1);
                self.keycode_picker.tap_dance_names = load_tap_dance_names(&device_name);
                if self.firmware == FirmwareProtocol::Vial {
                    const USER_BASE: u16 = 0x7E40;
                    self.keycode_picker.custom_keycodes = r
                        .layout
                        .custom_keycodes
                        .iter()
                        .enumerate()
                        .map(|(i, custom)| {
                            (
                                custom.name.clone(),
                                custom.label.clone(),
                                custom.title.clone(),
                                USER_BASE + i as u16,
                            )
                        })
                        .collect();
                } else {
                    self.keycode_picker.zmk_behaviors = r.layout.zmk_behaviors.clone();
                    self.keycode_picker.zmk_layer_count = r.layer_count;
                }
                self.keycode_picker.layer_names = self.layer_names.clone();

                self.layout = Some(r.layout);
                self.refresh_layer_picker_content_flags();

                // Open persistent HID connection for Vial real-time writes
                if self.firmware == FirmwareProtocol::Vial {
                    if let Some(dev) = self
                        .selected_device
                        .and_then(|i| self.device_manager.devices().get(i))
                    {
                        match crate::hid::HidDevice::open(&dev.path) {
                            Ok(v) => {
                                self.hid_device = Some(v);
                            }
                            Err(e) => log::warn!("Could not open persistent HID: {e}"),
                        }
                    }
                }

                log::info!(
                    "Connected: {} ({} layers, {:?})",
                    r.device_name,
                    r.layer_count,
                    self.firmware
                );
            }
            Err(e) => {
                self.status_msg = e;
            }
        }
    }

    /// Start polling for ZMK unlock in background.
    #[cfg(not(target_arch = "wasm32"))]
    fn start_zmk_lock_poll(&mut self, ctx: &egui::Context) {
        // Take the ZMK connection for the background thread
        let conn = match self.zmk_conn.take() {
            Some(c) => c,
            None => return,
        };

        let (tx, rx) = mpsc::channel();
        self.zmk_op_rx = Some(rx);

        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            let mut conn = conn;
            // Poll lock state until unlocked
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                match conn.get_lock_state() {
                    Ok(state) => {
                        let _ = tx.send(ZmkOpResult::LockStateChanged(state));
                        ctx_clone.request_repaint();
                        if state == crate::zmk_proto::core::LockState::Unlocked as i32 {
                            break;
                        }
                    }
                    Err(e) => {
                        log::warn!("Lock poll error: {e}");
                    }
                }
            }
            // Return connection via a special channel — we just drop it here since
            // we can't easily return it. The UI will reconnect.
            drop(conn);
        });
    }

    /// Poll ZMK background operation results.
    #[cfg(not(target_arch = "wasm32"))]
    fn poll_zmk_ops(&mut self, ctx: &egui::Context) {
        let msg = match &self.zmk_op_rx {
            Some(rx) => match rx.try_recv() {
                Ok(m) => m,
                Err(_) => return,
            },
            None => return,
        };

        match msg {
            ZmkOpResult::LockStateChanged(state) => {
                self.zmk_lock_state = state;
                if state == crate::zmk_proto::core::LockState::Unlocked as i32 {
                    self.zmk_op_rx = None;
                    // Reconnect to load the keymap now that it's unlocked
                    if let Some(idx) = self.selected_device {
                        self.start_connect(idx);
                    }
                } else {
                    ctx.request_repaint();
                }
            }
            ZmkOpResult::AddLayerOk {
                layer_idx,
                layer_name,
            } => {
                log::info!("Added layer at index {layer_idx}: {layer_name}");
                self.status_msg = "Added layer".into();
                self.zmk_has_unsaved = false;
                self.zmk_op_rx = None;
                // Reload to get updated layer list
                if let Some(idx) = self.selected_device {
                    self.start_connect(idx);
                }
            }
            ZmkOpResult::AddLayerFail(e) => {
                log::error!("Add layer failed: {e}");
                if e.contains(": 2") || e.contains("NoSpace") {
                    self.zmk_no_extra_layers = true;
                    self.status_msg =
                        "Firmware doesn't support extra layers (CONFIG_ZMK_KEYMAP_LAYERS_EXTRA)"
                            .to_string();
                } else {
                    self.status_msg = format!("Add layer failed: {e}");
                }
                self.zmk_op_rx = None;
            }
            ZmkOpResult::RemoveLayerOk => {
                log::info!("Removed layer");
                self.status_msg = "Removed layer".into();
                self.zmk_has_unsaved = false;
                self.zmk_op_rx = None;
                if let Some(idx) = self.selected_device {
                    self.start_connect(idx);
                }
            }
            ZmkOpResult::RemoveLayerFail(e) => {
                log::error!("RemoveLayer error: {e}");
                self.status_msg = format!("RemoveLayer error: {e}");
                self.zmk_op_rx = None;
            }
            ZmkOpResult::SaveOk => {
                log::info!("Saved");
                self.status_msg = "✓ Saved".into();
                self.zmk_has_unsaved = false;
                self.zmk_op_rx = None;
            }
            ZmkOpResult::SaveFail(e) => {
                log::error!("Save error: {e}");
                self.status_msg = format!("Save error: {e}");
                self.zmk_op_rx = None;
            }
            ZmkOpResult::DiscardOk => {
                log::info!("Discard ok");
                self.status_msg = "Discarded".into();
                self.zmk_has_unsaved = false;
                self.zmk_op_rx = None;
                // Reload after discard
                if let Some(idx) = self.selected_device {
                    self.start_connect(idx);
                }
            }
            ZmkOpResult::DiscardFail(e) => {
                log::error!("Discard error: {e}");
                self.status_msg = format!("Discard error: {e}");
                self.zmk_op_rx = None;
            }
        }
    }

    /// Assign keycode and immediately write to device (blocking, but single HID op — fast).
    #[cfg(not(target_arch = "wasm32"))]
    fn refresh_layer_picker_content_flags(&mut self) {
        if let Some(layout) = &self.layout {
            self.keycode_picker.layer_has_content = layout
                .layers
                .iter()
                .map(|keys| keys.iter().any(|&kc| kc != 0x0000 && kc != 0x0001))
                .collect();
        }
    }

    fn reset_matrix_tester_state(&mut self) {
        self.matrix_tester_pressed.clear();
        self.matrix_tester_ever_pressed.clear();
        self.matrix_tester_last_poll = std::time::Instant::now();
        self.matrix_tester_unlock_prompted = false;
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn is_vial_locked(&self) -> bool {
        self.firmware == FirmwareProtocol::Vial
            && self.layout.is_some()
            && !self.vial_unlock_polling
            && self
                .hid_device
                .as_ref()
                .and_then(|hid| hid.get_unlock_status().ok())
                .map(|(unlocked, _)| unlocked)
                .map(|unlocked| !unlocked)
                .unwrap_or(false)
    }

    #[cfg(target_arch = "wasm32")]
    fn is_vial_locked(&self) -> bool {
        false
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn reopen_vial_hid(&mut self) {
        if let Some(dev) = self
            .selected_device
            .and_then(|i| self.device_manager.devices().get(i))
        {
            match crate::hid::HidDevice::open(&dev.path) {
                Ok(hid) => {
                    self.hid_device = Some(hid);
                }
                Err(e) => {
                    self.hid_device = None;
                    self.status_msg = format!("Reconnect failed: {e}");
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn cancel_vial_unlock(&mut self, suppress_macro_auto_unlock: bool) {
        if let Some(hid) = &self.hid_device {
            match hid.lock() {
                Ok(()) => {
                    self.status_msg = "Keyboard unlock cancelled".into();
                }
                Err(e) => {
                    self.status_msg = format!("Cancel unlock failed: {e}");
                }
            }
        }
        self.unlock_open = false;
        self.vial_unlock_polling = false;
        self.vial_unlock_counter = 0;
        self.vial_unlock_best = 50;
        self.matrix_tester_unlock_prompted = false;
        if suppress_macro_auto_unlock {
            self.macro_auto_unlock_cancelled = true;
        }
        self.reopen_vial_hid();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn poll_matrix_tester(&mut self, ctx: &egui::Context, layout: &KeyboardLayout) {
        if self.main_menu_tab != MainMenuTab::Settings || self.firmware != FirmwareProtocol::Vial {
            return;
        }
        if self.unlock_open || self.vial_unlock_polling {
            return;
        }
        let Some(hid) = &self.hid_device else {
            return;
        };

        let now = std::time::Instant::now();
        if now.duration_since(self.matrix_tester_last_poll) >= std::time::Duration::from_millis(50)
        {
            self.matrix_tester_last_poll = now;
            match hid.get_switch_matrix(layout.rows, layout.cols) {
                Ok(pressed) => {
                    if self.matrix_tester_ever_pressed.len() != pressed.len() {
                        self.matrix_tester_ever_pressed = vec![false; pressed.len()];
                    }
                    for (idx, &is_pressed) in pressed.iter().enumerate() {
                        if is_pressed {
                            if let Some(seen) = self.matrix_tester_ever_pressed.get_mut(idx) {
                                *seen = true;
                            }
                        }
                    }
                    self.matrix_tester_pressed = pressed;
                }
                Err(e) => {
                    log::warn!("Matrix poll error: {e}");
                }
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }

    fn draw_settings_screen(
        &mut self,
        ui: &mut egui::Ui,
        layout: &KeyboardLayout,
        ctx: &egui::Context,
        content_top: f32,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_matrix_tester(ctx, layout);

        if let Some(id) = ctx.memory(|m| m.focused()) {
            ctx.memory_mut(|m| m.surrender_focus(id));
        }

        let dark = ui.visuals().dark_mode;
        let content_rect = egui::Rect::from_min_max(
            egui::pos2(ui.min_rect().left() + 20.0, content_top),
            egui::pos2(ui.min_rect().right() - 20.0, ui.max_rect().bottom() - 76.0),
        );

        let top_line_y = content_rect.top() + 18.0;
        let supported = self.firmware == FirmwareProtocol::Vial;
        let hid_ready = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.hid_device.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };

        if supported
            && hid_ready
            && self.is_vial_locked()
            && !self.unlock_open
            && !self.matrix_tester_unlock_prompted
        {
            self.unlock_open = true;
            self.matrix_tester_unlock_prompted = true;
            self.status_msg = "Keyboard is locked, unlock it to use Matrix Tester".into();
        }

        let total_keys = layout.keys.len();
        let tested_count = layout
            .keys
            .iter()
            .filter(|key| {
                let idx = key.row as usize * layout.cols + key.col as usize;
                self.matrix_tester_ever_pressed
                    .get(idx)
                    .copied()
                    .unwrap_or(false)
            })
            .count();

        ui.painter().text(
            egui::pos2(content_rect.left() + 4.0, top_line_y),
            egui::Align2::LEFT_CENTER,
            "Press switches on the keyboard to verify every matrix position.",
            FontId::proportional(13.0),
            app_muted_text(dark),
        );
        ui.painter().text(
            egui::pos2(content_rect.left() + 4.0, top_line_y + 22.0),
            egui::Align2::LEFT_CENTER,
            format!("Tested: {tested_count}/{total_keys}"),
            FontId::proportional(14.0),
            if tested_count == total_keys && total_keys > 0 {
                Color32::from_rgb(72, 168, 110)
            } else {
                app_muted_text(dark)
            },
        );

        let reset_rect = egui::Rect::from_min_size(
            egui::pos2(content_rect.right() - 88.0, top_line_y - 15.0),
            Vec2::new(84.0, 30.0),
        );
        if ui
            .put(
                reset_rect,
                egui::Button::new("Reset").sense(egui::Sense::CLICK),
            )
            .clicked()
        {
            self.reset_matrix_tester_state();
        }

        let painter = ui.painter();
        let idle_fill = if dark {
            Color32::from_rgb(34, 34, 38)
        } else {
            Color32::from_rgb(252, 252, 254)
        };
        let tested_fill = if dark {
            Color32::from_rgb(42, 68, 52)
        } else {
            Color32::from_rgb(232, 245, 236)
        };

        let board_top = content_rect.top() + 52.0;
        let board_rect = egui::Rect::from_min_max(
            egui::pos2(content_rect.left(), board_top),
            egui::pos2(content_rect.right(), content_rect.bottom()),
        );

        if !supported {
            painter.text(
                board_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Matrix Tester is currently available only for Vial keyboards.",
                FontId::proportional(15.0),
                app_muted_text(dark),
            );
            return;
        }

        if !hid_ready {
            painter.text(
                board_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Connect a Vial keyboard to start live switch testing.",
                FontId::proportional(15.0),
                app_muted_text(dark),
            );
            return;
        }

        let base_unit = 54.0_f32 * 1.15;
        let padding = 4.0_f32;
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for key in &layout.keys {
            min_x = min_x.min(key.x);
            min_y = min_y.min(key.y);
            max_x = max_x.max(key.x + key.w);
            max_y = max_y.max(key.y + key.h);
        }
        for encoder in &layout.encoders {
            min_x = min_x.min(encoder.x);
            min_y = min_y.min(encoder.y);
            max_x = max_x.max(encoder.x + encoder.w);
            max_y = max_y.max(encoder.y + encoder.h);
        }
        if min_x == f32::MAX {
            return;
        }

        let span_x = max_x - min_x;
        let span_y = max_y - min_y;
        let margin = 40.0_f32;
        let avail = ui.available_size();
        let scale_x = (avail.x - margin) / (span_x * base_unit).max(1.0);
        let scale_y = (avail.y - margin) / (span_y * base_unit).max(1.0);
        let scale = scale_x.min(scale_y).min(1.0);
        let unit = base_unit * scale;
        let layout_w = span_x * unit;
        let layout_h = span_y * unit;
        let offset_x = (avail.x - layout_w) / 2.0 + ui.min_rect().left() - min_x * unit;
        let offset_y = ((content_rect.top() + content_rect.bottom()) - layout_h) / 2.0 - min_y * unit;

        for key in &layout.keys {
            let matrix_idx = key.row as usize * layout.cols + key.col as usize;
            let is_pressed = self
                .matrix_tester_pressed
                .get(matrix_idx)
                .copied()
                .unwrap_or(false);
            let was_pressed = self
                .matrix_tester_ever_pressed
                .get(matrix_idx)
                .copied()
                .unwrap_or(false);
            let rect = egui::Rect::from_min_size(
                egui::pos2(
                    offset_x + key.x * unit + padding,
                    offset_y + key.y * unit + padding,
                ),
                Vec2::new(key.w * unit - padding * 2.0, key.h * unit - padding * 2.0),
            );

            let fill = if is_pressed {
                app_accent()
            } else if was_pressed {
                tested_fill
            } else {
                idle_fill
            };
            let stroke = if is_pressed {
                Color32::from_rgb(130, 140, 255)
            } else if was_pressed {
                Color32::from_rgb(104, 152, 120)
            } else {
                app_border_color(dark)
            };
            painter.rect(
                rect,
                6.0,
                fill,
                Stroke::new(1.0, stroke),
                egui::StrokeKind::Inside,
            );
        }
    }

    fn apply_picker_results(&mut self) {
        if let Some(kc_value) = self.keycode_picker.result.take() {
            if let Some((combo_idx, field)) = self.combo_pick_target.take() {
                self.push_combo_undo();
                if let Some(combo) = self.combo_entries.get_mut(combo_idx) {
                    match field {
                        ComboPickField::Trigger(key_idx) => combo.keys[key_idx] = kc_value,
                        ComboPickField::Output => combo.output = kc_value,
                    }
                    self.combo_dirty = true;
                }
                if self.combo_reopen_after_pick {
                    self.modal_focus_pending = true;
                    self.combo_window_open = true;
                    self.combo_reopen_after_pick = false;
                }
            } else if let Some(field) = self.key_override_pick_target.take() {
                let idx = self
                    .selected_key_override
                    .min(self.key_override_entries.len().saturating_sub(1));
                self.push_key_override_undo();
                if let Some(entry) = self.key_override_entries.get_mut(idx) {
                    match field {
                        KeyOverridePickField::Trigger => entry.trigger = kc_value,
                        KeyOverridePickField::Replacement => entry.replacement = kc_value,
                    }
                    Self::normalize_key_override_entry(entry);
                }
                self.write_key_override(idx);
                if self.key_override_reopen_after_pick {
                    self.modal_focus_pending = true;
                    self.key_override_window_open = true;
                    self.key_override_reopen_after_pick = false;
                }
            } else if let Some(field) = self.alt_repeat_pick_target.take() {
                let idx = self
                    .selected_alt_repeat
                    .min(self.alt_repeat_entries.len().saturating_sub(1));
                if let Some(entry) = self.alt_repeat_entries.get_mut(idx) {
                    match field {
                        AltRepeatPickField::LastKey => entry.keycode = kc_value,
                        AltRepeatPickField::AltKey => entry.alt_keycode = kc_value,
                    }
                }
                self.write_alt_repeat_entry(idx);
                if self.alt_repeat_reopen_after_pick {
                    self.modal_focus_pending = true;
                    self.alt_repeat_window_open = true;
                    self.alt_repeat_visible_count = self
                        .alt_repeat_visible_count
                        .clamp(1, self.alt_repeat_entries.len().max(1));
                    self.alt_repeat_reopen_after_pick = false;
                }
            } else if let Some((layer, encoder_visual_idx)) = self.selected_encoder {
                #[cfg(not(target_arch = "wasm32"))]
                self.assign_encoder_keycode(layer, encoder_visual_idx, kc_value);
                #[cfg(target_arch = "wasm32")]
                if let Some(layout) = &mut self.layout {
                    if let Some(layer_codes) = layout.encoder_layers.get_mut(layer) {
                        if let Some(slot) = layer_codes.get_mut(encoder_visual_idx) {
                            *slot = kc_value;
                        }
                    }
                }
                if is_alt_repeat_keycode(kc_value) {
                    self.open_alt_repeat_window_compact();
                }
            } else if let Some((layer, ki)) = self.selected_key {
                #[cfg(not(target_arch = "wasm32"))]
                self.assign_keycode(layer, ki, kc_value);
                #[cfg(target_arch = "wasm32")]
                if let Some(layout) = &mut self.layout {
                    layout.set_keycode(layer, ki, kc_value);
                }
                if is_alt_repeat_keycode(kc_value) {
                    self.open_alt_repeat_window_compact();
                }
            }
            self.selected_key = None;
            self.selected_encoder = None;
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(binding) = self.keycode_picker.zmk_result.take() {
            if let Some((layer, ki)) = self.selected_key {
                self.assign_zmk_binding(layer, ki, binding);
            }
            self.selected_key = None;
            self.selected_encoder = None;
        }
    }

    fn assign_encoder_keycode(&mut self, layer: usize, encoder_visual_idx: usize, kc_value: u16) {
        let encoder = match self
            .layout
            .as_ref()
            .and_then(|l| l.encoders.get(encoder_visual_idx))
        {
            Some(e) => e.clone(),
            None => return,
        };
        let old_kc = self
            .layout
            .as_ref()
            .map(|l| l.get_encoder_keycode(layer, encoder_visual_idx))
            .unwrap_or(0);
        self.undo_stack.push(UndoAction::Encoder {
            layer,
            encoder_visual_idx,
            old_kc,
        });

        if let Some(layout) = &mut self.layout {
            if let Some(layer_codes) = layout.encoder_layers.get_mut(layer) {
                if let Some(slot) = layer_codes.get_mut(encoder_visual_idx) {
                    *slot = kc_value;
                }
            }
        }

        let result = if let Some(conn) = &self.hid_device {
            conn.set_encoder(
                layer as u8,
                encoder.encoder_idx,
                encoder.direction,
                kc_value,
            )
        } else if let Some(dev) = self
            .selected_device
            .and_then(|i| self.device_manager.devices().get(i))
        {
            match crate::hid::HidDevice::open(&dev.path) {
                Ok(conn) => conn.set_encoder(
                    layer as u8,
                    encoder.encoder_idx,
                    encoder.direction,
                    kc_value,
                ),
                Err(e) => Err(anyhow::anyhow!("{e}")),
            }
        } else {
            return;
        };

        match result {
            Ok(()) => {
                self.status_msg = format!(
                    "Assigned encoder {} direction {} on layer {}",
                    encoder.encoder_idx,
                    encoder.direction,
                    layer + 1
                );
            }
            Err(e) => {
                self.status_msg = format!("Set encoder failed: {e}");
            }
        }
    }

    fn open_picker_for_target(
        &mut self,
        key_target: Option<usize>,
        encoder_target: Option<usize>,
        is_zmk: bool,
    ) {
        self.selected_key = key_target.map(|ki| (self.selected_layer, ki));
        self.selected_encoder = encoder_target.map(|ei| (self.selected_layer, ei));
        self.keycode_picker.open = true;
        self.keycode_picker.result = None;
        self.keycode_picker.search_query.clear();
        self.keycode_picker.layer_names = self.layer_names.clone();
        self.keycode_picker.firmware = self.firmware;
        self.keycode_picker.vial_quantum_pending_mod = None;
        self.keycode_picker.vial_quantum_pending_mt = None;
        self.keycode_picker.vial_layer_pending = None;
        self.keycode_picker.tap_dance_editor_open = None;
        self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::Basic;
        if is_zmk {
            self.keycode_picker.zmk_behaviors = self
                .layout
                .as_ref()
                .map(|l| l.zmk_behaviors.clone())
                .unwrap_or_default();
            self.keycode_picker.zmk_layer_count = self.layer_count;
        }
    }

    fn handle_secondary_target(
        &mut self,
        ctrl_held: bool,
        is_zmk: bool,
        kc: u16,
        key_target: Option<usize>,
        encoder_target: Option<usize>,
    ) {
        if is_zmk {
            return;
        }
        if !ctrl_held {
            let target_layer = if kc >= 0x5200 && kc < 0x5300 {
                Some((kc & 0x1F) as usize)
            } else if kc & 0xF000 == 0x4000 {
                Some(((kc >> 8) & 0xF) as usize)
            } else {
                None
            };
            if let Some(target_layer) = target_layer {
                if target_layer != self.selected_layer {
                    self.jump_back_stack.push(self.selected_layer);
                    self.selected_layer = target_layer;
                    self.hover_layer = None;
                }
                self.secondary_click_handled = true;
                return;
            }
        }
        if ctrl_held {
            if let Some(swapped) = toggle_handed_modifier(kc) {
                if let Some(visual_idx) = encoder_target {
                    self.assign_encoder_keycode(self.selected_layer, visual_idx, swapped);
                } else if let Some(ki) = key_target {
                    self.pending_handed_swap = Some((self.selected_layer, ki, swapped));
                }
                self.secondary_click_handled = true;
            } else {
                let layer_base: Option<u16> = if kc >= 0x5200 && kc < 0x5300 {
                    Some(kc & 0xFFE0)
                } else if kc & 0xF000 == 0x4000 {
                    Some(0x4000)
                } else {
                    None
                };
                if let Some(base) = layer_base {
                    self.open_picker_for_target(key_target, encoder_target, is_zmk);
                    self.keycode_picker.vial_layer_pending = Some(base);
                    self.secondary_click_handled = true;
                }
            }
            if self.secondary_click_handled {
                return;
            }
        }
        if kc >= 0x7700 && kc <= 0x77FF {
            let macro_n = (kc - 0x7700) as u8;
            self.open_picker_for_target(key_target, encoder_target, is_zmk);
            self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::Macro;
            self.keycode_picker.macro_inline_selected = Some(macro_n);
            self.secondary_click_handled = true;
            return;
        }
        if kc >= 0x5700 && kc <= 0x57FF {
            let td_n = (kc - 0x5700) as u8;
            self.open_picker_for_target(key_target, encoder_target, is_zmk);
            self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::TapDance;
            self.keycode_picker.tap_dance_editor_open = Some(td_n);
            self.secondary_click_handled = true;
            return;
        }
        if is_mouse_keycode(kc) {
            self.mouse_keys_window_open = true;
            self.secondary_click_handled = true;
            return;
        }
        if is_alt_repeat_keycode(kc) {
            self.open_alt_repeat_window_compact();
            self.secondary_click_handled = true;
            return;
        }
        let is_layer_key = (kc >= 0x5200 && kc < 0x5300) || (kc & 0xF000 == 0x4000);
        let pending_base: Option<u16> = if is_layer_key {
            None
        } else if kc >= 0x2000 && kc < 0x4000 {
            Some(kc & 0xFF00)
        } else if kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0 {
            Some(kc & 0xFF00)
        } else {
            None
        };
        if let Some(base) = pending_base {
            self.open_picker_for_target(key_target, encoder_target, is_zmk);
            if kc >= 0x2000 {
                self.keycode_picker.vial_quantum_pending_mt = Some(base);
                self.keycode_picker.vial_quantum_pending_mod = None;
            } else {
                self.keycode_picker.vial_quantum_pending_mod = Some(base);
                self.keycode_picker.vial_quantum_pending_mt = None;
            }
            self.secondary_click_handled = true;
        }
    }

    fn assign_keycode(&mut self, layer: usize, ki: usize, kc_value: u16) {
        // Save old value for undo
        let old_kc = self
            .layout
            .as_ref()
            .map(|l| l.get_keycode(layer, ki))
            .unwrap_or(0);
        self.undo_stack.push(UndoAction::Key {
            layer,
            key_idx: ki,
            old_kc,
            old_binding: ZmkBinding::none(),
        });
        // Update in-memory layout
        if let Some(layout) = &mut self.layout {
            layout.set_keycode(layer, ki, kc_value);
        }
        self.refresh_layer_picker_content_flags();

        let key = match self.layout.as_ref().and_then(|l| l.keys.get(ki)) {
            Some(k) => k.clone(),
            None => return,
        };

        // Use persistent connection if available, otherwise open fresh
        let result = if let Some(conn) = &self.hid_device {
            conn.set_keycode(layer as u8, key.row, key.col, kc_value)
        } else if let Some(dev) = self
            .selected_device
            .and_then(|i| self.device_manager.devices().get(i))
        {
            match crate::hid::HidDevice::open(&dev.path) {
                Ok(conn) => conn.set_keycode(layer as u8, key.row, key.col, kc_value),
                Err(e) => Err(anyhow::anyhow!("{e}")),
            }
        } else {
            return;
        };

        match result {
            Ok(()) => self.status_msg = "✓ Saved".into(),
            Err(e) => {
                self.status_msg = format!("Write error: {e}");
                // Connection lost — reopen
                self.hid_device = None;
            }
        }
    }

    /// Reload all keycodes from device in background.
    #[cfg(not(target_arch = "wasm32"))]
    fn load_from_device(&mut self) {
        if let Some(idx) = self.selected_device {
            self.start_connect(idx);
        }
    }

    /// ZMK: assign binding and write to device.
    #[cfg(not(target_arch = "wasm32"))]
    fn assign_zmk_binding(&mut self, layer: usize, ki: usize, binding: ZmkBinding) {
        let layer_id = self
            .layout
            .as_ref()
            .and_then(|l| l.zmk_layer_ids.get(layer).copied())
            .unwrap_or(layer as u32);

        // Save old binding for undo
        let old_binding = self
            .layout
            .as_ref()
            .map(|l| l.get_zmk_binding(layer, ki))
            .unwrap_or_else(ZmkBinding::none);
        self.undo_stack.push(UndoAction::Key {
            layer,
            key_idx: ki,
            old_kc: 0,
            old_binding,
        });

        if let Some(layout) = &mut self.layout {
            layout.set_zmk_binding(layer, ki, binding.clone());
        }

        let conn = match self.zmk_conn.take() {
            Some(c) => c,
            None => {
                self.status_msg = "ZMK not connected".into();
                return;
            }
        };
        let mut conn = conn;
        match conn.set_layer_binding(layer_id, ki as i32, &binding) {
            Ok(()) => {
                self.status_msg = "✓ Saved".into();
                self.zmk_has_unsaved = true;
            }
            Err(e) => {
                self.status_msg = format!("ZMK write error: {e}");
            }
        }
        self.zmk_conn = Some(conn);
    }

    /// ZMK: save changes to flash in background.
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(not(target_arch = "wasm32"))]
    fn undo(&mut self) {
        let Some(action) = self.undo_stack.pop() else {
            return;
        };
        match action {
            UndoAction::Key {
                layer,
                key_idx,
                old_kc,
                old_binding,
            } => {
                if self.firmware == FirmwareProtocol::Zmk {
                    self.assign_zmk_binding(layer, key_idx, old_binding);
                    self.undo_stack.pop();
                } else {
                    self.assign_keycode(layer, key_idx, old_kc);
                    self.undo_stack.pop();
                }
            }
            UndoAction::Encoder {
                layer,
                encoder_visual_idx,
                old_kc,
            } => {
                self.assign_encoder_keycode(layer, encoder_visual_idx, old_kc);
                self.undo_stack.pop();
            }
        }
    }

    fn zmk_save(&mut self) {
        if self.zmk_op_rx.is_some() {
            return;
        } // operation in progress
        let conn = match self.zmk_conn.take() {
            Some(c) => c,
            None => {
                self.status_msg = "No ZMK connection".into();
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.zmk_op_rx = Some(rx);

        std::thread::spawn(move || {
            let mut conn = conn;
            match conn.save_changes() {
                Ok(()) => {
                    let _ = tx.send(ZmkOpResult::SaveOk);
                }
                Err(e) => {
                    let _ = tx.send(ZmkOpResult::SaveFail(e.to_string()));
                }
            }
            // Put connection back — we drop it here since we can't easily return it
            // The caller will need to reconnect if needed
            drop(conn);
        });
        self.status_msg = "Saving…".into();
    }

    /// ZMK: discard unsaved changes in background.
    #[cfg(not(target_arch = "wasm32"))]
    fn zmk_discard(&mut self) {
        if self.zmk_op_rx.is_some() {
            return;
        }
        // Discard = just reload from device
        self.zmk_has_unsaved = false;
        self.status_msg = "Discarded".into();
        if let Some(idx) = self.selected_device {
            self.start_connect(idx);
        }
    }

    /// ZMK: add layer in background.
    #[cfg(not(target_arch = "wasm32"))]
    fn zmk_add_layer(&mut self) {
        if self.zmk_op_rx.is_some() {
            return;
        }
        let conn = match self.zmk_conn.take() {
            Some(c) => c,
            None => {
                self.status_msg = "No ZMK connection".into();
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.zmk_op_rx = Some(rx);

        std::thread::spawn(move || {
            let mut conn = conn;
            let res = conn.add_layer().and_then(|(idx, name)| {
                log::info!("save_changes after add_layer: layer={idx}");
                conn.save_changes()?;
                Ok((idx, name))
            });
            match res {
                Ok((idx, name)) => {
                    let _ = tx.send(ZmkOpResult::AddLayerOk {
                        layer_idx: idx,
                        layer_name: name,
                    });
                }
                Err(e) => {
                    let _ = tx.send(ZmkOpResult::AddLayerFail(e.to_string()));
                }
            }
            drop(conn);
        });
        self.status_msg = "Adding layer…".into();
    }

    /// ZMK: remove last layer in background.
    #[cfg(not(target_arch = "wasm32"))]
    fn zmk_remove_layer(&mut self) {
        if self.zmk_op_rx.is_some() {
            return;
        }
        if self.layer_count <= 1 {
            self.status_msg = "Cannot remove last layer".into();
            return;
        }
        let last_idx = (self.layer_count - 1) as u32;
        let conn = match self.zmk_conn.take() {
            Some(c) => c,
            None => {
                self.status_msg = "No ZMK connection".into();
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.zmk_op_rx = Some(rx);

        std::thread::spawn(move || {
            let mut conn = conn;
            let res = conn.remove_layer(last_idx).and_then(|()| {
                log::info!("save_changes after remove_layer: layer={last_idx}");
                conn.save_changes()
            });
            match res {
                Ok(()) => {
                    let _ = tx.send(ZmkOpResult::RemoveLayerOk);
                }
                Err(e) => {
                    let _ = tx.send(ZmkOpResult::RemoveLayerFail(e.to_string()));
                }
            }
            drop(conn);
        });
        self.status_msg = "Removing layer…".into();
    }
}

impl eframe::App for EntropyApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        std::process::exit(0);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-scan for new devices every ~2 seconds (120 frames at 60fps)
        self.secondary_click_handled = false;
        if let Some((layer, ki, kc)) = self.pending_handed_swap {
            if !ctx.input(|i| i.modifiers.ctrl) {
                #[cfg(not(target_arch = "wasm32"))]
                self.assign_keycode(layer, ki, kc);
                #[cfg(target_arch = "wasm32")]
                if let Some(layout) = &mut self.layout {
                    layout.set_keycode(layer, ki, kc);
                }
                self.pending_handed_swap = None;
            }
        }
        self.scan_frame += 1;
        if self.scan_frame >= 120 && !self.vial_unlock_polling {
            self.scan_frame = 0;
            let prev_count = self.device_manager.devices().len();
            self.device_manager.scan();
            // Auto-connect if a new device appeared and nothing is connected
            if self.device_manager.devices().len() > prev_count
                && self.layout.is_none()
                && !matches!(self.connect_state, ConnectState::Loading(_))
            {
                self.start_connect(0);
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        self.poll_single_instance_signal(ctx);

        // Apply theme
        if self.dark_mode {
            let mut v = egui::Visuals::dark();
            v.panel_fill = app_panel_fill(true);
            v.window_fill = app_window_fill(true);
            v.faint_bg_color = app_window_fill(true);
            v.extreme_bg_color = Color32::from_rgb(24, 24, 24);
            v.widgets.noninteractive.bg_fill = app_window_fill(true);
            v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, app_border_color(true));
            v.widgets.inactive.bg_fill = app_surface_fill(true);
            v.widgets.inactive.weak_bg_fill = app_surface_fill(true);
            v.widgets.inactive.bg_stroke = Stroke::new(1.0, app_border_color(true));
            v.widgets.hovered.bg_fill = app_hover_fill(true);
            v.widgets.hovered.weak_bg_fill = app_hover_fill(true);
            v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(96, 96, 104));
            v.widgets.active.bg_fill = app_accent();
            v.widgets.active.weak_bg_fill = app_accent();
            v.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(130, 140, 255));
            v.selection.bg_fill = Color32::from_rgba_unmultiplied(91, 104, 223, 120);
            v.interact_cursor = Some(egui::CursorIcon::PointingHand);
            ctx.set_visuals(v);
        } else {
            let mut v = egui::Visuals::light();
            v.panel_fill = app_panel_fill(false);
            v.window_fill = app_window_fill(false);
            v.faint_bg_color = app_panel_fill(false);
            v.extreme_bg_color = Color32::from_rgb(235, 235, 235);
            v.widgets.noninteractive.bg_fill = app_panel_fill(false);
            v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, app_border_color(false));
            v.widgets.inactive.bg_fill = app_surface_fill(false);
            v.widgets.inactive.weak_bg_fill = app_surface_fill(false);
            v.widgets.inactive.bg_stroke = Stroke::new(1.0, app_border_color(false));
            v.widgets.hovered.bg_fill = app_hover_fill(false);
            v.widgets.hovered.weak_bg_fill = app_hover_fill(false);
            v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(206, 206, 216));
            v.widgets.active.bg_fill = app_accent();
            v.widgets.active.weak_bg_fill = app_accent();
            v.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(126, 138, 236));
            v.selection.bg_fill = Color32::from_rgba_unmultiplied(91, 104, 223, 80);
            v.interact_cursor = Some(egui::CursorIcon::PointingHand);
            ctx.set_visuals(v);
        }

        // Poll background connect thread
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_connect(ctx);

        // Poll ZMK background operations
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_zmk_ops(ctx);

        self.apply_picker_results();

        // Deselect key when picker is closed without choosing
        if !self.keycode_picker.open
            && (self.selected_key.is_some() || self.selected_encoder.is_some())
            && self.keycode_picker.result.is_none()
            && self.keycode_picker.zmk_result.is_none()
        {
            self.selected_key = None;
            self.selected_encoder = None;
        }

        if !self.keycode_picker.open || self.keycode_picker.selected_tab != KeycodeTab::Macro {
            self.macro_auto_unlock_cancelled = false;
        }

        if self.firmware == FirmwareProtocol::Vial
            && self.keycode_picker.open
            && self.keycode_picker.selected_tab == KeycodeTab::Macro
            && !self.unlock_open
            && !self.vial_unlock_polling
            && !self.macro_auto_unlock_cancelled
            && self.is_vial_locked()
        {
            self.unlock_open = true;
            self.status_msg = "Keyboard is locked, unlock it to edit macros".into();
        }

        // Arrow keys Left/Right switch layers (when picker is closed)
        if !self.keycode_picker.open {
            let layer_count = self.layer_count;
            ctx.input(|i| {
                if i.key_pressed(egui::Key::ArrowLeft) && self.selected_layer > 0 {
                    self.selected_layer -= 1;
                    self.jump_back_stack.clear();
                }
                if i.key_pressed(egui::Key::ArrowRight) && self.selected_layer + 1 < layer_count {
                    self.selected_layer += 1;
                    self.jump_back_stack.clear();
                }
            });
        }

        // Check if loading
        #[cfg(not(target_arch = "wasm32"))]
        let is_loading = matches!(self.connect_state, ConnectState::Loading(_));
        #[cfg(target_arch = "wasm32")]
        let is_loading = false;

        // ZMK unlock modal — show before everything else if locked
        #[cfg(not(target_arch = "wasm32"))]
        {
            let is_zmk_locked = self.firmware == FirmwareProtocol::Zmk
                && self.zmk_lock_state == crate::zmk_proto::core::LockState::Locked as i32
                && !is_loading;

            if is_zmk_locked {
                self.show_zmk_unlock_modal(ctx);
                return;
            }
        }

        // Top bar
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⌨ Entropy").size(18.0).strong());
                ui.separator();

                #[cfg(not(target_arch = "wasm32"))]
                if !self.undo_stack.is_empty() {
                    if ui
                        .button("↩ Undo")
                        .on_hover_text("Undo last change")
                        .clicked()
                    {
                        self.undo();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // Vial: Unlock button (don't poll status during unlock process)
                        if self.firmware == FirmwareProtocol::Vial
                            && self.layout.is_some()
                            && !self.vial_unlock_polling
                            && !self.unlock_open
                        {
                            let is_unlocked = self
                                .hid_device
                                .as_ref()
                                .and_then(|hid| hid.get_unlock_status().ok())
                                .map(|(unlocked, _keys)| unlocked)
                                .unwrap_or(false);
                            if !is_unlocked {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("🔒 Unlock")
                                                .color(Color32::from_rgb(220, 120, 60)),
                                        )
                                        .fill(Color32::TRANSPARENT),
                                    )
                                    .on_hover_text("Keyboard is locked — click to unlock")
                                    .clicked()
                                {
                                    self.unlock_open = true;
                                }
                            } else {
                                if ui
                                    .add(
                                        egui::Button::new(RichText::new("🔓 Lock"))
                                            .fill(Color32::TRANSPARENT),
                                    )
                                    .on_hover_text("Lock keyboard — prevents accidental changes")
                                    .clicked()
                                {
                                    if let Some(hid) = &self.hid_device {
                                        match hid.lock() {
                                            Ok(()) => self.status_msg = "Keyboard locked".into(),
                                            Err(e) => self.status_msg = format!("Lock failed: {e}"),
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !self.status_msg.is_empty() {
                        let color = if self.status_msg.starts_with("✓") {
                            Color32::from_rgb(100, 200, 100)
                        } else if self.status_msg.contains("error")
                            || self.status_msg.contains("failed")
                        {
                            Color32::from_rgb(220, 80, 80)
                        } else {
                            Color32::from_rgb(180, 180, 100)
                        };
                        ui.label(RichText::new(&self.status_msg).size(11.0).color(color));
                    }

                    if is_loading {
                        ui.spinner();
                    }
                });
            });
        });

        // Main canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.selected_device.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("Connect a keyboard and press Refresh")
                            .size(16.0)
                            .color(Color32::GRAY),
                    );
                });
                return;
            }

            if is_loading {
                ui.centered_and_justified(|ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new("Loading keyboard…")
                                .size(16.0)
                                .color(Color32::GRAY),
                        );
                    });
                });
                return;
            }

            if self.layout.is_some() {
                let layout = self.layout.clone().unwrap();
                self.draw_layout(ui, &layout, ctx);
            } else {
                self.draw_placeholder(ui);
            }
        });

        // Bottom bar — theme toggle
        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let theme_label = if self.dark_mode {
                    "☀ Light"
                } else {
                    "🌙 Dark"
                };
                if ui.small_button(theme_label).clicked() {
                    self.dark_mode = !self.dark_mode;
                }
            });
        });

        // Keycode picker modal
        // Vial unlock modal
        if self.unlock_open && self.firmware == FirmwareProtocol::Vial {
            // Start unlock if not yet polling
            if !self.vial_unlock_polling {
                if let Some(hid) = &self.hid_device {
                    // Get unlock keys from get_unlock_status
                    match hid.get_unlock_status() {
                        Ok((_, keys)) => {
                            self.vial_unlock_keys = keys;
                        }
                        Err(_) => {}
                    }
                    // Start the unlock process
                    match hid.unlock_start() {
                        Ok(()) => {
                            self.vial_unlock_polling = true;
                            self.vial_unlock_best = 50;
                        }
                        Err(e) => {
                            self.status_msg = format!("Unlock start failed: {e}");
                            self.unlock_open = false;
                        }
                    }
                }
            }
            // Poll unlock
            if self.vial_unlock_polling {
                if let Some(hid) = &self.hid_device {
                    match hid.unlock_poll() {
                        Ok((unlocked, _in_progress, counter)) => {
                            self.vial_unlock_counter = counter;
                            if counter < self.vial_unlock_best {
                                self.vial_unlock_best = counter;
                            }
                            if unlocked {
                                self.status_msg = "Keyboard unlocked!".into();
                                self.unlock_open = false;
                                self.vial_unlock_polling = false;
                                self.macro_auto_unlock_cancelled = false;
                            }
                        }
                        Err(_) => {}
                    }
                }
                // Poll at ~120ms intervals (firmware timer threshold is 100ms)
                ctx.request_repaint_after(std::time::Duration::from_millis(120));
            }
            // Fullscreen overlay with layout and highlighted keys
            let unlock_keys = self.vial_unlock_keys.clone();
            let counter = self.vial_unlock_best;
            let total = self.vial_unlock_total;

            egui::Area::new(egui::Id::new("unlock_overlay"))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let screen = ui.ctx().screen_rect();
                    // Dim background
                    ui.painter().rect_filled(
                        screen,
                        0.0,
                        Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                    );

                    let center_x = screen.center().x;
                    let top_y = screen.min.y + 40.0;

                    // Title
                    ui.painter().text(
                        egui::pos2(center_x, top_y),
                        egui::Align2::CENTER_CENTER,
                        "🔓 Unlock Keyboard",
                        FontId::proportional(24.0),
                        Color32::WHITE,
                    );

                    ui.painter().text(
                        egui::pos2(center_x, top_y + 30.0),
                        egui::Align2::CENTER_CENTER,
                        "Press and hold the highlighted keys one by one",
                        FontId::proportional(14.0),
                        Color32::from_gray(180),
                    );

                    // Progress bar
                    let progress = if total > 0 {
                        1.0 - (counter as f32 / total as f32)
                    } else {
                        0.0
                    };
                    let bar_w = 300.0f32;
                    let bar_h = 12.0f32;
                    let bar_y = top_y + 55.0;
                    let bar_rect = egui::Rect::from_min_size(
                        egui::pos2(center_x - bar_w / 2.0, bar_y),
                        egui::Vec2::new(bar_w, bar_h),
                    );
                    ui.painter().rect(
                        bar_rect,
                        4.0,
                        Color32::from_gray(40),
                        egui::Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );
                    let fill_rect = egui::Rect::from_min_size(
                        bar_rect.min,
                        egui::Vec2::new(bar_w * progress, bar_h),
                    );
                    ui.painter().rect(
                        fill_rect,
                        4.0,
                        Color32::from_rgb(91, 104, 223),
                        egui::Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );

                    // Draw layout keys with highlighted unlock keys
                    if let Some(layout) = &self.layout {
                        let base_unit = 54.0f32 * 0.85;
                        let padding = 3.0f32;
                        let mut min_x = f32::MAX;
                        let mut min_y = f32::MAX;
                        let mut max_x = f32::MIN;
                        let mut max_y = f32::MIN;
                        for key in &layout.keys {
                            min_x = min_x.min(key.x);
                            min_y = min_y.min(key.y);
                            max_x = max_x.max(key.x + key.w);
                            max_y = max_y.max(key.y + key.h);
                        }
                        let span_x = max_x - min_x;
                        let span_y = max_y - min_y;
                        let avail_w = screen.width() - 80.0;
                        let avail_h = screen.height() - 160.0;
                        let scale = (avail_w / (span_x * base_unit))
                            .min(avail_h / (span_y * base_unit))
                            .min(1.0);
                        let unit = base_unit * scale;
                        let layout_w = span_x * unit;
                        let layout_h = span_y * unit;
                        let off_x = center_x - layout_w / 2.0 - min_x * unit;
                        let off_y = bar_y + 30.0 + (avail_h - layout_h) / 2.0 - min_y * unit;

                        for (ki, key) in layout.keys.iter().enumerate() {
                            let is_unlock = unlock_keys
                                .iter()
                                .any(|(r, c)| key.row == *r && key.col == *c);
                            let rect = egui::Rect::from_min_size(
                                egui::pos2(
                                    off_x + key.x * unit + padding,
                                    off_y + key.y * unit + padding,
                                ),
                                egui::Vec2::new(
                                    key.w * unit - padding * 2.0,
                                    key.h * unit - padding * 2.0,
                                ),
                            );
                            let bg = if is_unlock {
                                Color32::from_rgb(91, 104, 223)
                            } else {
                                Color32::from_rgba_unmultiplied(48, 48, 52, 120)
                            };
                            let border = if is_unlock {
                                Color32::from_rgb(120, 130, 255)
                            } else {
                                Color32::from_gray(60)
                            };
                            ui.painter().rect(
                                rect,
                                5.0,
                                bg,
                                Stroke::new(1.0, border),
                                egui::StrokeKind::Inside,
                            );

                            let kc = layout.get_keycode(0, ki);
                            let label = keycode_label_with_macro_names(
                                kc,
                                &layout.custom_keycodes,
                                &self.layer_names,
                                &self.keycode_picker.macro_names,
                                &self.keycode_picker.tap_dance_names,
                            );
                            let text_color = if is_unlock {
                                Color32::WHITE
                            } else {
                                Color32::from_gray(80)
                            };
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &label,
                                FontId::proportional(9.0 * scale),
                                text_color,
                            );
                        }
                    }
                });
        }

        let any_floating_window_open = self.combo_window_open
            || self.auto_shift_window_open
            || self.mouse_keys_window_open
            || self.alt_repeat_window_open
            || self.rgb_window_open
            || self.encoder_visibility_window_open
            || self.key_override_window_open
            || self.keycode_picker.open;
        if any_floating_window_open && !self.prev_any_floating_window_open {
            self.modal_focus_pending = true;
        }
        if any_floating_window_open {
            let screen_rect = ctx.screen_rect();
            egui::Area::new("window_backdrop".into())
                .order(egui::Order::Foreground)
                .fixed_pos(screen_rect.min)
                .show(ctx, |ui| {
                    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, screen_rect.size());
                    let response =
                        ui.interact(rect, ui.id().with("backdrop_click"), egui::Sense::click());
                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        Color32::from_black_alpha(crate::ui_style::modal_backdrop_alpha(
                            ctx.style().visuals.dark_mode,
                        )),
                    );
                    if response.clicked() {
                        self.combo_window_open = false;
                        self.auto_shift_window_open = false;
                        self.mouse_keys_window_open = false;
                        self.alt_repeat_window_open = false;
                        self.rgb_window_open = false;
                        self.encoder_visibility_window_open = false;
                        self.key_override_window_open = false;
                        self.keycode_picker.open = false;
                        if let Some(id) = ctx.memory(|m| m.focused()) {
                            ctx.memory_mut(|m| m.surrender_focus(id));
                        }
                    }
                });
        }

        if self.combo_window_open {
            self.show_combo_window(ctx);
        }
        if self.auto_shift_window_open {
            self.show_auto_shift_window(ctx);
        }
        if self.mouse_keys_window_open {
            self.show_mouse_keys_window(ctx);
        }
        if self.alt_repeat_window_open {
            self.show_alt_repeat_window(ctx);
        }
        if self.rgb_window_open {
            self.show_rgb_window(ctx);
        }
        if self.encoder_visibility_window_open {
            self.show_encoder_visibility_window(ctx);
        }
        if self.key_override_window_open {
            self.show_key_override_window(ctx);
        }

        if !self.unlock_open && !self.vial_unlock_polling {
            self.keycode_picker.show(ctx);
            self.apply_picker_results();
        }

        self.prev_any_floating_window_open = self.combo_window_open
            || self.auto_shift_window_open
            || self.mouse_keys_window_open
            || self.alt_repeat_window_open
            || self.rgb_window_open
            || self.encoder_visibility_window_open
            || self.key_override_window_open
            || self.keycode_picker.open;
        if self.combo_pick_target.is_some()
            && !self.keycode_picker.open
            && self.keycode_picker.result.is_none()
        {
            self.combo_pick_target = None;
            if self.combo_reopen_after_pick {
                self.combo_window_open = true;
                self.combo_reopen_after_pick = false;
            }
        }
        if self.key_override_pick_target.is_some()
            && !self.keycode_picker.open
            && self.keycode_picker.result.is_none()
        {
            self.key_override_pick_target = None;
            if self.key_override_reopen_after_pick {
                self.key_override_window_open = true;
                self.key_override_reopen_after_pick = false;
            }
        }
        if self.alt_repeat_pick_target.is_some()
            && !self.keycode_picker.open
            && self.keycode_picker.result.is_none()
        {
            self.alt_repeat_pick_target = None;
            if self.alt_repeat_reopen_after_pick {
                self.modal_focus_pending = true;
                self.alt_repeat_window_open = true;
                self.alt_repeat_reopen_after_pick = false;
            }
        }

        // Write macros to device if changed
        if self.keycode_picker.macros_dirty {
            if self.unlock_open || self.vial_unlock_polling {
                // Defer macro write until unlock flow fully finishes.
            } else if self.is_vial_locked() {
                self.unlock_open = true;
                self.status_msg = "Keyboard is locked, unlock it to save macros".into();
            } else {
                self.keycode_picker.macros_dirty = false;
                if let Some(hid) = &self.hid_device {
                    if let Ok(size) = hid.get_macro_buffer_size() {
                        let buf = crate::hid::HidDevice::encode_macros(
                            &self.keycode_picker.macro_texts,
                            size,
                        );
                        match hid.set_macro_buffer(&buf) {
                            Ok(()) => self.status_msg = "Macros saved".into(),
                            Err(e) => self.status_msg = format!("Macro write error: {e}"),
                        }
                    }
                }
            }
        }

        // Write combos to device if changed
        if self.combo_dirty && !self.keycode_picker.open {
            let mut combo_save_ok = true;
            if let Some(hid) = &self.hid_device {
                for (i, combo) in self.combo_entries.iter().enumerate() {
                    match hid.set_combo(i as u8, combo.keys, combo.output) {
                        Ok(()) => {}
                        Err(e) => {
                            self.status_msg = format!("Combo write error: {e}");
                            combo_save_ok = false;
                            break;
                        }
                    }
                }
            }
            if combo_save_ok {
                self.combo_dirty = false;
                self.status_msg = "Combos saved".into();
            }
        }

        if self.combo_term_dirty && !self.keycode_picker.open {
            let mut term_save_ok = true;
            if let (Some(hid), Some(value)) = (&self.hid_device, self.combo_term) {
                if let Err(e) = hid.set_qmk_setting_u16(2, value) {
                    self.status_msg = format!("Combo timeout write error: {e}");
                    term_save_ok = false;
                }
            }
            if term_save_ok {
                self.combo_term_dirty = false;
                self.status_msg = "Combo timeout saved".into();
            }
        }

        if self.combo_names_dirty {
            save_combo_names(&self.combo_names, &self.current_device_name);
            self.combo_names_dirty = false;
        }

        // Write tap dance to device if changed
        if self.keycode_picker.tap_dance_dirty && !self.keycode_picker.open {
            let mut td_save_ok = true;
            if let Some(hid) = &self.hid_device {
                for (i, td) in self.keycode_picker.tap_dance_entries.iter().enumerate() {
                    match hid.set_tap_dance(
                        i as u8,
                        td.on_tap,
                        td.on_hold,
                        td.on_double_tap,
                        td.on_tap_hold,
                        td.tapping_term,
                    ) {
                        Ok(()) => {}
                        Err(e) => {
                            self.status_msg = format!("Tap dance write error: {e}");
                            td_save_ok = false;
                            break;
                        }
                    }
                }
            }
            if td_save_ok {
                save_tap_dance_names(
                    &self.keycode_picker.tap_dance_names,
                    &self.current_device_name,
                );
                self.keycode_picker.tap_dance_dirty = false;
                if self.status_msg.is_empty() || self.status_msg.starts_with("✓") {
                    self.status_msg = "✓ Tap dance saved".into();
                }
            }
        }

        // Right-click anywhere = pop back one step (only if NOT hovering a layer key and not handled by key)
        if !self.jump_back_stack.is_empty()
            && !self.keycode_picker.open
            && !self.secondary_click_handled
        {
            let esc_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
            let rclick = self.hover_layer.is_none() && ctx.input(|i| i.pointer.secondary_clicked());
            if rclick || esc_pressed {
                if let Some(back_layer) = self.jump_back_stack.pop() {
                    self.selected_layer = back_layer;
                }
            }
        }
    }
}

impl EntropyApp {
    /// Show ZMK unlock modal overlay.
    #[cfg(not(target_arch = "wasm32"))]
    fn show_zmk_unlock_modal(&self, ctx: &egui::Context) {
        // Draw dimmed background panels
        egui::TopBottomPanel::top("topbar_locked").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⌨ Entropy").size(18.0).strong());
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        RichText::new("🔒 Keyboard locked")
                            .size(24.0)
                            .strong()
                            .color(Color32::from_rgb(220, 160, 60)),
                    );
                    ui.add_space(16.0);

                    // Check if there's a studio unlock behavior
                    let has_unlock_key = self.zmk_conn.is_none(); // after poll the conn is taken
                    if has_unlock_key {
                        ui.label(
                            RichText::new(
                                "Press the unlock key on your keyboard to allow editing.",
                            )
                            .size(14.0),
                        );
                    } else {
                        ui.label(
                            RichText::new(
                                "Hold the unlock combo on your keyboard to allow editing.",
                            )
                            .size(14.0),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Hold simultaneously: Studio Unlock key")
                                .size(12.0)
                                .color(Color32::GRAY),
                        );
                    }

                    ui.add_space(24.0);
                    ui.label(
                        RichText::new("Keep holding… Unlock ZMK Studio")
                            .size(12.0)
                            .color(Color32::GRAY),
                    );
                    ui.add_space(16.0);
                    ui.spinner();
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Waiting for unlock…")
                            .size(11.0)
                            .color(Color32::GRAY),
                    );
                });
            });
        });

        // Keep repainting while waiting
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}

impl EntropyApp {
    fn push_combo_undo(&mut self) {
        self.combo_undo_stack.push((
            self.combo_entries.clone(),
            self.combo_names.clone(),
            self.combo_term,
            self.selected_combo,
            self.combo_visible_count,
        ));
        if self.combo_undo_stack.len() > 64 {
            self.combo_undo_stack.remove(0);
        }
    }

    fn apply_combo_capture(&mut self) {
        if !(2..=4).contains(&self.combo_capture_keys.len()) {
            return;
        }
        self.push_combo_undo();
        if let Some(combo) = self.combo_entries.get_mut(self.selected_combo) {
            combo.keys = [0; 4];
            for (idx, kc) in self.combo_capture_keys.iter().copied().enumerate().take(4) {
                combo.keys[idx] = kc;
            }
            self.combo_dirty = true;
        }
        self.combo_capture_open = false;
        self.combo_capture_keys.clear();
    }

    fn cancel_combo_capture(&mut self) {
        self.combo_capture_open = false;
        self.combo_capture_keys.clear();
    }

    fn push_key_override_undo(&mut self) {
        self.key_override_undo_stack.push((
            self.key_override_entries.clone(),
            self.key_override_names.clone(),
            self.selected_key_override,
            self.key_override_visible_count,
        ));
        if self.key_override_undo_stack.len() > 64 {
            self.key_override_undo_stack.remove(0);
        }
    }

    fn write_all_key_overrides(&mut self) {
        for idx in 0..self.key_override_entries.len() {
            self.write_key_override(idx);
        }
    }

    fn key_override_entry_exists(entry: &KeyOverrideEntry) -> bool {
        entry.trigger != 0
            || entry.replacement != 0
            || entry.layers != 0
            || entry.trigger_mods != 0
            || entry.negative_mod_mask != 0
            || entry.suppressed_mods != 0
            || entry.options.activation_trigger_down
            || entry.options.activation_required_mod_down
            || entry.options.activation_negative_mod_up
            || entry.options.one_mod
            || entry.options.no_reregister_trigger
            || entry.options.no_unregister_on_other_key_down
    }

    fn normalize_key_override_entry(entry: &mut KeyOverrideEntry) {
        entry.options.enabled = Self::key_override_entry_exists(entry);
    }

    fn write_key_override(&mut self, idx: usize) {
        let Some(entry) = self.key_override_entries.get_mut(idx) else {
            return;
        };
        Self::normalize_key_override_entry(entry);
        let entry = entry.clone();
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_key_override(
            idx as u8,
            entry.trigger,
            entry.replacement,
            entry.layers,
            entry.trigger_mods,
            entry.negative_mod_mask,
            entry.suppressed_mods,
            entry.options.bits(),
        ) {
            self.status_msg = format!("Failed to save Key Override {}: {}", idx + 1, e);
            log::warn!("set_key_override({idx}) failed: {e}");
        }
    }

    fn open_key_override_picker(&mut self, target: KeyOverridePickField) {
        self.key_override_pick_target = Some(target);
        self.key_override_reopen_after_pick = true;
        self.key_override_window_open = false;
        self.keycode_picker.result = None;
        self.keycode_picker.open = true;
    }

    fn write_alt_repeat_entry(&mut self, idx: usize) {
        let Some(entry) = self.alt_repeat_entries.get(idx).cloned() else {
            return;
        };
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_alt_repeat_key(
            idx as u8,
            entry.keycode,
            entry.alt_keycode,
            entry.allowed_mods,
            entry.options.bits(),
        ) {
            self.status_msg = format!("Failed to save Alt Repeat {}: {}", idx + 1, e);
            log::warn!("set_alt_repeat_key({idx}) failed: {e}");
        }
    }

    fn open_alt_repeat_picker(&mut self, target: AltRepeatPickField) {
        self.alt_repeat_pick_target = Some(target);
        self.alt_repeat_reopen_after_pick = true;
        self.alt_repeat_window_open = false;
        self.keycode_picker.result = None;
        self.keycode_picker.open = true;
    }

    fn open_alt_repeat_window_compact(&mut self) {
        self.selected_alt_repeat = 0;
        self.alt_repeat_visible_count = 1;
        self.modal_focus_pending = true;
        self.alt_repeat_window_open = true;
    }

    fn close_top_dropdowns(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new("device_dropdown_open"), false);
            d.insert_temp(egui::Id::new("advanced_dropdown_open"), false);
            d.insert_temp(egui::Id::new("settings_dropdown_open"), false);
        });
    }

    fn focus_modal_window<T>(&mut self, shown: &Option<egui::InnerResponse<T>>) {
        if self.modal_focus_pending {
            if let Some(shown) = shown {
                shown.response.request_focus();
                self.modal_focus_pending = false;
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn poll_single_instance_signal(&mut self, ctx: &egui::Context) {
        let signal = read_single_instance_signal();
        if signal.is_empty() || signal == self.last_single_instance_signal {
            return;
        }
        self.last_single_instance_signal = signal;
        self.status_msg = "Entropy refreshed from a repeated launch".into();
        self.device_manager.scan();
        if let Some(device_idx) = self.selected_device {
            if !matches!(self.connect_state, ConnectState::Loading(_)) {
                self.start_connect(device_idx);
            }
        }
        ctx.request_repaint();
    }

    fn write_auto_shift_flags(&mut self) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u8(3, self.auto_shift_options.bits()) {
            self.status_msg = format!("Failed to save Auto Shift flags: {}", e);
            log::warn!("set_qmk_setting_u8(auto_shift_flags) failed: {e}");
        }
    }

    fn write_auto_shift_timeout(&mut self) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let Some(timeout) = self.auto_shift_timeout else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u16(4, timeout) {
            self.status_msg = format!("Failed to save Auto Shift timeout: {}", e);
            log::warn!("set_qmk_setting_u16(auto_shift_timeout) failed: {e}");
        }
    }

    fn draw_key_override_layers(ui: &mut egui::Ui, layers: &mut u16) -> bool {
        let mut changed = false;
        egui::Grid::new(ui.id().with("ko_layers_grid"))
            .num_columns(6)
            .spacing([10.0, 6.0])
            .show(ui, |ui| {
                for row in 0..3 {
                    for col in 0..6 {
                        let idx = row * 6 + col;
                        if idx >= 16 {
                            ui.label("");
                            continue;
                        }
                        let mut checked = (*layers & (1 << idx)) != 0;
                        if ui.checkbox(&mut checked, idx.to_string()).changed() {
                            if checked {
                                *layers |= 1 << idx;
                            } else {
                                *layers &= !(1 << idx);
                            }
                            changed = true;
                        }
                    }
                    ui.end_row();
                }
            });
        ui.horizontal(|ui| {
            if ui.button("Enable all").clicked() {
                if *layers != u16::MAX {
                    *layers = u16::MAX;
                    changed = true;
                }
            }
            if ui.button("Disable all").clicked() {
                if *layers != 0 {
                    *layers = 0;
                    changed = true;
                }
            }
        });
        changed
    }

    fn draw_key_override_mod_mask(ui: &mut egui::Ui, mask: &mut u8, id: &str) -> bool {
        let mut changed = false;
        let gui = crate::keycode::gui_mod_name();
        let labels = vec![
            "Left Ctrl".to_string(),
            "Left Shift".to_string(),
            "Left Alt".to_string(),
            format!("Left {}", gui),
            "Right Ctrl".to_string(),
            "Right Shift".to_string(),
            "Right Alt".to_string(),
            format!("Right {}", gui),
        ];
        egui::Grid::new(ui.id().with(id))
            .num_columns(2)
            .spacing([18.0, 4.0])
            .show(ui, |ui| {
                for row in 0..4 {
                    for col in 0..2 {
                        let idx = row * 2 + col;
                        let mut checked = (*mask & (1 << idx)) != 0;
                        if ui.checkbox(&mut checked, labels[idx].as_str()).changed() {
                            if checked {
                                *mask |= 1 << idx;
                            } else {
                                *mask &= !(1 << idx);
                            }
                            changed = true;
                        }
                    }
                    ui.end_row();
                }
            });
        changed
    }

    fn set_rgb_effect(&mut self, effect: u16) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = match self.rgb_settings.kind {
            RgbSupportKind::QmkRgblight => {
                hid.set_qmk_rgblight_effect(effect.min(u8::MAX as u16) as u8)
            }
            RgbSupportKind::VialRgb => hid.set_vialrgb_mode(
                effect,
                self.rgb_settings.speed,
                self.rgb_settings.hue,
                self.rgb_settings.saturation,
                self.rgb_settings.brightness,
            ),
            RgbSupportKind::None => return,
        };
        match result {
            Ok(()) => {
                self.rgb_settings.effect = effect;
                if effect != 0 {
                    self.rgb_settings.last_enabled_effect = effect;
                }
            }
            Err(e) => {
                self.status_msg = format!("Failed to update RGB effect: {}", e);
                log::warn!("set_rgb_effect failed: {e}");
            }
        }
    }

    fn set_rgb_brightness(&mut self, brightness: u8) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = match self.rgb_settings.kind {
            RgbSupportKind::QmkRgblight => hid.set_qmk_rgblight_brightness(brightness),
            RgbSupportKind::VialRgb => hid.set_vialrgb_mode(
                self.rgb_settings.effect,
                self.rgb_settings.speed,
                self.rgb_settings.hue,
                self.rgb_settings.saturation,
                brightness,
            ),
            RgbSupportKind::None => return,
        };
        match result {
            Ok(()) => self.rgb_settings.brightness = brightness,
            Err(e) => {
                self.status_msg = format!("Failed to update RGB brightness: {}", e);
                log::warn!("set_rgb_brightness failed: {e}");
            }
        }
    }

    fn set_rgb_color(&mut self, hue: u8, saturation: u8) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = match self.rgb_settings.kind {
            RgbSupportKind::QmkRgblight => hid.set_qmk_rgblight_color(hue, saturation),
            RgbSupportKind::VialRgb => hid.set_vialrgb_mode(
                self.rgb_settings.effect,
                self.rgb_settings.speed,
                hue,
                saturation,
                self.rgb_settings.brightness,
            ),
            RgbSupportKind::None => return,
        };
        match result {
            Ok(()) => {
                self.rgb_settings.hue = hue;
                self.rgb_settings.saturation = saturation;
            }
            Err(e) => {
                self.status_msg = format!("Failed to update RGB color: {}", e);
                log::warn!("set_rgb_color failed: {e}");
            }
        }
    }

    fn set_rgb_speed(&mut self, speed: u8) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = match self.rgb_settings.kind {
            RgbSupportKind::QmkRgblight => hid.set_qmk_rgblight_effect_speed(speed),
            RgbSupportKind::VialRgb => hid.set_vialrgb_mode(
                self.rgb_settings.effect,
                speed,
                self.rgb_settings.hue,
                self.rgb_settings.saturation,
                self.rgb_settings.brightness,
            ),
            RgbSupportKind::None => return,
        };
        match result {
            Ok(()) => self.rgb_settings.speed = speed,
            Err(e) => {
                self.status_msg = format!("Failed to update RGB speed: {}", e);
                log::warn!("set_rgb_speed failed: {e}");
            }
        }
    }

    fn save_rgb_settings(&mut self) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.save_rgb() {
            self.status_msg = format!("Failed to save RGB settings: {}", e);
            log::warn!("save_rgb failed: {e}");
        } else {
            self.status_msg = "RGB settings saved".to_string();
        }
    }

    fn show_rgb_window(&mut self, ctx: &egui::Context) {
        let mut open = self.rgb_window_open;
        let mut close_after_save = false;
        let dark = ctx.style().visuals.dark_mode;
        let style = ctx.style().as_ref().clone();
        let frame = crate::ui_style::modal_window_frame(&style, dark);

        let shown = egui::Window::new("RGB")
            .id(egui::Id::new("rgb_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .movable(true)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(500.0, 270.0))
            .frame(frame)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.add_space(6.0);

                if !self.rgb_settings.supported {
                    ui.vertical_centered(|ui| {
                        ui.add_space(52.0);
                        ui.label(
                            RichText::new("RGB settings are not available on this firmware.")
                                .size(13.0)
                                .color(app_muted_text(dark)),
                        );
                    });
                    return;
                }

                let options = rgb_effect_options(&self.rgb_settings);
                let mut enabled = self.rgb_settings.is_enabled();
                let mut selected_effect = self.rgb_settings.effect;
                let brightness_max = self.rgb_settings.max_brightness.max(1);
                let current_percent = ((self.rgb_settings.brightness as f32 / brightness_max as f32)
                    * 100.0)
                    .round()
                    .clamp(0.0, 100.0);
                let mut brightness_percent = current_percent;
                let selected_effect_name = options
                    .iter()
                    .find(|(id, _)| *id == self.rgb_settings.effect)
                    .map(|(_, name)| *name)
                    .unwrap_or("Unknown");
                let speed_max = 255.0_f32;
                let mut speed_percent = ((self.rgb_settings.speed as f32 / speed_max) * 100.0)
                    .round()
                    .clamp(0.0, 100.0);
                let mut color_hsva = egui::ecolor::Hsva {
                    h: self.rgb_settings.hue as f32 / 255.0,
                    s: self.rgb_settings.saturation as f32 / 255.0,
                    v: 1.0,
                    a: 1.0,
                };

                ui.vertical_centered(|ui| {
                    ui.set_width(500.0);
                    ui.add_space(4.0);

                    ui.allocate_ui_with_layout(
                        Vec2::new(380.0, 0.0),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            let label_width = 96.0_f32;
                            let control_width = 252.0_f32;

                            ui.horizontal(|ui| {
                                ui.set_width(label_width + control_width);
                                ui.add_sized(
                                    [label_width, 24.0],
                                    egui::Label::new(RichText::new("Enable").size(12.5)),
                                );
                                let enable_resp = ui.checkbox(&mut enabled, "");
                                if enable_resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if enable_resp.changed() {
                                    let next_effect = if enabled {
                                        self.rgb_settings.effect_or_default()
                                    } else {
                                        if self.rgb_settings.effect != 0 {
                                            self.rgb_settings.last_enabled_effect =
                                                self.rgb_settings.effect;
                                        }
                                        0
                                    };
                                    self.set_rgb_effect(next_effect);
                                    selected_effect = self.rgb_settings.effect;
                                }
                            });

                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                ui.set_width(label_width + control_width);
                                ui.add_sized(
                                    [label_width, 24.0],
                                    egui::Label::new(RichText::new("Effect").size(12.5)),
                                );
                                egui::ComboBox::from_id_salt("rgb_effect_combo")
                                    .selected_text(selected_effect_name)
                                    .width(control_width)
                                    .show_ui(ui, |ui| {
                                        for (id, label) in &options {
                                            if ui
                                                .selectable_value(
                                                    &mut selected_effect,
                                                    *id,
                                                    *label,
                                                )
                                                .changed()
                                            {
                                                self.set_rgb_effect(selected_effect);
                                            }
                                        }
                                    });
                            });

                            ui.add_space(10.0);

                            let color_enabled = rgb_effect_supports_color(self.rgb_settings.kind, selected_effect);
                            ui.horizontal(|ui| {
                                ui.set_width(label_width + control_width);
                                ui.add_sized(
                                    [label_width, 24.0],
                                    egui::Label::new(
                                        RichText::new("Color")
                                            .size(12.5)
                                            .color(if color_enabled {
                                                ui.visuals().text_color()
                                            } else {
                                                app_muted_text(dark)
                                            }),
                                    ),
                                );

                                let popup_id = ui.make_persistent_id("rgb_color_popup");
                                let popup_hsva_id = popup_id.with("hsva");
                                let popup_open = ui.memory(|m| m.is_popup_open(popup_id));
                                let border = if dark {
                                    Color32::from_gray(95)
                                } else {
                                    Color32::from_gray(185)
                                };
                                let swatch_border = if color_enabled && popup_open {
                                    app_accent()
                                } else {
                                    border
                                };
                                let swatch_color: Color32 = color_hsva.into();
                                let swatch_sense = if color_enabled { Sense::click() } else { Sense::hover() };
                                let (swatch_rect, swatch_resp) = ui.allocate_exact_size(
                                    Vec2::new(56.0, 32.0),
                                    swatch_sense,
                                );
                                if color_enabled && swatch_resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if color_enabled && swatch_resp.clicked() {
                                    ui.ctx().data_mut(|d| d.insert_temp(popup_hsva_id, color_hsva));
                                    ui.memory_mut(|m| m.toggle_popup(popup_id));
                                }
                                ui.painter().rect(
                                    swatch_rect,
                                    8.0,
                                    app_surface_fill(dark),
                                    Stroke::new(1.0, swatch_border),
                                    egui::StrokeKind::Inside,
                                );
                                ui.painter().rect(
                                    swatch_rect.shrink(5.0),
                                    5.0,
                                    if color_enabled {
                                        swatch_color
                                    } else {
                                        swatch_color.gamma_multiply(0.45)
                                    },
                                    Stroke::new(1.0, swatch_border.gamma_multiply(0.85)),
                                    egui::StrokeKind::Inside,
                                );

                                if color_enabled {
                                    let mut picked_hsva = ui
                                        .ctx()
                                        .data(|d| d.get_temp::<egui::ecolor::Hsva>(popup_hsva_id))
                                        .unwrap_or(color_hsva);
                                    egui::popup_below_widget(
                                        ui,
                                        popup_id,
                                        &swatch_resp,
                                        egui::PopupCloseBehavior::CloseOnClickOutside,
                                        |ui| {
                                            ui.spacing_mut().slider_width = 136.0;
                                            if compact_rgb_color_picker(ui, &mut picked_hsva) {
                                                let hue = (picked_hsva.h.rem_euclid(1.0) * 255.0)
                                                    .round()
                                                    .clamp(0.0, 255.0)
                                                    as u8;
                                                let saturation = (picked_hsva
                                                    .s
                                                    .clamp(0.0, 1.0)
                                                    * 255.0)
                                                    .round()
                                                    .clamp(0.0, 255.0)
                                                    as u8;
                                                self.set_rgb_color(hue, saturation);
                                                color_hsva = picked_hsva;
                                                ui.ctx().data_mut(|d| d.insert_temp(popup_hsva_id, picked_hsva));
                                            }
                                        },
                                    );
                                }
                            });

                            ui.add_space(10.0);

                            let speed_enabled = rgb_effect_supports_speed(self.rgb_settings.kind, selected_effect);
                            ui.horizontal(|ui| {
                                ui.set_width(label_width + control_width);
                                ui.add_sized(
                                    [label_width, 24.0],
                                    egui::Label::new(
                                        RichText::new("Speed")
                                            .size(12.5)
                                            .color(if speed_enabled {
                                                ui.visuals().text_color()
                                            } else {
                                                app_muted_text(dark)
                                            }),
                                    ),
                                );
                                ui.add_enabled_ui(speed_enabled, |ui| {
                                    ui.scope(|ui| {
                                        ui.spacing_mut().slider_width = 184.0;
                                        let slider = egui::Slider::new(
                                            &mut speed_percent,
                                            0.0..=100.0,
                                        )
                                        .step_by(1.0)
                                        .show_value(false)
                                        .trailing_fill(true);
                                        let resp = ui.add_sized([192.0, 24.0], slider);
                                        if resp.changed() {
                                            let raw_value = ((speed_percent / 100.0) * speed_max)
                                                .round()
                                                .clamp(0.0, speed_max)
                                                as u8;
                                            self.set_rgb_speed(raw_value);
                                        }
                                    });
                                });
                                ui.add_space(8.0);
                                ui.add_sized(
                                    [52.0, 28.0],
                                    egui::Label::new(
                                        RichText::new(format!("{}%", speed_percent as u8))
                                            .size(12.0)
                                            .color(if speed_enabled {
                                                if dark {
                                                    Color32::from_gray(230)
                                                } else {
                                                    Color32::from_gray(55)
                                                }
                                            } else {
                                                app_muted_text(dark)
                                            }),
                                    )
                                    .sense(egui::Sense::hover()),
                                );
                            });

                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                ui.set_width(label_width + control_width);
                                ui.add_sized(
                                    [label_width, 24.0],
                                    egui::Label::new(RichText::new("Brightness").size(12.5)),
                                );
                                ui.scope(|ui| {
                                    ui.spacing_mut().slider_width = 184.0;
                                    let slider = egui::Slider::new(
                                        &mut brightness_percent,
                                        0.0..=100.0,
                                    )
                                    .step_by(1.0)
                                    .show_value(false)
                                    .trailing_fill(true);
                                    let resp = ui.add_sized([192.0, 24.0], slider);
                                    if resp.changed() {
                                        let raw_value =
                                            ((brightness_percent / 100.0) * brightness_max as f32)
                                                .round()
                                                .clamp(0.0, brightness_max as f32)
                                                as u8;
                                        self.set_rgb_brightness(raw_value);
                                    }
                                });
                                ui.add_space(8.0);
                                ui.add_sized(
                                    [52.0, 28.0],
                                    egui::Label::new(
                                        RichText::new(format!("{}%", brightness_percent as u8))
                                            .size(12.0)
                                            .color(if dark {
                                                Color32::from_gray(230)
                                            } else {
                                                Color32::from_gray(55)
                                            }),
                                    )
                                    .sense(egui::Sense::hover()),
                                );
                            });

                            ui.add_space(22.0);

                            ui.horizontal_centered(|ui| {
                                let btn = egui::Button::new(RichText::new("Save").size(13.0))
                                    .min_size(crate::ui_style::modal_action_button_size());
                                if ui.add(btn).clicked() {
                                    self.save_rgb_settings();
                                    close_after_save = true;
                                }
                            });
                        },
                    );
                });
            });

        if close_after_save {
            open = false;
        }
        self.focus_modal_window(&shown);
        self.rgb_window_open = open;
    }

    fn show_encoder_visibility_window(&mut self, ctx: &egui::Context) {
        let mut open = self.encoder_visibility_window_open;
        let shown = egui::Window::new("Encoders")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .movable(true)
            .order(egui::Order::Foreground)
            .default_width(280.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(crate::ui_style::modal_window_frame(
                &ctx.style(),
                ctx.style().visuals.dark_mode,
            ))
            .show(ctx, |ui| {
                if self.encoder_visibility.is_empty() {
                    ui.label("No encoders found for this device.");
                    return;
                }
                ui.label(
                    RichText::new("Choose which encoders are visible in the main layout")
                        .size(12.0)
                        .color(app_muted_text(ui.visuals().dark_mode)),
                );
                ui.add_space(10.0);
                let mut changed = false;
                for (idx, visible) in self.encoder_visibility.iter_mut().enumerate() {
                    let resp = ui.checkbox(visible, format!("Show Encoder {}", idx + 1));
                    if resp.changed() {
                        changed = true;
                    }
                }
                if changed && !self.current_device_name.is_empty() {
                    save_encoder_visibility(&self.encoder_visibility, &self.current_device_name);
                }
            });
        self.focus_modal_window(&shown);
        self.encoder_visibility_window_open = open;
    }

    fn show_alt_repeat_window(&mut self, ctx: &egui::Context) {
        if !self.keycode_picker.open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.alt_repeat_window_open = false;
            return;
        }

        let dark = ctx.style().visuals.dark_mode;
        let mut open = self.alt_repeat_window_open;
        let shown = egui::Window::new("Alt Repeat")
            .id(egui::Id::new("alt_repeat_window_v2"))
            .order(egui::Order::Foreground)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(444.0, 500.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .frame(crate::ui_style::modal_window_frame(ctx.style().as_ref(), dark))
            .show(ctx, |ui| {
                if self.alt_repeat_entries.is_empty() {
                    ui.label("Alt Repeat is not supported by this keyboard.");
                    return;
                }

                if self.selected_alt_repeat >= self.alt_repeat_entries.len() {
                    self.selected_alt_repeat = 0;
                }
                self.selected_alt_repeat = self
                    .selected_alt_repeat
                    .min(self.alt_repeat_entries.len().saturating_sub(1));

                ui.add_space(2.0);
                let idx = self.selected_alt_repeat;
                let current = self.alt_repeat_entries[idx].clone();
                let mut edited = current.clone();
                let content_width = 360.0_f32;
                let field_width = 220.0_f32;
                let custom = self
                    .layout
                    .as_ref()
                    .map(|l| l.custom_keycodes.as_slice())
                    .unwrap_or(&[]);
                let last_key_label = if edited.keycode == 0 {
                    "Pick key".to_string()
                } else {
                    keycode_label_with_macro_names(
                        edited.keycode,
                        custom,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    )
                    .replace("\n", " ")
                };
                let alt_key_label = if edited.alt_keycode == 0 {
                    "Pick key".to_string()
                } else {
                    keycode_label_with_macro_names(
                        edited.alt_keycode,
                        custom,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    )
                    .replace("\n", " ")
                };
                let last_key_tip = keycode_tooltip_with_macro_names(
                    edited.keycode,
                    custom,
                    &self.layer_names,
                    &self.keycode_picker.macro_names,
                    &self.keycode_picker.tap_dance_names,
                );
                let alt_key_tip = keycode_tooltip_with_macro_names(
                    edited.alt_keycode,
                    custom,
                    &self.layer_names,
                    &self.keycode_picker.macro_names,
                    &self.keycode_picker.tap_dance_names,
                );

                ui.horizontal(|ui| {
                    let left_pad = ((ui.available_width() - content_width).max(0.0)) * 0.5;
                    ui.add_space(left_pad);
                    ui.vertical(|ui| {
                        ui.set_width(content_width);

                        egui::ComboBox::from_id_salt("alt_repeat_entry_select")
                            .selected_text(format!("AR{}", self.selected_alt_repeat))
                            .width(140.0)
                            .show_ui(ui, |ui| {
                                for idx in 0..self.alt_repeat_entries.len() {
                                    let resp = ui.selectable_value(
                                        &mut self.selected_alt_repeat,
                                        idx,
                                        format!("AR{}", idx),
                                    );
                                    if resp.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                }
                            });

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Enable").size(12.5));
                            let resp = ui.checkbox(&mut edited.options.enabled, "");
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                        });

                        ui.add_space(8.0);
                        ui.label(RichText::new("Last key").size(12.0).strong());
                        ui.add_space(4.0);
                        ui.horizontal_centered(|ui| {
                            let resp = ui
                                .add_sized(
                                    [field_width, 34.0],
                                    egui::Button::new(RichText::new(last_key_label).size(12.0)),
                                )
                                .on_hover_cursor(egui::CursorIcon::PointingHand);
                            if resp.clicked() {
                                self.open_alt_repeat_picker(AltRepeatPickField::LastKey);
                            }
                            resp.on_hover_text(last_key_tip);
                        });

                        ui.add_space(8.0);
                        ui.label(RichText::new("Alt key").size(12.0).strong());
                        ui.add_space(4.0);
                        ui.horizontal_centered(|ui| {
                            let resp = ui
                                .add_sized(
                                    [field_width, 34.0],
                                    egui::Button::new(RichText::new(alt_key_label).size(12.0)),
                                )
                                .on_hover_cursor(egui::CursorIcon::PointingHand);
                            if resp.clicked() {
                                self.open_alt_repeat_picker(AltRepeatPickField::AltKey);
                            }
                            resp.on_hover_text(alt_key_tip);
                        });

                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("Allowed mods")
                                .size(11.0)
                                .color(app_muted_text(dark)),
                        );
                        ui.add_space(4.0);
                        Self::draw_key_override_mod_mask(
                            ui,
                            &mut edited.allowed_mods,
                            "alt_repeat_allowed_mods",
                        );

                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("Options")
                                .size(11.0)
                                .color(app_muted_text(dark)),
                        );
                        ui.add_space(4.0);
                        let row = |ui: &mut egui::Ui, label: &str, value: &mut bool| {
                            let resp = ui.checkbox(value, label);
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                        };
                        row(
                            ui,
                            "Default to this alt key",
                            &mut edited.options.default_to_this_alt_key,
                        );
                        row(ui, "Bidirectional", &mut edited.options.bidirectional);
                        row(
                            ui,
                            "Ignore mod handedness",
                            &mut edited.options.ignore_mod_handedness,
                        );
                    });
                });

                if edited != current {
                    if let Some(slot) = self.alt_repeat_entries.get_mut(idx) {
                        *slot = edited;
                    }
                    self.write_alt_repeat_entry(idx);
                }
            });

        self.focus_modal_window(&shown);
        self.alt_repeat_window_open = open;
    }

    fn show_auto_shift_window(&mut self, ctx: &egui::Context) {
        let mut open = self.auto_shift_window_open;
        let dark = ctx.style().visuals.dark_mode;
        let style = ctx.style().as_ref().clone();
        let frame = crate::ui_style::modal_window_frame(&style, dark);

        let shown = egui::Window::new("Auto Shift")
            .id(egui::Id::new("auto_shift_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .movable(true)
            .default_pos(egui::pos2(
                ctx.screen_rect().center().x - 220.0,
                ctx.screen_rect().center().y - 140.0,
            ))
            .fixed_size(Vec2::new(440.0, 280.0))
            .frame(frame)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                if self.auto_shift_timeout.is_none() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(64.0);
                        ui.label(
                            RichText::new("Auto Shift is not enabled in this firmware.")
                                .size(13.0)
                                .color(app_muted_text(dark)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Enable AUTO_SHIFT_ENABLE in the keyboard rules.mk to use this window.")
                                .size(11.5)
                                .color(app_muted_text(dark)),
                        );
                    });
                    return;
                }

                let mut timeout_value = self.auto_shift_timeout.unwrap_or(175);
                let mut timeout_text = timeout_value.to_string();
                let content_width = 360.0_f32;

                ui.vertical_centered(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(content_width, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            egui::Grid::new(ui.id().with("auto_shift_grid"))
                                .num_columns(2)
                                .spacing([18.0, 10.0])
                                .show(ui, |ui| {
                                    let mut checkbox_row = |ui: &mut egui::Ui, label: &str, value: &mut bool| -> bool {
                                        ui.label(RichText::new(label).size(12.5));
                                        let resp = ui.checkbox(value, "");
                                        if resp.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        ui.end_row();
                                        resp.changed()
                                    };

                                    let mut options_changed = false;
                                    options_changed |= checkbox_row(ui, "Enable", &mut self.auto_shift_options.enabled);
                                    options_changed |= checkbox_row(ui, "Enable for modifiers", &mut self.auto_shift_options.enable_for_modifiers);

                                    ui.label(RichText::new("Timeout").size(12.5));
                                    ui.horizontal(|ui| {
                                        let resp = ui.add(
                                            egui::TextEdit::singleline(&mut timeout_text)
                                                .desired_width(52.0)
                                        );
                                        if resp.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
                                        }
                                        ui.label(RichText::new("ms").size(11.5).color(app_muted_text(dark)));
                                        if resp.changed() {
                                            let filtered: String = timeout_text.chars().filter(|c: &char| c.is_ascii_digit()).collect();
                                            if let Ok(parsed) = filtered.parse::<u16>() {
                                                timeout_value = parsed.max(1);
                                                self.auto_shift_timeout = Some(timeout_value);
                                                self.write_auto_shift_timeout();
                                            }
                                        }
                                    });
                                    ui.end_row();

                                    options_changed |= checkbox_row(ui, "Do not Auto Shift special keys", &mut self.auto_shift_options.no_special);
                                    options_changed |= checkbox_row(ui, "Do not Auto Shift numeric keys", &mut self.auto_shift_options.no_numeric);
                                    options_changed |= checkbox_row(ui, "Do not Auto Shift alpha characters", &mut self.auto_shift_options.no_alpha);
                                    options_changed |= checkbox_row(ui, "Enable keyrepeat", &mut self.auto_shift_options.enable_keyrepeat);
                                    options_changed |= checkbox_row(ui, "Disable keyrepeat when timeout is exceeded", &mut self.auto_shift_options.disable_keyrepeat_timeout);

                                    if options_changed {
                                        self.write_auto_shift_flags();
                                    }
                                });
                        },
                    );
                });
            });

        self.focus_modal_window(&shown);
        self.auto_shift_window_open = open;
    }

    /// Write a single mouse-keys QMK setting to device. In Vial qmk_settings these are width=1.
    fn write_mouse_keys_setting(&mut self, qsid: u16, value: u16) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let value = value.min(u8::MAX as u16) as u8;
        if let Err(e) = hid.set_qmk_setting_u8(qsid, value) {
            self.status_msg = format!("Failed to save Mouse keys setting (qsid {qsid}): {}", e);
            log::warn!("set_qmk_setting_u8(mouse_keys qsid {qsid}) failed: {e}");
        }
    }

    fn show_mouse_keys_window(&mut self, ctx: &egui::Context) {
        if !self.keycode_picker.open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.mouse_keys_window_open = false;
            return;
        }

        let mut open = self.mouse_keys_window_open;
        let dark = ctx.style().visuals.dark_mode;
        let style = ctx.style().as_ref().clone();
        let frame = crate::ui_style::modal_window_frame(&style, dark);

        let shown = egui::Window::new("Mouse keys")
            .id(egui::Id::new("mouse_keys_window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .movable(true)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(520.0, 360.0))
            .frame(frame)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.add_space(8.0);

                if !self.mouse_keys_settings.supported {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.label(
                            RichText::new("Mouse keys settings are not available on this firmware.")
                                .size(13.0)
                                .color(app_muted_text(dark)),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Enable MOUSEKEY_ENABLE and QMK_SETTINGS in the keyboard rules.mk to use this window.")
                                .size(11.5)
                                .color(app_muted_text(dark)),
                        );
                    });
                    return;
                }

                let content_width = 480.0_f32;
                // (qsid, label, max, &mut value)
                // Limits match Vial GUI qmk_settings.json.
                let rows: [(u16, &str, u32); 9] = [
                    (9,  "Delay between pressing a movement key and cursor movement", 10000),
                    (10, "Time between cursor movements in milliseconds",             10000),
                    (11, "Step size",                                                  1000),
                    (12, "Maximum cursor speed at which acceleration stops",           1000),
                    (13, "Time until maximum cursor speed is reached",                 1000),
                    (14, "Delay between pressing a wheel key and wheel movement",     10000),
                    (15, "Time between wheel movements",                              10000),
                    (16, "Maximum number of scroll steps per scroll action",           1000),
                    (17, "Time until maximum scroll speed is reached",                 1000),
                ];

                ui.vertical_centered(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(content_width, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            egui::Grid::new(ui.id().with("mouse_keys_grid"))
                                .num_columns(2)
                                .spacing([18.0, 10.0])
                                .show(ui, |ui| {
                                    for (qsid, label, max) in rows.iter().copied() {
                                        let (current, write_back): (u16, Box<dyn FnOnce(&mut MouseKeysSettingsState, u16)>) = match qsid {
                                            9  => (self.mouse_keys_settings.delay,
                                                   Box::new(|s: &mut MouseKeysSettingsState, v| s.delay = v)),
                                            10 => (self.mouse_keys_settings.interval,
                                                   Box::new(|s: &mut MouseKeysSettingsState, v| s.interval = v)),
                                            11 => (self.mouse_keys_settings.move_delta,
                                                   Box::new(|s: &mut MouseKeysSettingsState, v| s.move_delta = v)),
                                            12 => (self.mouse_keys_settings.max_speed,
                                                   Box::new(|s: &mut MouseKeysSettingsState, v| s.max_speed = v)),
                                            13 => (self.mouse_keys_settings.time_to_max,
                                                   Box::new(|s: &mut MouseKeysSettingsState, v| s.time_to_max = v)),
                                            14 => (self.mouse_keys_settings.wheel_delay,
                                                   Box::new(|s: &mut MouseKeysSettingsState, v| s.wheel_delay = v)),
                                            15 => (self.mouse_keys_settings.wheel_interval,
                                                   Box::new(|s: &mut MouseKeysSettingsState, v| s.wheel_interval = v)),
                                            16 => (self.mouse_keys_settings.wheel_max_speed,
                                                   Box::new(|s: &mut MouseKeysSettingsState, v| s.wheel_max_speed = v)),
                                            17 => (self.mouse_keys_settings.wheel_time_to_max,
                                                   Box::new(|s: &mut MouseKeysSettingsState, v| s.wheel_time_to_max = v)),
                                            _ => continue,
                                        };

                                        ui.label(RichText::new(label).size(12.5));

                                        let edit_id = egui::Id::new(("mouse_keys_edit", qsid));
                                        let mut text = ui.ctx().data_mut(|d| {
                                            d.get_temp::<String>(edit_id).unwrap_or_else(|| current.to_string())
                                        });
                                        // If external value changed and user is not editing, resync text.
                                        if text.parse::<u16>().ok() != Some(current) && !ui.memory(|m| m.has_focus(edit_id)) {
                                            text = current.to_string();
                                        }

                                        let resp = ui.add(
                                            egui::TextEdit::singleline(&mut text)
                                                .id(edit_id)
                                                .desired_width(72.0),
                                        );
                                        if resp.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
                                        }
                                        ui.end_row();

                                        if resp.changed() {
                                            let filtered: String = text.chars().filter(|c: &char| c.is_ascii_digit()).collect();
                                            let parsed = filtered.parse::<u32>().unwrap_or(0).min(max);
                                            let new_value = parsed as u16;
                                            if new_value != current {
                                                write_back(&mut self.mouse_keys_settings, new_value);
                                                self.write_mouse_keys_setting(qsid, new_value);
                                            }
                                            text = filtered;
                                        }
                                        ui.ctx().data_mut(|d| d.insert_temp(edit_id, text));
                                    }
                                });

                            ui.add_space(10.0);
                            ui.label(
                                RichText::new("Changes are written to the keyboard immediately.")
                                    .size(11.0)
                                    .color(app_muted_text(dark)),
                            );
                        },
                    );
                });
            });

        self.focus_modal_window(&shown);
        self.mouse_keys_window_open = open;
    }

    fn show_key_override_window(&mut self, ctx: &egui::Context) {
        if !self.keycode_picker.open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.key_override_window_open = false;
            return;
        }

        let dark = ctx.style().visuals.dark_mode;
        let mut open = self.key_override_window_open;
        let shown = egui::Window::new("Key Overrides")
            .order(egui::Order::Foreground)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(448.0, 468.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .frame(
                crate::ui_style::modal_window_frame(ctx.style().as_ref(), dark)
            )
            .show(ctx, |ui| {
                if self.key_override_entries.is_empty() {
                    ui.label("Key Overrides are not supported by this keyboard.");
                    return;
                }

                    if self.selected_key_override >= self.key_override_entries.len() {
                        self.selected_key_override = 0;
                    }
                    self.key_override_names
                        .resize(self.key_override_entries.len(), String::new());
                    self.key_override_visible_count = self.key_override_visible_count
                        .max(1)
                        .min(self.key_override_entries.len().max(1));
                    self.selected_key_override = self.selected_key_override.min(self.key_override_visible_count.saturating_sub(1));

                    ui.horizontal_wrapped(|ui| {
                        for idx in 0..self.key_override_visible_count {
                            let active = idx == self.selected_key_override;
                            let label = key_override_display_name(&self.key_override_names, idx);
                            let resp = ui.add(
                                egui::Button::new(RichText::new(label).size(11.0))
                                    .min_size(Vec2::new(52.0, 28.0))
                                    .fill(if active { app_hover_fill(dark) } else { app_surface_fill(dark) })
                                    .stroke(egui::Stroke::new(1.0, app_border_color(dark))),
                            ).on_hover_cursor(egui::CursorIcon::PointingHand);
                            if resp.clicked() {
                                self.selected_key_override = idx;
                            }
                        }

                        if self.key_override_visible_count < self.key_override_entries.len() {
                            let add_resp = ui.add(
                                egui::Button::new(RichText::new("+").size(14.0))
                                    .min_size(Vec2::new(28.0, 28.0))
                                    .fill(app_surface_fill(dark))
                                    .stroke(egui::Stroke::new(1.0, app_border_color(dark))),
                            ).on_hover_cursor(egui::CursorIcon::PointingHand);
                            add_resp.clone().on_hover_text("Add Key Override");
                            if add_resp.clicked() {
                                self.key_override_visible_count += 1;
                                self.selected_key_override = self.key_override_visible_count.saturating_sub(1);
                            }
                        }
                    });

                    ui.add_space(6.0);
                    let idx = self.selected_key_override;
                    let current = self.key_override_entries[idx].clone();
                    let mut edited = current.clone();
                    let content_width = 360.0_f32;
                    let field_width = 180.0_f32;
                    let name_field_width = 118.0_f32;
                    let action_button_size = crate::ui_style::modal_action_button_size();
                    let combo_outline_stroke = crate::ui_style::modal_outline_stroke(ui.visuals().dark_mode);

                    ui.vertical_centered(|ui| {
                        egui::ScrollArea::vertical()
                            .max_height(344.0)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(content_width, 0.0),
                                    egui::Layout::top_down(egui::Align::Min),
                                    |ui| {
                                        if let Some(name) = self.key_override_names.get_mut(idx) {
                                            let resp = ui.add(
                                                egui::TextEdit::singleline(name)
                                                    .desired_width(name_field_width)
                                                    .hint_text("Name")
                                                    .char_limit(9),
                                            );
                                            if resp.changed() {
                                                save_key_override_names(&self.key_override_names, &self.current_device_name);
                                            }
                                            resp.clone().on_hover_text("Stored locally in Entropy.");
                                            if resp.hovered() {
                                                ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
                                            }
                                        }

                                        let custom = self.layout.as_ref().map(|l| l.custom_keycodes.as_slice()).unwrap_or(&[]);
                                        let trigger_label = if edited.trigger == 0 {
                                            "Pick trigger".to_string()
                                        } else {
                                            keycode_label_with_macro_names(
                                                edited.trigger,
                                                custom,
                                                &self.layer_names,
                                                &self.keycode_picker.macro_names,
                                                &self.keycode_picker.tap_dance_names,
                                            ).replace('\n', " ")
                                        };
                                        let replacement_label = if edited.replacement == 0 {
                                            "Pick replacement".to_string()
                                        } else {
                                            keycode_label_with_macro_names(
                                                edited.replacement,
                                                custom,
                                                &self.layer_names,
                                                &self.keycode_picker.macro_names,
                                                &self.keycode_picker.tap_dance_names,
                                            ).replace('\n', " ")
                                        };
                                        let trigger_tip = keycode_tooltip_with_macro_names(
                                            edited.trigger,
                                            custom,
                                            &self.layer_names,
                                            &self.keycode_picker.macro_names,
                                            &self.keycode_picker.tap_dance_names,
                                        );
                                        let replacement_tip = keycode_tooltip_with_macro_names(
                                            edited.replacement,
                                            custom,
                                            &self.layer_names,
                                            &self.keycode_picker.macro_names,
                                            &self.keycode_picker.tap_dance_names,
                                        );

                                        ui.add_space(6.0);
                                        ui.label(RichText::new("Trigger").size(12.0).strong());
                                        ui.add_space(4.0);
                                        ui.horizontal_centered(|ui| {
                                            let trigger_resp = ui.add(
                                                egui::Button::new(RichText::new(trigger_label).size(12.0))
                                                    .min_size(Vec2::new(field_width, 34.0)),
                                            ).on_hover_cursor(egui::CursorIcon::PointingHand);
                                            if trigger_resp.clicked() {
                                                self.open_key_override_picker(KeyOverridePickField::Trigger);
                                            }
                                            trigger_resp.on_hover_text(trigger_tip);
                                        });

                                        ui.add_space(6.0);
                                        let suppressed_resp = egui::CollapsingHeader::new(
                                            RichText::new("Suppressed mods").size(11.0).color(app_muted_text(dark))
                                        )
                                        .default_open(false)
                                        .id_salt(format!("ko_suppressed_mods_{}", idx))
                                        .show(ui, |ui| {
                                            Self::draw_key_override_mod_mask(ui, &mut edited.suppressed_mods, "ko_suppressed_mods");
                                        });
                                        if suppressed_resp.header_response.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }

                                        ui.add_space(4.0);
                                        let trigger_mods_resp = egui::CollapsingHeader::new(
                                            RichText::new("Trigger mods").size(11.0).color(app_muted_text(dark))
                                        )
                                        .default_open(false)
                                        .id_salt(format!("ko_trigger_mods_{}", idx))
                                        .show(ui, |ui| {
                                            Self::draw_key_override_mod_mask(ui, &mut edited.trigger_mods, "ko_trigger_mods");
                                        });
                                        if trigger_mods_resp.header_response.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }

                                        ui.add_space(4.0);
                                        let negative_mods_resp = egui::CollapsingHeader::new(
                                            RichText::new("Negative mods").size(11.0).color(app_muted_text(dark))
                                        )
                                        .default_open(false)
                                        .id_salt(format!("ko_negative_mods_{}", idx))
                                        .show(ui, |ui| {
                                            Self::draw_key_override_mod_mask(ui, &mut edited.negative_mod_mask, "ko_negative_mods");
                                        });
                                        if negative_mods_resp.header_response.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }

                                        ui.add_space(6.0);
                                        ui.label(RichText::new("Replacement").size(12.0).strong());
                                        ui.add_space(4.0);
                                        ui.horizontal_centered(|ui| {
                                            let replacement_resp = ui.add(
                                                egui::Button::new(RichText::new(replacement_label).size(12.0))
                                                    .min_size(Vec2::new(field_width, 34.0)),
                                            ).on_hover_cursor(egui::CursorIcon::PointingHand);
                                            if replacement_resp.clicked() {
                                                self.open_key_override_picker(KeyOverridePickField::Replacement);
                                            }
                                            replacement_resp.on_hover_text(replacement_tip);
                                        });

                                        ui.add_space(6.0);
                                        let layers_resp = egui::CollapsingHeader::new(
                                            RichText::new("Enable on layers").size(11.0).color(app_muted_text(dark))
                                        )
                                        .default_open(false)
                                        .id_salt(format!("ko_layers_{}", idx))
                                        .show(ui, |ui| {
                                            Self::draw_key_override_layers(ui, &mut edited.layers);
                                        });
                                        if layers_resp.header_response.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }

                                        ui.add_space(6.0);
                                        ui.label(RichText::new("How this override behaves").size(11.0).color(app_muted_text(dark)));
                                        ui.add_space(2.0);
                                        ui.checkbox(&mut edited.options.activation_trigger_down, "Activate as soon as the trigger key is pressed");
                                        ui.checkbox(&mut edited.options.activation_required_mod_down, "Activate as soon as a required modifier is pressed");
                                        ui.checkbox(&mut edited.options.activation_negative_mod_up, "Activate when a blocked modifier is released");
                                        ui.checkbox(&mut edited.options.one_mod, "Any one trigger modifier is enough");
                                        ui.checkbox(&mut edited.options.no_reregister_trigger, "Do not send the original trigger key after the override ends");
                                        ui.checkbox(&mut edited.options.no_unregister_on_other_key_down, "Keep the override active even if another key is pressed");
                                    },
                                );
                            });
                    });

                    ui.add_space(0.0);
                    ui.horizontal_centered(|ui| {
                        let clear_btn = egui::Button::new(RichText::new("Clear").size(13.0))
                            .min_size(action_button_size)
                            .frame(true)
                            .stroke(combo_outline_stroke);
                        let clear_enabled = Self::key_override_entry_exists(&self.key_override_entries[idx])
                            || self.key_override_names.get(idx).map(|s| !s.trim().is_empty()).unwrap_or(false);
                        let clear_resp = ui.add_enabled(clear_enabled, clear_btn);
                        if clear_resp.hovered() && clear_enabled {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if clear_resp.clicked() {
                            self.push_key_override_undo();
                            self.key_override_entries[idx] = KeyOverrideEntry::default();
                            if let Some(name) = self.key_override_names.get_mut(idx) {
                                name.clear();
                            }
                            save_key_override_names(&self.key_override_names, &self.current_device_name);
                            self.write_key_override(idx);
                        }

                        let delete_btn = egui::Button::new(RichText::new("Delete").size(13.0))
                            .min_size(action_button_size)
                            .frame(true)
                            .stroke(combo_outline_stroke);
                        let delete_enabled = self.key_override_visible_count > 1;
                        let delete_resp = ui.add_enabled(delete_enabled, delete_btn);
                        if delete_resp.hovered() && delete_enabled {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if delete_resp.clicked() {
                            self.push_key_override_undo();
                            for move_idx in idx..self.key_override_visible_count.saturating_sub(1) {
                                self.key_override_entries[move_idx] = self.key_override_entries[move_idx + 1].clone();
                                self.key_override_names[move_idx] = self.key_override_names.get(move_idx + 1).cloned().unwrap_or_default();
                            }
                            let last_idx = self.key_override_visible_count.saturating_sub(1);
                            if last_idx < self.key_override_entries.len() {
                                self.key_override_entries[last_idx] = KeyOverrideEntry::default();
                            }
                            if last_idx < self.key_override_names.len() {
                                self.key_override_names[last_idx].clear();
                            }
                            self.key_override_visible_count = self.key_override_visible_count.saturating_sub(1).max(1);
                            self.selected_key_override = idx.min(self.key_override_visible_count.saturating_sub(1));
                            save_key_override_names(&self.key_override_names, &self.current_device_name);
                            self.write_all_key_overrides();
                        }

                        let undo_btn = egui::Button::new(RichText::new("Undo").size(13.0))
                            .min_size(action_button_size)
                            .frame(true)
                            .stroke(combo_outline_stroke);
                        let undo_enabled = !self.key_override_undo_stack.is_empty();
                        let undo_resp = ui.add_enabled(undo_enabled, undo_btn);
                        if undo_resp.hovered() && undo_enabled {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if undo_resp.clicked() {
                            if let Some((entries, names, selected, visible_count)) = self.key_override_undo_stack.pop() {
                                self.key_override_entries = entries;
                                self.key_override_names = names;
                                self.key_override_visible_count = visible_count.clamp(1, self.key_override_entries.len().max(1));
                                self.selected_key_override = selected.min(self.key_override_visible_count.saturating_sub(1));
                                save_key_override_names(&self.key_override_names, &self.current_device_name);
                                self.write_all_key_overrides();
                            }
                        }
                    });

                Self::normalize_key_override_entry(&mut edited);
                if edited != current {
                    self.push_key_override_undo();
                    self.key_override_entries[idx] = edited;
                    self.write_key_override(idx);
                }
            });
        self.focus_modal_window(&shown);
        self.key_override_window_open = open;
    }

    fn show_combo_window(&mut self, ctx: &egui::Context) {
        if !self.keycode_picker.open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.combo_capture_open {
                self.cancel_combo_capture();
            } else {
                self.combo_window_open = false;
                self.combo_capture_open = false;
                self.combo_capture_keys.clear();
                return;
            }
        }

        if self.combo_capture_open {
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.apply_combo_capture();
            } else {
                for event in ctx.input(|i| i.events.clone()) {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        if matches!(key, egui::Key::Enter | egui::Key::Escape) {
                            continue;
                        }
                        if let Some(kc) = egui_key_to_qmk(key, modifiers) {
                            if !self.combo_capture_keys.contains(&kc)
                                && self.combo_capture_keys.len() < 4
                            {
                                self.combo_capture_keys.push(kc);
                            }
                        }
                    }
                }
            }
        }

        let mut open = self.combo_window_open;
        let shown = egui::Window::new("Combo")
            .order(egui::Order::Foreground)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(360.0)
            .min_width(360.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                egui::Frame::window(ctx.style().as_ref())
                    .fill(app_window_fill(ctx.style().visuals.dark_mode))
                    .stroke(egui::Stroke::NONE)
                    .inner_margin(egui::Margin::same(10)),
            )
            .show(ctx, |ui| {
                ui.style_mut().visuals.button_frame = true;
                if ui.visuals().dark_mode {
                    ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(48, 48, 58);
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                        Color32::from_rgb(48, 48, 58);
                    ui.style_mut().visuals.widgets.inactive.bg_stroke =
                        Stroke::new(1.0, Color32::from_gray(110));
                    ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(68, 68, 88);
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                        Color32::from_rgb(68, 68, 88);
                    ui.style_mut().visuals.widgets.hovered.bg_stroke =
                        Stroke::new(1.0, Color32::from_rgb(130, 130, 160));
                    ui.style_mut().visuals.widgets.active.bg_fill = Color32::from_rgb(78, 78, 102);
                    ui.style_mut().visuals.widgets.active.weak_bg_fill =
                        Color32::from_rgb(78, 78, 102);
                    ui.style_mut().visuals.widgets.active.bg_stroke =
                        Stroke::new(1.0, Color32::from_rgb(150, 150, 184));
                } else {
                    ui.style_mut().visuals.widgets.inactive.bg_fill =
                        Color32::from_rgb(255, 255, 255);
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                        Color32::from_rgb(255, 255, 255);
                    ui.style_mut().visuals.widgets.inactive.bg_stroke =
                        Stroke::new(1.0, Color32::from_rgb(222, 222, 228));
                    ui.style_mut().visuals.widgets.hovered.bg_fill =
                        Color32::from_rgb(234, 232, 242);
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                        Color32::from_rgb(234, 232, 242);
                    ui.style_mut().visuals.widgets.hovered.bg_stroke =
                        Stroke::new(1.0, Color32::from_rgb(210, 206, 223));
                    ui.style_mut().visuals.widgets.active.bg_fill =
                        Color32::from_rgb(228, 225, 238);
                    ui.style_mut().visuals.widgets.active.weak_bg_fill =
                        Color32::from_rgb(228, 225, 238);
                    ui.style_mut().visuals.widgets.active.bg_stroke =
                        Stroke::new(1.0, Color32::from_rgb(202, 198, 216));
                }

                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Press multiple keys together to send a separate keycode.")
                            .size(12.0)
                            .color(Color32::from_gray(150)),
                    );
                });

                if self.firmware != FirmwareProtocol::Vial {
                    ui.label(
                        RichText::new("Dynamic combos are not supported for this firmware.")
                            .color(Color32::from_gray(140)),
                    );
                    return;
                }

                if self.combo_entries.is_empty() {
                    ui.label(
                        RichText::new("This keyboard does not report any dynamic combo slots.")
                            .color(Color32::from_gray(140)),
                    );
                    return;
                }

                self.selected_combo = self
                    .selected_combo
                    .min(self.combo_entries.len().saturating_sub(1));
                self.combo_names
                    .resize(self.combo_entries.len(), String::new());

                let combo_undo_snapshot = (
                    self.combo_entries.clone(),
                    self.combo_names.clone(),
                    self.combo_term,
                    self.selected_combo,
                    self.combo_visible_count,
                );

                self.combo_visible_count = self
                    .combo_visible_count
                    .clamp(1, self.combo_entries.len().max(1));
                self.selected_combo = self
                    .selected_combo
                    .min(self.combo_visible_count.saturating_sub(1));

                let combo_outline_stroke = if ui.visuals().dark_mode {
                    Stroke::new(1.0, Color32::from_gray(110))
                } else {
                    Stroke::new(1.0, Color32::from_gray(175))
                };

                ui.horizontal_wrapped(|ui| {
                    for idx in 0..self.combo_visible_count {
                        let tab_stroke = if idx == self.selected_combo {
                            Stroke::new(1.5, Color32::from_rgb(91, 104, 223))
                        } else {
                            combo_outline_stroke
                        };
                        let tab_label = match self.combo_names.get(idx) {
                            Some(name) if !name.trim().is_empty() => name.trim().to_string(),
                            _ => format!("C{}", idx),
                        };
                        let tab_text = if idx == self.selected_combo {
                            RichText::new(tab_label)
                                .size(12.5)
                                .color(ui.visuals().widgets.inactive.fg_stroke.color)
                        } else {
                            RichText::new(tab_label).size(12.5)
                        };
                        let tab = egui::Button::new(tab_text)
                            .frame(true)
                            .stroke(tab_stroke)
                            .min_size(Vec2::new(48.0, 28.0));
                        let resp = ui.add(tab);
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if resp.clicked() {
                            self.selected_combo = idx;
                        }
                    }

                    let add_combo_btn = egui::Button::new(RichText::new("+").size(16.0))
                        .frame(true)
                        .stroke(combo_outline_stroke);
                    let resp = ui.add_enabled(
                        self.combo_visible_count < self.combo_entries.len(),
                        add_combo_btn,
                    );
                    if resp.hovered() && self.combo_visible_count < self.combo_entries.len() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if resp.clicked() {
                        let next_idx = self
                            .combo_visible_count
                            .min(self.combo_entries.len().saturating_sub(1));
                        self.combo_visible_count =
                            (self.combo_visible_count + 1).min(self.combo_entries.len());
                        self.selected_combo = next_idx;
                    }
                });

                ui.add_space(12.0);
                let combo_idx = self.selected_combo;
                let content_width = 340.0_f32;
                let compact_field_width = ((content_width - 110.0) * 0.5).round();
                let name_field_width = ((content_width * 0.66) * 0.5).round();
                let action_button_size = crate::ui_style::modal_action_button_size();

                ui.vertical_centered(|ui| {
                    ui.allocate_ui_with_layout(
                        Vec2::new(content_width, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            let mut combo_name_changed = false;
                            if let Some(name) = self.combo_names.get_mut(combo_idx) {
                                let resp = ui.add(
                                    egui::TextEdit::singleline(name)
                                        .desired_width(name_field_width)
                                        .hint_text("Name")
                                        .char_limit(9),
                                );
                                combo_name_changed = resp.changed();
                                resp.clone().on_hover_text("Stored locally in Entropy.");
                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
                                }
                            }
                            if combo_name_changed {
                                self.combo_undo_stack.push(combo_undo_snapshot.clone());
                                self.combo_names_dirty = true;
                            }

                            let output_label = if self.combo_entries[combo_idx].output == 0 {
                                "Pick output".to_string()
                            } else {
                                keycode_label_with_macro_names(
                                    self.combo_entries[combo_idx].output,
                                    self.layout
                                        .as_ref()
                                        .map(|l| l.custom_keycodes.as_slice())
                                        .unwrap_or(&[]),
                                    &self.layer_names,
                                    &self.keycode_picker.macro_names,
                                    &self.keycode_picker.tap_dance_names,
                                )
                                .replace('\n', " ")
                            };

                            ui.add_space(12.0);
                            ui.label(RichText::new("Input keys").size(13.0).strong());
                            ui.add_space(6.0);
                            let input_summary = {
                                let keys: Vec<String> = if self.combo_capture_open {
                                    self.combo_capture_keys
                                        .iter()
                                        .copied()
                                        .map(|kc| {
                                            keycode_label_with_macro_names(
                                                kc,
                                                self.layout
                                                    .as_ref()
                                                    .map(|l| l.custom_keycodes.as_slice())
                                                    .unwrap_or(&[]),
                                                &self.layer_names,
                                                &self.keycode_picker.macro_names,
                                                &self.keycode_picker.tap_dance_names,
                                            )
                                            .replace('\n', " ")
                                        })
                                        .collect()
                                } else {
                                    self.combo_entries[combo_idx]
                                        .keys
                                        .iter()
                                        .copied()
                                        .filter(|&kc| kc != 0)
                                        .map(|kc| {
                                            keycode_label_with_macro_names(
                                                kc,
                                                self.layout
                                                    .as_ref()
                                                    .map(|l| l.custom_keycodes.as_slice())
                                                    .unwrap_or(&[]),
                                                &self.layer_names,
                                                &self.keycode_picker.macro_names,
                                                &self.keycode_picker.tap_dance_names,
                                            )
                                            .replace('\n', " ")
                                        })
                                        .collect()
                                };
                                if keys.is_empty() {
                                    if self.combo_capture_open {
                                        "Press 2-4 keys".to_string()
                                    } else {
                                        "Record 2-4 keys".to_string()
                                    }
                                } else {
                                    keys.join(" + ")
                                }
                            };
                            let field_resp = ui
                                .horizontal_centered(|ui| {
                                    let field_btn =
                                        egui::Button::new(RichText::new(input_summary).size(13.0))
                                            .frame(true)
                                            .stroke(combo_outline_stroke);
                                    ui.add_sized(Vec2::new(compact_field_width, 32.0), field_btn)
                                })
                                .inner;
                            if field_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if field_resp.clicked() {
                                self.combo_capture_keys.clear();
                                self.combo_capture_open = true;
                            }
                            if self.combo_capture_open {
                                let clicked_outside_input = ctx.input(|i| {
                                    i.pointer.any_pressed()
                                        && i.pointer
                                            .interact_pos()
                                            .map(|pos| !field_resp.rect.contains(pos))
                                            .unwrap_or(false)
                                });
                                if clicked_outside_input {
                                    self.apply_combo_capture();
                                }
                            }

                            ui.add_space(10.0);
                            ui.label(RichText::new("Output key").size(13.0).strong());
                            ui.add_space(6.0);
                            let resp = ui
                                .horizontal_centered(|ui| {
                                    let btn =
                                        egui::Button::new(RichText::new(&output_label).size(13.0))
                                            .frame(true)
                                            .stroke(combo_outline_stroke);
                                    ui.add_sized(Vec2::new(compact_field_width, 32.0), btn)
                                })
                                .inner;
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if resp.clicked() {
                                self.combo_pick_target = Some((combo_idx, ComboPickField::Output));
                                self.combo_reopen_after_pick = true;
                                self.combo_window_open = false;
                                self.keycode_picker.result = None;
                                self.keycode_picker.selected_tab = KeycodeTab::Basic;
                                self.keycode_picker.open = true;
                            }

                            if let Some(current_combo_term) = self.combo_term {
                                ui.add_space(14.0);
                                ui.separator();
                                ui.add_space(10.0);
                                ui.label(
                                    RichText::new("Time out period for combos")
                                        .size(13.0)
                                        .strong(),
                                );
                                ui.add_space(4.0);
                                let mut combo_term_text = current_combo_term.to_string();
                                ui.horizontal_centered(|ui| {
                                    let resp = ui.add(
                                        egui::TextEdit::singleline(&mut combo_term_text)
                                            .desired_width(45.0)
                                            .hint_text("ms"),
                                    );
                                    if resp.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
                                    }
                                    ui.label("ms");
                                    if resp.changed() {
                                        let filtered: String = combo_term_text
                                            .chars()
                                            .filter(|c| c.is_ascii_digit())
                                            .collect();
                                        if let Ok(parsed) = filtered.parse::<u16>() {
                                            self.combo_undo_stack.push(combo_undo_snapshot.clone());
                                            self.combo_term = Some(parsed.max(1));
                                            self.combo_term_dirty = true;
                                        }
                                    }
                                });
                            }
                            ui.add_space(12.0);
                            ui.horizontal_centered(|ui| {
                                let clear_btn =
                                    egui::Button::new(RichText::new("Clear combo").size(13.0))
                                        .min_size(action_button_size)
                                        .frame(true)
                                        .stroke(combo_outline_stroke);
                                let clear_enabled = combo_idx < self.combo_entries.len()
                                    && (self.combo_entries[combo_idx].keys.iter().any(|&k| k != 0)
                                        || self.combo_entries[combo_idx].output != 0
                                        || self
                                            .combo_names
                                            .get(combo_idx)
                                            .map(|s| !s.trim().is_empty())
                                            .unwrap_or(false));
                                let clear_resp = ui.add_enabled(clear_enabled, clear_btn);
                                if clear_resp.hovered() && clear_enabled {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if clear_resp.clicked() {
                                    self.push_combo_undo();
                                    self.combo_entries[combo_idx] = ComboEntry::default();
                                    if let Some(name) = self.combo_names.get_mut(combo_idx) {
                                        name.clear();
                                    }
                                    self.combo_dirty = true;
                                    self.combo_names_dirty = true;
                                }

                                let delete_btn =
                                    egui::Button::new(RichText::new("Delete combo").size(13.0))
                                        .min_size(action_button_size)
                                        .frame(true)
                                        .stroke(combo_outline_stroke);
                                let delete_resp = ui.add_enabled(
                                    combo_idx > 0 && self.combo_visible_count > 1,
                                    delete_btn,
                                );
                                if delete_resp.hovered()
                                    && combo_idx > 0
                                    && self.combo_visible_count > 1
                                {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if delete_resp.clicked() {
                                    self.push_combo_undo();
                                    for idx in combo_idx..self.combo_visible_count.saturating_sub(1)
                                    {
                                        self.combo_entries[idx] =
                                            self.combo_entries[idx + 1].clone();
                                        self.combo_names[idx] = self
                                            .combo_names
                                            .get(idx + 1)
                                            .cloned()
                                            .unwrap_or_default();
                                    }
                                    let last_idx = self.combo_visible_count.saturating_sub(1);
                                    if last_idx < self.combo_entries.len() {
                                        self.combo_entries[last_idx] = ComboEntry::default();
                                    }
                                    if last_idx < self.combo_names.len() {
                                        self.combo_names[last_idx].clear();
                                    }
                                    self.combo_visible_count =
                                        self.combo_visible_count.saturating_sub(1).max(1);
                                    self.selected_combo =
                                        combo_idx.min(self.combo_visible_count.saturating_sub(1));
                                    self.combo_dirty = true;
                                    self.combo_names_dirty = true;
                                }

                                let undo_btn = egui::Button::new(RichText::new("Undo").size(13.0))
                                    .min_size(action_button_size)
                                    .frame(true)
                                    .stroke(combo_outline_stroke);
                                let undo_resp =
                                    ui.add_enabled(!self.combo_undo_stack.is_empty(), undo_btn);
                                if undo_resp.hovered() && !self.combo_undo_stack.is_empty() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if undo_resp.clicked() {
                                    if let Some((entries, names, term, selected, visible_count)) =
                                        self.combo_undo_stack.pop()
                                    {
                                        self.combo_entries = entries;
                                        self.combo_names = names;
                                        self.combo_term = term;
                                        self.combo_visible_count =
                                            visible_count.clamp(1, self.combo_entries.len().max(1));
                                        self.selected_combo = selected
                                            .min(self.combo_visible_count.saturating_sub(1));
                                        self.combo_dirty = true;
                                        self.combo_names_dirty = true;
                                        self.combo_term_dirty = true;
                                    }
                                }
                            });
                        },
                    );
                });
            });
        self.focus_modal_window(&shown);
        self.combo_window_open = open;
        if !self.combo_window_open {
            self.combo_capture_open = false;
            self.combo_capture_keys.clear();
        }
    }

    fn draw_layout(&mut self, ui: &mut egui::Ui, layout: &KeyboardLayout, ctx: &egui::Context) {
        let base_unit = 54.0_f32 * 1.15; // +15%
        let padding = 4.0_f32;

        let avail = ui.available_size();

        // Calculate layout bounding box (min AND max to handle rx offsets)
        let mut min_x: f32 = f32::MAX;
        let mut min_y: f32 = f32::MAX;
        let mut max_x: f32 = f32::MIN;
        let mut max_y: f32 = f32::MIN;
        for key in &layout.keys {
            min_x = min_x.min(key.x);
            min_y = min_y.min(key.y);
            max_x = max_x.max(key.x + key.w);
            max_y = max_y.max(key.y + key.h);
        }
        for encoder in &layout.encoders {
            min_x = min_x.min(encoder.x);
            min_y = min_y.min(encoder.y);
            max_x = max_x.max(encoder.x + encoder.w);
            max_y = max_y.max(encoder.y + encoder.h);
        }
        if min_x == f32::MAX {
            min_x = 0.0;
            min_y = 0.0;
            max_x = 1.0;
            max_y = 1.0;
        }

        let span_x = max_x - min_x;
        let span_y = max_y - min_y;

        // Scale unit to fit available space with some margin
        let margin = 40.0_f32;
        let scale_x = (avail.x - margin) / (span_x * base_unit).max(1.0);
        let scale_y = (avail.y - margin) / (span_y * base_unit).max(1.0);
        let scale = scale_x.min(scale_y).min(1.0);
        let unit = base_unit * scale;

        let layout_w = span_x * unit;
        let layout_h = span_y * unit;
        // Reserve space at top for main tabs + layer switcher
        let main_tabs_h = 32.0_f32;
        let main_tabs_gap = 4.0_f32;
        let layer_bar_h = 68.0_f32;
        let top_reserved_h = main_tabs_h + main_tabs_gap + layer_bar_h;
        let bottom_reserved_h = 76.0_f32;
        let top_base_y = ui.min_rect().top() + 6.0;
        let offset_x = (avail.x - layout_w) / 2.0 + ui.min_rect().left() - min_x * unit;
        let content_top = ui.min_rect().top() + top_reserved_h;
        let content_bottom = ui.max_rect().bottom() - bottom_reserved_h;
        let offset_y = ((content_top + content_bottom) - layout_h) / 2.0 - min_y * unit;

        // ── Main menu tabs ────────────────────────────────────────────────
        {
            let center_x = ui.min_rect().center().x;
            let tabs_y = top_base_y;
            let tab_size = Vec2::new(112.0, 28.0);
            let tab_gap = 8.0;
            let total_w = tab_size.x * 3.0 + tab_gap * 2.0;
            let start_x = center_x - total_w / 2.0;
            let tabs = [
                (MainMenuTab::Keyboard, "Device"),
                (MainMenuTab::Advanced, "Advanced"),
                (MainMenuTab::Settings, "Settings"),
            ];
            let mut device_tab_rect = None;
            let mut device_tab_hovered = false;
            let mut advanced_tab_rect = None;
            let mut advanced_tab_hovered = false;
            let mut settings_tab_rect = None;
            let mut settings_tab_hovered = false;

            for (idx, (tab, label)) in tabs.iter().enumerate() {
                let slot_rect = egui::Rect::from_min_size(
                    egui::pos2(start_x + idx as f32 * (tab_size.x + tab_gap), tabs_y),
                    tab_size,
                );
                let text_rect = {
                    let text_w = ui.fonts(|f| {
                        f.layout_no_wrap(
                            (*label).to_owned(),
                            FontId::proportional(15.0),
                            ui.visuals().widgets.inactive.fg_stroke.color,
                        )
                        .size()
                        .x
                    });
                    egui::Rect::from_center_size(
                        slot_rect.center(),
                        Vec2::new(text_w + 20.0, tab_size.y),
                    )
                };
                let resp = ui.allocate_rect(text_rect, Sense::CLICK);
                if matches!(tab, MainMenuTab::Keyboard) {
                    device_tab_rect = Some(text_rect);
                    device_tab_hovered = resp.hovered();
                }
                if matches!(tab, MainMenuTab::Advanced) {
                    advanced_tab_rect = Some(text_rect);
                    advanced_tab_hovered = resp.hovered();
                }
                if matches!(tab, MainMenuTab::Settings) {
                    settings_tab_rect = Some(text_rect);
                    settings_tab_hovered = resp.hovered();
                }
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if resp.clicked() {
                    match tab {
                        MainMenuTab::Keyboard => {
                            self.main_menu_tab = MainMenuTab::Keyboard;
                        }
                        MainMenuTab::Advanced => {
                            self.main_menu_tab = MainMenuTab::Keyboard;
                        }
                        MainMenuTab::Settings => {
                            if self.main_menu_tab != MainMenuTab::Settings {
                                self.reset_matrix_tester_state();
                            }
                            self.matrix_tester_unlock_prompted = false;
                            self.main_menu_tab = MainMenuTab::Settings;
                        }
                    }
                }

                let is_active = !matches!(tab, MainMenuTab::Advanced) && self.main_menu_tab == *tab;
                let text_color = if is_active {
                    ui.visuals().widgets.inactive.fg_stroke.color
                } else if resp.hovered() {
                    if ui.visuals().dark_mode {
                        Color32::from_gray(135)
                    } else {
                        Color32::from_gray(120)
                    }
                } else if ui.visuals().dark_mode {
                    Color32::from_gray(90)
                } else {
                    Color32::from_gray(150)
                };

                ui.painter().text(
                    slot_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    *label,
                    FontId::proportional(15.0),
                    text_color,
                );
            }

            let divider_color = if ui.visuals().dark_mode {
                Color32::from_gray(105)
            } else {
                Color32::from_gray(170)
            };
            let divider_top = tabs_y + 4.0;
            let divider_bottom = tabs_y + tab_size.y - 4.0;
            for sep_idx in 1..3 {
                let x = start_x + sep_idx as f32 * tab_size.x + (sep_idx as f32 - 0.5) * tab_gap;
                ui.painter().line_segment(
                    [egui::pos2(x, divider_top), egui::pos2(x, divider_bottom)],
                    egui::Stroke::new(1.5, divider_color),
                );
            }

            if let Some(device_rect) = device_tab_rect {
                let dropdown_id = ui.make_persistent_id("device_dropdown_open");
                let was_open = ui
                    .ctx()
                    .data(|d| d.get_temp::<bool>(dropdown_id))
                    .unwrap_or(false);
                let has_lock_button = self.firmware == FirmwareProtocol::Vial
                    && self.layout.is_some()
                    && !self.vial_unlock_polling
                    && !self.unlock_open;
                let is_unlocked = if has_lock_button {
                    self.hid_device
                        .as_ref()
                        .and_then(|hid| hid.get_unlock_status().ok())
                        .map(|(unlocked, _keys)| unlocked)
                        .unwrap_or(false)
                } else {
                    false
                };
                let device_count = self.device_manager.devices().len();
                let device_rows = device_count.max(1) as f32;
                let devices_h = 8.0 + device_rows * 26.0;
                let lock_h = if has_lock_button { 32.0 } else { 0.0 };
                let dropdown_size = Vec2::new(240.0, devices_h + lock_h + 8.0);
                let dropdown_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        device_rect.center().x - dropdown_size.x / 2.0,
                        device_rect.bottom() + 6.0,
                    ),
                    dropdown_size,
                );
                let hover_bridge_rect = device_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = !advanced_tab_hovered
                    && !settings_tab_hovered
                    && (device_tab_hovered || (was_open && pointer_over_bridge));

                if show_dropdown {
                    let area_id = ui.make_persistent_id("device_dropdown_area");
                    egui::Area::new(area_id)
                        .order(egui::Order::Foreground)
                        .fixed_pos(dropdown_rect.min)
                        .show(ctx, |ui| {
                            let dropdown_fill = if ui.visuals().dark_mode {
                                Color32::from_gray(32)
                            } else {
                                Color32::from_gray(248)
                            };
                            egui::Frame::new()
                                .fill(dropdown_fill)
                                .stroke(egui::Stroke::NONE)
                                .corner_radius(8.0)
                                .inner_margin(egui::Margin::same(8))
                                .show(ui, |ui| {
                                    ui.set_min_width(dropdown_size.x);

                                    let prev_selected = self.selected_device;
                                    if self.device_manager.devices().is_empty() {
                                        ui.add_sized(
                                            [dropdown_size.x - 16.0, 26.0],
                                            egui::Label::new("No devices found"),
                                        );
                                    } else {
                                        for (i, dev) in
                                            self.device_manager.devices().iter().enumerate()
                                        {
                                            let is_selected = self.selected_device == Some(i);
                                            let label = if is_selected {
                                                format!("✓ {}", dev.name)
                                            } else {
                                                dev.name.clone()
                                            };
                                            let resp = ui.add_sized(
                                                [dropdown_size.x - 16.0, 26.0],
                                                egui::Button::new(RichText::new(label).color(
                                                    if is_selected {
                                                        ui.visuals()
                                                            .widgets
                                                            .inactive
                                                            .fg_stroke
                                                            .color
                                                    } else if ui.visuals().dark_mode {
                                                        Color32::from_gray(170)
                                                    } else {
                                                        Color32::from_gray(90)
                                                    },
                                                ))
                                                .fill(Color32::TRANSPARENT)
                                                .stroke(egui::Stroke::NONE),
                                            );
                                            if resp.clicked() {
                                                self.selected_device = Some(i);
                                            }
                                        }
                                    }

                                    #[cfg(not(target_arch = "wasm32"))]
                                    if self.selected_device != prev_selected {
                                        if let Some(idx) = self.selected_device {
                                            self.start_connect(idx);
                                        }
                                    }

                                    if has_lock_button {
                                        ui.add_space(6.0);
                                        let lock_label = if is_unlocked {
                                            "🔓 Lock"
                                        } else {
                                            "🔒 Unlock"
                                        };
                                        let lock_text = if !is_unlocked {
                                            RichText::new(lock_label)
                                                .color(Color32::from_rgb(220, 120, 60))
                                        } else {
                                            RichText::new(lock_label)
                                        };
                                        if ui
                                            .add_sized(
                                                [dropdown_size.x - 16.0, 26.0],
                                                egui::Button::new(lock_text)
                                                    .fill(Color32::TRANSPARENT)
                                                    .stroke(egui::Stroke::NONE),
                                            )
                                            .clicked()
                                        {
                                            if is_unlocked {
                                                if let Some(hid) = &self.hid_device {
                                                    match hid.lock() {
                                                        Ok(()) => {
                                                            self.status_msg =
                                                                "Keyboard locked".into()
                                                        }
                                                        Err(e) => {
                                                            self.status_msg =
                                                                format!("Lock failed: {e}")
                                                        }
                                                    }
                                                }
                                            } else {
                                                self.unlock_open = true;
                                            }
                                        }
                                    }
                                });
                        });

                    ui.ctx().data_mut(|d| {
                        d.insert_temp(dropdown_id, device_tab_hovered || pointer_over_bridge)
                    });
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, false));
                }
            }

            if let Some(advanced_rect) = advanced_tab_rect {
                let dropdown_id = ui.make_persistent_id("advanced_dropdown_open");
                let was_open = ui
                    .ctx()
                    .data(|d| d.get_temp::<bool>(dropdown_id))
                    .unwrap_or(false);
                let dropdown_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        advanced_rect.center().x - 76.0,
                        advanced_rect.bottom() + 6.0,
                    ),
                    Vec2::new(152.0, 106.0),
                );
                let hover_bridge_rect = advanced_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = !device_tab_hovered
                    && !settings_tab_hovered
                    && (advanced_tab_hovered || (was_open && pointer_over_bridge));

                if show_dropdown {
                    let dark = ui.visuals().dark_mode;
                    let dropdown_fill = if dark {
                        Color32::from_gray(32)
                    } else {
                        Color32::from_gray(248)
                    };
                    let auto_shift_supported = self.auto_shift_timeout.is_some();
                    let (combo_hovered, auto_shift_hovered, key_override_hovered) = egui::Area::new(egui::Id::new("advanced_dropdown_area"))
                        .order(egui::Order::Foreground)
                        .fixed_pos(dropdown_rect.min)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::NONE
                                .fill(dropdown_fill)
                                .corner_radius(8.0)
                                .inner_margin(egui::Margin::symmetric(6, 4))
                                .show(ui, |ui| {
                                    ui.set_min_width(dropdown_rect.width() - 12.0);
                                    let combo_resp = ui.add_sized([dropdown_rect.width() - 12.0, 30.0], egui::Button::new("Combo").frame(false));
                                    let auto_shift_color = if auto_shift_supported { ui.visuals().widgets.inactive.fg_stroke.color } else { app_muted_text(dark) };
                                    let auto_shift_resp = ui.add_sized([dropdown_rect.width() - 12.0, 30.0], egui::Button::new(RichText::new("Auto Shift").color(auto_shift_color)).frame(false));
                                    let key_override_resp = ui.add_sized([dropdown_rect.width() - 12.0, 30.0], egui::Button::new("Key Overrides").frame(false));
                                    if combo_resp.hovered() || auto_shift_resp.hovered() || key_override_resp.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    if combo_resp.clicked() {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.modal_focus_pending = true;
                                        self.combo_window_open = true;
                                        if self.combo_visible_count == 0 { self.combo_visible_count = 1; }
                                    }
                                    if auto_shift_resp.clicked() && auto_shift_supported {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.modal_focus_pending = true;
                                        self.auto_shift_window_open = true;
                                    }
                                    if key_override_resp.clicked() {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.modal_focus_pending = true;
                                        self.key_override_window_open = true;
                                    }
                                    if !auto_shift_supported {
                                        let _ = auto_shift_resp.clone().on_hover_text("Auto Shift is not enabled in this keyboard firmware");
                                    }
                                    (combo_resp.hovered(), auto_shift_resp.hovered(), key_override_resp.hovered())
                                })
                                .inner
                        })
                        .inner;
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            dropdown_id,
                            advanced_tab_hovered
                                || combo_hovered
                                || auto_shift_hovered
                                || key_override_hovered
                                || pointer_over_bridge,
                        )
                    });
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, false));
                }
            }

            if let Some(settings_rect) = settings_tab_rect {
                let dropdown_id = ui.make_persistent_id("settings_dropdown_open");
                let was_open = ui
                    .ctx()
                    .data(|d| d.get_temp::<bool>(dropdown_id))
                    .unwrap_or(false);
                let dropdown_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        settings_rect.center().x - 76.0,
                        settings_rect.bottom() + 6.0,
                    ),
                    Vec2::new(152.0, 102.0),
                );
                let hover_bridge_rect = settings_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = !device_tab_hovered
                    && !advanced_tab_hovered
                    && (settings_tab_hovered || (was_open && pointer_over_bridge));

                if show_dropdown {
                    let dark = ui.visuals().dark_mode;
                    let dropdown_fill = if dark {
                        Color32::from_gray(32)
                    } else {
                        Color32::from_gray(248)
                    };
                    let rgb_available = self.rgb_settings.supported
                        || self
                            .layout
                            .as_ref()
                            .map(|l| l.supports_rgb)
                            .unwrap_or(false);
                    let (matrix_hovered, rgb_hovered, encoders_hovered) =
                        egui::Area::new(egui::Id::new("settings_dropdown_area"))
                            .order(egui::Order::Foreground)
                            .fixed_pos(dropdown_rect.min)
                            .show(ui.ctx(), |ui| {
                                egui::Frame::NONE
                                    .fill(dropdown_fill)
                                    .corner_radius(8.0)
                                    .inner_margin(egui::Margin::symmetric(6, 4))
                                    .show(ui, |ui| {
                                        ui.set_min_width(dropdown_rect.width() - 12.0);
                                        let matrix_resp = ui.add_sized(
                                            [dropdown_rect.width() - 12.0, 30.0],
                                            egui::Button::new("Matrix Tester").frame(false),
                                        );
                                        let rgb_color = if rgb_available {
                                            ui.visuals().widgets.inactive.fg_stroke.color
                                        } else {
                                            app_muted_text(dark)
                                        };
                                        let rgb_resp = ui.add_sized(
                                            [dropdown_rect.width() - 12.0, 30.0],
                                            egui::Button::new(RichText::new("RGB").color(rgb_color)).frame(false),
                                        );
                                        let encoders_resp = ui.add_sized(
                                            [dropdown_rect.width() - 12.0, 30.0],
                                            egui::Button::new("Encoders").frame(false),
                                        );
                                        if matrix_resp.hovered()
                                            || rgb_resp.hovered()
                                            || encoders_resp.hovered()
                                        {
                                            ui.ctx()
                                                .set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        if matrix_resp.clicked() {
                                            self.close_top_dropdowns(ui.ctx());
                                            self.settings_tab = SettingsTab::MatrixTester;
                                            if self.main_menu_tab != MainMenuTab::Settings {
                                                self.reset_matrix_tester_state();
                                            }
                                            self.matrix_tester_unlock_prompted = false;
                                            self.main_menu_tab = MainMenuTab::Settings;
                                        }
                                        if rgb_resp.clicked() && rgb_available {
                                            self.close_top_dropdowns(ui.ctx());
                                            self.modal_focus_pending = true;
                                            self.rgb_window_open = true;
                                        }
                                        if !rgb_available {
                                            let _ = rgb_resp.clone().on_hover_text(
                                                "RGB settings are not available on this firmware",
                                            );
                                        }
                                        if encoders_resp.clicked() {
                                            self.close_top_dropdowns(ui.ctx());
                                            self.modal_focus_pending = true;
                                            self.encoder_visibility_window_open = true;
                                        }
                                        (
                                            matrix_resp.hovered(),
                                            rgb_resp.hovered(),
                                            encoders_resp.hovered(),
                                        )
                                    })
                                    .inner
                            })
                            .inner;
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            dropdown_id,
                            settings_tab_hovered
                                || matrix_hovered
                                || rgb_hovered
                                || encoders_hovered
                                || pointer_over_bridge,
                        )
                    });
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, false));
                }
            }
            if self.main_menu_tab == MainMenuTab::Settings {
                self.draw_settings_screen(ui, layout, ctx, ui.min_rect().top() + top_reserved_h);
                return;
            }

            // ── Layer switcher ─────────────────────────────────────────────────
            {
                let layer_count = self.layer_count;
                let selected = self.selected_layer;
                // raw_name — чистое имя без префикса, хранится в layer_names
                let raw_name = self
                    .layer_names
                    .get(selected)
                    .cloned()
                    .unwrap_or_else(|| selected.to_string());
                let visible_raw_name: String = raw_name.chars().take(12).collect();
                // display_name — с префиксом для отображения
                let display_name = if !raw_name.is_empty() && raw_name != selected.to_string() {
                    format!("{}. {}", selected, visible_raw_name)
                } else {
                    visible_raw_name.clone()
                };
                let name = display_name;
                let center_x = ui.max_rect().center().x;
                let bar_y = top_base_y + main_tabs_h + 24.0;
                let any_top_dropdown_open = ui.memory(|m| {
                    m.data
                        .get_temp::<bool>(ui.make_persistent_id("device_dropdown_open"))
                        .unwrap_or(false)
                        || m.data
                            .get_temp::<bool>(ui.make_persistent_id("advanced_dropdown_open"))
                            .unwrap_or(false)
                        || m.data
                            .get_temp::<bool>(ui.make_persistent_id("settings_dropdown_open"))
                            .unwrap_or(false)
                });

                // ZMK layer management buttons (+ Add / - Remove)
                if !any_top_dropdown_open && self.firmware == FirmwareProtocol::Zmk {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let op_busy = self.zmk_op_rx.is_some();
                        let can_remove = !op_busy && layer_count > self.zmk_base_layer_count.max(1);
                        // Place +/− just to the right of the → arrow
                        let fixed_half = 85.0_f32;
                        let gap = 16.0_f32;
                        let right_arrow_x = center_x + fixed_half + gap + 24.0;
                        let btn_x = right_arrow_x + 36.0;
                        let mid_y = bar_y + layer_bar_h / 2.0;
                        let sym_font = FontId::proportional(14.0);
                        let active_color = if self.dark_mode {
                            Color32::from_gray(200)
                        } else {
                            Color32::from_gray(60)
                        };
                        let disabled_color = if self.dark_mode {
                            Color32::from_gray(70)
                        } else {
                            Color32::from_gray(180)
                        };

                        let add_rect = egui::Rect::from_center_size(
                            egui::pos2(btn_x, mid_y - 8.0),
                            Vec2::splat(16.0),
                        );
                        let remove_rect = egui::Rect::from_center_size(
                            egui::pos2(btn_x, mid_y + 8.0),
                            Vec2::splat(16.0),
                        );
                        let can_add = !op_busy && !self.zmk_no_extra_layers;
                        let add_resp = ui.allocate_rect(
                            add_rect,
                            if can_add {
                                Sense::click()
                            } else {
                                Sense::hover()
                            },
                        );
                        let remove_resp = ui.allocate_rect(
                            remove_rect,
                            if can_remove {
                                Sense::click()
                            } else {
                                Sense::hover()
                            },
                        );
                        if add_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if remove_resp.hovered() && can_remove {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        let add_col = if add_resp.hovered() && can_add {
                            Color32::from_rgb(91, 104, 223)
                        } else if can_add {
                            active_color
                        } else {
                            disabled_color
                        };
                        let rem_col = if remove_resp.hovered() && can_remove {
                            Color32::from_rgb(91, 104, 223)
                        } else if can_remove {
                            active_color
                        } else {
                            disabled_color
                        };
                        ui.painter().text(
                            add_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "+",
                            sym_font.clone(),
                            add_col,
                        );
                        ui.painter().text(
                            remove_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "−",
                            sym_font,
                            rem_col,
                        );
                        let add_hover_text = if self.zmk_no_extra_layers {
                            "Firmware doesn't support extra layers"
                        } else {
                            "Add layer"
                        };
                        let add_clicked = add_resp.on_hover_text(add_hover_text).clicked();
                        let rem_clicked = remove_resp
                            .on_hover_text(if can_remove {
                                "Remove last layer"
                            } else {
                                "Cannot remove base layers"
                            })
                            .clicked();
                        if add_clicked && can_add {
                            self.zmk_add_layer();
                        }
                        if rem_clicked && can_remove {
                            self.zmk_remove_layer();
                        }
                    }
                }

                // Layer name / edit field
                let name_rect = egui::Rect::from_min_size(
                    egui::pos2(center_x - 85.0, bar_y),
                    Vec2::new(170.0, 52.0),
                );

                let display_name_len = visible_raw_name.chars().count();
                let display_label_size = if display_name_len > 10 {
                    26.0
                } else if display_name_len > 7 {
                    31.0
                } else {
                    39.0
                };
                let label_font = egui::FontId {
                    size: display_label_size,
                    family: egui::FontFamily::Proportional,
                };
                let text_color = if self.dark_mode {
                    Color32::from_gray(245)
                } else {
                    Color32::from_gray(60)
                };

                if self.editing_layer == Some(selected) {
                    // Limit input to 12 chars
                    if self.editing_layer_text.chars().count() > 12 {
                        let s: String = self.editing_layer_text.chars().take(12).collect();
                        self.editing_layer_text = s;
                    }
                    let editing_font = egui::FontId {
                        size: 39.0,
                        family: egui::FontFamily::Proportional,
                    };
                    let resp = ui.put(
                        name_rect,
                        egui::TextEdit::singleline(&mut self.editing_layer_text)
                            .font(editing_font)
                            .horizontal_align(egui::Align::Center)
                            .char_limit(12)
                            .frame(false),
                    );
                    // Request focus only on the first frame so lost_focus() works correctly.
                    if !self.editing_layer_focus_requested {
                        resp.request_focus();
                        self.editing_layer_focus_requested = true;
                    }
                    // Commit on Enter or lost focus (click outside); cancel on Escape.
                    let commit =
                        resp.lost_focus() || ui.input(|inp| inp.key_pressed(egui::Key::Enter));
                    let cancel = ui.input(|inp| inp.key_pressed(egui::Key::Escape));
                    if commit || cancel {
                        if commit && !self.editing_layer_text.trim().is_empty() {
                            let new_name = self.editing_layer_text.trim().to_string();
                            while self.layer_names.len() <= selected {
                                self.layer_names.push(self.layer_names.len().to_string());
                            }
                            self.layer_names[selected] = new_name.clone();
                            #[cfg(not(target_arch = "wasm32"))]
                            save_layer_names(&self.layer_names, &self.current_device_name);
                            #[cfg(target_arch = "wasm32")]
                            save_layer_names(&self.layer_names, "default");
                            // Also write name back to the connected device
                            #[cfg(not(target_arch = "wasm32"))]
                            if self.firmware == FirmwareProtocol::Zmk {
                                if let Some(conn) = &mut self.zmk_conn {
                                    let layer_id = self
                                        .layout
                                        .as_ref()
                                        .and_then(|l| l.zmk_layer_ids.get(selected).copied())
                                        .unwrap_or(selected as u32);
                                    if let Err(e) = conn.set_layer_name(layer_id, &new_name) {
                                        log::warn!("ZMK set_layer_name failed: {e}");
                                    }
                                }
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            if self.firmware == FirmwareProtocol::Vial {
                                if let Some(dev) = &self.hid_device {
                                    if let Err(e) =
                                        dev.set_qmk_setting_string(200 + selected as u16, &new_name)
                                    {
                                        log::warn!(
                                            "Vial set_qmk_setting_string failed for layer {}: {}",
                                            selected,
                                            e
                                        );
                                    }
                                }
                            }
                        }
                        self.editing_layer = None;
                        self.editing_layer_focus_requested = false;
                    }
                } else {
                    let mid_y = bar_y + layer_bar_h / 2.0;

                    // Fixed arrow positions based on max 7-char name width so
                    // arrows never jump around as the layer name changes.
                    // name_rect is 170px wide → half = 85px; gap keeps arrows clear.
                    let fixed_half = 85.0_f32;
                    let gap = 16.0_f32;
                    let arrow_y = mid_y - 2.0;
                    let left_center = egui::pos2(center_x - fixed_half - gap - 24.0, arrow_y);
                    let right_center = egui::pos2(center_x + fixed_half + gap + 24.0, arrow_y);

                    // Still measure actual text width for painting the name and edit icon.
                    let text_w = ui.fonts(|f| {
                        f.layout_no_wrap(name.clone(), label_font.clone(), text_color)
                            .size()
                            .x
                    });

                    // Allocate name FIRST — arrows are allocated last and win in egui's
                    // hit-test order (last allocation = highest priority).
                    let name_hit = egui::Rect::from_center_size(
                        egui::pos2(center_x, mid_y),
                        Vec2::new(text_w + 12.0, 52.0),
                    );
                    let name_r = ui.allocate_rect(name_hit, Sense::click());

                    // Full layer switch zone from arrow to arrow for mouse wheel switching.
                    // Keep click/hover hitboxes close to the actual arrow glyph size.
                    let left_hit = egui::Rect::from_center_size(left_center, Vec2::new(28.0, 44.0));
                    let right_hit =
                        egui::Rect::from_center_size(right_center, Vec2::new(28.0, 44.0));
                    let wheel_hit = egui::Rect::from_min_max(
                        egui::pos2(left_hit.left(), mid_y - 26.0),
                        egui::pos2(right_hit.right(), mid_y + 26.0),
                    );
                    let wheel_r = ui.allocate_rect(wheel_hit, Sense::hover());

                    // Scroll wheel over the whole layer bar switches layers (down = next, up = prev)
                    if wheel_r.hovered() {
                        let scroll = ui.input(|i| i.raw_scroll_delta.y);
                        if scroll < 0.0 && selected > 0 {
                            self.selected_layer = selected - 1;
                        } else if scroll > 0.0 && selected + 1 < layer_count {
                            self.selected_layer = selected + 1;
                        }
                    }

                    // Allocate arrows LAST so they have click priority over the name rect.
                    let left_r = ui.allocate_rect(left_hit, Sense::click());
                    let right_r = ui.allocate_rect(right_hit, Sense::click());
                    if left_r.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if right_r.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if left_r.clicked() && selected > 0 {
                        self.selected_layer = selected - 1;
                        self.jump_back_stack.clear();
                    }
                    if right_r.clicked() && selected + 1 < layer_count {
                        self.selected_layer = selected + 1;
                        self.jump_back_stack.clear();
                    }
                    if name_r.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if name_r.clicked() {
                        self.editing_layer = Some(selected);
                        self.editing_layer_text = raw_name.clone();
                    }

                    // Paint
                    let dis = if self.dark_mode {
                        Color32::from_gray(60)
                    } else {
                        Color32::from_gray(200)
                    };
                    let ac_l = if left_r.hovered() {
                        Color32::from_rgb(91, 104, 223)
                    } else if self.dark_mode {
                        Color32::from_gray(140)
                    } else {
                        Color32::from_gray(120)
                    };
                    let ac_r = if right_r.hovered() {
                        Color32::from_rgb(91, 104, 223)
                    } else if self.dark_mode {
                        Color32::from_gray(140)
                    } else {
                        Color32::from_gray(120)
                    };
                    ui.painter().text(
                        left_center,
                        egui::Align2::CENTER_CENTER,
                        "‹",
                        FontId::proportional(52.0),
                        if selected == 0 { dis } else { ac_l },
                    );
                    ui.painter().text(
                        right_center,
                        egui::Align2::CENTER_CENTER,
                        "›",
                        FontId::proportional(52.0),
                        if selected + 1 >= layer_count {
                            dis
                        } else {
                            ac_r
                        },
                    );
                    ui.painter().text(
                        egui::pos2(center_x, mid_y),
                        egui::Align2::CENTER_CENTER,
                        &name,
                        label_font,
                        text_color,
                    );

                    // Hint text below layer name
                    let hint_color = if self.dark_mode {
                        Color32::from_gray(100)
                    } else {
                        Color32::from_gray(160)
                    };
                    let hint_font = FontId::proportional(11.0);
                    let secondary_hint_font = hint_font.clone();
                    let hint_y = ui.max_rect().bottom() - 36.0;
                    let any_hovered = self.prev_hovered_key.is_some() || self.prev_hovered_encoder;
                    if let Some(hl) = self.hover_layer {
                        let hl_name = self
                            .layer_names
                            .get(hl)
                            .cloned()
                            .unwrap_or_else(|| hl.to_string());
                        let mut line = 0i32;
                        let line_h = 13.0f32;
                        let base_y = hint_y - 15.0;
                        // Line 1: always
                        ui.painter().text(
                            egui::pos2(center_x, base_y + line as f32 * line_h),
                            egui::Align2::CENTER_CENTER,
                            "Left click to change this key",
                            hint_font.clone(),
                            hint_color,
                        );
                        line += 1;
                        // Line 2: go to layer (if not current)
                        if hl != self.selected_layer {
                            ui.painter().text(
                                egui::pos2(center_x, base_y + line as f32 * line_h),
                                egui::Align2::CENTER_CENTER,
                                &format!("Right click to go to layer {}: {}", hl, hl_name),
                                hint_font.clone(),
                                hint_color,
                            );
                            line += 1;
                        }
                        // Line 3: change layer number
                        ui.painter().text(
                            egui::pos2(center_x, base_y + line as f32 * line_h),
                            egui::Align2::CENTER_CENTER,
                            "Ctrl + Right click to change layer number",
                            hint_font.clone(),
                            hint_color,
                        );
                        line += 1;
                        // Line 4: go back (if in jump mode)
                        if !self.jump_back_stack.is_empty() {
                            ui.painter().text(
                                egui::pos2(center_x, base_y + line as f32 * line_h),
                                egui::Align2::CENTER_CENTER,
                                "Esc to go back",
                                hint_font.clone(),
                                hint_color,
                            );
                        }
                        let _ = hint_font;
                    } else if !self.jump_back_stack.is_empty() {
                        if any_hovered {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 9.0),
                                egui::Align2::CENTER_CENTER,
                                "Left click to change this key",
                                hint_font.clone(),
                                hint_color,
                            );
                        }
                        ui.painter().text(
                            egui::pos2(center_x, if any_hovered { hint_y + 5.0 } else { hint_y }),
                            egui::Align2::CENTER_CENTER,
                            "Right-click or Esc to go back",
                            hint_font,
                            hint_color,
                        );
                    } else if any_hovered {
                        // Check if hovered key is a mod key
                        let (
                            hovered_is_mod,
                            hovered_can_swap_side,
                            hovered_is_macro,
                            hovered_is_tap_dance,
                            hovered_is_mouse,
                            hovered_is_alt_repeat,
                            hovered_is_layer,
                        ) = if self.firmware == FirmwareProtocol::Vial {
                            let hint_kc = self
                                .prev_hovered_key
                                .and_then(|ki| {
                                    self.layout
                                        .as_ref()
                                        .map(|l| l.get_keycode(self.selected_layer, ki))
                                })
                                .or(self.prev_hovered_encoder_keycode)
                                .or_else(|| {
                                    self.selected_key.and_then(|(selected_layer, selected_ki)| {
                                        (selected_layer == self.selected_layer)
                                            .then(|| {
                                                self.layout.as_ref().map(|l| {
                                                    l.get_keycode(self.selected_layer, selected_ki)
                                                })
                                            })
                                            .flatten()
                                    })
                                });
                            hint_kc
                                .map(|kc| {
                                    let is_plain_mod = (0x00E0..=0x00E7).contains(&kc)
                                        || matches!(
                                            kc,
                                            0x52A1
                                                | 0x52A2
                                                | 0x52A4
                                                | 0x52A7
                                                | 0x52A8
                                                | 0x52AF
                                                | 0x52B1
                                                | 0x52B2
                                                | 0x52B4
                                                | 0x52B8
                                        );
                                    let is_mod = is_plain_mod
                                        || (kc >= 0x2000 && kc < 0x4000)
                                        || (kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0);
                                    let can_swap_side = toggle_handed_modifier(kc).is_some();
                                    let is_macro = kc >= 0x7700 && kc <= 0x77FF;
                                    let is_tap_dance = kc >= 0x5700 && kc <= 0x57FF;
                                    let is_mouse = is_mouse_keycode(kc);
                                    let is_alt_repeat = is_alt_repeat_keycode(kc);
                                    let is_layer =
                                        (kc >= 0x5200 && kc < 0x5300) || (kc & 0xF000 == 0x4000);
                                    (
                                        is_mod,
                                        can_swap_side,
                                        is_macro,
                                        is_tap_dance,
                                        is_mouse,
                                        is_alt_repeat,
                                        is_layer,
                                    )
                                })
                                .unwrap_or((false, false, false, false, false, false, false))
                        } else {
                            (false, false, false, false, false, false, false)
                        };
                        if hovered_is_mod {
                            if hovered_can_swap_side {
                                ui.painter().text(
                                    egui::pos2(center_x, hint_y - 22.0),
                                    egui::Align2::CENTER_CENTER,
                                    "Left click to change this key",
                                    hint_font.clone(),
                                    hint_color,
                                );
                                ui.painter().text(
                                    egui::pos2(center_x, hint_y - 4.0),
                                    egui::Align2::CENTER_CENTER,
                                    "Right click to change the modifier key",
                                    secondary_hint_font.clone(),
                                    hint_color,
                                );
                                ui.painter().text(
                                    egui::pos2(center_x, hint_y + 12.0),
                                    egui::Align2::CENTER_CENTER,
                                    "Ctrl+right-click to switch left/right side",
                                    secondary_hint_font,
                                    hint_color,
                                );
                            } else {
                                ui.painter().text(
                                    egui::pos2(center_x, hint_y - 14.0),
                                    egui::Align2::CENTER_CENTER,
                                    "Left click to change this key",
                                    hint_font.clone(),
                                    hint_color,
                                );
                                ui.painter().text(
                                    egui::pos2(center_x, hint_y + 4.0),
                                    egui::Align2::CENTER_CENTER,
                                    "Right click to change the modifier key",
                                    secondary_hint_font,
                                    hint_color,
                                );
                            }
                        } else if hovered_is_macro {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 14.0),
                                egui::Align2::CENTER_CENTER,
                                "Left click to change this key",
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 4.0),
                                egui::Align2::CENTER_CENTER,
                                "Right click to edit macro",
                                secondary_hint_font.clone(),
                                hint_color,
                            );
                        } else if hovered_is_tap_dance {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 14.0),
                                egui::Align2::CENTER_CENTER,
                                "Left click to change this key",
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 4.0),
                                egui::Align2::CENTER_CENTER,
                                "Right click to edit tap dance",
                                secondary_hint_font,
                                hint_color,
                            );
                        } else if hovered_is_mouse {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 14.0),
                                egui::Align2::CENTER_CENTER,
                                "Left click to change this key",
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 4.0),
                                egui::Align2::CENTER_CENTER,
                                "Right click to open Mouse Keys settings",
                                secondary_hint_font.clone(),
                                hint_color,
                            );
                        } else if hovered_is_alt_repeat {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 14.0),
                                egui::Align2::CENTER_CENTER,
                                "Left click to change this key",
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 4.0),
                                egui::Align2::CENTER_CENTER,
                                "Right click to open Alt Repeat settings",
                                secondary_hint_font,
                                hint_color,
                            );
                        } else if hovered_is_layer {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 22.0),
                                egui::Align2::CENTER_CENTER,
                                "Left click to change this key",
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 4.0),
                                egui::Align2::CENTER_CENTER,
                                "Right click to go to that layer",
                                secondary_hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 12.0),
                                egui::Align2::CENTER_CENTER,
                                "Ctrl+right-click to change layer target",
                                secondary_hint_font,
                                hint_color,
                            );
                        } else {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y),
                                egui::Align2::CENTER_CENTER,
                                "Left click to change this key",
                                hint_font,
                                hint_color,
                            );
                        }
                    } else if name_r.hovered() {
                        ui.painter().text(
                            egui::pos2(center_x, hint_y),
                            egui::Align2::CENTER_CENTER,
                            "Click to rename layer",
                            hint_font,
                            hint_color,
                        );
                    }
                }
            }
        }

        // Pass 1: allocate
        let key_rects: Vec<(usize, egui::Rect)> = layout
            .keys
            .iter()
            .enumerate()
            .map(|(ki, key)| {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(
                        offset_x + key.x * unit + padding,
                        offset_y + key.y * unit + padding,
                    ),
                    Vec2::new(key.w * unit - padding * 2.0, key.h * unit - padding * 2.0),
                );
                (ki, rect)
            })
            .collect();
        let encoder_rects: Vec<(usize, egui::Rect)> = layout
            .encoders
            .iter()
            .enumerate()
            .map(|(ei, encoder)| {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(
                        offset_x + encoder.x * unit + padding,
                        offset_y + encoder.y * unit + padding,
                    ),
                    Vec2::new(
                        encoder.w * unit - padding * 2.0,
                        encoder.h * unit - padding * 2.0,
                    ),
                );
                (ei, rect)
            })
            .collect();
        let mut encoder_groups: Vec<(u8, egui::Rect, Option<(usize, u16)>, Option<(usize, u16)>)> =
            Vec::new();
        for (ei, rect) in &encoder_rects {
            let encoder = &layout.encoders[*ei];
            if !self
                .encoder_visibility
                .get(encoder.encoder_idx as usize)
                .copied()
                .unwrap_or(true)
            {
                continue;
            }
            let kc = layout.get_encoder_keycode(self.selected_layer, *ei);
            if let Some((_, group_rect, ccw, cw)) = encoder_groups
                .iter_mut()
                .find(|(idx, _, _, _)| *idx == encoder.encoder_idx)
            {
                *group_rect = group_rect.union(*rect);
                if encoder.direction == 0 {
                    *ccw = Some((*ei, kc));
                } else {
                    *cw = Some((*ei, kc));
                }
            } else {
                encoder_groups.push((
                    encoder.encoder_idx,
                    *rect,
                    if encoder.direction == 0 {
                        Some((*ei, kc))
                    } else {
                        None
                    },
                    if encoder.direction == 0 {
                        None
                    } else {
                        Some((*ei, kc))
                    },
                ));
            }
        }
        let mut encoder_press_rects: Vec<(usize, egui::Rect)> = Vec::new();
        for (_, group_rect, _, _) in &encoder_groups {
            let center = group_rect.center();
            let radius = group_rect.width().min(group_rect.height()) * 0.5;
            let mut best_key: Option<(usize, f32)> = None;
            for (ki, key_rect) in &key_rects {
                if encoder_press_rects
                    .iter()
                    .any(|(assigned_ki, _)| assigned_ki == ki)
                {
                    continue;
                }
                let dist = key_rect.center().distance(center);
                if dist > radius * 0.38 {
                    continue;
                }
                match best_key {
                    Some((_, best_dist)) if dist >= best_dist => {}
                    _ => best_key = Some((*ki, dist)),
                }
            }
            if let Some((ki, _)) = best_key {
                let press_rect = egui::Rect::from_center_size(
                    center,
                    Vec2::new(
                        (radius * 0.88).min(group_rect.width() * 0.44),
                        (radius * 0.48).min(group_rect.height() * 0.22),
                    ),
                );
                encoder_press_rects.push((ki, press_rect));
            }
        }
        let mut rects: Vec<(usize, egui::Rect, egui::Response)> =
            Vec::with_capacity(layout.keys.len());
        for (ki, rect) in &key_rects {
            let response_rect = encoder_press_rects
                .iter()
                .find(|(press_ki, _)| press_ki == ki)
                .map(|(_, press_rect)| *press_rect)
                .unwrap_or(*rect);
            let response = ui.allocate_rect(response_rect, Sense::click());
            rects.push((*ki, response_rect, response));
        }

        let is_zmk = self.firmware == FirmwareProtocol::Zmk;

        // Reset hover_layer each frame — will be set again if a layer key is hovered
        let prev_hover = self.hover_layer;
        self.hover_layer = None;

        // Pass 2: hover + clicks + tooltips
        let mut hovered_key: Option<usize> = None;
        for (ki, _, response) in &mut rects {
            if response.hovered() {
                hovered_key = Some(*ki);
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                self.selected_key = Some((self.selected_layer, *ki));
                self.keycode_picker.open = true;
                self.keycode_picker.search_query.clear();
                self.keycode_picker.layer_names = self.layer_names.clone();
                self.keycode_picker.firmware = self.firmware;
                self.keycode_picker.vial_quantum_pending_mod = None;
                self.keycode_picker.vial_quantum_pending_mt = None;
                // Reset all editor states so picker opens normally
                self.keycode_picker.tap_dance_editor_open = None;
                self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::Basic;
                if is_zmk {
                    self.keycode_picker.zmk_behaviors = self
                        .layout
                        .as_ref()
                        .map(|l| l.zmk_behaviors.clone())
                        .unwrap_or_default();
                    self.keycode_picker.zmk_layer_count = self.layer_count;
                }
            }

            // Right-click on mod key — open second picker to change tap/key
            if response.secondary_clicked() && !is_zmk {
                let kc = layout.get_keycode(self.selected_layer, *ki);
                let ctrl_held = ui.input(|i| i.modifiers.ctrl);

                // Ctrl+RClick on layer keys: change layer number
                if ctrl_held {
                    if let Some(swapped) = toggle_handed_modifier(kc) {
                        self.pending_handed_swap = Some((self.selected_layer, *ki, swapped));
                        self.secondary_click_handled = true;
                    } else {
                        let layer_base: Option<u16> = if kc >= 0x5200 && kc < 0x5300 {
                            Some(kc & 0xFFE0)
                        } else if kc & 0xF000 == 0x4000 {
                            Some(0x4000)
                        } else {
                            None
                        };
                        if let Some(base) = layer_base {
                            self.selected_key = Some((self.selected_layer, *ki));
                            self.keycode_picker.open = true;
                            self.keycode_picker.layer_names = self.layer_names.clone();
                            self.keycode_picker.firmware = self.firmware;
                            self.keycode_picker.vial_layer_pending = Some(base);
                            self.secondary_click_handled = true;
                        }
                    }
                    if self.secondary_click_handled {
                        continue;
                    }
                }
                // Macro keys: 0x7700..0x77FF — RClick opens editor
                if kc >= 0x7700 && kc <= 0x77FF {
                    let macro_n = (kc - 0x7700) as u8;
                    self.selected_key = Some((self.selected_layer, *ki));
                    self.keycode_picker.open = true;
                    self.keycode_picker.layer_names = self.layer_names.clone();
                    self.keycode_picker.firmware = self.firmware;
                    self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::Macro;
                    self.keycode_picker.macro_inline_selected = Some(macro_n);
                    self.secondary_click_handled = true;
                }
                // Tap Dance keys: 0x5700..0x57FF — RClick opens editor
                if kc >= 0x5700 && kc <= 0x57FF {
                    let td_n = (kc - 0x5700) as u8;
                    self.selected_key = Some((self.selected_layer, *ki));
                    self.keycode_picker.open = true;
                    self.keycode_picker.layer_names = self.layer_names.clone();
                    self.keycode_picker.firmware = self.firmware;
                    self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::TapDance;
                    self.keycode_picker.tap_dance_editor_open = Some(td_n);
                    self.secondary_click_handled = true;
                }
                // Mouse keys — RClick opens Mouse keys settings
                if is_mouse_keycode(kc) {
                    self.modal_focus_pending = true;
                    self.mouse_keys_window_open = true;
                    self.secondary_click_handled = true;
                }
                if is_alt_repeat_keycode(kc) {
                    self.open_alt_repeat_window_compact();
                    self.secondary_click_handled = true;
                }
                // MT: 0x2000..0x3FFF, Mod+Key: 0x0100..0x1FFF with kc != 0
                let is_layer_key = (kc >= 0x5200 && kc < 0x5300) || (kc & 0xF000 == 0x4000);
                let pending_base: Option<u16> = if is_layer_key {
                    None // layer keys handled above (Ctrl+RClick)
                } else if kc >= 0x2000 && kc < 0x4000 {
                    Some(kc & 0xFF00) // MT base
                } else if kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0 {
                    Some(kc & 0xFF00) // Mod+Key base
                } else {
                    None
                };
                if let Some(base) = pending_base {
                    self.selected_key = Some((self.selected_layer, *ki));
                    self.keycode_picker.open = true;
                    self.keycode_picker.layer_names = self.layer_names.clone();
                    self.keycode_picker.firmware = self.firmware;
                    if kc >= 0x2000 {
                        self.keycode_picker.vial_quantum_pending_mt = Some(base);
                        self.keycode_picker.vial_quantum_pending_mod = None;
                    } else {
                        self.keycode_picker.vial_quantum_pending_mod = Some(base);
                        self.keycode_picker.vial_quantum_pending_mt = None;
                    }
                    self.secondary_click_handled = true;
                }
            }

            // Tooltip — for layer keys show mini layout preview
            let preview_layer: Option<usize> = if is_zmk {
                let binding = layout.get_zmk_binding(self.selected_layer, *ki);
                let beh_name = layout
                    .zmk_behaviors
                    .iter()
                    .find(|b| b.id == binding.behavior_id as u32)
                    .map(|b| b.display_name.as_str())
                    .unwrap_or("");
                match beh_name {
                    "Momentary Layer" | "Toggle Layer" | "To Layer" | "Sticky Layer"
                    | "Layer-Tap" => Some(binding.param1 as usize),
                    _ => None,
                }
            } else {
                let kc = layout.get_keycode(self.selected_layer, *ki);
                // MO/TG/TO/OSL/TT/DF range: 0x5200..0x52FF, LT: 0x4000..0x4FFF
                if kc >= 0x5200 && kc < 0x5300 {
                    Some((kc & 0x1F) as usize)
                } else if kc & 0xF000 == 0x4000 {
                    Some(((kc >> 8) & 0xF) as usize)
                } else {
                    None
                }
            };

            if let Some(preview_layer_idx) = preview_layer {
                if response.hovered() {
                    self.hover_layer = Some(preview_layer_idx);
                    hovered_key = Some(*ki); // keep hovered_key for layer keys too
                }
                if response.secondary_clicked() && preview_layer_idx != self.selected_layer {
                    // Right-click: jump to that layer
                    self.jump_back_stack.push(self.selected_layer);
                    self.selected_layer = preview_layer_idx;
                    self.hover_layer = None;
                    self.secondary_click_handled = true;
                }
            } else if is_zmk {
                let binding = layout.get_zmk_binding(self.selected_layer, *ki);
                let tip = zmk_binding_tooltip(&binding, &layout.zmk_behaviors, &self.layer_names);
                *response = response.clone().on_hover_text(tip);
            } else {
                let kc = layout.get_keycode(self.selected_layer, *ki);
                let tip = keycode_tooltip_with_macro_names(
                    kc,
                    &layout.custom_keycodes,
                    &self.layer_names,
                    &self.keycode_picker.macro_names,
                    &self.keycode_picker.tap_dance_names,
                );
                *response = response.clone().on_hover_text(tip);
            }
        }

        // Animate hover_layer_progress
        let target_progress = if self.hover_layer.is_some() {
            1.0f32
        } else {
            0.0f32
        };
        let speed = 4.0f32;
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        self.hover_layer_progress +=
            (target_progress - self.hover_layer_progress) * (speed * dt).min(1.0);
        if (self.hover_layer_progress - target_progress).abs() > 0.01 {
            ctx.request_repaint();
        }

        // Pass 3: paint
        let painter = ui.painter();
        let mut hovered_encoder = false;
        let mut hovered_encoder_keycode = None;
        let hover_target = self
            .hover_layer
            .unwrap_or(prev_hover.unwrap_or(self.selected_layer));
        let hover_alpha = self.hover_layer_progress;
        let dark = self.dark_mode;
        // Use hover layer for logic (TRNS resolution etc) when mostly visible
        let layer = if hover_alpha > 0.5 {
            hover_target
        } else {
            self.selected_layer
        };
        for (ki, rect, _) in &rects {
            let key = &layout.keys[*ki];
            let is_selected = self.selected_key == Some((layer, *ki));
            let is_hovered = hovered_key == Some(*ki);
            // Accent: #5B68DF indigo
            let bg = if is_selected {
                Color32::from_rgb(91, 104, 223)
            } else if is_hovered {
                if dark {
                    Color32::from_rgb(60, 60, 65)
                } else {
                    Color32::from_rgb(232, 232, 240)
                }
            } else {
                if dark {
                    Color32::from_rgb(48, 48, 52)
                } else {
                    Color32::from_rgb(255, 255, 255)
                }
            };

            let press_rect_override = encoder_press_rects
                .iter()
                .find(|(press_ki, _)| *press_ki == *ki)
                .map(|(_, press_rect)| *press_rect);
            let draw_rect = if let Some(press_rect) = press_rect_override {
                press_rect
            } else if key.rotation != 0.0 {
                let angle_rad = key.rotation.to_radians();
                let ax = offset_x + key.rotation_x * unit;
                let ay = offset_y + key.rotation_y * unit;
                let anchor = egui::pos2(ax, ay);
                let center = rect.center();
                let dx = center.x - anchor.x;
                let dy = center.y - anchor.y;
                let rx = anchor.x + dx * angle_rad.cos() - dy * angle_rad.sin();
                let ry = anchor.y + dx * angle_rad.sin() + dy * angle_rad.cos();
                egui::Rect::from_center_size(egui::pos2(rx, ry), rect.size())
            } else {
                *rect
            };

            let is_hovering = hover_alpha > 0.05;

            if press_rect_override.is_some() {
                continue;
            }

            if is_zmk {
                // ZMK binding display
                let binding = layout.get_zmk_binding(layer, *ki);
                let is_trans = layout
                    .zmk_behaviors
                    .iter()
                    .find(|b| b.id == binding.behavior_id as u32)
                    .map(|b| b.display_name == "Transparent")
                    .unwrap_or(false);
                let border = if dark {
                    Color32::from_rgb(55, 55, 60)
                } else {
                    Color32::from_rgb(210, 210, 218)
                };
                painter.rect(
                    draw_rect,
                    6.0,
                    bg,
                    Stroke::new(1.0, border),
                    egui::StrokeKind::Inside,
                );
                if is_trans && layer > 0 {
                    if is_hovering {
                        // During hover preview — TRNS keys are empty (no text)
                    } else {
                        // Normal display — show TRNS with fallback
                        let fallback = (0..layer)
                            .rev()
                            .map(|l| layout.get_zmk_binding(l, *ki))
                            .find(|b| {
                                !layout
                                    .zmk_behaviors
                                    .iter()
                                    .find(|beh| beh.id == b.behavior_id as u32)
                                    .map(|beh| beh.display_name == "Transparent")
                                    .unwrap_or(false)
                            });
                        let label = if let Some(fb) = fallback {
                            zmk_binding_label(&fb, &layout.zmk_behaviors, &self.layer_names)
                        } else {
                            "▽".to_string()
                        };
                        draw_key_label_dimmed(&painter, draw_rect, &label, dark);
                    }
                } else {
                    let label =
                        zmk_binding_label(&binding, &layout.zmk_behaviors, &self.layer_names);
                    draw_key_label(&painter, draw_rect, &label, dark);
                }
            } else {
                let kc = layout.get_keycode(layer, *ki);

                if kc == 0x0001 {
                    painter.rect(
                        draw_rect,
                        6.0,
                        bg,
                        Stroke::new(
                            1.0,
                            if dark {
                                Color32::from_rgb(55, 55, 60)
                            } else {
                                Color32::from_rgb(210, 210, 218)
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    if !is_hovering {
                        let fallback_kc = (0..layer)
                            .rev()
                            .map(|l| layout.get_keycode(l, *ki))
                            .find(|&k| k != 0x0001)
                            .unwrap_or(0x0000);
                        let label = if fallback_kc == 0x0000 || fallback_kc == 0x0001 {
                            String::new()
                        } else {
                            keycode_label_with_macro_names(
                                fallback_kc,
                                &layout.custom_keycodes,
                                &self.layer_names,
                                &self.keycode_picker.macro_names,
                                &self.keycode_picker.tap_dance_names,
                            )
                        };
                        draw_key_label_dimmed(&painter, draw_rect, &label, dark);
                    }
                } else if kc == 0x0000 {
                    let no_bg = if dark {
                        Color32::from_rgb(20, 20, 22)
                    } else {
                        Color32::from_rgb(255, 255, 255)
                    };
                    let no_border = if dark {
                        Color32::from_rgb(40, 40, 44)
                    } else {
                        Color32::from_rgb(210, 210, 218)
                    };
                    let fill = if is_selected || is_hovered { bg } else { no_bg };
                    painter.rect(
                        draw_rect,
                        6.0,
                        fill,
                        Stroke::new(1.0, no_border),
                        egui::StrokeKind::Inside,
                    );
                } else {
                    let border = if dark {
                        Color32::from_rgb(55, 55, 60)
                    } else {
                        Color32::from_rgb(210, 210, 218)
                    };
                    painter.rect(
                        draw_rect,
                        6.0,
                        bg,
                        Stroke::new(1.0, border),
                        egui::StrokeKind::Inside,
                    );
                    let label = keycode_label_with_macro_names(
                        kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    );
                    draw_key_label(&painter, draw_rect, &label, dark);
                }
            }
        }

        let encoder_custom_keycodes = layout.custom_keycodes.clone();
        let encoder_layer_names = self.layer_names.clone();
        let encoder_macro_names = self.keycode_picker.macro_names.clone();
        let encoder_tap_dance_names = self.keycode_picker.tap_dance_names.clone();
        let encoder_label = |kc: u16| -> String {
            match kc {
                0x0000 => "✕".to_string(),
                0x0001 => "▽".to_string(),
                _ => keycode_label_with_macro_names(
                    kc,
                    &encoder_custom_keycodes,
                    &encoder_layer_names,
                    &encoder_macro_names,
                    &encoder_tap_dance_names,
                )
                .replace('\n', " "),
            }
        };

        let draw_encoder_arrow = |painter: &egui::Painter,
                                  center: egui::Pos2,
                                  encoder_radius: f32,
                                  top: bool,
                                  color: Color32| {
            let (start_deg, end_deg) = if top {
                (240.0_f32, 300.0_f32)
            } else {
                (120.0_f32, 60.0_f32)
            };
            let r = encoder_radius * 1.22;
            let mut points = Vec::new();
            for step in 0..=12 {
                let t = step as f32 / 12.0;
                let deg = start_deg + (end_deg - start_deg) * t;
                let rad = deg.to_radians();
                points.push(egui::pos2(
                    center.x + rad.cos() * r,
                    center.y + rad.sin() * r,
                ));
            }
            painter.add(egui::Shape::line(points.clone(), Stroke::new(1.7, color)));
            if points.len() >= 2 {
                let end = points[points.len() - 1];
                let prev = points[points.len() - 2];
                let dir = egui::vec2(end.x - prev.x, end.y - prev.y).normalized();
                let left = egui::vec2(-dir.y, dir.x);
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        end,
                        egui::pos2(
                            end.x - dir.x * 3.6 + left.x * 2.4,
                            end.y - dir.y * 3.6 + left.y * 2.4,
                        ),
                        egui::pos2(
                            end.x - dir.x * 3.6 - left.x * 2.4,
                            end.y - dir.y * 3.6 - left.y * 2.4,
                        ),
                    ],
                    color,
                    Stroke::NONE,
                ));
            }
        };

        for (_encoder_idx, rect, ccw, cw) in &encoder_groups {
            let center = rect.center();
            let radius = rect.width().min(rect.height()) * 0.58;
            let circle_bounds =
                egui::Rect::from_center_size(center, egui::vec2(radius * 2.0, radius * 2.0));
            let press_slot = encoder_press_rects
                .iter()
                .find(|(_, press_rect)| press_rect.center().distance(center) < 1.0)
                .map(|(press_ki, press_rect)| (*press_ki, *press_rect));
            let (top_rect, middle_rect, bottom_rect) = if let Some((_, press_rect)) = press_slot {
                let divider_gap = radius * 0.06;
                let top_divider_y = press_rect.top() - divider_gap;
                let bottom_divider_y = press_rect.bottom() + divider_gap;
                (
                    egui::Rect::from_min_max(
                        circle_bounds.min,
                        egui::pos2(circle_bounds.max.x, top_divider_y),
                    ),
                    Some(egui::Rect::from_min_max(
                        egui::pos2(circle_bounds.min.x, top_divider_y),
                        egui::pos2(circle_bounds.max.x, bottom_divider_y),
                    )),
                    egui::Rect::from_min_max(
                        egui::pos2(circle_bounds.min.x, bottom_divider_y),
                        circle_bounds.max,
                    ),
                )
            } else {
                (
                    egui::Rect::from_min_max(
                        circle_bounds.min,
                        egui::pos2(circle_bounds.max.x, center.y),
                    ),
                    None,
                    egui::Rect::from_min_max(
                        egui::pos2(circle_bounds.min.x, center.y),
                        circle_bounds.max,
                    ),
                )
            };
            let top_resp = ui.allocate_rect(top_rect, Sense::click());
            let middle_resp =
                middle_rect.map(|middle_rect| ui.allocate_rect(middle_rect, Sense::click()));
            let bottom_resp = ui.allocate_rect(bottom_rect, Sense::click());
            if top_resp.hovered()
                || middle_resp.as_ref().map(|r| r.hovered()).unwrap_or(false)
                || bottom_resp.hovered()
            {
                hovered_encoder = true;
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let ctrl_held = ui.input(|i| i.modifiers.ctrl);
            if top_resp.hovered() {
                if let Some((_, kc)) = cw {
                    hovered_encoder_keycode = Some(*kc);
                    let tip = keycode_tooltip_with_macro_names(
                        *kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    );
                    let _ = top_resp.clone().on_hover_text(tip);
                }
            }
            if top_resp.secondary_clicked() {
                if let Some((visual_idx, kc)) = cw {
                    self.handle_secondary_target(ctrl_held, is_zmk, *kc, None, Some(*visual_idx));
                }
            }
            if top_resp.clicked() {
                if let Some((visual_idx, _)) = cw {
                    self.selected_key = None;
                    self.selected_encoder = Some((layer, *visual_idx));
                    self.keycode_picker.open = true;
                    self.keycode_picker.result = None;
                }
            }
            if let (Some((press_ki, _)), Some(middle_resp)) = (press_slot, middle_resp.as_ref()) {
                if middle_resp.hovered() {
                    hovered_key = Some(press_ki);
                    let kc = layout.get_keycode(self.selected_layer, press_ki);
                    hovered_encoder_keycode = Some(kc);
                    let tip = keycode_tooltip_with_macro_names(
                        kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    );
                    let _ = middle_resp.clone().on_hover_text(tip);
                }
                if middle_resp.secondary_clicked() {
                    let kc = layout.get_keycode(self.selected_layer, press_ki);
                    self.handle_secondary_target(ctrl_held, is_zmk, kc, Some(press_ki), None);
                }
                if middle_resp.clicked() {
                    self.open_picker_for_target(Some(press_ki), None, is_zmk);
                    self.selected_encoder = None;
                }
            }
            if bottom_resp.hovered() {
                if let Some((_, kc)) = ccw {
                    hovered_encoder_keycode = Some(*kc);
                    let tip = keycode_tooltip_with_macro_names(
                        *kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    );
                    let _ = bottom_resp.clone().on_hover_text(tip);
                }
            }
            if bottom_resp.secondary_clicked() {
                if let Some((visual_idx, kc)) = ccw {
                    self.handle_secondary_target(ctrl_held, is_zmk, *kc, None, Some(*visual_idx));
                }
            }
            if bottom_resp.clicked() {
                if let Some((visual_idx, _)) = ccw {
                    self.selected_key = None;
                    self.selected_encoder = Some((self.selected_layer, *visual_idx));
                    self.keycode_picker.open = true;
                    self.keycode_picker.result = None;
                }
            }

            let top_selected = cw
                .map(|(visual_idx, _)| Some((layer, visual_idx)) == self.selected_encoder)
                .unwrap_or(false);
            let bottom_selected = ccw
                .map(|(visual_idx, _)| Some((layer, visual_idx)) == self.selected_encoder)
                .unwrap_or(false);
            let middle_selected = press_slot
                .map(|(press_ki, _)| self.selected_key == Some((layer, press_ki)))
                .unwrap_or(false);
            let middle_hovered = middle_resp.as_ref().map(|r| r.hovered()).unwrap_or(false);
            let visuals = &ui.visuals().widgets;
            let fill_radius = radius + 1.5;
            let top_fill = if top_selected {
                visuals.active.bg_fill
            } else if top_resp.hovered() {
                visuals.hovered.bg_fill
            } else {
                visuals.inactive.bg_fill
            };
            let bottom_fill = if bottom_selected {
                visuals.active.bg_fill
            } else if bottom_resp.hovered() {
                visuals.hovered.bg_fill
            } else {
                visuals.inactive.bg_fill
            };
            let middle_fill = if middle_selected {
                visuals.active.bg_fill
            } else if middle_hovered {
                visuals.hovered.bg_fill
            } else {
                visuals.inactive.bg_fill
            };
            let outline = if top_selected || bottom_selected || middle_selected {
                visuals.active.bg_stroke
            } else if top_resp.hovered() || middle_hovered || bottom_resp.hovered() {
                visuals.hovered.bg_stroke
            } else {
                visuals.inactive.bg_stroke
            };

            let painter = ui.painter();
            painter.circle_filled(center, fill_radius, visuals.inactive.bg_fill);
            painter
                .with_clip_rect(top_rect)
                .circle_filled(center, fill_radius, top_fill);
            if let Some(middle_rect) = middle_rect {
                painter
                    .with_clip_rect(middle_rect)
                    .circle_filled(center, fill_radius, middle_fill);
            }
            painter
                .with_clip_rect(bottom_rect)
                .circle_filled(center, fill_radius, bottom_fill);
            painter.circle_stroke(center, radius, outline);

            let has_press_button = encoder_press_rects
                .iter()
                .any(|(_, press_rect)| press_rect.center().distance(center) < 1.0);
            let top_label = encoder_label(cw.map(|(_, kc)| kc).unwrap_or(0x0000));
            let bottom_label = encoder_label(ccw.map(|(_, kc)| kc).unwrap_or(0x0000));
            let top_font = if has_press_button {
                egui::FontId::proportional(if top_label.chars().count() > 9 {
                    6.6
                } else {
                    7.4
                })
            } else {
                egui::FontId::proportional(if top_label.chars().count() > 9 {
                    8.5
                } else {
                    9.5
                })
            };
            let bottom_font = if has_press_button {
                egui::FontId::proportional(if bottom_label.chars().count() > 9 {
                    6.6
                } else {
                    7.4
                })
            } else {
                egui::FontId::proportional(if bottom_label.chars().count() > 9 {
                    8.5
                } else {
                    9.5
                })
            };
            let top_label_y = center.y - radius * if has_press_button { 0.52 } else { 0.30 };
            let bottom_label_y = center.y + radius * if has_press_button { 0.52 } else { 0.30 };
            let top_text_color = if top_selected {
                visuals.active.fg_stroke.color
            } else if top_resp.hovered() {
                visuals.hovered.fg_stroke.color
            } else {
                visuals.inactive.fg_stroke.color
            };
            let bottom_text_color = if bottom_selected {
                visuals.active.fg_stroke.color
            } else if bottom_resp.hovered() {
                visuals.hovered.fg_stroke.color
            } else {
                visuals.inactive.fg_stroke.color
            };
            painter.text(
                egui::pos2(center.x, top_label_y),
                egui::Align2::CENTER_CENTER,
                top_label,
                top_font,
                top_text_color,
            );
            painter.text(
                egui::pos2(center.x, bottom_label_y),
                egui::Align2::CENTER_CENTER,
                bottom_label,
                bottom_font,
                bottom_text_color,
            );

            let arrow_color_top = outline.color;
            let arrow_color_bottom = outline.color;
            draw_encoder_arrow(painter, center, radius, true, arrow_color_top);
            draw_encoder_arrow(painter, center, radius, false, arrow_color_bottom);

            if let Some((press_ki, _)) = press_slot {
                let middle_rect = middle_rect.unwrap();
                let top_divider_y = middle_rect.top();
                let bottom_divider_y = middle_rect.bottom();
                let divider_extend = 0.75;
                let top_divider_half_width = (((radius * radius)
                    - (top_divider_y - center.y) * (top_divider_y - center.y))
                    .max(0.0)
                    .sqrt())
                    + divider_extend;
                let bottom_divider_half_width = (((radius * radius)
                    - (bottom_divider_y - center.y) * (bottom_divider_y - center.y))
                    .max(0.0)
                    .sqrt())
                    + divider_extend;
                painter.line_segment(
                    [
                        egui::pos2(center.x - top_divider_half_width, top_divider_y),
                        egui::pos2(center.x + top_divider_half_width, top_divider_y),
                    ],
                    Stroke::new(1.0, outline.color),
                );
                painter.line_segment(
                    [
                        egui::pos2(center.x - bottom_divider_half_width, bottom_divider_y),
                        egui::pos2(center.x + bottom_divider_half_width, bottom_divider_y),
                    ],
                    Stroke::new(1.0, outline.color),
                );
                let is_hovering = hover_alpha > 0.05;
                let text_color = if middle_selected {
                    visuals.active.fg_stroke.color
                } else if middle_hovered {
                    visuals.hovered.fg_stroke.color
                } else {
                    visuals.inactive.fg_stroke.color
                };
                let press_text_rect = middle_rect.shrink2(egui::vec2(4.0, 2.0));

                let press_label = if is_zmk {
                    let binding = layout.get_zmk_binding(layer, press_ki);
                    let is_trans = layout
                        .zmk_behaviors
                        .iter()
                        .find(|b| b.id == binding.behavior_id as u32)
                        .map(|b| b.display_name == "Transparent")
                        .unwrap_or(false);
                    if is_trans && layer > 0 && !is_hovering {
                        let fallback = (0..layer)
                            .rev()
                            .map(|l| layout.get_zmk_binding(l, press_ki))
                            .find(|b| {
                                !layout
                                    .zmk_behaviors
                                    .iter()
                                    .find(|beh| beh.id == b.behavior_id as u32)
                                    .map(|beh| beh.display_name == "Transparent")
                                    .unwrap_or(false)
                            });
                        if let Some(fb) = fallback {
                            zmk_binding_label(&fb, &layout.zmk_behaviors, &self.layer_names)
                        } else {
                            "▽".to_string()
                        }
                    } else {
                        zmk_binding_label(&binding, &layout.zmk_behaviors, &self.layer_names)
                    }
                } else {
                    let kc = layout.get_keycode(layer, press_ki);
                    if kc == 0x0001 && !is_hovering {
                        let fallback_kc = (0..layer)
                            .rev()
                            .map(|l| layout.get_keycode(l, press_ki))
                            .find(|&k| k != 0x0001)
                            .unwrap_or(0x0000);
                        if fallback_kc == 0x0000 || fallback_kc == 0x0001 {
                            "▽".to_string()
                        } else {
                            keycode_label_with_macro_names(
                                fallback_kc,
                                &layout.custom_keycodes,
                                &self.layer_names,
                                &self.keycode_picker.macro_names,
                                &self.keycode_picker.tap_dance_names,
                            )
                        }
                    } else if kc == 0x0001 {
                        "▽".to_string()
                    } else if kc == 0x0000 {
                        "✕".to_string()
                    } else {
                        keycode_label_with_macro_names(
                            kc,
                            &layout.custom_keycodes,
                            &self.layer_names,
                            &self.keycode_picker.macro_names,
                            &self.keycode_picker.tap_dance_names,
                        )
                    }
                }
                .replace('\n', " ");
                let press_font = FontId::proportional(if press_label.chars().count() > 8 {
                    7.2
                } else {
                    8.2
                });
                painter.with_clip_rect(press_text_rect).text(
                    press_text_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    press_label,
                    press_font,
                    text_color,
                );
            } else {
                let divider_extend = 1.5;
                let divider_half_width = radius + divider_extend;
                painter.line_segment(
                    [
                        egui::pos2(center.x - divider_half_width, center.y),
                        egui::pos2(center.x + divider_half_width, center.y),
                    ],
                    Stroke::new(1.0, outline.color),
                );
            }
        }

        self.prev_hovered_key = hovered_key;
        self.prev_hovered_encoder = hovered_encoder;
        self.prev_hovered_encoder_keycode = hovered_encoder_keycode;

        if layout_h > avail.y {
            ui.allocate_space(Vec2::new(0.0, (layout_h - avail.y).max(0.0)));
        }
    }
}

fn draw_key_label_dimmed(painter: &egui::Painter, rect: egui::Rect, label: &str, dark: bool) {
    let dim = if dark {
        Color32::from_rgb(60, 60, 65)
    } else {
        Color32::from_rgb(200, 200, 208)
    };
    let dim_top = if dark {
        Color32::from_rgb(45, 45, 50)
    } else {
        Color32::from_rgb(215, 215, 220)
    };
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos + 1..])
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);

    if let Some(top_str) = top {
        let center = rect.center();
        painter.text(
            egui::pos2(center.x, center.y - 7.0),
            egui::Align2::CENTER_CENTER,
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            dim_top,
        );
        painter.text(
            egui::pos2(center.x, center.y + 6.0),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(bottom_size),
            dim,
        );
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(font_size),
            dim,
        );
    }
}

fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    let a = (color.a() as f32 * alpha.clamp(0.0, 1.0)) as u8;
    Color32::from_rgba_premultiplied(
        (color.r() as f32 * alpha.clamp(0.0, 1.0)) as u8,
        (color.g() as f32 * alpha.clamp(0.0, 1.0)) as u8,
        (color.b() as f32 * alpha.clamp(0.0, 1.0)) as u8,
        a,
    )
}

fn draw_key_label_alpha(
    painter: &egui::Painter,
    rect: egui::Rect,
    label: &str,
    dark: bool,
    alpha: f32,
) {
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos + 1..])
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);
    let top_color = with_alpha(
        if dark {
            Color32::from_rgb(130, 130, 145)
        } else {
            Color32::from_rgb(130, 130, 150)
        },
        alpha,
    );
    let main_color = with_alpha(
        if dark {
            Color32::from_rgb(232, 232, 240)
        } else {
            Color32::from_rgb(26, 26, 30)
        },
        alpha,
    );
    if let Some(top_str) = top {
        let center = rect.center();
        painter.text(
            egui::pos2(center.x, center.y - 7.0),
            egui::Align2::CENTER_CENTER,
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            top_color,
        );
        painter.text(
            egui::pos2(center.x, center.y + 6.0),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(bottom_size),
            main_color,
        );
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(font_size),
            main_color,
        );
    }
}

fn draw_key_label_dimmed_alpha(
    painter: &egui::Painter,
    rect: egui::Rect,
    label: &str,
    dark: bool,
    alpha: f32,
) {
    let dim = with_alpha(
        if dark {
            Color32::from_rgb(80, 80, 90)
        } else {
            Color32::from_rgb(180, 180, 195)
        },
        alpha,
    );
    let dim_top = with_alpha(
        if dark {
            Color32::from_rgb(60, 60, 70)
        } else {
            Color32::from_rgb(190, 190, 205)
        },
        alpha,
    );
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos + 1..])
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);
    if let Some(top_str) = top {
        let center = rect.center();
        painter.text(
            egui::pos2(center.x, center.y - 7.0),
            egui::Align2::CENTER_CENTER,
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            dim_top,
        );
        painter.text(
            egui::pos2(center.x, center.y + 6.0),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(bottom_size),
            dim,
        );
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(font_size),
            dim,
        );
    }
}

fn draw_key_label(painter: &egui::Painter, rect: egui::Rect, label: &str, dark: bool) {
    // Split on "\n" first, then on "/" — show top part small+dim, bottom part normal
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        let t = &label[..pos];
        let b = &label[pos + 1..];
        (Some(t), b)
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);

    if let Some(top_str) = top {
        // Two-line layout
        let center = rect.center();
        let top_pos = egui::pos2(center.x, center.y - 7.0);
        let bot_pos = egui::pos2(center.x, center.y + 6.0);

        let top_color = if dark {
            Color32::from_rgb(130, 130, 145)
        } else {
            Color32::from_rgb(130, 130, 150)
        };
        let main_color = if dark {
            Color32::from_rgb(232, 232, 240)
        } else {
            Color32::from_rgb(26, 26, 30)
        };
        painter.text(
            top_pos,
            egui::Align2::CENTER_CENTER,
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            top_color,
        );
        painter.text(
            bot_pos,
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(bottom_size),
            main_color,
        );
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(font_size),
            if dark {
                Color32::from_rgb(232, 232, 240)
            } else {
                Color32::from_rgb(26, 26, 30)
            },
        );
    }
}

impl EntropyApp {
    fn draw_placeholder(&mut self, ui: &mut egui::Ui) {
        let key_w = 52.0_f32;
        let key_h = 52.0_f32;
        let gap = 6.0_f32;
        let total_w = 6.0 * (key_w + gap);
        let start_x = (ui.available_width() - total_w * 2.0 - 40.0) / 2.0;
        let start_y = ui.min_rect().top() + 40.0;

        let mut keys: Vec<(usize, egui::Rect, egui::Response)> = vec![];
        for half in 0..2_usize {
            let half_offset = if half == 0 { 0.0 } else { total_w + 40.0 };
            for row in 0..4_usize {
                for col in 0..6_usize {
                    let key_idx = half * 24 + row * 6 + col;
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(
                            start_x + half_offset + col as f32 * (key_w + gap),
                            start_y + row as f32 * (key_h + gap),
                        ),
                        Vec2::new(key_w, key_h),
                    );
                    let response = ui.allocate_rect(rect, Sense::click());
                    keys.push((key_idx, rect, response));
                }
            }
        }

        for (key_idx, _, response) in &keys {
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                self.selected_key = Some((self.selected_layer, *key_idx));
            }
        }

        let painter = ui.painter();
        for (key_idx, rect, _) in &keys {
            let is_selected = self.selected_key == Some((self.selected_layer, *key_idx));
            let bg = if is_selected {
                Color32::from_rgb(70, 110, 190)
            } else {
                Color32::from_gray(45)
            };
            painter.rect(
                *rect,
                6.0,
                bg,
                Stroke::new(1.0, Color32::from_gray(80)),
                egui::StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("K{key_idx}"),
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
    }
}
