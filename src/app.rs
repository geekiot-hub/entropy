use crate::device::DeviceManager;
use crate::firmware::FirmwareProtocol;
use crate::zmk::{zmk_binding_label, zmk_binding_tooltip, ZmkBinding};

/// Sanitize a device name into a filesystem-safe slug.
fn device_id_slug(device_name: &str) -> String {
    device_name
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
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

fn load_layer_names(device_name: &str) -> Vec<String> {
    let path = layer_names_path(device_name);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(mut v) = serde_json::from_str::<Vec<String>>(&data) {
            if !v.is_empty() {
                // Pad to at least 16 so indexing is always safe
                while v.len() < 16 {
                    let n = v.len();
                    v.push(n.to_string());
                }
                return v;
            }
        }
    }
    let mut v: Vec<String> = (0..16).map(|i| i.to_string()).collect();
    v[0] = "Main".to_string();
    v
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
    macro_names.get(idx)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn macro_display_name(macro_names: &[String], idx: usize) -> String {
    macro_custom_name(macro_names, idx).unwrap_or_else(|| format!("M{}", idx))
}

fn tap_dance_custom_name(tap_dance_names: &[String], idx: usize) -> Option<String> {
    tap_dance_names.get(idx)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn tap_dance_display_name(tap_dance_names: &[String], idx: usize) -> String {
    tap_dance_custom_name(tap_dance_names, idx).unwrap_or_else(|| format!("TD{}", idx))
}

fn app_accent() -> Color32 { crate::ui_style::accent() }
fn app_panel_fill(dark: bool) -> Color32 { crate::ui_style::panel_fill(dark) }
fn app_window_fill(dark: bool) -> Color32 { crate::ui_style::window_fill(dark) }
fn app_surface_fill(dark: bool) -> Color32 { crate::ui_style::surface_fill(dark) }
fn app_hover_fill(dark: bool) -> Color32 { crate::ui_style::hover_fill(dark) }
fn app_border_color(dark: bool) -> Color32 { crate::ui_style::border_color(dark) }
fn app_muted_text(dark: bool) -> Color32 { crate::ui_style::muted_text(dark) }

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
    /// Key Override entries
    key_override_entries: Vec<KeyOverrideEntry>,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyOverridePickField {
    Trigger,
    Replacement,
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
    /// Undo stack: each entry is (layer, key_idx, old_vial_kc, old_zmk_binding)
    undo_stack: Vec<(usize, usize, u16, ZmkBinding)>,
    /// Frame counter for periodic device scan
    scan_frame: u32,
    /// Layer to preview on hover (None = show selected_layer)
    hover_layer: Option<usize>,
    /// Key index hovered in previous frame (for hint display)
    prev_hovered_key: Option<usize>,
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
            secondary_click_handled: false,
            pending_handed_swap: None,
            hover_layer_progress: 0.0,
            jump_back_stack: Vec::new(),
            device_manager: DeviceManager::new(),
            selected_device: None,
            selected_layer: 0,
            selected_key: None,
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
                        let dev_conn = HidDevice::open(&dev.path)
                            .map_err(|e| format!("Open failed: {e}"))?;

                        log::info!("Getting protocol version…");
                        match dev_conn.get_protocol_version() {
                            Ok(v) => log::info!("VIA protocol version: {v}"),
                            Err(e) => log::warn!("get_protocol_version failed: {e}"),
                        }

                        log::info!("Getting layer count…");
                        let layer_count = dev_conn.get_layer_count()
                            .map(|c| c as usize)
                            .unwrap_or_else(|e| { log::warn!("get_layer_count failed: {e}, defaulting to 4"); 4 });
                        log::info!("Layer count: {layer_count}");

                        log::info!("Getting layout JSON…");
                        let json = dev_conn.get_layout_json()
                            .map_err(|e| format!("Layout read failed: {e}"))?;

                        let mut layout = KeyboardLayout::from_vial_json(&json)
                            .map_err(|e| format!("Layout parse failed: {e}"))?;

                        // Override coords from embedded layout
                        log::info!("Vial: looking up embedded layout for '{}'", dev.name);
                        if let Some((emb, ref_keys)) = crate::layouts::lookup_layout(&dev.name) {
                            log::info!("Vial: found embedded layout '{}' with {} keys", emb.name, ref_keys.len());
                            use std::collections::HashMap;
                            let ref_map: HashMap<(u8,u8), &crate::keyboard::PhysicalKey> =
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

                        // Read macros
                        let macro_texts = match dev_conn.get_macro_count() {
                            Ok(count) => {
                                log::info!("Macro count: {count}");
                                match dev_conn.get_macro_buffer_size() {
                                    Ok(size) => {
                                        log::info!("Macro buffer size: {size}");
                                        match dev_conn.get_macro_buffer(size) {
                                            Ok(buf) => crate::hid::HidDevice::parse_macros(&buf, count),
                                            Err(e) => { log::warn!("get_macro_buffer: {e}"); vec![String::new(); count as usize] }
                                        }
                                    }
                                    Err(e) => { log::warn!("get_macro_buffer_size: {e}"); vec![String::new(); count as usize] }
                                }
                            }
                            Err(e) => { log::warn!("get_macro_count: {e}"); vec![String::new(); 16] }
                        };

                        let combo_entries = match dev_conn.get_combo_count() {
                            Ok(count) => {
                                log::info!("Combo count: {count}");
                                let mut entries = Vec::new();
                                for i in 0..count {
                                    match dev_conn.get_combo(i) {
                                        Ok((keys, output)) => entries.push(ComboEntry { keys, output }),
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

                        // Read tap dance entries
                        let tap_dance_entries = match dev_conn.get_tap_dance_count() {
                            Ok(count) => {
                                log::info!("Tap dance count: {count}");
                                let mut entries = Vec::new();
                                for i in 0..count {
                                    match dev_conn.get_tap_dance(i) {
                                        Ok((tap, hold, dtap, taphold, term)) => {
                                            entries.push(crate::keycode_picker::TapDanceEntry {
                                                on_tap: tap, on_hold: hold, on_double_tap: dtap,
                                                on_tap_hold: taphold, tapping_term: term,
                                            });
                                        }
                                        Err(e) => { log::warn!("get_tap_dance({i}): {e}"); entries.push(Default::default()); }
                                    }
                                }
                                entries
                            }
                            Err(e) => { log::warn!("get_tap_dance_count: {e}"); vec![] }
                        };
                        let key_override_entries = match dev_conn.get_key_override_count() {
                            Ok(count) => {
                                log::info!("Key Override count: {count}");
                                let mut entries = Vec::new();
                                for i in 0..count {
                                    match dev_conn.get_key_override(i) {
                                        Ok((trigger, replacement, layers, trigger_mods, negative_mod_mask, suppressed_mods, options)) => {
                                            entries.push(KeyOverrideEntry {
                                                trigger,
                                                replacement,
                                                layers,
                                                trigger_mods,
                                                negative_mod_mask,
                                                suppressed_mods,
                                                options: KeyOverrideOptionsState::from_bits(options),
                                            });
                                        }
                                        Err(e) => { log::warn!("get_key_override({i}): {e}"); entries.push(Default::default()); }
                                    }
                                }
                                entries
                            }
                            Err(e) => { log::warn!("get_key_override_count: {e}"); vec![] }
                        };

                        Ok(ConnectResult {
                            device_name: dev.name.clone(),
                            macro_texts,
                            tap_dance_entries,
                            combo_entries,
                            combo_term,
                            auto_shift_options: auto_shift_options.unwrap_or_default(),
                            auto_shift_timeout,
                            key_override_entries,
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
                        let lock_state = conn.get_lock_state()
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
                                layers: vec![],
                                custom_keycodes: vec![],
                                supports_rgb: false,
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
                                key_override_entries: vec![],
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
                        let keymap = conn.get_keymap()
                            .map_err(|e| format!("ZMK keymap failed: {e}"))?;

                        log::info!("Fetching ZMK physical layouts…");
                        let phys = conn.get_physical_layouts()
                            .map_err(|e| format!("ZMK layouts failed: {e}"))?;

                        let layer_count = keymap.layers.len();
                        log::info!("ZMK: {} layers", layer_count);

                        // Build layout from device physical layout
                        let mut layout = crate::layouts::build_layout_from_zmk(&phys, &keymap);
                        layout.firmware = FirmwareProtocol::Zmk;
                        layout.zmk_behaviors = conn.behaviors.clone();

                        // Extract bindings
                        layout.zmk_layer_ids = keymap.layers.iter().map(|l| l.id).collect();
                        layout.zmk_layer_names = keymap.layers.iter().map(|l| l.name.clone()).collect();

                        let num_keys = layout.keys.len();
                        layout.zmk_bindings = keymap.layers.iter().map(|layer| {
                            let mut bindings = vec![crate::zmk::ZmkBinding::none(); num_keys];
                            for (i, b) in layer.bindings.iter().enumerate() {
                                if i < num_keys {
                                    bindings[i] = crate::zmk::ZmkBinding::from_proto(b);
                                }
                            }
                            bindings
                        }).collect();

                        Ok(ConnectResult {
                            device_name: dev.name.clone(),
                            macro_texts: vec![],
                            tap_dance_entries: vec![],
                            combo_entries: vec![],
                            combo_term: None,
                            auto_shift_options: AutoShiftOptionsState::default(),
                            auto_shift_timeout: None,
                            key_override_entries: vec![],
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
                self.key_override_names = load_key_override_names(&self.current_device_name);
                self.key_override_names.resize(self.key_override_entries.len(), String::new());
                self.key_override_visible_count = 1;
                self.key_override_undo_stack.clear();
                self.selected_key_override = 0;
                self.combo_names = load_combo_names(&self.current_device_name);
                self.combo_names.resize(self.combo_entries.len(), String::new());
                self.combo_term = r.combo_term.or(Some(50));
                self.auto_shift_options = r.auto_shift_options;
                self.auto_shift_timeout = r.auto_shift_timeout;
                let highest_used_combo = self
                    .combo_entries
                    .iter()
                    .enumerate()
                    .filter(|(i, combo)| {
                        combo.output != 0
                            || combo.keys.iter().any(|&k| k != 0)
                            || self.combo_names.get(*i).map(|n| !n.trim().is_empty()).unwrap_or(false)
                    })
                    .map(|(i, _)| i + 1)
                    .max()
                    .unwrap_or(1);
                self.combo_visible_count = highest_used_combo.min(self.combo_entries.len().max(1));
                self.selected_combo = self.selected_combo.min(self.combo_visible_count.saturating_sub(1));
                if !r.macro_texts.is_empty() {
                    self.keycode_picker.macro_count = r.macro_texts.len();
                    self.keycode_picker.macro_texts = r.macro_texts.clone();
                    self.keycode_picker.macro_names = vec![String::new(); r.macro_texts.len()];
                    // Parse macro texts into actions
                    // Parse macro texts → actions (Vial protocol v2: prefix 0x01 before actions)
                    self.keycode_picker.macro_actions = r.macro_texts.iter().map(|text| {
                        let bytes = text.as_bytes();
                        let mut actions = Vec::new();
                        let mut i = 0;
                        while i < bytes.len() {
                            if bytes[i] == 1 && i + 1 < bytes.len() {
                                // SS_QMK_PREFIX
                                match bytes[i + 1] {
                                    1 if i + 2 < bytes.len() => { // SS_TAP
                                        actions.push(crate::keycode_picker::MacroAction::Tap(bytes[i+2]));
                                        i += 3;
                                    }
                                    2 if i + 2 < bytes.len() => { // SS_DOWN
                                        actions.push(crate::keycode_picker::MacroAction::Down(bytes[i+2]));
                                        i += 3;
                                    }
                                    3 if i + 2 < bytes.len() => { // SS_UP
                                        actions.push(crate::keycode_picker::MacroAction::Up(bytes[i+2]));
                                        i += 3;
                                    }
                                    4 if i + 3 < bytes.len() => { // SS_DELAY
                                        let ms = (bytes[i+2] as u16 - 1) + (bytes[i+3] as u16 - 1) * 255;
                                        actions.push(crate::keycode_picker::MacroAction::Delay(ms));
                                        i += 4;
                                    }
                                    _ => { i += 2; } // skip unknown
                                }
                            } else {
                                // Text character
                                let start = i;
                                while i < bytes.len() && bytes[i] != 1 { i += 1; }
                                if let Ok(s) = std::str::from_utf8(&bytes[start..i]) {
                                    actions.push(crate::keycode_picker::MacroAction::Text(s.to_string()));
                                }
                            }
                        }
                        actions
                    }).collect();
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
                // For ZMK, use device layer names if available; otherwise load from file
                if !r.layout.zmk_layer_names.is_empty() {
                    self.layer_names = r.layout.zmk_layer_names.clone();
                    while self.layer_names.len() < 16 {
                        let n = self.layer_names.len();
                        self.layer_names.push(n.to_string());
                    }
                } else {
                    self.layer_names = load_layer_names(&device_name);
                }

                // Populate picker based on firmware
                self.keycode_picker.firmware = self.firmware;
                self.keycode_picker.supports_rgb = r.layout.supports_rgb;
                self.keycode_picker.layer_count = r.layout.layers.len().max(1);
                self.keycode_picker.tap_dance_names = load_tap_dance_names(&device_name);
                if self.firmware == FirmwareProtocol::Vial {
                    const USER_BASE: u16 = 0x7E40;
                    self.keycode_picker.custom_keycodes = r.layout.custom_keycodes.iter().enumerate()
                        .map(|(i, custom)| (custom.name.clone(), custom.label.clone(), custom.title.clone(), USER_BASE + i as u16))
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
                    if let Some(dev) = self.selected_device.and_then(|i| self.device_manager.devices().get(i)) {
                        match crate::hid::HidDevice::open(&dev.path) {
                            Ok(v) => { self.hid_device = Some(v); }
                            Err(e) => log::warn!("Could not open persistent HID: {e}"),
                        }
                    }
                }

                log::info!("Connected: {} ({} layers, {:?})", r.device_name, r.layer_count, self.firmware);
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
            ZmkOpResult::AddLayerOk { layer_idx, layer_name } => {
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
                    self.status_msg = "Firmware doesn't support extra layers (CONFIG_ZMK_KEYMAP_LAYERS_EXTRA)".to_string();
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
            self.keycode_picker.layer_has_content = layout.layers.iter()
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
            && self.hid_device.as_ref()
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
        if let Some(dev) = self.selected_device.and_then(|i| self.device_manager.devices().get(i)) {
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
        if now.duration_since(self.matrix_tester_last_poll) >= std::time::Duration::from_millis(50) {
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

    fn draw_settings_screen(&mut self, ui: &mut egui::Ui, layout: &KeyboardLayout, ctx: &egui::Context, content_top: f32) {
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_matrix_tester(ctx, layout);

        if let Some(id) = ctx.memory(|m| m.focused()) {
            ctx.memory_mut(|m| m.surrender_focus(id));
        }

        let dark = ui.visuals().dark_mode;
        let content_rect = egui::Rect::from_min_max(
            egui::pos2(ui.min_rect().left() + 20.0, content_top),
            egui::pos2(ui.min_rect().right() - 20.0, ui.max_rect().bottom() - 16.0),
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

        if supported && hid_ready && self.is_vial_locked() && !self.unlock_open && !self.matrix_tester_unlock_prompted {
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
                self.matrix_tester_ever_pressed.get(idx).copied().unwrap_or(false)
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
        if ui.put(reset_rect, egui::Button::new("Reset").sense(egui::Sense::CLICK)).clicked() {
            self.reset_matrix_tester_state();
        }

        let painter = ui.painter();
        let idle_fill = if dark { Color32::from_rgb(34, 34, 38) } else { Color32::from_rgb(252, 252, 254) };
        let tested_fill = if dark { Color32::from_rgb(42, 68, 52) } else { Color32::from_rgb(232, 245, 236) };

        let board_top = top_line_y + 48.0;
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
        if min_x == f32::MAX {
            return;
        }

        let span_x = max_x - min_x;
        let span_y = max_y - min_y;
        let margin = 40.0_f32;
        let scale_x = (board_rect.width() - margin) / (span_x * base_unit).max(1.0);
        let scale_y = (content_rect.height() - margin) / (span_y * base_unit).max(1.0);
        let scale = scale_x.min(scale_y).min(1.0);
        let unit = base_unit * scale;
        let layout_w = span_x * unit;
        let layout_h = span_y * unit;
        let offset_x = board_rect.center().x - layout_w / 2.0 - min_x * unit;
        let offset_y = board_rect.center().y - layout_h / 2.0 - min_y * unit;

        for key in &layout.keys {
            let matrix_idx = key.row as usize * layout.cols + key.col as usize;
            let is_pressed = self.matrix_tester_pressed.get(matrix_idx).copied().unwrap_or(false);
            let was_pressed = self.matrix_tester_ever_pressed.get(matrix_idx).copied().unwrap_or(false);
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
                    self.combo_window_open = true;
                    self.combo_reopen_after_pick = false;
                }
            } else if let Some(field) = self.key_override_pick_target.take() {
                let idx = self.selected_key_override.min(self.key_override_entries.len().saturating_sub(1));
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
                    self.key_override_window_open = true;
                    self.key_override_reopen_after_pick = false;
                }
            } else if let Some((layer, ki)) = self.selected_key {
                #[cfg(not(target_arch = "wasm32"))]
                self.assign_keycode(layer, ki, kc_value);
                #[cfg(target_arch = "wasm32")]
                if let Some(layout) = &mut self.layout {
                    layout.set_keycode(layer, ki, kc_value);
                }
            }
            self.selected_key = None;
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(binding) = self.keycode_picker.zmk_result.take() {
            if let Some((layer, ki)) = self.selected_key {
                self.assign_zmk_binding(layer, ki, binding);
            }
            self.selected_key = None;
        }
    }

    fn assign_keycode(&mut self, layer: usize, ki: usize, kc_value: u16) {
        // Save old value for undo
        let old_kc = self.layout.as_ref().map(|l| l.get_keycode(layer, ki)).unwrap_or(0);
        self.undo_stack.push((layer, ki, old_kc, ZmkBinding::none()));
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
        } else if let Some(dev) = self.selected_device
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
        let layer_id = self.layout.as_ref()
            .and_then(|l| l.zmk_layer_ids.get(layer).copied())
            .unwrap_or(layer as u32);

        // Save old binding for undo
        let old_binding = self.layout.as_ref().map(|l| l.get_zmk_binding(layer, ki)).unwrap_or_else(ZmkBinding::none);
        self.undo_stack.push((layer, ki, 0, old_binding));

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
        let Some((layer, ki, old_kc, old_binding)) = self.undo_stack.pop() else { return };
        if self.firmware == FirmwareProtocol::Zmk {
            self.assign_zmk_binding(layer, ki, old_binding);
            // Remove the undo entry that assign_zmk_binding just pushed
            self.undo_stack.pop();
        } else {
            self.assign_keycode(layer, ki, old_kc);
            // Remove the undo entry that assign_keycode just pushed
            self.undo_stack.pop();
        }
    }

    fn zmk_save(&mut self) {
        if self.zmk_op_rx.is_some() { return; } // operation in progress
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
        if self.zmk_op_rx.is_some() { return; }
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
        if self.zmk_op_rx.is_some() { return; }
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
                    let _ = tx.send(ZmkOpResult::AddLayerOk { layer_idx: idx, layer_name: name });
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
        if self.zmk_op_rx.is_some() { return; }
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
            && self.selected_key.is_some()
            && self.keycode_picker.result.is_none()
            && self.keycode_picker.zmk_result.is_none()
        {
            self.selected_key = None;
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
                    if ui.button("↩ Undo").on_hover_text("Undo last change").clicked() {
                        self.undo();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {


                        // Vial: Unlock button (don't poll status during unlock process)
                        if self.firmware == FirmwareProtocol::Vial && self.layout.is_some() && !self.vial_unlock_polling && !self.unlock_open {
                            let is_unlocked = self.hid_device.as_ref()
                                .and_then(|hid| hid.get_unlock_status().ok())
                                .map(|(unlocked, _keys)| unlocked)
                                .unwrap_or(false);
                            if !is_unlocked {
                                if ui.add(egui::Button::new(RichText::new("🔒 Unlock")
                                    .color(Color32::from_rgb(220, 120, 60)))
                                    .fill(Color32::TRANSPARENT))
                                    .on_hover_text("Keyboard is locked — click to unlock")
                                    .clicked()
                                {
                                    self.unlock_open = true;
                                }
                            } else {
                                if ui.add(egui::Button::new(RichText::new("🔓 Lock"))
                                    .fill(Color32::TRANSPARENT))
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
                        } else if self.status_msg.contains("error") || self.status_msg.contains("failed") {
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
                        ui.label(RichText::new("Loading keyboard…").size(16.0).color(Color32::GRAY));
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
                let theme_label = if self.dark_mode { "☀ Light" } else { "🌙 Dark" };
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
                    ui.painter().rect_filled(screen, 0.0, Color32::from_rgba_unmultiplied(0, 0, 0, 180));

                    let center_x = screen.center().x;
                    let top_y = screen.min.y + 40.0;

                    // Title
                    ui.painter().text(egui::pos2(center_x, top_y), egui::Align2::CENTER_CENTER,
                        "🔓 Unlock Keyboard", FontId::proportional(24.0), Color32::WHITE);

                    ui.painter().text(egui::pos2(center_x, top_y + 30.0), egui::Align2::CENTER_CENTER,
                        "Press and hold the highlighted keys one by one", FontId::proportional(14.0), Color32::from_gray(180));

                    // Progress bar
                    let progress = if total > 0 { 1.0 - (counter as f32 / total as f32) } else { 0.0 };
                    let bar_w = 300.0f32;
                    let bar_h = 12.0f32;
                    let bar_y = top_y + 55.0;
                    let bar_rect = egui::Rect::from_min_size(
                        egui::pos2(center_x - bar_w / 2.0, bar_y),
                        egui::Vec2::new(bar_w, bar_h),
                    );
                    ui.painter().rect(bar_rect, 4.0, Color32::from_gray(40), egui::Stroke::NONE, egui::StrokeKind::Inside);
                    let fill_rect = egui::Rect::from_min_size(
                        bar_rect.min,
                        egui::Vec2::new(bar_w * progress, bar_h),
                    );
                    ui.painter().rect(fill_rect, 4.0, Color32::from_rgb(91, 104, 223), egui::Stroke::NONE, egui::StrokeKind::Inside);

                    // Draw layout keys with highlighted unlock keys
                    if let Some(layout) = &self.layout {
                        let base_unit = 54.0f32 * 0.85;
                        let padding = 3.0f32;
                        let mut min_x = f32::MAX; let mut min_y = f32::MAX;
                        let mut max_x = f32::MIN; let mut max_y = f32::MIN;
                        for key in &layout.keys {
                            min_x = min_x.min(key.x); min_y = min_y.min(key.y);
                            max_x = max_x.max(key.x + key.w); max_y = max_y.max(key.y + key.h);
                        }
                        let span_x = max_x - min_x; let span_y = max_y - min_y;
                        let avail_w = screen.width() - 80.0;
                        let avail_h = screen.height() - 160.0;
                        let scale = (avail_w / (span_x * base_unit)).min(avail_h / (span_y * base_unit)).min(1.0);
                        let unit = base_unit * scale;
                        let layout_w = span_x * unit;
                        let layout_h = span_y * unit;
                        let off_x = center_x - layout_w / 2.0 - min_x * unit;
                        let off_y = bar_y + 30.0 + (avail_h - layout_h) / 2.0 - min_y * unit;

                        for (ki, key) in layout.keys.iter().enumerate() {
                            let is_unlock = unlock_keys.iter().any(|(r, c)| key.row == *r && key.col == *c);
                            let rect = egui::Rect::from_min_size(
                                egui::pos2(off_x + key.x * unit + padding, off_y + key.y * unit + padding),
                                egui::Vec2::new(key.w * unit - padding * 2.0, key.h * unit - padding * 2.0),
                            );
                            let bg = if is_unlock {
                                Color32::from_rgb(91, 104, 223)
                            } else {
                                Color32::from_rgba_unmultiplied(48, 48, 52, 120)
                            };
                            let border = if is_unlock { Color32::from_rgb(120, 130, 255) } else { Color32::from_gray(60) };
                            ui.painter().rect(rect, 5.0, bg, Stroke::new(1.0, border), egui::StrokeKind::Inside);

                            let kc = layout.get_keycode(0, ki);
                            let label = keycode_label_with_macro_names(kc, &layout.custom_keycodes, &self.layer_names, &self.keycode_picker.macro_names, &self.keycode_picker.tap_dance_names);
                            let text_color = if is_unlock { Color32::WHITE } else { Color32::from_gray(80) };
                            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, &label,
                                FontId::proportional(9.0 * scale), text_color);
                        }
                    }

                });
        }

        let any_floating_window_open = self.combo_window_open
            || self.auto_shift_window_open
            || self.key_override_window_open
            || self.keycode_picker.open;
        if any_floating_window_open {
            let screen_rect = ctx.screen_rect();
            egui::Area::new("window_backdrop".into())
                .order(egui::Order::Middle)
                .fixed_pos(screen_rect.min)
                .show(ctx, |ui| {
                    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, screen_rect.size());
                    let response = ui.interact(rect, ui.id().with("backdrop_click"), egui::Sense::click());
                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        Color32::from_black_alpha(crate::ui_style::modal_backdrop_alpha(ctx.style().visuals.dark_mode)),
                    );
                    if response.clicked() {
                        self.combo_window_open = false;
                        self.auto_shift_window_open = false;
                        self.key_override_window_open = false;
                        self.keycode_picker.open = false;
                    }
                });
        }

        if self.combo_window_open {
            self.show_combo_window(ctx);
        }
        if self.auto_shift_window_open {
            self.show_auto_shift_window(ctx);
        }
        if self.key_override_window_open {
            self.show_key_override_window(ctx);
        }

        if !self.unlock_open && !self.vial_unlock_polling {
            self.keycode_picker.show(ctx);
            self.apply_picker_results();
        }
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
                        let buf = crate::hid::HidDevice::encode_macros(&self.keycode_picker.macro_texts, size);
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
                    match hid.set_tap_dance(i as u8, td.on_tap, td.on_hold, td.on_double_tap, td.on_tap_hold, td.tapping_term) {
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
                save_tap_dance_names(&self.keycode_picker.tap_dance_names, &self.current_device_name);
                self.keycode_picker.tap_dance_dirty = false;
                if self.status_msg.is_empty() || self.status_msg.starts_with("✓") {
                    self.status_msg = "✓ Tap dance saved".into();
                }
            }
        }

        // Right-click anywhere = pop back one step (only if NOT hovering a layer key and not handled by key)
        if !self.jump_back_stack.is_empty() && !self.keycode_picker.open && !self.secondary_click_handled {
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
                            RichText::new("Press the unlock key on your keyboard to allow editing.")
                                .size(14.0),
                        );
                    } else {
                        ui.label(
                            RichText::new("Hold the unlock combo on your keyboard to allow editing.")
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
        let Some(entry) = self.key_override_entries.get_mut(idx) else { return; };
        Self::normalize_key_override_entry(entry);
        let entry = entry.clone();
        let Some(hid) = &self.hid_device else { return; };
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

    fn write_auto_shift_flags(&mut self) {
        let Some(hid) = &self.hid_device else { return; };
        if let Err(e) = hid.set_qmk_setting_u8(3, self.auto_shift_options.bits()) {
            self.status_msg = format!("Failed to save Auto Shift flags: {}", e);
            log::warn!("set_qmk_setting_u8(auto_shift_flags) failed: {e}");
        }
    }

    fn write_auto_shift_timeout(&mut self) {
        let Some(hid) = &self.hid_device else { return; };
        let Some(timeout) = self.auto_shift_timeout else { return; };
        if let Err(e) = hid.set_qmk_setting_u16(4, timeout) {
            self.status_msg = format!("Failed to save Auto Shift timeout: {}", e);
            log::warn!("set_qmk_setting_u16(auto_shift_timeout) failed: {e}");
        }
    }

    fn draw_key_override_layers(ui: &mut egui::Ui, layers: &mut u16) -> bool {
        let mut changed = false;
        egui::Grid::new(ui.id().with("ko_layers_grid")).num_columns(6).spacing([10.0, 6.0]).show(ui, |ui| {
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
        egui::Grid::new(ui.id().with(id)).num_columns(2).spacing([18.0, 4.0]).show(ui, |ui| {
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

    fn show_auto_shift_window(&mut self, ctx: &egui::Context) {
        let mut open = self.auto_shift_window_open;
        let dark = ctx.style().visuals.dark_mode;
        let style = ctx.style().as_ref().clone();
        let frame = crate::ui_style::modal_window_frame(&style, dark);

        egui::Window::new("Auto Shift")
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

        self.auto_shift_window_open = open;
    }

    fn show_key_override_window(&mut self, ctx: &egui::Context) {
        if !self.keycode_picker.open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.key_override_window_open = false;
            return;
        }

        let dark = ctx.style().visuals.dark_mode;
        let mut open = self.key_override_window_open;
        egui::Window::new("Key Overrides")
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
                    if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                        if matches!(key, egui::Key::Enter | egui::Key::Escape) {
                            continue;
                        }
                        if let Some(kc) = egui_key_to_qmk(key, modifiers) {
                            if !self.combo_capture_keys.contains(&kc) && self.combo_capture_keys.len() < 4 {
                                self.combo_capture_keys.push(kc);
                            }
                        }
                    }
                }
            }
        }

        let mut open = self.combo_window_open;
        egui::Window::new("Combo")
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
                    .inner_margin(egui::Margin::same(10))
            )
            .show(ctx, |ui| {
                ui.style_mut().visuals.button_frame = true;
                if ui.visuals().dark_mode {
                    ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(48, 48, 58);
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(48, 48, 58);
                    ui.style_mut().visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_gray(110));
                    ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(68, 68, 88);
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(68, 68, 88);
                    ui.style_mut().visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(130, 130, 160));
                    ui.style_mut().visuals.widgets.active.bg_fill = Color32::from_rgb(78, 78, 102);
                    ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(78, 78, 102);
                    ui.style_mut().visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(150, 150, 184));
                } else {
                    ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(255, 255, 255);
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(255, 255, 255);
                    ui.style_mut().visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(222, 222, 228));
                    ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(234, 232, 242);
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(234, 232, 242);
                    ui.style_mut().visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(210, 206, 223));
                    ui.style_mut().visuals.widgets.active.bg_fill = Color32::from_rgb(228, 225, 238);
                    ui.style_mut().visuals.widgets.active.weak_bg_fill = Color32::from_rgb(228, 225, 238);
                    ui.style_mut().visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(202, 198, 216));
                }

                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("Press multiple keys together to send a separate keycode.")
                            .size(12.0)
                            .color(Color32::from_gray(150)),
                    );
                });

                if self.firmware != FirmwareProtocol::Vial {
                    ui.label(RichText::new("Dynamic combos are not supported for this firmware.").color(Color32::from_gray(140)));
                    return;
                }

                if self.combo_entries.is_empty() {
                    ui.label(RichText::new("This keyboard does not report any dynamic combo slots.").color(Color32::from_gray(140)));
                    return;
                }

                self.selected_combo = self.selected_combo.min(self.combo_entries.len().saturating_sub(1));
                self.combo_names.resize(self.combo_entries.len(), String::new());

                let combo_undo_snapshot = (
                    self.combo_entries.clone(),
                    self.combo_names.clone(),
                    self.combo_term,
                    self.selected_combo,
                    self.combo_visible_count,
                );

                self.combo_visible_count = self.combo_visible_count.clamp(1, self.combo_entries.len().max(1));
                self.selected_combo = self.selected_combo.min(self.combo_visible_count.saturating_sub(1));

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
                            RichText::new(tab_label).size(12.5).color(ui.visuals().widgets.inactive.fg_stroke.color)
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
                    let resp = ui.add_enabled(self.combo_visible_count < self.combo_entries.len(), add_combo_btn);
                    if resp.hovered() && self.combo_visible_count < self.combo_entries.len() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if resp.clicked() {
                        let next_idx = self.combo_visible_count.min(self.combo_entries.len().saturating_sub(1));
                        self.combo_visible_count = (self.combo_visible_count + 1).min(self.combo_entries.len());
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
                                    self.layout.as_ref().map(|l| l.custom_keycodes.as_slice()).unwrap_or(&[]),
                                    &self.layer_names,
                                    &self.keycode_picker.macro_names,
                                    &self.keycode_picker.tap_dance_names,
                                ).replace('\n', " ")
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
                                                self.layout.as_ref().map(|l| l.custom_keycodes.as_slice()).unwrap_or(&[]),
                                                &self.layer_names,
                                                &self.keycode_picker.macro_names,
                                                &self.keycode_picker.tap_dance_names,
                                            ).replace('\n', " ")
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
                                                self.layout.as_ref().map(|l| l.custom_keycodes.as_slice()).unwrap_or(&[]),
                                                &self.layer_names,
                                                &self.keycode_picker.macro_names,
                                                &self.keycode_picker.tap_dance_names,
                                            ).replace('\n', " ")
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
                            let field_resp = ui.horizontal_centered(|ui| {
                                let field_btn = egui::Button::new(RichText::new(input_summary).size(13.0))
                                    .frame(true)
                                    .stroke(combo_outline_stroke);
                                ui.add_sized(Vec2::new(compact_field_width, 32.0), field_btn)
                            }).inner;
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
                            let resp = ui.horizontal_centered(|ui| {
                                let btn = egui::Button::new(RichText::new(&output_label).size(13.0))
                                    .frame(true)
                                    .stroke(combo_outline_stroke);
                                ui.add_sized(Vec2::new(compact_field_width, 32.0), btn)
                            }).inner;
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
                                ui.label(RichText::new("Time out period for combos").size(13.0).strong());
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
                                        let filtered: String = combo_term_text.chars().filter(|c| c.is_ascii_digit()).collect();
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
                                let clear_btn = egui::Button::new(RichText::new("Clear combo").size(13.0))
                                    .min_size(action_button_size)
                                    .frame(true)
                                    .stroke(combo_outline_stroke);
                                let clear_enabled = combo_idx < self.combo_entries.len() && (
                                    self.combo_entries[combo_idx].keys.iter().any(|&k| k != 0)
                                        || self.combo_entries[combo_idx].output != 0
                                        || self.combo_names.get(combo_idx).map(|s| !s.trim().is_empty()).unwrap_or(false)
                                );
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

                                let delete_btn = egui::Button::new(RichText::new("Delete combo").size(13.0))
                                    .min_size(action_button_size)
                                    .frame(true)
                                    .stroke(combo_outline_stroke);
                                let delete_resp = ui.add_enabled(combo_idx > 0 && self.combo_visible_count > 1, delete_btn);
                                if delete_resp.hovered() && combo_idx > 0 && self.combo_visible_count > 1 {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if delete_resp.clicked() {
                                    self.push_combo_undo();
                                    for idx in combo_idx..self.combo_visible_count.saturating_sub(1) {
                                        self.combo_entries[idx] = self.combo_entries[idx + 1].clone();
                                        self.combo_names[idx] = self.combo_names.get(idx + 1).cloned().unwrap_or_default();
                                    }
                                    let last_idx = self.combo_visible_count.saturating_sub(1);
                                    if last_idx < self.combo_entries.len() {
                                        self.combo_entries[last_idx] = ComboEntry::default();
                                    }
                                    if last_idx < self.combo_names.len() {
                                        self.combo_names[last_idx].clear();
                                    }
                                    self.combo_visible_count = self.combo_visible_count.saturating_sub(1).max(1);
                                    self.selected_combo = combo_idx.min(self.combo_visible_count.saturating_sub(1));
                                    self.combo_dirty = true;
                                    self.combo_names_dirty = true;
                                }

                                let undo_btn = egui::Button::new(RichText::new("Undo").size(13.0))
                                    .min_size(action_button_size)
                                    .frame(true)
                                    .stroke(combo_outline_stroke);
                                let undo_resp = ui.add_enabled(!self.combo_undo_stack.is_empty(), undo_btn);
                                if undo_resp.hovered() && !self.combo_undo_stack.is_empty() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if undo_resp.clicked() {
                                    if let Some((entries, names, term, selected, visible_count)) = self.combo_undo_stack.pop() {
                                        self.combo_entries = entries;
                                        self.combo_names = names;
                                        self.combo_term = term;
                                        self.combo_visible_count = visible_count.clamp(1, self.combo_entries.len().max(1));
                                        self.selected_combo = selected.min(self.combo_visible_count.saturating_sub(1));
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
        if min_x == f32::MAX { min_x = 0.0; min_y = 0.0; max_x = 1.0; max_y = 1.0; }

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
        let top_base_y = ui.min_rect().top() + 6.0;
        let offset_x = (avail.x - layout_w) / 2.0 + ui.min_rect().left() - min_x * unit;
        let offset_y = (avail.y - layout_h - top_reserved_h) / 2.0 + ui.min_rect().top() - min_y * unit + top_reserved_h;

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
                    egui::Rect::from_center_size(slot_rect.center(), Vec2::new(text_w + 20.0, tab_size.y))
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
                    self.hid_device.as_ref()
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
                    egui::pos2(device_rect.center().x - dropdown_size.x / 2.0, device_rect.bottom() + 6.0),
                    dropdown_size,
                );
                let hover_bridge_rect = device_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = !advanced_tab_hovered && !settings_tab_hovered
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
                                        for (i, dev) in self.device_manager.devices().iter().enumerate() {
                                            let is_selected = self.selected_device == Some(i);
                                            let label = if is_selected {
                                                format!("✓ {}", dev.name)
                                            } else {
                                                dev.name.clone()
                                            };
                                            let resp = ui.add_sized(
                                                [dropdown_size.x - 16.0, 26.0],
                                                egui::Button::new(
                                                    RichText::new(label).color(if is_selected {
                                                        ui.visuals().widgets.inactive.fg_stroke.color
                                                    } else if ui.visuals().dark_mode {
                                                        Color32::from_gray(170)
                                                    } else {
                                                        Color32::from_gray(90)
                                                    })
                                                )
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
                                        let lock_label = if is_unlocked { "🔓 Lock" } else { "🔒 Unlock" };
                                        let lock_text = if !is_unlocked {
                                            RichText::new(lock_label).color(Color32::from_rgb(220, 120, 60))
                                        } else {
                                            RichText::new(lock_label)
                                        };
                                        if ui.add_sized(
                                            [dropdown_size.x - 16.0, 26.0],
                                            egui::Button::new(lock_text)
                                                .fill(Color32::TRANSPARENT)
                                                .stroke(egui::Stroke::NONE),
                                        ).clicked() {
                                            if is_unlocked {
                                                if let Some(hid) = &self.hid_device {
                                                    match hid.lock() {
                                                        Ok(()) => self.status_msg = "Keyboard locked".into(),
                                                        Err(e) => self.status_msg = format!("Lock failed: {e}"),
                                                    }
                                                }
                                            } else {
                                                self.unlock_open = true;
                                            }
                                        }
                                    }
                                });
                        });

                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, device_tab_hovered || pointer_over_bridge));
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
                    egui::pos2(advanced_rect.center().x - 76.0, advanced_rect.bottom() + 6.0),
                    Vec2::new(152.0, 106.0),
                );
                let hover_bridge_rect = advanced_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = !device_tab_hovered && !settings_tab_hovered
                    && (advanced_tab_hovered || (was_open && pointer_over_bridge));

                if show_dropdown {
                    let dropdown_fill = if ui.visuals().dark_mode {
                        Color32::from_gray(32)
                    } else {
                        Color32::from_gray(248)
                    };
                    ui.painter().rect(
                        dropdown_rect,
                        8.0,
                        dropdown_fill,
                        egui::Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );

                    let combo_rect = egui::Rect::from_min_max(
                        dropdown_rect.min + Vec2::new(6.0, 4.0),
                        egui::pos2(dropdown_rect.max.x - 6.0, dropdown_rect.min.y + 34.0),
                    );
                    let combo_resp = ui.allocate_rect(combo_rect, Sense::CLICK);
                    if combo_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if combo_resp.clicked() {
                        self.main_menu_tab = MainMenuTab::Keyboard;
                        self.combo_window_open = true;
                    }

                    let auto_shift_supported = self.auto_shift_timeout.is_some();
                    let auto_shift_rect = egui::Rect::from_min_max(
                        dropdown_rect.min + Vec2::new(6.0, 38.0),
                        egui::pos2(dropdown_rect.max.x - 6.0, dropdown_rect.min.y + 68.0),
                    );
                    let mut auto_shift_resp = ui.allocate_rect(auto_shift_rect, Sense::CLICK);
                    if auto_shift_supported && auto_shift_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if auto_shift_supported && auto_shift_resp.clicked() {
                        self.main_menu_tab = MainMenuTab::Keyboard;
                        self.auto_shift_window_open = true;
                    }
                    if !auto_shift_supported {
                        auto_shift_resp = auto_shift_resp.on_hover_text("Auto Shift is not enabled in this firmware.");
                    }

                    let key_override_rect = egui::Rect::from_min_max(
                        dropdown_rect.min + Vec2::new(6.0, 72.0),
                        dropdown_rect.max - Vec2::new(6.0, 4.0),
                    );
                    let key_override_resp = ui.allocate_rect(key_override_rect, Sense::CLICK);
                    if key_override_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if key_override_resp.clicked() {
                        self.main_menu_tab = MainMenuTab::Keyboard;
                        self.key_override_window_open = true;
                    }

                    let item_fill = |hovered: bool, dark_mode: bool| {
                        if hovered {
                            if dark_mode {
                                Color32::from_gray(46)
                            } else {
                                Color32::from_gray(238)
                            }
                        } else {
                            Color32::TRANSPARENT
                        }
                    };
                    ui.painter().rect(
                        combo_rect,
                        6.0,
                        item_fill(combo_resp.hovered(), ui.visuals().dark_mode),
                        egui::Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        combo_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Combo",
                        FontId::proportional(14.0),
                        ui.visuals().widgets.inactive.fg_stroke.color,
                    );
                    ui.painter().rect(
                        auto_shift_rect,
                        6.0,
                        item_fill(auto_shift_supported && auto_shift_resp.hovered(), ui.visuals().dark_mode),
                        egui::Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        auto_shift_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Auto Shift",
                        FontId::proportional(14.0),
                        if auto_shift_supported {
                            ui.visuals().widgets.inactive.fg_stroke.color
                        } else {
                            app_muted_text(ui.visuals().dark_mode)
                        },
                    );
                    ui.painter().rect(
                        key_override_rect,
                        6.0,
                        item_fill(key_override_resp.hovered(), ui.visuals().dark_mode),
                        egui::Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        key_override_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Key Overrides",
                        FontId::proportional(14.0),
                        ui.visuals().widgets.inactive.fg_stroke.color,
                    );

                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, advanced_tab_hovered || combo_resp.hovered() || auto_shift_resp.hovered() || key_override_resp.hovered() || pointer_over_bridge));
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
                    egui::pos2(settings_rect.center().x - 76.0, settings_rect.bottom() + 6.0),
                    Vec2::new(152.0, 36.0),
                );
                let hover_bridge_rect = settings_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = !device_tab_hovered && !advanced_tab_hovered
                    && (settings_tab_hovered || (was_open && pointer_over_bridge));

                if show_dropdown {
                    let dropdown_fill = if ui.visuals().dark_mode {
                        Color32::from_gray(32)
                    } else {
                        Color32::from_gray(248)
                    };
                    ui.painter().rect(
                        dropdown_rect,
                        8.0,
                        dropdown_fill,
                        egui::Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );

                    let matrix_rect = dropdown_rect.shrink2(Vec2::new(6.0, 4.0));
                    let matrix_resp = ui.allocate_rect(matrix_rect, Sense::CLICK);
                    if matrix_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if matrix_resp.clicked() {
                        self.settings_tab = SettingsTab::MatrixTester;
                        if self.main_menu_tab != MainMenuTab::Settings {
                            self.reset_matrix_tester_state();
                        }
                        self.matrix_tester_unlock_prompted = false;
                        self.main_menu_tab = MainMenuTab::Settings;
                    }

                    let matrix_fill = if matrix_resp.hovered() {
                        if ui.visuals().dark_mode {
                            Color32::from_gray(46)
                        } else {
                            Color32::from_gray(238)
                        }
                    } else {
                        Color32::TRANSPARENT
                    };
                    ui.painter().rect(
                        matrix_rect,
                        6.0,
                        matrix_fill,
                        egui::Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        matrix_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Matrix Tester",
                        FontId::proportional(14.0),
                        ui.visuals().widgets.inactive.fg_stroke.color,
                    );

                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, settings_tab_hovered || matrix_resp.hovered() || pointer_over_bridge));
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, false));
                }
            }
        }

        if self.main_menu_tab == MainMenuTab::Settings {
            self.draw_settings_screen(ui, layout, ctx, top_base_y + main_tabs_h + 14.0);
            return;
        }

        // ── Layer switcher ─────────────────────────────────────────────────
        {
            let layer_count = self.layer_count;
            let selected = self.selected_layer;
            // raw_name — чистое имя без префикса, хранится в layer_names
            let raw_name = self.layer_names.get(selected).cloned().unwrap_or_else(|| selected.to_string());
            // display_name — с префиксом для отображения
            let display_name = if !raw_name.is_empty() && raw_name != selected.to_string() {
                format!("{}. {}", selected, raw_name)
            } else {
                raw_name.clone()
            };
            let name = display_name;
            let center_x = ui.min_rect().center().x;
            let bar_y = ui.min_rect().top() + (avail.y - layout_h - layer_bar_h) / 2.0 + 4.0;

            // ZMK layer management buttons (+ Add / - Remove)
            if self.firmware == FirmwareProtocol::Zmk {
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
                    let active_color = if self.dark_mode { Color32::from_gray(200) } else { Color32::from_gray(60) };
                    let disabled_color = if self.dark_mode { Color32::from_gray(70) } else { Color32::from_gray(180) };

                    let add_rect = egui::Rect::from_center_size(egui::pos2(btn_x, mid_y - 8.0), Vec2::splat(16.0));
                    let remove_rect = egui::Rect::from_center_size(egui::pos2(btn_x, mid_y + 8.0), Vec2::splat(16.0));
                    let can_add = !op_busy && !self.zmk_no_extra_layers;
                    let add_resp = ui.allocate_rect(add_rect, if can_add { Sense::click() } else { Sense::hover() });
                    let remove_resp = ui.allocate_rect(remove_rect, if can_remove { Sense::click() } else { Sense::hover() });
                    if add_resp.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                    if remove_resp.hovered() && can_remove { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                    let add_col = if add_resp.hovered() && can_add { Color32::from_rgb(91,104,223) } else if can_add { active_color } else { disabled_color };
                    let rem_col = if remove_resp.hovered() && can_remove { Color32::from_rgb(91,104,223) } else if can_remove { active_color } else { disabled_color };
                    ui.painter().text(add_rect.center(), egui::Align2::CENTER_CENTER, "+", sym_font.clone(), add_col);
                    ui.painter().text(remove_rect.center(), egui::Align2::CENTER_CENTER, "−", sym_font, rem_col);
                    let add_hover_text = if self.zmk_no_extra_layers { "Firmware doesn't support extra layers" } else { "Add layer" };
                    let add_clicked = add_resp.on_hover_text(add_hover_text).clicked();
                    let rem_clicked = remove_resp.on_hover_text(if can_remove { "Remove last layer" } else { "Cannot remove base layers" }).clicked();
                    if add_clicked && can_add { self.zmk_add_layer(); }
                    if rem_clicked && can_remove { self.zmk_remove_layer(); }
                }
            }


            // Layer name / edit field
            let name_rect = egui::Rect::from_min_size(egui::pos2(center_x - 85.0, bar_y), Vec2::new(170.0, 52.0));

            let label_font = egui::FontId { size: 39.0, family: egui::FontFamily::Proportional };
            let text_color = if self.dark_mode { Color32::from_gray(245) } else { Color32::from_gray(60) };

            if self.editing_layer == Some(selected) {
                // Limit input to 7 chars
                if self.editing_layer_text.chars().count() > 7 {
                    let s: String = self.editing_layer_text.chars().take(7).collect();
                    self.editing_layer_text = s;
                }
                let resp = ui.put(name_rect,
                    egui::TextEdit::singleline(&mut self.editing_layer_text)
                        .font(label_font.clone())
                        .horizontal_align(egui::Align::Center)
                        .char_limit(7)
                        .frame(false)
                );
                // Request focus only on the first frame so lost_focus() works correctly.
                if !self.editing_layer_focus_requested {
                    resp.request_focus();
                    self.editing_layer_focus_requested = true;
                }
                // Commit on Enter or lost focus (click outside); cancel on Escape.
                let commit = resp.lost_focus() || ui.input(|inp| inp.key_pressed(egui::Key::Enter));
                let cancel = ui.input(|inp| inp.key_pressed(egui::Key::Escape));
                if commit || cancel {
                    if commit && !self.editing_layer_text.trim().is_empty() {
                        let new_name = self.editing_layer_text.trim().to_string();
                        while self.layer_names.len() <= selected { self.layer_names.push(self.layer_names.len().to_string()); }
                        self.layer_names[selected] = new_name.clone();
                        #[cfg(not(target_arch = "wasm32"))]
                        save_layer_names(&self.layer_names, &self.current_device_name);
                        #[cfg(target_arch = "wasm32")]
                        save_layer_names(&self.layer_names, "default");
                        // ZMK: also write name to device
                        #[cfg(not(target_arch = "wasm32"))]
                        if self.firmware == FirmwareProtocol::Zmk {
                            if let Some(conn) = &mut self.zmk_conn {
                                let layer_id = self.layout.as_ref()
                                    .and_then(|l| l.zmk_layer_ids.get(selected).copied())
                                    .unwrap_or(selected as u32);
                                if let Err(e) = conn.set_layer_name(layer_id, &new_name) {
                                    log::warn!("ZMK set_layer_name failed: {e}");
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
                let left_center  = egui::pos2(center_x - fixed_half - gap - 24.0, mid_y);
                let right_center = egui::pos2(center_x + fixed_half + gap + 24.0, mid_y);

                // Still measure actual text width for painting the name and edit icon.
                let text_w = ui.fonts(|f| f.layout_no_wrap(name.clone(), label_font.clone(), text_color).size().x);

                // Allocate name FIRST — arrows are allocated last and win in egui's
                // hit-test order (last allocation = highest priority).
                let name_hit = egui::Rect::from_center_size(egui::pos2(center_x, mid_y), Vec2::new(text_w + 12.0, 52.0));
                let name_r = ui.allocate_rect(name_hit, Sense::click());

                // Full layer switch zone from arrow to arrow for mouse wheel switching.
                // Keep click/hover hitboxes close to the actual arrow glyph size.
                let left_hit  = egui::Rect::from_center_size(left_center,  Vec2::new(28.0, 44.0));
                let right_hit = egui::Rect::from_center_size(right_center, Vec2::new(28.0, 44.0));
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
                let left_r  = ui.allocate_rect(left_hit,  Sense::click());
                let right_r = ui.allocate_rect(right_hit, Sense::click());
                if left_r.hovered()  { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                if right_r.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                if left_r.clicked()  && selected > 0              { self.selected_layer = selected - 1; self.jump_back_stack.clear(); }
                if right_r.clicked() && selected + 1 < layer_count { self.selected_layer = selected + 1; self.jump_back_stack.clear(); }
                if name_r.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                if name_r.clicked() {
                    self.editing_layer = Some(selected);
                    self.editing_layer_text = raw_name.clone();
                }

                // Paint
                let dis = if self.dark_mode { Color32::from_gray(60) } else { Color32::from_gray(200) };
                let ac_l = if left_r.hovered()  { Color32::from_rgb(91,104,223) } else if self.dark_mode { Color32::from_gray(140) } else { Color32::from_gray(120) };
                let ac_r = if right_r.hovered() { Color32::from_rgb(91,104,223) } else if self.dark_mode { Color32::from_gray(140) } else { Color32::from_gray(120) };
                ui.painter().text(left_center,  egui::Align2::CENTER_CENTER, "‹", FontId::proportional(52.0), if selected == 0 { dis } else { ac_l });
                ui.painter().text(right_center, egui::Align2::CENTER_CENTER, "›", FontId::proportional(52.0), if selected + 1 >= layer_count { dis } else { ac_r });
                ui.painter().text(egui::pos2(center_x, mid_y), egui::Align2::CENTER_CENTER, &name, label_font, text_color);

                // Hint text below layer name
                let hint_color = if self.dark_mode { Color32::from_gray(100) } else { Color32::from_gray(160) };
                let hint_font = FontId::proportional(11.0);
                let secondary_hint_font = FontId::proportional(10.5);
                let hint_y = bar_y + layer_bar_h + 18.0;
                let any_hovered = self.prev_hovered_key.is_some();
                if let Some(hl) = self.hover_layer {
                    let hl_name = self.layer_names.get(hl).cloned().unwrap_or_else(|| hl.to_string());
                    let mut line = 0i32;
                    let line_h = 13.0f32;
                    let base_y = hint_y - 15.0;
                    // Line 1: always
                    ui.painter().text(egui::pos2(center_x, base_y + line as f32 * line_h), egui::Align2::CENTER_CENTER,
                        "Left click to change this key", hint_font.clone(), hint_color);
                    line += 1;
                    // Line 2: go to layer (if not current)
                    if hl != self.selected_layer {
                        ui.painter().text(egui::pos2(center_x, base_y + line as f32 * line_h), egui::Align2::CENTER_CENTER,
                            &format!("Right click to go to layer {}: {}", hl, hl_name), hint_font.clone(), hint_color);
                        line += 1;
                    }
                    // Line 3: change layer number
                    ui.painter().text(egui::pos2(center_x, base_y + line as f32 * line_h), egui::Align2::CENTER_CENTER,
                        "Ctrl + Right click to change layer number", hint_font.clone(), hint_color);
                    line += 1;
                    // Line 4: go back (if in jump mode)
                    if !self.jump_back_stack.is_empty() {
                        ui.painter().text(egui::pos2(center_x, base_y + line as f32 * line_h), egui::Align2::CENTER_CENTER,
                            "Esc to go back", hint_font.clone(), hint_color);
                    }
                    let _ = hint_font;
                } else if !self.jump_back_stack.is_empty() {
                    if any_hovered {
                        ui.painter().text(egui::pos2(center_x, hint_y - 9.0), egui::Align2::CENTER_CENTER,
                            "Left click to change this key", hint_font.clone(), hint_color);
                    }
                    ui.painter().text(egui::pos2(center_x, if any_hovered { hint_y + 5.0 } else { hint_y }), egui::Align2::CENTER_CENTER,
                        "Right-click or Esc to go back", hint_font, hint_color);
                } else if any_hovered {
                    // Check if hovered key is a mod key
                    let (hovered_is_mod, hovered_can_swap_side, hovered_is_macro, hovered_is_tap_dance) = if self.firmware == FirmwareProtocol::Vial {
                        self.prev_hovered_key.and_then(|ki| {
                            self.layout.as_ref().map(|l| {
                                let kc = l.get_keycode(self.selected_layer, ki);
                                let is_plain_mod = (0x00E0..=0x00E7).contains(&kc) || matches!(kc, 0x52A1 | 0x52A2 | 0x52A4 | 0x52A7 | 0x52A8 | 0x52AF | 0x52B1 | 0x52B2 | 0x52B4 | 0x52B8);
                                let is_mod = is_plain_mod || (kc >= 0x2000 && kc < 0x4000) || (kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0);
                                let can_swap_side = toggle_handed_modifier(kc).is_some();
                                let is_macro = kc >= 0x7700 && kc <= 0x77FF;
                                let is_tap_dance = kc >= 0x5700 && kc <= 0x57FF;
                                (is_mod, can_swap_side, is_macro, is_tap_dance)
                            })
                        }).unwrap_or((false, false, false, false))
                    } else { (false, false, false, false) };
                    if hovered_is_mod {
                        if hovered_can_swap_side {
                            ui.painter().text(egui::pos2(center_x, hint_y - 22.0), egui::Align2::CENTER_CENTER,
                                "Left click to change this key", hint_font.clone(), hint_color);
                            ui.painter().text(egui::pos2(center_x, hint_y - 4.0), egui::Align2::CENTER_CENTER,
                                "Right click to change the modifier key", secondary_hint_font.clone(), hint_color);
                            ui.painter().text(egui::pos2(center_x, hint_y + 12.0), egui::Align2::CENTER_CENTER,
                                "Ctrl+right-click to switch left/right side", secondary_hint_font, hint_color);
                        } else {
                            ui.painter().text(egui::pos2(center_x, hint_y - 14.0), egui::Align2::CENTER_CENTER,
                                "Left click to change this key", hint_font.clone(), hint_color);
                            ui.painter().text(egui::pos2(center_x, hint_y + 4.0), egui::Align2::CENTER_CENTER,
                                "Right click to change the modifier key", secondary_hint_font, hint_color);
                        }
                    } else if hovered_is_macro {
                        ui.painter().text(egui::pos2(center_x, hint_y - 14.0), egui::Align2::CENTER_CENTER,
                            "Left click to change this key", hint_font.clone(), hint_color);
                        ui.painter().text(egui::pos2(center_x, hint_y + 4.0), egui::Align2::CENTER_CENTER,
                            "Right click to edit macro", secondary_hint_font.clone(), hint_color);
                    } else if hovered_is_tap_dance {
                        ui.painter().text(egui::pos2(center_x, hint_y - 14.0), egui::Align2::CENTER_CENTER,
                            "Left click to change this key", hint_font.clone(), hint_color);
                        ui.painter().text(egui::pos2(center_x, hint_y + 4.0), egui::Align2::CENTER_CENTER,
                            "Right click to edit tap dance", secondary_hint_font, hint_color);
                    } else {
                        ui.painter().text(egui::pos2(center_x, hint_y), egui::Align2::CENTER_CENTER,
                            "Left click to change this key", hint_font, hint_color);
                    }
                } else if name_r.hovered() {
                    ui.painter().text(egui::pos2(center_x, hint_y), egui::Align2::CENTER_CENTER,
                        "Click to rename layer", hint_font, hint_color);
                }
            }
        }

        // Pass 1: allocate
        let mut rects: Vec<(usize, egui::Rect, egui::Response)> =
            Vec::with_capacity(layout.keys.len());
        for (ki, key) in layout.keys.iter().enumerate() {
            let rect = egui::Rect::from_min_size(
                egui::pos2(
                    offset_x + key.x * unit + padding,
                    offset_y + key.y * unit + padding,
                ),
                Vec2::new(key.w * unit - padding * 2.0, key.h * unit - padding * 2.0),
            );
            let response = ui.allocate_rect(rect, Sense::click());
            rects.push((ki, rect, response));
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
                    self.keycode_picker.zmk_behaviors = self.layout.as_ref()
                        .map(|l| l.zmk_behaviors.clone()).unwrap_or_default();
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
                let beh_name = layout.zmk_behaviors.iter()
                    .find(|b| b.id == binding.behavior_id as u32)
                    .map(|b| b.display_name.as_str())
                    .unwrap_or("");
                match beh_name {
                    "Momentary Layer" | "Toggle Layer" | "To Layer" | "Sticky Layer" | "Layer-Tap" => {
                        Some(binding.param1 as usize)
                    }
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
                let is_mod_key = (kc >= 0x2000 && kc < 0x4000) || (kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0);
                let is_macro_key = kc >= 0x7700 && kc <= 0x77FF;
                let tip = keycode_tooltip_with_macro_names(kc, &layout.custom_keycodes, &self.layer_names, &self.keycode_picker.macro_names, &self.keycode_picker.tap_dance_names);
                *response = response.clone().on_hover_text(tip);
            }
        }

        // Animate hover_layer_progress
        let target_progress = if self.hover_layer.is_some() { 1.0f32 } else { 0.0f32 };
        let speed = 4.0f32;
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        self.hover_layer_progress += (target_progress - self.hover_layer_progress) * (speed * dt).min(1.0);
        if (self.hover_layer_progress - target_progress).abs() > 0.01 {
            ctx.request_repaint();
        }

        // Pass 3: paint
        let painter = ui.painter();
        let hover_target = self.hover_layer.unwrap_or(prev_hover.unwrap_or(self.selected_layer));
        let hover_alpha = self.hover_layer_progress;
        let dark = self.dark_mode;
        // Use hover layer for logic (TRNS resolution etc) when mostly visible
        let layer = if hover_alpha > 0.5 { hover_target } else { self.selected_layer };
        for (ki, rect, _) in &rects {
            let key = &layout.keys[*ki];
            let is_selected = self.selected_key == Some((layer, *ki));
            let is_hovered = hovered_key == Some(*ki);
            // Accent: #5B68DF indigo
            let bg = if is_selected {
                Color32::from_rgb(91, 104, 223)
            } else if is_hovered {
                if dark { Color32::from_rgb(60, 60, 65) } else { Color32::from_rgb(232, 232, 240) }
            } else {
                if dark { Color32::from_rgb(48, 48, 52) } else { Color32::from_rgb(255, 255, 255) }
            };

            let draw_rect = if key.rotation != 0.0 {
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

            if is_zmk {
                // ZMK binding display
                let binding = layout.get_zmk_binding(layer, *ki);
                let is_trans = layout.zmk_behaviors.iter()
                    .find(|b| b.id == binding.behavior_id as u32)
                    .map(|b| b.display_name == "Transparent")
                    .unwrap_or(false);
                let border = if dark { Color32::from_rgb(55, 55, 60) } else { Color32::from_rgb(210, 210, 218) };
                painter.rect(draw_rect, 6.0, bg, Stroke::new(1.0, border), egui::StrokeKind::Inside);
                if is_trans && layer > 0 {
                    if is_hovering {
                        // During hover preview — TRNS keys are empty (no text)
                    } else {
                        // Normal display — show TRNS with fallback
                        let fallback = (0..layer).rev()
                            .map(|l| layout.get_zmk_binding(l, *ki))
                            .find(|b| {
                                !layout.zmk_behaviors.iter()
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
                    let label = zmk_binding_label(&binding, &layout.zmk_behaviors, &self.layer_names);
                    draw_key_label(&painter, draw_rect, &label, dark);
                }
            } else {
                let kc = layout.get_keycode(layer, *ki);

                if kc == 0x0001 {
                    painter.rect(draw_rect, 6.0, bg, Stroke::new(1.0, if dark { Color32::from_rgb(55, 55, 60) } else { Color32::from_rgb(210, 210, 218) }), egui::StrokeKind::Inside);
                    if !is_hovering {
                        let fallback_kc = (0..layer).rev()
                            .map(|l| layout.get_keycode(l, *ki))
                            .find(|&k| k != 0x0001)
                            .unwrap_or(0x0000);
                        let label = if fallback_kc == 0x0000 || fallback_kc == 0x0001 {
                            "\u{25BD}".to_string()
                        } else {
                            keycode_label_with_macro_names(fallback_kc, &layout.custom_keycodes, &self.layer_names, &self.keycode_picker.macro_names, &self.keycode_picker.tap_dance_names)
                        };
                        draw_key_label_dimmed(&painter, draw_rect, &label, dark);
                    }
                } else if kc == 0x0000 {
                    let no_bg = if dark { Color32::from_rgb(20, 20, 22) } else { Color32::from_rgb(238, 238, 242) };
                    let no_border = if dark { Color32::from_rgb(40, 40, 44) } else { Color32::from_rgb(210, 210, 218) };
                    let no_text = if dark { Color32::from_rgb(55, 55, 65) } else { Color32::from_rgb(180, 180, 195) };
                    painter.rect(draw_rect, 6.0, no_bg, Stroke::new(1.0, no_border), egui::StrokeKind::Inside);
                    painter.text(draw_rect.center(), egui::Align2::CENTER_CENTER, "\u{2715}", FontId::proportional(10.0), no_text);
                } else {
                    let border = if dark { Color32::from_rgb(55, 55, 60) } else { Color32::from_rgb(210, 210, 218) };
                    painter.rect(draw_rect, 6.0, bg, Stroke::new(1.0, border), egui::StrokeKind::Inside);
                    let label = keycode_label_with_macro_names(kc, &layout.custom_keycodes, &self.layer_names, &self.keycode_picker.macro_names, &self.keycode_picker.tap_dance_names);
                    draw_key_label(&painter, draw_rect, &label, dark);
                }
            }


        }

        self.prev_hovered_key = hovered_key;

        if layout_h > avail.y {
            ui.allocate_space(Vec2::new(0.0, (layout_h - avail.y).max(0.0)));
        }
    }
}

fn draw_key_label_dimmed(painter: &egui::Painter, rect: egui::Rect, label: &str, dark: bool) {
    let dim = if dark { Color32::from_rgb(60, 60, 65) } else { Color32::from_rgb(200, 200, 208) };
    let dim_top = if dark { Color32::from_rgb(45, 45, 50) } else { Color32::from_rgb(215, 215, 220) };
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos+1..])
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);

    if let Some(top_str) = top {
        let center = rect.center();
        painter.text(egui::pos2(center.x, center.y - 7.0), egui::Align2::CENTER_CENTER, top_str, FontId::proportional(top_size.unwrap_or(9.0)), dim_top);
        painter.text(egui::pos2(center.x, center.y + 6.0), egui::Align2::CENTER_CENTER, bottom, FontId::proportional(bottom_size), dim);
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, bottom, FontId::proportional(font_size), dim);
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

fn draw_key_label_alpha(painter: &egui::Painter, rect: egui::Rect, label: &str, dark: bool, alpha: f32) {
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos+1..])
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);
    let top_color = with_alpha(if dark { Color32::from_rgb(130, 130, 145) } else { Color32::from_rgb(130, 130, 150) }, alpha);
    let main_color = with_alpha(if dark { Color32::from_rgb(232, 232, 240) } else { Color32::from_rgb(26, 26, 30) }, alpha);
    if let Some(top_str) = top {
        let center = rect.center();
        painter.text(egui::pos2(center.x, center.y - 7.0), egui::Align2::CENTER_CENTER, top_str, FontId::proportional(top_size.unwrap_or(9.0)), top_color);
        painter.text(egui::pos2(center.x, center.y + 6.0), egui::Align2::CENTER_CENTER, bottom, FontId::proportional(bottom_size), main_color);
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, bottom, FontId::proportional(font_size), main_color);
    }
}

fn draw_key_label_dimmed_alpha(painter: &egui::Painter, rect: egui::Rect, label: &str, dark: bool, alpha: f32) {
    let dim = with_alpha(if dark { Color32::from_rgb(80, 80, 90) } else { Color32::from_rgb(180, 180, 195) }, alpha);
    let dim_top = with_alpha(if dark { Color32::from_rgb(60, 60, 70) } else { Color32::from_rgb(190, 190, 205) }, alpha);
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos+1..])
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);
    if let Some(top_str) = top {
        let center = rect.center();
        painter.text(egui::pos2(center.x, center.y - 7.0), egui::Align2::CENTER_CENTER, top_str, FontId::proportional(top_size.unwrap_or(9.0)), dim_top);
        painter.text(egui::pos2(center.x, center.y + 6.0), egui::Align2::CENTER_CENTER, bottom, FontId::proportional(bottom_size), dim);
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, bottom, FontId::proportional(font_size), dim);
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
        let b = &label[pos+1..];
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

        let top_color = if dark { Color32::from_rgb(130, 130, 145) } else { Color32::from_rgb(130, 130, 150) };
        let main_color = if dark { Color32::from_rgb(232, 232, 240) } else { Color32::from_rgb(26, 26, 30) };
        painter.text(top_pos, egui::Align2::CENTER_CENTER, top_str, FontId::proportional(top_size.unwrap_or(9.0)), top_color);
        painter.text(bot_pos, egui::Align2::CENTER_CENTER, bottom, FontId::proportional(bottom_size), main_color);
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(font_size),
            if dark { Color32::from_rgb(232, 232, 240) } else { Color32::from_rgb(26, 26, 30) },
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
            let bg = if is_selected { Color32::from_rgb(70, 110, 190) } else { Color32::from_gray(45) };
            painter.rect(*rect, 6.0, bg, Stroke::new(1.0, Color32::from_gray(80)), egui::StrokeKind::Inside);
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
