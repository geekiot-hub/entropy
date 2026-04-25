/// Keycode picker modal — supports both Vial (QMK keycodes) and ZMK (behaviors).

fn inactive_picker_entry_text(dark: bool) -> egui::Color32 {
    if dark {
        egui::Color32::from_gray(55)
    } else {
        egui::Color32::from_gray(175)
    }
}

use crate::firmware::FirmwareProtocol;
use crate::popup_state::{PopupKey, PopupState};
use crate::keycode::{gui_label, gui_mod_name, gui_sym, key_label_font_sizes, keycode_label_with_names, keycode_tooltip, KeycodeCategory, KEYCODES};
use crate::zmk::{BehaviorInfo, ZmkBinding};
use egui::{Color32, Key, RichText, Vec2};

#[derive(Clone, Debug, Default)]
pub struct TapDanceEntry {
    pub on_tap: u16,
    pub on_hold: u16,
    pub on_double_tap: u16,
    pub on_tap_hold: u16,
    pub tapping_term: u16,
}

#[derive(Clone, Debug)]
pub enum MacroAction {
    Text(String),
    Tap(u8),    // QMK keycode
    Down(u8),   // key press
    Up(u8),     // key release
    Delay(u16), // milliseconds
}

pub struct KeycodePicker {
    pub open: bool,
    pub selected_tab: KeycodeTab,
    pub search_query: String,
    pub result: Option<u16>,
    pub custom_keycodes: Vec<(String, String, String, u16)>,
    pub supports_rgb: bool,
    pub layer_names: Vec<String>,
    pub layer_count: usize,
    pub layer_has_content: Vec<bool>,
    pub listening: bool,
    // ZMK
    pub firmware: FirmwareProtocol,
    pub zmk_behaviors: Vec<BehaviorInfo>,
    pub zmk_result: Option<ZmkBinding>,
    pub zmk_selected_behavior: Option<usize>,
    pub zmk_layer_count: usize,
    // Vial Quantum tab pending state
    pub vial_quantum_pending_mod: Option<u16>,
    pub vial_quantum_pending_mt: Option<u16>,
    pub vial_layer_pending: Option<u16>,
    /// Open macro editor for this macro number (0..15), None = closed
    pub macro_count: usize,
    pub tap_dance_entries: Vec<TapDanceEntry>,
    pub tap_dance_names: Vec<String>,
    pub tap_dance_undo_stack: Vec<(usize, TapDanceEntry, String)>,
    pub tap_dance_editor_open: Option<u8>,
    pub tap_dance_dirty: bool,
    /// Which field is being edited: (td_idx, field: 0=tap,1=hold,2=dtap,3=taphold)
    pub td_key_pick: Option<(usize, u8)>,
    pub macro_inline_selected: Option<u8>,
    /// Macro editor text buffers (one per macro)
    pub macro_texts: Vec<String>,
    /// User-visible names for macros (optional)
    pub macro_names: Vec<String>,
    /// Macro actions for editor UI
    pub macro_actions: Vec<Vec<MacroAction>>,
    /// Flag: macro texts changed, need to write to device
    pub macros_dirty: bool,
    /// Undo stack for macro editor: (macro_idx, previous_actions)
    macro_undo_stack: Vec<(usize, Vec<MacroAction>)>,
    /// Macro key picker: (macro_idx, action_idx) being edited
    macro_key_pick: Option<(usize, usize)>,
    popup_state: PopupState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeycodeTab {
    Basic,
    Symbols,
    Function,
    Navigation,
    Modifiers,
    Layers,
    Media,
    Mouse,
    Numpad,
    Special,
    Rgb,
    Macro,
    TapDance,
    Quantum,
    Custom,
    ZmkAdvanced,
}

impl KeycodeTab {
    pub const VIAL_TABS: &'static [KeycodeTab] = &[
        KeycodeTab::Basic,
        KeycodeTab::Symbols,
        KeycodeTab::Modifiers,
        KeycodeTab::Special,
        KeycodeTab::Rgb,
        KeycodeTab::Macro,
        KeycodeTab::TapDance,
        KeycodeTab::Custom,
    ];

    pub const ZMK_TABS: &'static [KeycodeTab] = &[
        KeycodeTab::Basic,
        KeycodeTab::Symbols,
        KeycodeTab::Function,
        KeycodeTab::Navigation,
        KeycodeTab::Modifiers,
        KeycodeTab::Layers,
        KeycodeTab::Media,
        KeycodeTab::Special,
        KeycodeTab::ZmkAdvanced,
    ];

    pub fn label(self) -> &'static str {
        match self {
            KeycodeTab::Basic       => "Basic",
            KeycodeTab::Symbols     => "Symbols",
            KeycodeTab::Function    => "F1-F24",
            KeycodeTab::Navigation  => "Nav",
            KeycodeTab::Modifiers   => "Mods",
            KeycodeTab::Layers      => "Layers",
            KeycodeTab::Media       => "Media, Apps, System",
            KeycodeTab::Mouse       => "Mouse",
            KeycodeTab::Numpad      => "Numpad",
            KeycodeTab::Special     => "Special",
            KeycodeTab::Rgb         => "RGB",
            KeycodeTab::Macro       => "Macros",
            KeycodeTab::TapDance   => "Tap Dance",
            KeycodeTab::Quantum     => "Quantum",
            KeycodeTab::Custom      => "Custom",
            KeycodeTab::ZmkAdvanced => "Advanced",
        }
    }

    fn vial_matches(&self, kc: &crate::keycode::Keycode) -> bool {
        match self {
            KeycodeTab::Basic      => matches!(kc.category, KeycodeCategory::Basic) && !is_symbol(kc.value),
            KeycodeTab::Symbols    => matches!(kc.category, KeycodeCategory::Basic) && is_symbol(kc.value),
            KeycodeTab::Function   => matches!(kc.category, KeycodeCategory::Function) && kc.value <= 0x0045,
            KeycodeTab::Navigation => matches!(kc.category, KeycodeCategory::Navigation),
            KeycodeTab::Modifiers  => matches!(kc.category, KeycodeCategory::Modifier),
            KeycodeTab::Layers     => matches!(kc.category, KeycodeCategory::Layer),
            KeycodeTab::Media      => matches!(kc.category, KeycodeCategory::Media | KeycodeCategory::Mouse),
            KeycodeTab::Mouse      => matches!(kc.category, KeycodeCategory::Mouse),
            KeycodeTab::Numpad     => matches!(kc.category, KeycodeCategory::Numpad),
            KeycodeTab::Special    => matches!(kc.category, KeycodeCategory::Special)
                || (matches!(kc.category, KeycodeCategory::Function) && kc.value >= 0x0068),
            _ => false,
        }
    }
}

fn is_symbol(value: u16) -> bool {
    matches!(value,
        0x002D..=0x0038 |
        0x0032 | 0x0064 |
        0x021E..=0x0238
    )
}

impl Default for KeycodePicker {
    fn default() -> Self {
        Self {
            open: false,
            selected_tab: KeycodeTab::Basic,
            search_query: String::new(),
            result: None,
            custom_keycodes: vec![],
            supports_rgb: true,
            layer_names: (0..16).map(|i| i.to_string()).collect(),
            layer_count: 4,
            layer_has_content: vec![true; 16],
            listening: false,
            firmware: FirmwareProtocol::Vial,
            zmk_behaviors: vec![],
            zmk_result: None,
            zmk_selected_behavior: None,
            zmk_layer_count: 4,
            vial_quantum_pending_mod: None,
            vial_quantum_pending_mt: None,
            vial_layer_pending: None,
            macro_inline_selected: None,
            macro_count: 16,
            tap_dance_entries: vec![],
            tap_dance_names: vec![],
            tap_dance_undo_stack: vec![],
            tap_dance_editor_open: None,
            tap_dance_dirty: false,
            td_key_pick: None,
            macro_texts: vec![String::new(); 16],
            macro_names: vec![String::new(); 16],
            macro_actions: vec![vec![]; 16],
            macro_undo_stack: Vec::new(),
            macro_key_pick: None,
            macros_dirty: false,
            popup_state: PopupState::default(),
        }
    }
}

pub fn egui_key_to_qmk(key: Key, mods: egui::Modifiers) -> Option<u16> {
    let base: u16 = match key {
        Key::A => 0x04, Key::B => 0x05, Key::C => 0x06, Key::D => 0x07,
        Key::E => 0x08, Key::F => 0x09, Key::G => 0x0A, Key::H => 0x0B,
        Key::I => 0x0C, Key::J => 0x0D, Key::K => 0x0E, Key::L => 0x0F,
        Key::M => 0x10, Key::N => 0x11, Key::O => 0x12, Key::P => 0x13,
        Key::Q => 0x14, Key::R => 0x15, Key::S => 0x16, Key::T => 0x17,
        Key::U => 0x18, Key::V => 0x19, Key::W => 0x1A, Key::X => 0x1B,
        Key::Y => 0x1C, Key::Z => 0x1D,
        Key::Num1 => 0x1E, Key::Num2 => 0x1F, Key::Num3 => 0x20,
        Key::Num4 => 0x21, Key::Num5 => 0x22, Key::Num6 => 0x23,
        Key::Num7 => 0x24, Key::Num8 => 0x25, Key::Num9 => 0x26,
        Key::Num0 => 0x27,
        Key::Enter => 0x28, Key::Escape => 0x29, Key::Backspace => 0x2A,
        Key::Tab => 0x2B, Key::Space => 0x2C,
        Key::Minus => 0x2D, Key::Equals => 0x2E,
        Key::OpenBracket => 0x2F, Key::CloseBracket => 0x30,
        Key::Backslash => 0x31, Key::Semicolon => 0x33,
        Key::Quote => 0x34, Key::Backtick => 0x35,
        Key::Comma => 0x36, Key::Period => 0x37, Key::Slash => 0x38,
        Key::F1 => 0x3A, Key::F2 => 0x3B, Key::F3 => 0x3C, Key::F4 => 0x3D,
        Key::F5 => 0x3E, Key::F6 => 0x3F, Key::F7 => 0x40, Key::F8 => 0x41,
        Key::F9 => 0x42, Key::F10 => 0x43, Key::F11 => 0x44, Key::F12 => 0x45,
        Key::Insert => 0x49, Key::Home => 0x4A, Key::PageUp => 0x4B,
        Key::Delete => 0x4C, Key::End => 0x4D, Key::PageDown => 0x4E,
        Key::ArrowRight => 0x4F, Key::ArrowLeft => 0x50,
        Key::ArrowDown => 0x51, Key::ArrowUp => 0x52,
        _ => return None,
    };
    let mut mod_mask: u16 = 0;
    if mods.ctrl  { mod_mask |= 0x0100; }
    if mods.shift { mod_mask |= 0x0200; }
    if mods.alt   { mod_mask |= 0x0400; }
    if mods.mac_cmd || mods.command { mod_mask |= 0x0800; }
    if mod_mask != 0 { Some(mod_mask | base) } else { Some(base) }
}

/// Convert egui key to ZMK HID usage (keyboard page 0x07)
fn egui_key_to_zmk_usage(key: Key) -> Option<u32> {
    let hid: u16 = match key {
        Key::A => 0x04, Key::B => 0x05, Key::C => 0x06, Key::D => 0x07,
        Key::E => 0x08, Key::F => 0x09, Key::G => 0x0A, Key::H => 0x0B,
        Key::I => 0x0C, Key::J => 0x0D, Key::K => 0x0E, Key::L => 0x0F,
        Key::M => 0x10, Key::N => 0x11, Key::O => 0x12, Key::P => 0x13,
        Key::Q => 0x14, Key::R => 0x15, Key::S => 0x16, Key::T => 0x17,
        Key::U => 0x18, Key::V => 0x19, Key::W => 0x1A, Key::X => 0x1B,
        Key::Y => 0x1C, Key::Z => 0x1D,
        Key::Num1 => 0x1E, Key::Num2 => 0x1F, Key::Num3 => 0x20,
        Key::Num4 => 0x21, Key::Num5 => 0x22, Key::Num6 => 0x23,
        Key::Num7 => 0x24, Key::Num8 => 0x25, Key::Num9 => 0x26,
        Key::Num0 => 0x27,
        Key::Enter => 0x28, Key::Escape => 0x29, Key::Backspace => 0x2A,
        Key::Tab => 0x2B, Key::Space => 0x2C,
        Key::F1 => 0x3A, Key::F2 => 0x3B, Key::F3 => 0x3C, Key::F4 => 0x3D,
        Key::F5 => 0x3E, Key::F6 => 0x3F, Key::F7 => 0x40, Key::F8 => 0x41,
        Key::F9 => 0x42, Key::F10 => 0x43, Key::F11 => 0x44, Key::F12 => 0x45,
        Key::ArrowRight => 0x4F, Key::ArrowLeft => 0x50,
        Key::ArrowDown => 0x51, Key::ArrowUp => 0x52,
        _ => return None,
    };
    Some(0x0007_0000u32 | hid as u32)
}

fn apply_picker_button_visuals(ui: &mut egui::Ui) {
    let dark_mode = ui.visuals().dark_mode;
    let visuals = ui.visuals_mut();
    visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    let picker_hover_fill = if dark_mode {
        Color32::from_rgb(60, 60, 65)
    } else {
        Color32::from_rgb(232, 232, 240)
    };
    visuals.widgets.hovered.bg_fill = picker_hover_fill;
    visuals.widgets.hovered.weak_bg_fill = picker_hover_fill;
    visuals.widgets.active.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.active.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.open.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.open.weak_bg_fill = Color32::TRANSPARENT;
    if dark_mode {
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(55, 55, 60));
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(55, 55, 60));
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(184, 92, 112));
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(184, 92, 112));
    } else {
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(210, 210, 218));
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(210, 210, 218));
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(184, 92, 112));
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(184, 92, 112));
    }
}

fn popup_key_group_title(kc: &crate::keycode::Keycode) -> &'static str {
    match kc.value {
        0x0004..=0x001D => "Letters",
        0x001E..=0x0027 => "Numbers",
        0x002D..=0x0038 | 0x0064 | 0x021E..=0x0227 | 0x022D..=0x0238 => "Symbols",
        0x0028..=0x002C | 0x0076 => "Editing",
        _ if matches!(kc.category, KeycodeCategory::Navigation) => "Navigation",
        _ if matches!(kc.category, KeycodeCategory::Function) => "Function keys",
        _ if matches!(kc.category, KeycodeCategory::Modifier) => "Modifiers",
        _ => "Other keys",
    }
}

fn popup_key_button_label(kc: &crate::keycode::Keycode, friendly_mods: bool) -> String {
    if friendly_mods {
        let gui = crate::keycode::gui_mod_name();
        match kc.value {
            0x00E0 => return "Left\nCtrl".into(),
            0x00E4 => return "Right\nCtrl".into(),
            0x00E1 => return "Left\nShift".into(),
            0x00E5 => return "Right\nShift".into(),
            0x00E2 => return "Left\nAlt".into(),
            0x00E6 => return "Right\nAlt".into(),
            0x00E3 => return format!("Left\n{}", gui),
            0x00E7 => return format!("Right\n{}", gui),
            _ => {}
        }
    }
    kc.label.to_string()
}

fn popup_key_button_size(label: &str) -> Vec2 {
    if label.contains('\n') {
        Vec2::new(60.0, 36.0)
    } else if label.len() > 5 {
        Vec2::new(68.0, 34.0)
    } else if label.len() > 3 {
        Vec2::new(56.0, 34.0)
    } else {
        Vec2::new(44.0, 34.0)
    }
}

const KEY_PICKER_POPUP_WIDTH: f32 = 760.0;
const KEY_PICKER_POPUP_HEIGHT: f32 = 560.0;
const KEY_PICKER_SCROLL_HEIGHT: f32 = 430.0;

fn show_grouped_popup_key_buttons(
    ui: &mut egui::Ui,
    keys: Vec<&'static crate::keycode::Keycode>,
    layer_names: &[String],
    friendly_mods: bool,
) -> Option<u16> {
    let group_order = [
        "Letters",
        "Numbers",
        "Symbols",
        "Editing",
        "Navigation",
        "Function keys",
        "Modifiers",
        "Other keys",
    ];
    let mut selected = None;

    for title in group_order {
        let group: Vec<&'static crate::keycode::Keycode> = keys
            .iter()
            .copied()
            .filter(|kc| popup_key_group_title(kc) == title)
            .collect();
        if group.is_empty() {
            continue;
        }

        ui.add_space(2.0);
        ui.label(RichText::new(title).size(11.0).color(Color32::from_gray(150)).strong());
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for kc in &group {
                let label = popup_key_button_label(kc, friendly_mods);
                let size = popup_key_button_size(&label);
                let resp = ui.add(
                    egui::Button::new(RichText::new(label).size(10.5))
                        .min_size(size),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    selected = Some(kc.value);
                }
                resp.on_hover_text(keycode_tooltip(kc.value, &[], layer_names));
            }
        });
        ui.add_space(8.0);
    }

    selected
}

fn show_grouped_popup_choice_buttons(
    ui: &mut egui::Ui,
    groups: Vec<(&'static str, Vec<(u16, String, String)>)>,
) -> Option<u16> {
    let mut selected = None;

    for (title, choices) in groups {
        if choices.is_empty() {
            continue;
        }
        ui.add_space(2.0);
        ui.label(RichText::new(title).size(11.0).color(Color32::from_gray(150)).strong());
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (value, label, tooltip) in choices {
                let size = popup_key_button_size(&label);
                let resp = ui.add(
                    egui::Button::new(RichText::new(label).size(10.5)).min_size(size),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    selected = Some(value);
                }
                resp.on_hover_text(tooltip);
            }
        });
        ui.add_space(8.0);
    }

    selected
}

impl KeycodePicker {
    fn zmk_find_behavior<'a>(&'a self, name: &str) -> Option<&'a BehaviorInfo> {
        self.zmk_behaviors.iter().find(|b| b.display_name == name)
    }

    fn zmk_assign(&mut self, behavior_id: u32, param1: u32, param2: u32) {
        self.zmk_result = Some(ZmkBinding { behavior_id: behavior_id as i32, param1, param2 });
        self.open = false;
    }

    fn finalize_vial_special_tab_close(&mut self) {
        if self.selected_tab == KeycodeTab::Macro {
            if let Some(raw_n) = self.macro_inline_selected {
                if (raw_n as usize) < self.macro_count {
                    self.encode_macro(raw_n as usize);
                    self.result = Some(0x7700 + raw_n as u16);
                    self.macros_dirty = true;
                }
            }
        }
        if self.selected_tab == KeycodeTab::TapDance {
            let td_n = self.tap_dance_editor_open.unwrap_or(0);
            if (td_n as usize) < self.tap_dance_entries.len() {
                self.result = Some(0x5700 + td_n as u16);
                self.tap_dance_dirty = true;
            }
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let macro_key_pick_open = self.macro_key_pick.is_some();
        let layer_pick_open = self.vial_layer_pending.is_some();
        let pending_key_pick_open =
            self.vial_quantum_pending_mod.is_some() || self.vial_quantum_pending_mt.is_some();
        let tap_dance_editor_open = self.tap_dance_editor_open.is_some();
        let td_key_pick_open = self.td_key_pick.is_some();

        self.popup_state.begin_frame(PopupKey::PickerWindow, self.open);
        self.popup_state.begin_frame(PopupKey::MacroKeyPickWindow, macro_key_pick_open);
        self.popup_state.begin_frame(PopupKey::PickLayerWindow, layer_pick_open);
        self.popup_state.begin_frame(PopupKey::PendingKeyPickWindow, pending_key_pick_open);
        self.popup_state.begin_frame(PopupKey::TapDanceEditorWindow, tap_dance_editor_open);
        self.popup_state.begin_frame(PopupKey::TdKeyPickWindow, td_key_pick_open);

        if !self.open { return; }

        // If pending mod/MT — show only the minimal second picker, not the full picker
        let has_pending = self.vial_quantum_pending_mod.is_some() || self.vial_quantum_pending_mt.is_some() ||
            self.vial_layer_pending.is_some();
        if has_pending && self.firmware == FirmwareProtocol::Vial {
            self.show_pending_picker(ctx);
            return;
        }

        // Macro key picker (sub-window of macro editor)
        if let Some((macro_idx, action_idx)) = self.macro_key_pick {
            let mut pick_open = true;
            crate::ui_style::centered_modal_window(
                ctx,
                "Pick key",
                self.popup_state.id(PopupKey::MacroKeyPickWindow),
                &mut pick_open,
                Vec2::new(KEY_PICKER_POPUP_WIDTH, KEY_PICKER_POPUP_HEIGHT),
            )
                .show(ctx, |ui| {
                    apply_picker_button_visuals(ui);
                    crate::ui_style::modal_intro(ui, "Press a key on your keyboard, or click below");
                    crate::ui_style::modal_hint(ui, "Best for normal keys, navigation, media and special actions");
                    ui.add_space(crate::ui_style::modal_space_xs());
                    // Physical key capture
                    ctx.input(|i| {
                        for event in &i.events {
                            if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                                if *key == Key::Escape {
                                    self.macro_key_pick = None;
                                    return;
                                }
                                if let Some(qmk) = egui_key_to_qmk(*key, *modifiers) {
                                    if qmk > 0 && qmk < 0x0100 {
                                        if let Some(action) = self.macro_actions.get_mut(macro_idx).and_then(|a| a.get_mut(action_idx)) {
                                            match action {
                                                MacroAction::Tap(kc) => *kc = qmk as u8,
                                                MacroAction::Down(kc) => *kc = qmk as u8,
                                                MacroAction::Up(kc) => *kc = qmk as u8,
                                                _ => {}
                                            }
                                        }
                                        self.macro_key_pick = None;
                                    }
                                }
                            }
                        }
                    });
                    if ui.add(egui::Button::new(RichText::new("None (clear)").size(12.0))
                        .min_size(crate::ui_style::modal_action_button_size())).clicked() {
                        if let Some(action) = self.macro_actions.get_mut(macro_idx).and_then(|a| a.get_mut(action_idx)) {
                            match action {
                                MacroAction::Tap(kc) => *kc = 0,
                                MacroAction::Down(kc) => *kc = 0,
                                MacroAction::Up(kc) => *kc = 0,
                                _ => {}
                            }
                        }
                        self.macro_key_pick = None;
                    }
                    ui.add_space(4.0);
                    let key_choices: Vec<&'static crate::keycode::Keycode> = KEYCODES.iter()
                        .filter(|kc| {
                            kc.value != 0
                                && kc.value != 0x0001
                                && !kc.name.starts_with("RGB_")
                                && matches!(kc.category, KeycodeCategory::Basic | KeycodeCategory::Function | KeycodeCategory::Navigation | KeycodeCategory::Media | KeycodeCategory::Special)
                                && kc.value < 0x0100
                        })
                        .collect();
                    egui::ScrollArea::vertical()
                        .max_height(KEY_PICKER_SCROLL_HEIGHT)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if let Some(value) = show_grouped_popup_key_buttons(ui, key_choices, &self.layer_names, false) {
                                if let Some(action) = self.macro_actions.get_mut(macro_idx).and_then(|a| a.get_mut(action_idx)) {
                                    match action {
                                        MacroAction::Tap(k) => *k = value as u8,
                                        MacroAction::Down(k) => *k = value as u8,
                                        MacroAction::Up(k) => *k = value as u8,
                                        _ => {}
                                    }
                                }
                                self.macro_key_pick = None;
                            }
                        });
                });
            if !pick_open {
                self.macro_key_pick = None;
            }
            // Don't show macro editor while key picker is open
            return;
        }

        // Tap dance key picker
        if let Some((td_idx, field)) = self.td_key_pick {
            self.show_td_key_picker(ctx, td_idx, field);
            return;
        }


        match self.firmware {
            FirmwareProtocol::Vial => self.show_vial(ctx),
            FirmwareProtocol::Zmk  => self.show_zmk(ctx),
        }
    }

    // ─────────────────────────── VIAL PICKER ────────────────────────────────

    fn show_vial(&mut self, ctx: &egui::Context) {
        if self.selected_tab == KeycodeTab::Layers {
            self.selected_tab = KeycodeTab::Modifiers;
        }
        if self.selected_tab == KeycodeTab::Media {
            self.selected_tab = KeycodeTab::Special;
        }

        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            if self.vial_quantum_pending_mod.is_some() || self.vial_quantum_pending_mt.is_some() {
                self.vial_quantum_pending_mod = None;
                self.vial_quantum_pending_mt = None;
            } else {
                self.finalize_vial_special_tab_close();
                self.open = false;
            }
            return;
        }

        // Physical key capture is disabled on inline macro editing tab and while text inputs are focused
        if self.selected_tab != KeycodeTab::Macro && !ctx.wants_keyboard_input() {
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    // Physical key capture only when no pending mod (avoid accidental assignment)
                    if self.vial_quantum_pending_mod.is_none() && self.vial_quantum_pending_mt.is_none() {
                        if self.search_query.is_empty() || modifiers.any() {
                            if let Some(qmk) = egui_key_to_qmk(*key, *modifiers) {
                                self.result = Some(qmk);
                                self.open = false;
                            }
                        }
                    } else {
                        // Pending mod: only accept basic keys (no mods pressed)
                        if !modifiers.any() {
                            if let Some(qmk) = egui_key_to_qmk(*key, *modifiers) {
                                if qmk > 0 && qmk < 0x0100 {
                                    let base = self.vial_quantum_pending_mod
                                        .or(self.vial_quantum_pending_mt)
                                        .unwrap_or(0);
                                    self.result = Some(base | qmk);
                                    self.vial_quantum_pending_mod = None;
                                    self.vial_quantum_pending_mt = None;
                                    self.open = false;
                                }
                            }
                        }
                    }
                }
            }
        });
        }

        let mut still_open = true;
        let picker_size = Vec2::new(920.0, 560.0);
        crate::ui_style::centered_modal_window(
            ctx,
            "Key Editor",
            self.popup_state.id(PopupKey::PickerWindow),
            &mut still_open,
            picker_size,
        )
            .show(ctx, |ui| {
                apply_picker_button_visuals(ui);
                crate::ui_style::modal_intro(ui, "Press a key on your keyboard, or pick below");
                ui.add_space(4.0);

                if self.selected_tab == KeycodeTab::Rgb && !self.supports_rgb {
                    self.selected_tab = KeycodeTab::Basic;
                }

                // Tab bar
                ui.horizontal_wrapped(|ui| {
                    for tab in KeycodeTab::VIAL_TABS {
                        if *tab == KeycodeTab::Rgb && !self.supports_rgb { continue; }
                        if *tab == KeycodeTab::Custom && self.custom_keycodes.is_empty() { continue; }
                        let active = self.selected_tab == *tab;
                        let text = if active {
                            RichText::new(tab.label())
                                .size(12.0)
                                .color(ui.visuals().widgets.hovered.fg_stroke.color)
                        } else {
                            RichText::new(tab.label()).size(12.0)
                        };
                        let btn = egui::Button::new(text)
                            .fill(if active { Color32::from_rgb(184, 92, 112) } else { Color32::TRANSPARENT });
                        if ui.add(btn).clicked() {
                            self.selected_tab = *tab;
                            self.vial_quantum_pending_mod = None;
                            self.vial_quantum_pending_mt = None;
                            self.vial_layer_pending = None;
                        }
                    }
                });
                ui.separator();

                let content_height = 455.0;
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), content_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_min_height(content_height);
                        egui::ScrollArea::vertical()
                            .max_height(content_height)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.scope(|ui| {
                                    apply_picker_button_visuals(ui);

                                    if self.selected_tab == KeycodeTab::Basic {
                                        ui.add_space(88.0);
                                        self.show_vial_tab_content(ui);
                                    } else {
                                        let centered_width = 840.0_f32.min(ui.available_width());
                                        let x_offset = ((ui.available_width() - centered_width).max(0.0) * 0.5).floor();
                                        if self.selected_tab == KeycodeTab::Symbols {
                                            ui.add_space(88.0);
                                        }
                                        ui.horizontal(|ui| {
                                            if x_offset > 0.0 {
                                                ui.add_space(x_offset);
                                            }
                                            ui.allocate_ui_with_layout(
                                                Vec2::new(centered_width, 0.0),
                                                egui::Layout::top_down(egui::Align::Min),
                                                |ui| self.show_vial_tab_content(ui),
                                            );
                                        });
                                    }
                                });
                            });
                    },
                );
            });

        if !still_open {
            self.finalize_vial_special_tab_close();
            self.open = false;
        }
    }

    fn show_pending_picker(&mut self, ctx: &egui::Context) {
        // Layer picker
        if let Some(base) = self.vial_layer_pending {
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key { key, pressed: true, .. } = event {
                        if *key == egui::Key::Escape {
                            self.vial_layer_pending = None;
                            self.open = false;
                            return;
                        }
                    }
                }
            });
            let mut still_open = true;
            let resp_win = crate::ui_style::centered_modal_window(
                ctx,
                "Pick layer",
                self.popup_state.id(PopupKey::PickLayerWindow),
                &mut still_open,
                Vec2::new(300.0, 120.0),
            )
                .show(ctx, |ui| {
                    apply_picker_button_visuals(ui);
                    crate::ui_style::modal_intro(ui, "Choose which layer (Esc to cancel)");
                    ui.add_space(crate::ui_style::modal_space_sm());
                    ui.horizontal_wrapped(|ui| {
                        for n in 0u16..self.layer_names.len().max(4) as u16 {
                            let raw = self.layer_names.get(n as usize).cloned().unwrap_or(n.to_string());
                            let label = if !raw.is_empty() && raw != n.to_string() {
                                format!("{}: {}", n, raw)
                            } else {
                                format!("Layer {}", n)
                            };
                            let has_content = self.layer_has_content.get(n as usize).copied().unwrap_or(true);
                            let text_color = if has_content {
                                ui.visuals().widgets.inactive.fg_stroke.color
                            } else if ui.visuals().dark_mode {
                                Color32::from_gray(70)
                            } else {
                                Color32::from_gray(150)
                            };
                            let resp = ui.add(egui::Button::new(RichText::new(&label).size(11.0).color(text_color))
                                .min_size(crate::ui_style::modal_small_button_size(84.0)));
                            if resp.clicked() {
                                let value = if base == 0x4000 {
                                    // LT: layer in bits 8..11, tap kc in bits 0..7 (default 0)
                                    0x4000 | ((n & 0xF) << 8)
                                } else {
                                    base + n
                                };
                                self.result = Some(value);
                                self.vial_layer_pending = None;
                                self.open = false;
                            }
                        }
                    });
                });
            if !still_open {
                self.vial_layer_pending = None;
                self.open = false;
            }
            return;
        }

        let pending = self.vial_quantum_pending_mod.or(self.vial_quantum_pending_mt);
        let is_mt = self.vial_quantum_pending_mod.is_none() && self.vial_quantum_pending_mt.is_some();
        // Physical key capture for pending
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    if *key == egui::Key::Escape {
                        self.vial_quantum_pending_mod = None;
                        self.vial_quantum_pending_mt = None;
                        self.open = false;
                        return;
                    }
                    if !modifiers.any() {
                        if let Some(qmk) = egui_key_to_qmk(*key, *modifiers) {
                            if qmk > 0 && qmk < 0x0100 {
                                if let Some(base) = pending {
                                    self.result = Some(base | qmk);
                                    self.vial_quantum_pending_mod = None;
                                    self.vial_quantum_pending_mt = None;
                                    self.open = false;
                                }
                            }
                        }
                    }
                }
            }
        });

        if let Some(base) = pending {
            let title = if is_mt { "Pick tap key (hold = modifier)" } else { "Pick key for modifier combo" };
            let mut still_open = true;
            let resp_win = crate::ui_style::centered_modal_window(
                ctx,
                title,
                self.popup_state.id(PopupKey::PendingKeyPickWindow),
                &mut still_open,
                Vec2::new(KEY_PICKER_POPUP_WIDTH, KEY_PICKER_POPUP_HEIGHT),
            )
                .show(ctx, |ui| {
                    apply_picker_button_visuals(ui);
                    crate::ui_style::modal_intro(ui, "Press a key on your keyboard, or click below (Esc to cancel)");
                    ui.add_space(crate::ui_style::modal_space_sm());
                    let key_choices: Vec<&'static crate::keycode::Keycode> = KEYCODES.iter()
                        .filter(|kc| matches!(kc.category, KeycodeCategory::Basic | KeycodeCategory::Function | KeycodeCategory::Navigation))
                        .filter(|kc| kc.value != 0 && kc.value < 0x0100)
                        .collect();
                    egui::ScrollArea::vertical()
                        .max_height(KEY_PICKER_SCROLL_HEIGHT)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if let Some(value) = show_grouped_popup_key_buttons(ui, key_choices, &self.layer_names, false) {
                                self.result = Some(base | value);
                                self.vial_quantum_pending_mod = None;
                                self.vial_quantum_pending_mt = None;
                                self.open = false;
                            }
                        });
                });
            if !still_open {
                self.vial_quantum_pending_mod = None;
                self.vial_quantum_pending_mt = None;
                self.open = false;
            }
        }
    }

    fn basic_key_button_at(
        &mut self,
        ui: &mut egui::Ui,
        origin: egui::Pos2,
        cell_w: f32,
        cell_h: f32,
        gap: f32,
        row: usize,
        col: usize,
        span: usize,
        label: &str,
        value: u16,
    ) {
        let x = origin.x + col as f32 * (cell_w + gap);
        let right_nav_extra_gap = if col >= 16 && matches!(row, 1 | 2) { 14.0 } else { 0.0 };
        let y = origin.y + row as f32 * (cell_h + gap) + right_nav_extra_gap;
        let width = span as f32 * cell_w + span.saturating_sub(1) as f32 * gap;
        let rect = egui::Rect::from_min_size(egui::pos2(x, y), Vec2::new(width, cell_h));
        let tip = keycode_tooltip(value, &[], &self.layer_names);
        let inactive_stroke = if ui.visuals().dark_mode {
            egui::Stroke::new(1.0, Color32::from_rgb(55, 55, 60))
        } else {
            egui::Stroke::new(1.0, Color32::from_rgb(210, 210, 218))
        };
        let hover_stroke = if ui.visuals().dark_mode {
            egui::Stroke::new(1.0, Color32::from_rgb(55, 55, 60))
        } else {
            egui::Stroke::new(1.0, Color32::from_rgb(210, 210, 218))
        };
        let hover_fill = if ui.visuals().dark_mode {
            Color32::from_rgb(60, 60, 65)
        } else {
            Color32::from_rgb(232, 232, 240)
        };
        let resp = ui.put(
            rect,
            egui::Button::new("")
                .min_size(Vec2::new(width, cell_h))
                .fill(Color32::TRANSPARENT)
                .stroke(inactive_stroke),
        );
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            ui.painter().rect_filled(
                resp.rect,
                ui.visuals().widgets.hovered.corner_radius,
                hover_fill,
            );
            ui.painter().rect_stroke(
                resp.rect,
                ui.visuals().widgets.hovered.corner_radius,
                hover_stroke,
                egui::StrokeKind::Outside,
            );
        }
        let visuals = ui.style().interact(&resp);
        let (top_size, bottom_size) = key_label_font_sizes(label);
        if let Some((top, bottom)) = label.split_once('\n') {
            let top_color = visuals.fg_stroke.color.gamma_multiply(0.75);
            let top_galley = ui.painter().layout_no_wrap(
                top.to_owned(),
                egui::FontId::proportional(top_size.unwrap_or(9.0)),
                top_color,
            );
            let bottom_galley = ui.painter().layout_no_wrap(
                bottom.to_owned(),
                egui::FontId::proportional(bottom_size),
                visuals.fg_stroke.color,
            );
            ui.painter().galley(
                egui::pos2(
                    resp.rect.center().x - top_galley.size().x / 2.0,
                    resp.rect.center().y - 7.0 - top_galley.size().y / 2.0,
                ),
                top_galley,
                top_color,
            );
            ui.painter().galley(
                egui::pos2(
                    resp.rect.center().x - bottom_galley.size().x / 2.0,
                    resp.rect.center().y + 6.0 - bottom_galley.size().y / 2.0,
                ),
                bottom_galley,
                visuals.fg_stroke.color,
            );
        } else {
            let galley = ui.painter().layout_no_wrap(
                label.to_owned(),
                egui::FontId::proportional(bottom_size),
                visuals.fg_stroke.color,
            );
            ui.painter().galley(
                egui::pos2(
                    resp.rect.center().x - galley.size().x / 2.0,
                    resp.rect.center().y - galley.size().y / 2.0,
                ),
                galley,
                visuals.fg_stroke.color,
            );
        }
        if resp.clicked() {
            self.result = Some(value);
            self.open = false;
        }
        resp.on_hover_text(tip);
    }

    fn show_vial_basic(&mut self, ui: &mut egui::Ui) {
        const COLS: usize = 19;
        const ROWS: usize = 6;

        let cell_w = 44.0;
        let cell_h = 38.0;
        let gap = 3.0;
        let width = COLS as f32 * cell_w + (COLS.saturating_sub(1)) as f32 * gap;
        let height = ROWS as f32 * cell_h + (ROWS.saturating_sub(1)) as f32 * gap;

        let keys: &[(usize, usize, usize, &str, u16)] = &[
            (0, 0, 1, "Esc", 0x0029),
            (0, 1, 1, "F1", 0x003A),
            (0, 2, 1, "F2", 0x003B),
            (0, 3, 1, "F3", 0x003C),
            (0, 4, 1, "F4", 0x003D),
            (0, 6, 1, "F5", 0x003E),
            (0, 7, 1, "F6", 0x003F),
            (0, 8, 1, "F7", 0x0040),
            (0, 9, 1, "F8", 0x0041),
            (0, 11, 1, "F9", 0x0042),
            (0, 12, 1, "F10", 0x0043),
            (0, 13, 1, "F11", 0x0044),
            (0, 14, 1, "F12", 0x0045),
            (0, 16, 1, "PrtSc", 0x0046),
            (0, 17, 1, "ScrLk", 0x0047),
            (0, 18, 1, "Pause", 0x0048),
            (1, 0, 1, "`", 0x0035),
            (1, 1, 1, "1", 0x001E),
            (1, 2, 1, "2", 0x001F),
            (1, 3, 1, "3", 0x0020),
            (1, 4, 1, "4", 0x0021),
            (1, 5, 1, "5", 0x0022),
            (1, 6, 1, "6", 0x0023),
            (1, 7, 1, "7", 0x0024),
            (1, 8, 1, "8", 0x0025),
            (1, 9, 1, "9", 0x0026),
            (1, 10, 1, "0", 0x0027),
            (1, 11, 1, "-", 0x002D),
            (1, 12, 1, "=", 0x002E),
            (1, 13, 2, "Backspace", 0x002A),
            (2, 0, 2, "Tab", 0x002B),
            (2, 2, 1, "Q", 0x0014),
            (2, 3, 1, "W", 0x001A),
            (2, 4, 1, "E", 0x0008),
            (2, 5, 1, "R", 0x0015),
            (2, 6, 1, "T", 0x0017),
            (2, 7, 1, "Y", 0x001C),
            (2, 8, 1, "U", 0x0018),
            (2, 9, 1, "I", 0x000C),
            (2, 10, 1, "O", 0x0012),
            (2, 11, 1, "P", 0x0013),
            (2, 12, 1, "[", 0x002F),
            (2, 13, 1, "]", 0x0030),
            (2, 14, 1, "\\", 0x0031),
            (3, 0, 2, "Caps", 0x0039),
            (3, 2, 1, "A", 0x0004),
            (3, 3, 1, "S", 0x0016),
            (3, 4, 1, "D", 0x0007),
            (3, 5, 1, "F", 0x0009),
            (3, 6, 1, "G", 0x000A),
            (3, 7, 1, "H", 0x000B),
            (3, 8, 1, "J", 0x000D),
            (3, 9, 1, "K", 0x000E),
            (3, 10, 1, "L", 0x000F),
            (3, 11, 1, ";", 0x0033),
            (3, 12, 1, "'", 0x0034),
            (3, 13, 2, "Enter", 0x0028),
            (4, 0, 3, "Shift", 0x00E1),
            (4, 3, 1, "Z", 0x001D),
            (4, 4, 1, "X", 0x001B),
            (4, 5, 1, "C", 0x0006),
            (4, 6, 1, "V", 0x0019),
            (4, 7, 1, "B", 0x0005),
            (4, 8, 1, "N", 0x0011),
            (4, 9, 1, "M", 0x0010),
            (4, 10, 1, ",", 0x0036),
            (4, 11, 1, ".", 0x0037),
            (4, 12, 1, "/", 0x0038),
            (4, 13, 2, "Shift", 0x00E5),
            (5, 0, 2, "Ctrl", 0x00E0),
            (5, 2, 1, "Win", 0x00E3),
            (5, 3, 1, "Alt", 0x00E2),
            (5, 4, 7, "Space", 0x002C),
            (5, 11, 1, "Alt", 0x00E6),
            (5, 12, 1, "Menu", 0x0065),
            (5, 13, 2, "Ctrl", 0x00E4),
            (1, 16, 1, "Ins", 0x0049),
            (1, 17, 1, "Home", 0x004A),
            (1, 18, 1, "PgUp", 0x004B),
            (2, 16, 1, "Del", 0x004C),
            (2, 17, 1, "End", 0x004D),
            (2, 18, 1, "PgDn", 0x004E),
            (4, 17, 1, "↑", 0x0052),
            (5, 16, 1, "←", 0x0050),
            (5, 17, 1, "↓", 0x0051),
            (5, 18, 1, "→", 0x004F),
        ];

        let available_width = ui.available_width();
        let x_offset = ((available_width - width).max(0.0) * 0.5).floor();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(available_width, height), egui::Sense::hover());
        let origin = egui::pos2(rect.min.x + x_offset, rect.min.y);
        for &(row, col, span, fallback_label, value) in keys {
            let display_label = crate::keycode::find_keycode(value)
                .map(|_| keycode_label_with_names(value, &[], &self.layer_names))
                .unwrap_or_else(|| fallback_label.to_string());
            self.basic_key_button_at(ui, origin, cell_w, cell_h, gap, row, col, span, &display_label, value);
        }
    }

    fn show_vial_tab_content(&mut self, ui: &mut egui::Ui) {
        match self.selected_tab {
            KeycodeTab::Basic     => self.show_vial_basic(ui),
            KeycodeTab::Layers    => self.show_vial_layers(ui),
            KeycodeTab::Modifiers => self.show_vial_modifiers(ui),
            KeycodeTab::Quantum   => self.show_vial_quantum(ui),
            KeycodeTab::Rgb       => self.show_vial_rgb(ui),
            KeycodeTab::Macro     => self.show_vial_macros(ui),
            KeycodeTab::TapDance  => self.show_vial_tap_dance(ui),
            KeycodeTab::Special   => self.show_vial_special(ui),
            KeycodeTab::Custom    => self.show_vial_custom(ui),
            _ => self.show_vial_generic(ui),
        }
    }

    fn show_vial_generic(&mut self, ui: &mut egui::Ui) {
        let custom_pairs: Vec<crate::keyboard::CustomKeycode> = self.custom_keycodes.iter()
            .map(|(name, label, title, _)| crate::keyboard::CustomKeycode {
                name: name.clone(),
                label: label.clone(),
                title: title.clone(),
            })
            .collect();
        ui.horizontal_wrapped(|ui| {
            for kc in KEYCODES.iter() {
                if !self.selected_tab.vial_matches(kc) { continue; }
                let tip = keycode_tooltip(kc.value, &custom_pairs, &self.layer_names);
                let resp = ui.add(
                    egui::Button::new(RichText::new(kc.label).size(11.0))
                        .min_size(Vec2::new(52.0, 38.0)),
                ).on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() { self.result = Some(kc.value); self.open = false; }
                resp.on_hover_text(tip);
            }
        });
    }

    fn show_vial_custom(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (name, label, title, value) in &self.custom_keycodes {
                if label.is_empty() { continue; }
                let tip = if title.trim().is_empty() { name.as_str() } else { title.as_str() };
                let resp = ui.add(
                    egui::Button::new(RichText::new(label).size(11.0))
                        .min_size(Vec2::new(52.0, 38.0)),
                ).on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(tip);
            }
        });
    }

    fn show_vial_layers(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 8.0);
        let ops: &[(u16, &str, &str)] = &[
            (0x5220, "MO — Momentary",   "Hold to activate, release to return"),
            (0x5260, "TG — Toggle",      "Tap to toggle on/off"),
            (0x5280, "OSL — One-Shot",   "Active for next keypress only"),
            (0x52C0, "TT — Tap-Toggle",  "Hold = MO, tap = toggle"),
            (0x5200, "TO — Switch",      "Switch and stay on this layer"),
            (0x5240, "DF — Default",     "Set as permanent base layer"),
        ];

        ui.label(RichText::new("Layers: choose a layer action, then pick the target layer").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for (base, label, hint) in ops {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(10.5))
                    .min_size(Vec2::new(102.0, 34.0)))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    self.vial_layer_pending = Some(*base);
                }
                resp.on_hover_text(*hint);
            }
            let lt_resp = ui.add(egui::Button::new(RichText::new("LT — Layer-Tap").size(10.5))
                .min_size(Vec2::new(102.0, 34.0)))
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if lt_resp.clicked() {
                self.vial_layer_pending = Some(0x4000);
            }
            lt_resp.on_hover_text("Hold = activate layer, tap = keycode (set key via right-click afterwards)");

        });
    }

    fn show_vial_modifiers(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 8.0);
        let gui = gui_label(false);
        let lgui = gui_label(false);

        ui.label(RichText::new("Plain modifiers").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let plain: Vec<(String, u16, String)> = vec![
            ("Ctrl".into(),         0x00E0, "Left Control".into()),
            ("Shift".into(),        0x00E1, "Left Shift".into()),
            ("Alt".into(),          0x00E2, "Left Alt".into()),
            (gui.into(),            0x00E3, format!("Left {}", lgui)),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in &plain {
                let resp = ui.add(egui::Button::new(RichText::new(label.as_str()).size(10.5))
                    .min_size(Vec2::new(68.0, 34.0)))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(tip.as_str());
            }
        });

        ui.add_space(12.0);
        self.show_vial_layers(ui);

        ui.add_space(12.0);
        ui.label(RichText::new("Mod+Key — always sends modifier+key together").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let mk: Vec<(String, u16, String)> = vec![
            ("Ctrl+key".into(),         0x0100, "Always sends Ctrl+key".into()),
            ("Shift+key".into(),        0x0200, "Always sends Shift+key".into()),
            ("Alt+key".into(),          0x0400, "Always sends Alt+key".into()),
            (format!("{}+key", gui),   0x0800, format!("Always sends {}+key", lgui)),
            ("Ctrl+Shift+key".into(),  0x0300, "Ctrl+Shift+key".into()),
            ("Ctrl+Alt+key".into(),    0x0500, "Ctrl+Alt+key".into()),
            ("Shift+Alt+key".into(),   0x0600, "Shift+Alt+key (LSA)".into()),
            ("Meh+key".into(),         0x0700, "Ctrl+Shift+Alt+key".into()),
            (format!("{}+Sh+key", gui), 0x0A00, format!("{}+Shift+key", lgui)),
            ("Hyper+key".into(),       0x0F00, "Ctrl+Shift+Alt+Win+key".into()),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in &mk {
                let resp = ui.add(egui::Button::new(RichText::new(label.as_str()).size(10.5))
                    .min_size(Vec2::new(74.0, 34.0)))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    self.vial_quantum_pending_mod = Some(*value);
                }
                resp.on_hover_text(tip.as_str());
            }
        });

        ui.add_space(12.0);
        ui.label(RichText::new("Mod-Tap — hold for modifier, tap for regular key").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(2.0);
        ui.add_space(4.0);
        let mt: Vec<(String, u16, String)> = vec![
            ("MT Ctrl".into(),          0x2100, "Mod-Tap: hold=LCtrl".into()),
            ("MT Shift".into(),         0x2200, "Mod-Tap: hold=LShift".into()),
            ("MT Alt".into(),           0x2400, "Mod-Tap: hold=LAlt".into()),
            (format!("MT {}", lgui),   0x2800, format!("Mod-Tap: hold=L{}", lgui)),
            ("MT C+S".into(),          0x2300, "Mod-Tap: hold=Ctrl+Shift".into()),
            ("MT C+A".into(),          0x2500, "Mod-Tap: hold=Ctrl+Alt".into()),
            ("MT S+A".into(),          0x2600, "Mod-Tap: hold=Shift+Alt (LSA)".into()),
            ("MT Meh".into(),          0x2700, "Mod-Tap: hold=Meh (Ctrl+Shift+Alt)".into()),
            ("MT Hyper".into(),        0x2F00, "Mod-Tap: hold=Hyper (Ctrl+Shift+Alt+Win)".into()),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in &mt {
                let resp = ui.add(egui::Button::new(RichText::new(label.as_str()).size(10.5))
                    .min_size(Vec2::new(68.0, 34.0)))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    self.vial_quantum_pending_mt = Some(*value);
                }
                resp.on_hover_text(tip.as_str());
            }
        });

        ui.add_space(12.0);
        ui.label(RichText::new("One-Shot Mod — active for next keypress only").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let osm: Vec<(String, u16, String)> = vec![
            ("OSM Ctrl".into(),         0x52A2, "One-Shot Left Ctrl".into()),
            ("OSM Shift".into(),        0x52A1, "One-Shot Left Shift — tap to capitalise next letter".into()),
            ("OSM Alt".into(),          0x52A4, "One-Shot Left Alt".into()),
            (format!("OSM {}", lgui),   0x52A8, format!("One-Shot Left {}", lgui)),
            ("OSM Meh".into(),          0x52A7, "One-Shot Meh (Ctrl+Shift+Alt)".into()),
            ("OSM Hyper".into(),        0x52AF, "One-Shot Hyper (Ctrl+Shift+Alt+Win)".into()),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in &osm {
                let resp = ui.add(egui::Button::new(RichText::new(label.as_str()).size(10.5))
                    .min_size(Vec2::new(68.0, 34.0)))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(tip.as_str());
            }
        });
    }

    fn show_vial_quantum(&mut self, ui: &mut egui::Ui) {
        // Pending mod+key selection
        if let Some(base) = self.vial_quantum_pending_mod {
            ui.label(RichText::new("Now pick the KEY to add the modifier to:").size(11.5).strong());
            ui.label(RichText::new("Click any key below to create the combo, or Escape to cancel").size(10.5).color(Color32::from_gray(150)));
            ui.add_space(4.0);
            if ui.button("✕ Cancel").clicked() { self.vial_quantum_pending_mod = None; }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for kc in KEYCODES.iter() {
                    if !matches!(kc.category, KeycodeCategory::Basic | KeycodeCategory::Function | KeycodeCategory::Navigation) { continue; }
                    if kc.value >= 0x0200 { continue; }
                    let resp = ui.add(egui::Button::new(RichText::new(kc.label).size(11.0))
                        .min_size(Vec2::new(44.0, 34.0)))
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if resp.clicked() {
                        self.result = Some(base | kc.value);
                        self.vial_quantum_pending_mod = None;
                        self.open = false;
                    }
                    resp.on_hover_text(kc.name);
                }
            });
            return;
        }
        if let Some(base) = self.vial_quantum_pending_mt {
            ui.label(RichText::new("Now pick the TAP key:").size(11.5).strong());
            ui.label(RichText::new("Hold = modifier, tap = this key").size(10.5).color(Color32::from_gray(150)));
            ui.add_space(4.0);
            if ui.button("✕ Cancel").clicked() { self.vial_quantum_pending_mt = None; }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for kc in KEYCODES.iter() {
                    if !matches!(kc.category, KeycodeCategory::Basic | KeycodeCategory::Function | KeycodeCategory::Navigation) { continue; }
                    if kc.value >= 0x0200 { continue; }
                    let resp = ui.add(egui::Button::new(RichText::new(kc.label).size(11.0))
                        .min_size(Vec2::new(44.0, 34.0)))
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if resp.clicked() {
                        self.result = Some(base | kc.value);
                        self.vial_quantum_pending_mt = None;
                        self.open = false;
                    }
                    resp.on_hover_text(kc.name);
                }
            });
            return;
        }

        let gui = gui_sym();
        let lgui = gui_label(false);

        // Mod+Key section
        ui.label(RichText::new("Mod+Key — pick modifier, then key").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let mod_bases: Vec<(String, u16, String)> = vec![
            ("Ctrl+…".into(),       0x0100, "Left Ctrl + next key".into()),
            ("Shift+…".into(),      0x0200, "Left Shift + next key".into()),
            ("Alt+…".into(),        0x0400, "Left Alt + next key".into()),
            (format!("{}+…", gui), 0x0800, format!("{} + next key", lgui)),
            ("C+S+…".into(),       0x0300, "Ctrl+Shift + next key".into()),
            ("C+A+…".into(),       0x0500, "Ctrl+Alt + next key".into()),
            ("S+A+…".into(),       0x0600, "Shift+Alt + next key".into()),
            ("Meh+…".into(),       0x0700, "Ctrl+Shift+Alt + next key".into()),
            ("Hyper+…".into(),     0x0F00, "Ctrl+Shift+Alt+Win + next key".into()),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, base, tip) in &mod_bases {
                let resp = ui.add(egui::Button::new(RichText::new(label.as_str()).size(10.5))
                    .min_size(Vec2::new(72.0, 36.0)))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() { self.vial_quantum_pending_mod = Some(*base); }
                resp.on_hover_text(tip.as_str());
            }
        });

        ui.separator();
        ui.label(RichText::new("Mod-Tap — pick modifier, then tap key").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let mt_bases: Vec<(String, u16, String)> = vec![
            ("MT Ctrl".into(),       0x2100, "Hold=LCtrl, tap=…".into()),
            ("MT Shift".into(),      0x2200, "Hold=LShift, tap=…".into()),
            ("MT Alt".into(),        0x2400, "Hold=LAlt, tap=…".into()),
            (format!("MT {}", lgui), 0x2800, format!("Hold=L{}, tap=…", lgui)),
            ("MT Meh".into(),        0x2700, "Hold=Meh, tap=…".into()),
            ("MT Hyper".into(),      0x2F00, "Hold=Hyper, tap=…".into()),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, base, tip) in &mt_bases {
                let resp = ui.add(egui::Button::new(RichText::new(label.as_str()).size(10.5))
                    .min_size(Vec2::new(72.0, 36.0)))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() { self.vial_quantum_pending_mt = Some(*base); }
                resp.on_hover_text(tip.as_str());
            }
        });
    }

    fn show_macro_editor_contents(&mut self, ui: &mut egui::Ui, raw_n: u8, grid_id: &'static str, add_action_id: &'static str, footer_text: &'static str) -> u8 {
        let mut selected_macro = raw_n;
        ui.label("Choose macro:");
        egui::Frame::none().show(ui, |ui| {
            ui.set_max_height(80.0);
            egui::ScrollArea::vertical().max_height(80.0).auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new(grid_id).num_columns(16).spacing([2.0, 2.0]).show(ui, |ui| {
                    for i in 0..128u8 {
                        let is_active = i == selected_macro;
                        let has_content = self.macro_has_content(i as usize);
                        let display_name = self.macro_display_name(i as usize);
                        let fill = if is_active {
                            Color32::from_rgb(184, 92, 112)
                        } else {
                            Color32::TRANSPARENT
                        };
                        let mut resp = ui.add_sized(Vec2::new(48.0, 28.0), egui::Button::new("").fill(fill));
                        let rect = resp.rect;
                        let painter = ui.painter();
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            if !is_active {
                                let hover_fill = if ui.visuals().dark_mode {
                                    Color32::from_rgb(60, 60, 65)
                                } else {
                                    Color32::from_rgb(232, 232, 240)
                                };
                                painter.rect_filled(rect, ui.visuals().widgets.hovered.corner_radius, hover_fill);
                            }
                        }
                        let text_color = if is_active {
                            Color32::WHITE
                        } else if resp.hovered() {
                            ui.visuals().widgets.hovered.fg_stroke.color
                        } else if has_content {
                            ui.visuals().widgets.inactive.fg_stroke.color
                        } else {
                            inactive_picker_entry_text(ui.visuals().dark_mode)
                        };
                        let id_text = format!("M{}", i);
                        if display_name != id_text {
                            painter.text(
                                egui::pos2(rect.center().x, rect.top() + 8.0),
                                egui::Align2::CENTER_CENTER,
                                &id_text,
                                egui::FontId::proportional(11.0),
                                text_color,
                            );
                            painter.text(
                                egui::pos2(rect.center().x, rect.bottom() - 8.0),
                                egui::Align2::CENTER_CENTER,
                                display_name.clone(),
                                egui::FontId::proportional(11.0),
                                text_color,
                            );
                            resp = resp.on_hover_text(display_name.clone());
                        } else {
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &id_text,
                                egui::FontId::proportional(12.0),
                                text_color,
                            );
                        }
                        if resp.clicked() {
                            self.ensure_macro_meta_len(i as usize);
                            selected_macro = i;
                        }
                        if (i + 1) % 16 == 0 { ui.end_row(); }
                    }
                });
            });
        });
        ui.separator();

        if selected_macro == 254 {
            ui.label(RichText::new("Select a macro above to edit").size(16.0).color(Color32::from_gray(140)));
            return selected_macro;
        }

        let n = selected_macro as usize;
        self.ensure_macro_meta_len(n);

        let macro_font_size = 14.0;
        let macro_hint_font_size = 13.0;
        ui.add_space(4.0);
        if let Some(name) = self.macro_names.get_mut(n) {
            let resp = ui.add_sized(
                Vec2::new(110.0, 28.0),
                egui::TextEdit::singleline(name)
                    .hint_text("Macro name")
                    .font(egui::FontId::proportional(macro_font_size))
                    .horizontal_align(egui::Align::Center)
                    .vertical_align(egui::Align::Center)
            );
            if resp.changed() {
                let trimmed: String = name.chars().take(7).collect();
                *name = trimmed;
            }
        }
        ui.add_space(6.0);

        let mut remove_idx = None;
        let mut move_up: Option<usize> = None;
        let mut move_down: Option<usize> = None;
        let avail_w = ui.available_width();
        {
            let action_count = self.macro_actions[n].len();
            for (i, action) in self.macro_actions[n].iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    let arrow_size = Vec2::new(26.0, 26.0);
                    let up_btn = egui::Button::new(RichText::new("↑").size(macro_font_size))
                        .min_size(arrow_size)
                        .sense(if i > 0 { egui::Sense::click() } else { egui::Sense::hover() });
                    let down_btn = egui::Button::new(RichText::new("↓").size(macro_font_size))
                        .min_size(arrow_size)
                        .sense(if i + 1 < action_count { egui::Sense::click() } else { egui::Sense::hover() });
                    if ui.add(up_btn).on_hover_text("Move up").clicked() && i > 0 { move_up = Some(i); }
                    if ui.add(down_btn).on_hover_text("Move down").clicked() && i + 1 < action_count { move_down = Some(i); }

                    let (type_label, type_color, tooltip) = match action {
                        MacroAction::Text(_) => ("Text", Color32::from_rgb(184, 92, 112), "Types text characters one by one"),
                        MacroAction::Tap(_) => ("Tap", Color32::from_rgb(184, 92, 112), "Press and release a key"),
                        MacroAction::Down(_) => ("Down", Color32::from_rgb(200, 150, 50), "Press a key (hold until Up)"),
                        MacroAction::Up(_) => ("Up", Color32::from_rgb(150, 200, 50), "Release a previously pressed key"),
                        MacroAction::Delay(_) => ("Delay", Color32::from_gray(150), "Wait before next action"),
                    };
                    ui.allocate_ui(Vec2::new(55.0, 30.0), |ui| {
                        ui.add(egui::Label::new(RichText::new(type_label).size(macro_font_size).color(type_color).strong())
                            .sense(egui::Sense::hover()))
                            .on_hover_text(tooltip);
                    });

                    match action {
                        MacroAction::Text(text) => {
                            let text_w = (avail_w - 220.0).max(150.0);
                            ui.add_sized(Vec2::new(text_w, 30.0),
                                egui::TextEdit::singleline(text)
                                .hint_text("Type text here")
                                .font(egui::FontId::proportional(macro_font_size))
                                .horizontal_align(egui::Align::Min)
                                .vertical_align(egui::Align::Center))
                                .on_hover_text("Characters to type when this macro runs");
                        }
                        MacroAction::Tap(kc) => {
                            let label = crate::keycode::KEYCODES.iter().find(|k| k.value == *kc as u16).map(|k| k.label).unwrap_or("?");
                            if ui.add(egui::Button::new(RichText::new(label).size(macro_font_size)).min_size(Vec2::new(100.0, 30.0)))
                                .on_hover_text("Click to change key — press and release this key")
                                .clicked() { self.macro_key_pick = Some((n, i)); }
                        }
                        MacroAction::Down(kc) => {
                            let label = crate::keycode::KEYCODES.iter().find(|k| k.value == *kc as u16).map(|k| k.label).unwrap_or("?");
                            if ui.add(egui::Button::new(RichText::new(label).size(macro_font_size)).min_size(Vec2::new(100.0, 30.0)))
                                .on_hover_text("Click to change key — holds down until Up")
                                .clicked() { self.macro_key_pick = Some((n, i)); }
                        }
                        MacroAction::Up(kc) => {
                            let label = crate::keycode::KEYCODES.iter().find(|k| k.value == *kc as u16).map(|k| k.label).unwrap_or("?");
                            if ui.add(egui::Button::new(RichText::new(label).size(macro_font_size)).min_size(Vec2::new(100.0, 30.0)))
                                .on_hover_text("Click to change key — releases this key")
                                .clicked() { self.macro_key_pick = Some((n, i)); }
                        }
                        MacroAction::Delay(ms) => {
                            let mut ms_str = ms.to_string();
                            if ui.add_sized(Vec2::new(80.0, 30.0),
                                egui::TextEdit::singleline(&mut ms_str)
                                .font(egui::FontId::proportional(macro_font_size))
                                .horizontal_align(egui::Align::Center)
                                .vertical_align(egui::Align::Center)).changed() {
                                if let Ok(v) = ms_str.parse::<u16>() { *ms = v; }
                            }
                            ui.label(RichText::new("ms").size(macro_font_size).color(Color32::from_gray(140))).on_hover_text("Milliseconds to wait");
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new("✕").size(macro_font_size))
                            .min_size(Vec2::new(30.0, 30.0)))
                            .on_hover_text("Remove this action")
                            .clicked() {
                            remove_idx = Some(i);
                        }
                    });
                });
                ui.add_space(2.0);
            }
        }
        if let Some(idx) = remove_idx {
            if idx < self.macro_actions[n].len() {
                self.macro_undo_stack.push((n, self.macro_actions[n].clone()));
                self.macro_actions[n].remove(idx);
                if let Some((mn, ai)) = self.macro_key_pick {
                    if mn == n && ai >= idx {
                        self.macro_key_pick = None;
                    }
                }
            }
        }
        if let Some(idx) = move_up {
            if idx > 0 { self.macro_actions[n].swap(idx, idx - 1); }
        }
        if let Some(idx) = move_down {
            if idx + 1 < self.macro_actions[n].len() { self.macro_actions[n].swap(idx, idx + 1); }
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt(add_action_id)
                .selected_text(RichText::new("+ Add action").size(macro_font_size))
                .width(160.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(false, "Text — type characters").clicked() {
                        self.macro_actions[n].push(MacroAction::Text(String::new()));
                    }
                    if ui.selectable_label(false, "Tap — press and release a key").clicked() {
                        self.macro_actions[n].push(MacroAction::Tap(0x04));
                        self.macro_key_pick = Some((n, self.macro_actions[n].len() - 1));
                    }
                    if ui.selectable_label(false, "Down — hold a key").clicked() {
                        self.macro_actions[n].push(MacroAction::Down(0x04));
                        self.macro_key_pick = Some((n, self.macro_actions[n].len() - 1));
                    }
                    if ui.selectable_label(false, "Up — release a key").clicked() {
                        self.macro_actions[n].push(MacroAction::Up(0x04));
                        self.macro_key_pick = Some((n, self.macro_actions[n].len() - 1));
                    }
                    if ui.selectable_label(false, "Delay — pause (ms)").clicked() {
                        self.macro_actions[n].push(MacroAction::Delay(100));
                    }
                });

            let can_clear_macro = self.macro_has_content(n)
                || self.macro_names.get(n).map(|s| !s.trim().is_empty()).unwrap_or(false);
            if ui.add_enabled(can_clear_macro, egui::Button::new(RichText::new("Clear all").size(macro_font_size)))
                .on_hover_text("Remove all actions from this macro")
                .clicked() {
                self.macro_undo_stack.push((n, self.macro_actions[n].clone()));
                self.macro_actions[n].clear();
                if n < self.macro_texts.len() {
                    self.macro_texts[n].clear();
                }
                if n < self.macro_names.len() {
                    self.macro_names[n].clear();
                }
            }
            if !self.macro_undo_stack.is_empty() {
                if ui.add(egui::Button::new(RichText::new("↩ Undo").size(macro_font_size)))
                    .on_hover_text("Undo last change")
                    .clicked() {
                    if let Some((idx, prev)) = self.macro_undo_stack.pop() {
                        if idx < self.macro_actions.len() {
                            self.macro_actions[idx] = prev;
                        }
                    }
                }
            }
        });

        ui.add_space(4.0);
        ui.label(RichText::new(footer_text)
            .size(macro_hint_font_size).color(Color32::from_gray(120)));

        selected_macro
    }

    fn show_vial_tap_dance(&mut self, ui: &mut egui::Ui) {
        if self.tap_dance_entries.is_empty() {
            self.tap_dance_editor_open = None;
            ui.label(RichText::new("No Tap Dance slots available on this keyboard").size(16.0).color(Color32::from_gray(140)));
            return;
        }

        let selected = match self.tap_dance_editor_open {
            Some(n) if (n as usize) < self.tap_dance_entries.len() => n,
            _ => 0,
        };
        self.tap_dance_editor_open = Some(selected);
        self.ensure_tap_dance_name_len(selected as usize);

        ui.label("Choose tap dance:");
        egui::Frame::none().show(ui, |ui| {
            ui.set_max_height(80.0);
            egui::ScrollArea::vertical().max_height(80.0).auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("tap_dance_grid_inline").num_columns(16).spacing([2.0, 2.0]).show(ui, |ui| {
                    for n in 0..self.tap_dance_entries.len() as u8 {
                        self.ensure_tap_dance_name_len(n as usize);
                        let is_active = n == selected;
                        let display_name = self.tap_dance_display_name(n as usize);
                        let id_text = format!("TD{}", n);
                        let fill = if is_active { Color32::from_rgb(184, 92, 112) } else { Color32::TRANSPARENT };
                        let mut resp = ui.add_sized(Vec2::new(48.0, 28.0), egui::Button::new("").fill(fill));
                        let rect = resp.rect;
                        let painter = ui.painter();
                        if resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            if !is_active {
                                let hover_fill = if ui.visuals().dark_mode {
                                    Color32::from_rgb(60, 60, 65)
                                } else {
                                    Color32::from_rgb(232, 232, 240)
                                };
                                painter.rect_filled(rect, ui.visuals().widgets.hovered.corner_radius, hover_fill);
                            }
                        }
                        let has_content = {
                            let td = &self.tap_dance_entries[n as usize];
                            td.on_tap != 0 || td.on_hold != 0 || td.on_double_tap != 0 || td.on_tap_hold != 0 || td.tapping_term != 200
                        };
                        let text_color = if is_active {
                            Color32::WHITE
                        } else if resp.hovered() {
                            ui.visuals().widgets.hovered.fg_stroke.color
                        } else if has_content {
                            ui.visuals().widgets.inactive.fg_stroke.color
                        } else {
                            inactive_picker_entry_text(ui.visuals().dark_mode)
                        };
                        if display_name != id_text {
                            painter.text(
                                egui::pos2(rect.center().x, rect.top() + 8.0),
                                egui::Align2::CENTER_CENTER,
                                &id_text,
                                egui::FontId::proportional(10.5),
                                text_color,
                            );
                            painter.text(
                                egui::pos2(rect.center().x, rect.bottom() - 8.0),
                                egui::Align2::CENTER_CENTER,
                                display_name.clone(),
                                egui::FontId::proportional(10.5),
                                text_color,
                            );
                            resp = resp.on_hover_text(display_name.clone());
                        } else {
                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &id_text,
                                egui::FontId::proportional(12.0),
                                text_color,
                            );
                        }
                        if resp.clicked() {
                            self.tap_dance_editor_open = Some(n);
                        }
                        if (n + 1) % 16 == 0 { ui.end_row(); }
                    }
                });
            });
        });
        ui.separator();

        let n = self.tap_dance_editor_open.unwrap_or(0) as usize;
        self.ensure_tap_dance_name_len(n);
        let td_font_size = 14.0;
        let td_hint_font_size = 13.0;
        ui.add_space(4.0);
        let prev_name = self.tap_dance_names.get(n).cloned().unwrap_or_default();
        let mut edited_name = prev_name.clone();
        let resp = ui.add_sized(
            Vec2::new(110.0, 28.0),
            egui::TextEdit::singleline(&mut edited_name)
                .hint_text("TD name")
                .font(egui::FontId::proportional(td_font_size))
                .horizontal_align(egui::Align::Center)
                .vertical_align(egui::Align::Center)
        );
        if resp.changed() {
            let trimmed: String = edited_name.chars().take(7).collect();
            if trimmed != prev_name {
                self.push_tap_dance_undo(n);
                self.ensure_tap_dance_name_len(n);
                self.tap_dance_names[n] = trimmed;
                self.tap_dance_dirty = true;
            }
        }
        ui.add_space(8.0);

        let fields = [
            ("On Tap", "Key sent on single tap", 0u8),
            ("On Hold", "Key sent when held", 1),
            ("On Double Tap", "Key sent on double tap", 2),
            ("On Tap + Hold", "Key sent on tap then hold", 3),
        ];

        egui::Grid::new("td_fields_inline").spacing([8.0, 8.0]).show(ui, |ui| {
            for (label, tooltip, field_id) in &fields {
                ui.add(egui::Label::new(RichText::new(*label).size(td_font_size).strong())
                    .sense(egui::Sense::hover()))
                    .on_hover_text(*tooltip);

                let kc = match field_id {
                    0 => self.tap_dance_entries[n].on_tap,
                    1 => self.tap_dance_entries[n].on_hold,
                    2 => self.tap_dance_entries[n].on_double_tap,
                    3 => self.tap_dance_entries[n].on_tap_hold,
                    _ => 0,
                };
                let kc_label = if kc == 0 {
                    "None".to_string()
                } else {
                    crate::keycode::keycode_label_with_names(kc, &[], &self.layer_names)
                };
                if ui.add(egui::Button::new(RichText::new(&kc_label).size(td_font_size))
                    .min_size(Vec2::new(120.0, 30.0)))
                    .on_hover_text(if kc == 0 { "Click to assign a key".to_string() } else { keycode_tooltip(kc, &[], &self.layer_names) })
                    .clicked() {
                    self.td_key_pick = Some((n, *field_id));
                }
                ui.end_row();
            }

            ui.add(egui::Label::new(RichText::new("Tapping Term").size(td_font_size).strong())
                .sense(egui::Sense::hover()))
                .on_hover_text("Time in ms to distinguish tap from hold (default: 200)");
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let prev_term = self.tap_dance_entries[n].tapping_term;
                let mut term_str = prev_term.to_string();
                if ui.add_sized(Vec2::new(76.0, 30.0),
                    egui::TextEdit::singleline(&mut term_str)
                    .font(egui::FontId::proportional(td_font_size))
                    .horizontal_align(egui::Align::Center)
                    .vertical_align(egui::Align::Center)).changed() {
                    if let Ok(v) = term_str.parse::<u16>() {
                        let v = v.clamp(10, 3000);
                        if v != prev_term {
                            self.push_tap_dance_undo(n);
                            self.tap_dance_entries[n].tapping_term = v;
                            self.tap_dance_dirty = true;
                        }
                    }
                }
                ui.label(RichText::new("ms").size(td_font_size).color(Color32::from_gray(140)));
            });
            ui.end_row();
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let can_clear_tap_dance = self.tap_dance_entries.get(n).map(|td| {
                td.on_tap != 0 || td.on_hold != 0 || td.on_double_tap != 0 || td.on_tap_hold != 0 || td.tapping_term != 200
            }).unwrap_or(false)
                || self.tap_dance_names.get(n).map(|s| !s.trim().is_empty()).unwrap_or(false);
            if ui.add_enabled(can_clear_tap_dance, egui::Button::new(RichText::new("Clear all").size(td_font_size)))
                .on_hover_text("Clear all actions for this tap dance")
                .clicked() {
                self.push_tap_dance_undo(n);
                if let Some(td) = self.tap_dance_entries.get_mut(n) {
                    td.on_tap = 0;
                    td.on_hold = 0;
                    td.on_double_tap = 0;
                    td.on_tap_hold = 0;
                    td.tapping_term = 200;
                }
                if n < self.tap_dance_names.len() {
                    self.tap_dance_names[n].clear();
                }
                self.tap_dance_dirty = true;
            }
            let can_undo_current = self.tap_dance_undo_stack.iter().any(|(idx, _, _)| *idx == n);
            if can_undo_current {
                if ui.add(egui::Button::new(RichText::new("↩ Undo").size(td_font_size)))
                    .on_hover_text("Undo last tap dance change")
                    .clicked() {
                    if let Some(pos) = self.tap_dance_undo_stack.iter().rposition(|(idx, _, _)| *idx == n) {
                        let (idx, prev, prev_name) = self.tap_dance_undo_stack.remove(pos);
                        if idx < self.tap_dance_entries.len() {
                            self.tap_dance_entries[idx] = prev;
                        }
                        self.ensure_tap_dance_name_len(idx);
                        if idx < self.tap_dance_names.len() {
                            self.tap_dance_names[idx] = prev_name;
                        }
                        self.tap_dance_editor_open = Some(idx as u8);
                        self.tap_dance_dirty = true;
                    }
                }
            }
        });

        ui.add_space(4.0);
        ui.label(RichText::new("Saved to keyboard when you close the keycode picker")
            .size(td_hint_font_size).color(Color32::from_gray(120)));
    }

    fn show_tap_dance_editor(&mut self, ctx: &egui::Context, active_td: u8) {
        if ctx.input(|i| i.key_pressed(Key::Escape)) && self.td_key_pick.is_none() {
            self.tap_dance_editor_open = None;
            self.tap_dance_dirty = true;
            if active_td < self.tap_dance_entries.len() as u8 {
                self.result = Some(0x5700 + active_td as u16); // TD keycode
            }
            self.open = false;
            return;
        }

        let mut still_open = true;
        crate::ui_style::centered_modal_window(
            ctx,
            "Tap Dance Editor",
            self.popup_state.id(PopupKey::TapDanceEditorWindow),
            &mut still_open,
            Vec2::new(600.0, 420.0),
        )
            .show(ctx, |ui| {
                // Tabs
                ui.horizontal_wrapped(|ui| {
                    for n in 0..self.tap_dance_entries.len() as u8 {
                        let is_active = n == active_td;
                        let label = format!("TD{}", n);
                        let btn = egui::Button::new(
                            RichText::new(&label).size(14.0)
                                .color(if is_active { Color32::WHITE } else { Color32::from_gray(100) })
                        ).fill(if is_active { Color32::from_rgb(184, 92, 112) } else { Color32::TRANSPARENT })
                         .min_size(crate::ui_style::modal_tab_button_size());
                        if ui.add(btn).clicked() {
                            self.tap_dance_editor_open = Some(n);
                        }
                    }
                });
                ui.separator();

                if self.tap_dance_entries.is_empty() {
                    ui.label(RichText::new("No Tap Dance slots available on this keyboard").size(16.0).color(Color32::from_gray(140)));
                    return;
                }

                if active_td == 255 || active_td as usize >= self.tap_dance_entries.len() {
                    ui.label(RichText::new("Select a tap dance tab above to edit").size(16.0).color(Color32::from_gray(140)));
                    return;
                }

                let n = active_td as usize;
                ui.label(RichText::new(format!("TD{}", n)).size(18.0).strong());
                ui.add_space(8.0);

                let fields = [
                    ("On Tap", "Key sent on single tap", 0u8),
                    ("On Hold", "Key sent when held", 1),
                    ("On Double Tap", "Key sent on double tap", 2),
                    ("On Tap + Hold", "Key sent on tap then hold", 3),
                ];

                egui::Grid::new("td_fields").spacing([8.0, 8.0]).show(ui, |ui| {
                    for (label, tooltip, field_id) in &fields {
                        ui.add(egui::Label::new(RichText::new(*label).size(15.0).strong())
                            .sense(egui::Sense::hover()))
                            .on_hover_text(*tooltip);

                        let kc = match field_id {
                            0 => self.tap_dance_entries[n].on_tap,
                            1 => self.tap_dance_entries[n].on_hold,
                            2 => self.tap_dance_entries[n].on_double_tap,
                            3 => self.tap_dance_entries[n].on_tap_hold,
                            _ => 0,
                        };
                        let kc_label = if kc == 0 {
                            "None".to_string()
                        } else {
                            crate::keycode::keycode_label_with_names(kc, &[], &self.layer_names)
                        };
                        if ui.add(egui::Button::new(RichText::new(&kc_label).size(16.0))
                            .min_size(crate::ui_style::modal_field_button_size(132.0)))
                            .on_hover_text(if kc == 0 { "Click to assign a key".to_string() } else { keycode_tooltip(kc, &[], &self.layer_names) })
                            .clicked() {
                            self.td_key_pick = Some((n, *field_id));
                        }
                        ui.end_row();
                    }

                    // Tapping term
                    ui.add(egui::Label::new(RichText::new("Tapping Term").size(15.0).strong())
                        .sense(egui::Sense::hover()))
                        .on_hover_text("Time in ms to distinguish tap from hold (default: 200)");
                    let mut term_str = self.tap_dance_entries[n].tapping_term.to_string();
                    ui.horizontal(|ui| {
                        if ui.add_sized(crate::ui_style::modal_small_button_size(80.0),
                            egui::TextEdit::singleline(&mut term_str)
                            .font(egui::FontId::monospace(14.0))).changed() {
                            if let Ok(v) = term_str.parse::<u16>() {
                                self.tap_dance_entries[n].tapping_term = v;
                            }
                        }
                        ui.label(RichText::new("ms").size(14.0));
                    });
                    ui.end_row();
                });
            });

        if !still_open {
            if active_td < self.tap_dance_entries.len() as u8 {
                self.result = Some(0x5700 + active_td as u16);
            }
            self.tap_dance_editor_open = None;
            self.tap_dance_dirty = true;
            self.open = false;
        }
    }

    fn show_td_key_picker(&mut self, ctx: &egui::Context, td_idx: usize, field: u8) {
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.td_key_pick = None;
            return;
        }

        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    if let Some(qmk) = egui_key_to_qmk(*key, *modifiers) {
                        if qmk > 0 && qmk < 0x0100 {
                            self.set_tap_dance_field(td_idx, field, qmk);
                            self.td_key_pick = None;
                        }
                    }
                }
            }
        });

        let field_name = match field {
            0 => "On Tap", 1 => "On Hold", 2 => "On Double Tap", 3 => "On Tap+Hold", _ => "?"
        };
        let helper_text = match field {
            0 => "Best for normal keys, navigation, media and special actions",
            1 => "Hold actions are limited to left/right modifiers and layers",
            2 => "Best for a second tap action, usually another normal key or command",
            3 => "Tap-then-hold actions are limited to left/right modifiers and layers",
            _ => "Press a key on your keyboard, or click below (Esc to cancel)",
        };
        let td_choices: Vec<(u16, String, String)> = if matches!(field, 1 | 3) {
            let gui = gui_label(false).to_string();
            let mut out: Vec<(u16, String, String)> = vec![
                (0x00E0, "Left\nCtrl".into(), "Left Control".into()),
                (0x00E4, "Right\nCtrl".into(), "Right Control".into()),
                (0x00E1, "Left\nShift".into(), "Left Shift".into()),
                (0x00E5, "Right\nShift".into(), "Right Shift".into()),
                (0x00E2, "Left\nAlt".into(), "Left Alt".into()),
                (0x00E6, "Right\nAlt".into(), "Right Alt".into()),
                (0x00E3, format!("Left\n{}", gui), format!("Left {}", gui)),
                (0x00E7, format!("Right\n{}", gui), format!("Right {}", gui)),
            ];
            out.extend(self.tap_dance_layer_choices().into_iter().map(|(value, _label)| {
                let layer = (value & 0x1F) as usize;
                let layer_name = self.layer_names.get(layer).cloned().unwrap_or_else(|| layer.to_string());
                (value, format!("MO({})\n{}", layer, layer_name), format!("Momentarily activate layer {} while held", layer_name))
            }));
            out
        } else {
            KEYCODES.iter()
                .filter(|kc| {
                    kc.value != 0
                        && kc.value != 0x0001
                        && !kc.name.starts_with("RGB_")
                        && matches!(kc.category, KeycodeCategory::Basic | KeycodeCategory::Function | KeycodeCategory::Navigation | KeycodeCategory::Media | KeycodeCategory::Special)
                })
                .map(|kc| (
                    kc.value,
                    keycode_label_with_names(kc.value, &[], &self.layer_names),
                    keycode_tooltip(kc.value, &[], &self.layer_names),
                ))
                .collect()
        };
        let mut still_open = true;
        crate::ui_style::centered_modal_window(
            ctx,
            &format!("Pick key for {}", field_name),
            self.popup_state.id(PopupKey::TdKeyPickWindow),
            &mut still_open,
            Vec2::new(KEY_PICKER_POPUP_WIDTH, KEY_PICKER_POPUP_HEIGHT),
        )
            .show(ctx, |ui| {
                apply_picker_button_visuals(ui);
                crate::ui_style::modal_intro(ui, "Press a key on your keyboard, or click below (Esc to cancel)");
                crate::ui_style::modal_hint(ui, helper_text);
                ui.add_space(crate::ui_style::modal_space_xs());
                if ui.add(egui::Button::new(RichText::new("None (clear)").size(12.0))
                    .min_size(crate::ui_style::modal_action_button_size())).clicked() {
                    self.set_tap_dance_field(td_idx, field, 0);
                    self.td_key_pick = None;
                }
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .max_height(KEY_PICKER_SCROLL_HEIGHT)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if matches!(field, 1 | 3) {
                            let modifier_choices: Vec<(u16, String, String)> = td_choices.iter().take(8).cloned().collect();
                            let layer_choices: Vec<(u16, String, String)> = td_choices.iter().skip(8).cloned().collect();
                            let groups = vec![
                                ("Modifiers", modifier_choices),
                                ("Layers", layer_choices),
                            ];
                            if let Some(value) = show_grouped_popup_choice_buttons(ui, groups) {
                                self.set_tap_dance_field(td_idx, field, value);
                                self.td_key_pick = None;
                            }
                        } else {
                            let key_choices: Vec<&'static crate::keycode::Keycode> = KEYCODES.iter()
                                .filter(|kc| {
                                    kc.value != 0
                                        && kc.value != 0x0001
                                        && !kc.name.starts_with("RGB_")
                                        && matches!(kc.category, KeycodeCategory::Basic | KeycodeCategory::Function | KeycodeCategory::Navigation | KeycodeCategory::Media | KeycodeCategory::Special)
                                })
                                .collect();
                            if let Some(value) = show_grouped_popup_key_buttons(ui, key_choices, &self.layer_names, false) {
                                self.set_tap_dance_field(td_idx, field, value);
                                self.td_key_pick = None;
                            }
                        }
                    });
            });
        if !still_open { self.td_key_pick = None; }
    }

    fn macro_has_content(&self, n: usize) -> bool {
        self.macro_actions.get(n).map(|a| !a.is_empty()).unwrap_or(false)
            || self.macro_texts.get(n).map(|s| !s.is_empty()).unwrap_or(false)
    }

    fn ensure_macro_meta_len(&mut self, n: usize) {
        while self.macro_texts.len() <= n { self.macro_texts.push(String::new()); }
        while self.macro_names.len() <= n { self.macro_names.push(String::new()); }
        while self.macro_actions.len() <= n { self.macro_actions.push(vec![]); }
    }

    fn macro_display_name(&self, n: usize) -> String {
        match self.macro_names.get(n) {
            Some(name) if !name.trim().is_empty() => name.clone(),
            _ => format!("M{}", n),
        }
    }

    fn macro_custom_name(&self, n: usize) -> Option<String> {
        self.macro_names.get(n).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
    }

    fn ensure_tap_dance_name_len(&mut self, n: usize) {
        while self.tap_dance_names.len() <= n { self.tap_dance_names.push(String::new()); }
    }

    fn tap_dance_display_name(&self, n: usize) -> String {
        match self.tap_dance_names.get(n) {
            Some(name) if !name.trim().is_empty() => name.clone(),
            _ => format!("TD{}", n),
        }
    }

    fn push_tap_dance_undo(&mut self, n: usize) {
        self.ensure_tap_dance_name_len(n);
        if let Some(td) = self.tap_dance_entries.get(n).cloned() {
            let name = self.tap_dance_names.get(n).cloned().unwrap_or_default();
            self.tap_dance_undo_stack.push((n, td, name));
        }
    }

    fn tap_dance_layer_choices(&self) -> Vec<(u16, String)> {
        let count = self.layer_count.max(1).min(self.layer_names.len().max(1));
        (0..count)
            .map(|layer| {
                let name = self.layer_names.get(layer).cloned().unwrap_or_else(|| layer.to_string());
                (0x5220 | layer as u16, format!("MO({})", name))
            })
            .collect()
    }

    fn set_tap_dance_field(&mut self, n: usize, field: u8, value: u16) {
        if n >= self.tap_dance_entries.len() { return; }
        let current = match self.tap_dance_entries.get(n) {
            Some(td) => match field {
                0 => td.on_tap,
                1 => td.on_hold,
                2 => td.on_double_tap,
                3 => td.on_tap_hold,
                _ => return,
            },
            None => return,
        };
        if current == value { return; }
        self.push_tap_dance_undo(n);
        if let Some(td) = self.tap_dance_entries.get_mut(n) {
            match field {
                0 => td.on_tap = value,
                1 => td.on_hold = value,
                2 => td.on_double_tap = value,
                3 => td.on_tap_hold = value,
                _ => {}
            }
        }
        self.tap_dance_dirty = true;
    }

    fn encode_macro(&mut self, n: usize) {
        while self.macro_texts.len() <= n { self.macro_texts.push(String::new()); }
        while self.macro_actions.len() <= n { self.macro_actions.push(vec![]); }
        let mut encoded = Vec::new();
        for action in &self.macro_actions[n] {
            match action {
                MacroAction::Text(s) => encoded.extend_from_slice(s.as_bytes()),
                MacroAction::Tap(kc) => { encoded.push(1); encoded.push(1); encoded.push(*kc); }
                MacroAction::Down(kc) => { encoded.push(1); encoded.push(2); encoded.push(*kc); }
                MacroAction::Up(kc) => { encoded.push(1); encoded.push(3); encoded.push(*kc); }
                MacroAction::Delay(ms) => {
                    let hi = (*ms / 255 + 1) as u8;
                    let lo = (*ms % 255 + 1) as u8;
                    encoded.push(1); encoded.push(4); encoded.push(lo); encoded.push(hi);
                }
            }
        }
        self.macro_texts[n] = String::from_utf8_lossy(&encoded).to_string();
    }

    fn show_vial_macros(&mut self, ui: &mut egui::Ui) {
        let previous = self.macro_inline_selected.unwrap_or(0);
        let selected = self.show_macro_editor_contents(
            ui,
            previous,
            "macro_grid_inline",
            "add_action_inline",
            "Saved to keyboard when you close the keycode picker",
        );
        if selected != previous && (previous as usize) < self.macro_count {
            self.encode_macro(previous as usize);
            self.macros_dirty = true;
        }
        self.macro_inline_selected = Some(selected);
    }

    fn show_vial_rgb(&mut self, ui: &mut egui::Ui) {
        // Backlight
        ui.label(RichText::new("Backlight").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let bl_keys: &[(&str, u16, &str)] = &[
            ("Toggle",       0x7800, "Toggle backlight on/off"),
            ("Cycle",        0x7801, "Cycle through backlight brightness levels"),
            ("Breathing",    0x7802, "Toggle breathing effect on/off"),
            ("On",           0x7805, "Turn backlight on"),
            ("Off",          0x7806, "Turn backlight off"),
            ("Brightness -", 0x7804, "Decrease backlight brightness"),
            ("Brightness +", 0x7803, "Increase backlight brightness"),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in bl_keys {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(10.5))
                    .min_size(Vec2::new(80.0, 36.0)))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(*tip);
            }
        });

        ui.add_space(12.0);
        // RGB Underglow (QMK rgblight)
        ui.label(RichText::new("RGB Underglow").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let rgb_keys: &[(&str, u16, &str)] = &[
            ("Toggle",       0x7A00, "Toggle RGB lighting on/off"),
            ("Prev Mode",    0x7A02, "Switch to previous RGB animation mode"),
            ("Next Mode",    0x7A01, "Switch to next RGB animation mode"),
            ("Hue -",        0x7A04, "Decrease color hue"),
            ("Hue +",        0x7A03, "Increase color hue"),
            ("Saturation -", 0x7A06, "Decrease color saturation"),
            ("Saturation +", 0x7A05, "Increase color saturation"),
            ("Brightness -", 0x7A08, "Decrease brightness"),
            ("Brightness +", 0x7A07, "Increase brightness"),
            ("Speed -",      0x7A0A, "Decrease animation speed"),
            ("Speed +",      0x7A09, "Increase animation speed"),
            ("Effect -",     0x7A0C, "Previous RGB effect"),
            ("Effect +",     0x7A0B, "Next RGB effect"),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in rgb_keys {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(10.5))
                    .min_size(Vec2::new(80.0, 36.0)))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(*tip);
            }
        });

        ui.add_space(12.0);
        // RGB Matrix modes
        ui.label(RichText::new("RGB Matrix Modes").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let rgbm_keys: &[(&str, u16, &str)] = &[
            ("Plain",        0x7A0D, "RGB Matrix: solid color, no animation"),
            ("Breathe",      0x7A0E, "RGB Matrix: breathing effect — smooth brightness fade"),
            ("Rainbow",      0x7A0F, "RGB Matrix: rainbow gradient across all keys"),
            ("Swirl",        0x7A10, "RGB Matrix: swirling rainbow pattern"),
            ("Snake",        0x7A11, "RGB Matrix: snake animation moving across keys"),
            ("Knight",       0x7A12, "RGB Matrix: Knight Rider scanning effect"),
            ("Xmas",         0x7A13, "RGB Matrix: alternating red and green like Christmas lights"),
            ("Gradient",     0x7A14, "RGB Matrix: static gradient effect"),
            ("Test",         0x7A15, "RGB Matrix: test mode — cycles through R, G, B"),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in rgbm_keys {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(10.5))
                    .min_size(Vec2::new(80.0, 36.0)))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(*tip);
            }
        });

        ui.add_space(12.0);
        // RGB Matrix controls
        ui.label(RichText::new("RGB Matrix Controls").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let rgbm_ctrl: &[(&str, u16, &str)] = &[
            ("On",           0x7A16, "Turn RGB Matrix on"),
            ("Off",          0x7A17, "Turn RGB Matrix off"),
            ("Toggle",       0x7A18, "Toggle RGB Matrix on/off"),
            ("Previous",     0x7A1A, "Previous RGB Matrix animation"),
            ("Next",         0x7A19, "Next RGB Matrix animation"),
            ("Hue -",        0x7A1C, "Decrease RGB Matrix hue"),
            ("Hue +",        0x7A1B, "Increase RGB Matrix hue"),
            ("Saturation -", 0x7A1E, "Decrease RGB Matrix saturation"),
            ("Saturation +", 0x7A1D, "Increase RGB Matrix saturation"),
            ("Brightness -", 0x7A20, "Decrease RGB Matrix brightness"),
            ("Brightness +", 0x7A1F, "Increase RGB Matrix brightness"),
            ("Speed -",      0x7A22, "Decrease RGB Matrix animation speed"),
            ("Speed +",      0x7A21, "Increase RGB Matrix animation speed"),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in rgbm_ctrl {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(10.5))
                    .min_size(Vec2::new(80.0, 36.0)))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(*tip);
            }
        });
    }

    fn show_vial_special(&mut self, ui: &mut egui::Ui) {
        let special_keys: Vec<(String, u16, String)> = vec![
            ("✕
None".into(),       0x0000, "KC_NO — disables this key completely, it sends nothing when pressed".into()),
            ("▽
Inherit".into(),    0x0001, "KC_TRNS — inherits the key from the layer below".into()),
            ("Esc
~".into(),        0x7C16, format!("Grave/Escape — sends Esc normally, ` when Shift or {} is held", gui_mod_name())),
            ("⚡
Boot".into(),      0x7C00, "QK_BOOT — put keyboard into flash mode".into()),
            ("🐛
Debug".into(),     0x7C02, "DB_TOGG — toggle debug mode".into()),
            ("🔒
Lock".into(),      0x7800, "QK_LOCK — hold to lock remaining keys until pressed again".into()),
            ("Auto
Shift".into(),  0x7C15, "Toggles the state of the Auto Shift feature".into()),
            ("Combo
Toggle".into(),0x7C52, "Toggles Combo feature on and off".into()),
            ("Caps
Word".into(),   0x7C73, "Capitalizes until end of current word".into()),
            ("Repeat".into(),       0x7C79, "Repeats the last pressed key".into()),
            ("Alt
Repeat".into(),  0x7C7A, "Alt repeats the last pressed key".into()),
        ];
        let extra_fn_keys: &[(&str, u16)] = &[
            ("F13", 0x0068), ("F14", 0x0069), ("F15", 0x006A), ("F16", 0x006B),
            ("F17", 0x006C), ("F18", 0x006D), ("F19", 0x006E), ("F20", 0x006F),
            ("F21", 0x0070), ("F22", 0x0071), ("F23", 0x0072), ("F24", 0x0073),
        ];
        ui.label(RichText::new("Special QMK keys").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in &special_keys {
                let resp = ui.add_sized(Vec2::new(56.0, 42.0), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let visuals = ui.style().interact(&resp);
                let galleys: Vec<_> = label
                    .split('\n')
                    .map(|line| {
                        ui.painter().layout_no_wrap(
                            line.to_owned(),
                            egui::FontId::proportional(10.5),
                            visuals.fg_stroke.color,
                        )
                    })
                    .collect();
                let line_spacing = 1.0;
                let total_height: f32 = galleys.iter().map(|galley| galley.size().y).sum::<f32>()
                    + line_spacing * (galleys.len().saturating_sub(1) as f32);
                let mut y = resp.rect.center().y - total_height / 2.0;
                for galley in galleys {
                    let x = resp.rect.center().x - galley.size().x / 2.0;
                    let pos = egui::pos2(x, y);
                    let height = galley.size().y;
                    ui.painter().galley(pos, galley, visuals.fg_stroke.color);
                    y += height + line_spacing;
                }
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(tip.as_str());
            }
        });

        ui.add_space(10.0);
        ui.label(RichText::new("Mouse").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for kc in crate::keycode::KEYCODES.iter().filter(|kc| matches!(kc.category, crate::keycode::KeycodeCategory::Mouse)) {
                let resp = ui.add_sized(Vec2::new(56.0, 42.0), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let visuals = ui.style().interact(&resp);
                let icon_color = visuals.fg_stroke.color.gamma_multiply(0.6);
                if let Some(suffix) = kc.label.strip_prefix("🖱") {
                    let mouse_galley = ui.painter().layout_no_wrap(
                        "🖱".to_owned(),
                        egui::FontId::proportional(15.5),
                        icon_color,
                    );
                    let suffix_galley = ui.painter().layout_no_wrap(
                        suffix.to_owned(),
                        egui::FontId::proportional(10.5),
                        visuals.fg_stroke.color,
                    );
                    let spacing = if suffix.is_empty() { 0.0 } else { 1.0 };
                    let mouse_width = mouse_galley.size().x;
                    let mouse_height = mouse_galley.size().y;
                    let suffix_width = suffix_galley.size().x;
                    let suffix_height = suffix_galley.size().y;
                    let total_width = mouse_width + spacing + suffix_width;
                    let start_x = resp.rect.center().x - total_width / 2.0;
                    let mouse_pos = egui::pos2(start_x, resp.rect.center().y - mouse_height / 2.0);
                    ui.painter().galley(mouse_pos, mouse_galley, icon_color);
                    if !suffix.is_empty() {
                        let suffix_pos = egui::pos2(
                            start_x + mouse_width + spacing,
                            resp.rect.center().y - suffix_height / 2.0,
                        );
                        ui.painter().galley(suffix_pos, suffix_galley, visuals.fg_stroke.color);
                    }
                } else if let Some((icon, text)) = kc.label.split_once(' ') {
                    let icon_galley = ui.painter().layout_no_wrap(
                        icon.to_owned(),
                        egui::FontId::proportional(11.0),
                        icon_color,
                    );
                    let text_galley = ui.painter().layout_no_wrap(
                        text.to_owned(),
                        egui::FontId::proportional(10.5),
                        visuals.fg_stroke.color,
                    );
                    let spacing = 2.0;
                    let icon_width = icon_galley.size().x;
                    let icon_height = icon_galley.size().y;
                    let text_height = text_galley.size().y;
                    let total_width = icon_width + spacing + text_galley.size().x;
                    let start_x = resp.rect.center().x - total_width / 2.0;
                    let icon_pos = egui::pos2(start_x, resp.rect.center().y - icon_height / 2.0);
                    ui.painter().galley(icon_pos, icon_galley, icon_color);
                    let text_pos = egui::pos2(
                        start_x + icon_width + spacing,
                        resp.rect.center().y - text_height / 2.0,
                    );
                    ui.painter().galley(text_pos, text_galley, visuals.fg_stroke.color);
                } else {
                    let galley = ui.painter().layout_no_wrap(
                        kc.label.to_owned(),
                        egui::FontId::proportional(10.5),
                        visuals.fg_stroke.color,
                    );
                    let pos = egui::pos2(
                        resp.rect.center().x - galley.size().x / 2.0,
                        resp.rect.center().y - galley.size().y / 2.0,
                    );
                    ui.painter().galley(pos, galley, visuals.fg_stroke.color);
                }
                if resp.clicked() { self.result = Some(kc.value); self.open = false; }
                resp.on_hover_text(crate::keycode::keycode_tooltip(kc.value, &[], &self.layer_names));
            }
        });

        let media_keys: &[(&str, &str, u16)] = &[
            ("⏻", "Power",   0x00A5),
            ("🌙", "Sleep",   0x00A6),
            ("☀", "Wake",    0x00A7),
            ("🔇", "Mute",    0x00A8),
            ("🔊", "Vol+",    0x00A9),
            ("🔉", "Vol-",    0x00AA),
            ("⏭", "Next",    0x00AB),
            ("⏮", "Prev",    0x00AC),
            ("⏹", "Stop",    0x00AD),
            ("⏯", "Play",    0x00AE),
            ("🎵", "Media",   0x00AF),
            ("⏏", "Eject",   0x00B0),
            ("✉", "Mail",    0x00B1),
            ("∑", "Calc",    0x00B2),
            ("📁", "Files",   0x00B3),
            ("🔍", "Search",  0x00B4),
            ("🌐", "Home",    0x00B5),
            ("←", "Back",    0x00B6),
            ("→", "Fwd",     0x00B7),
            ("⏹", "Web",     0x00B8),
            ("↻", "Reload",  0x00B9),
            ("★", "Favs",    0x00BA),
            ("⏩", "Fast+",   0x00BB),
            ("⏪", "Rewind",  0x00BC),
            ("🔆", "Bright+", 0x00BD),
            ("🔅", "Bright-", 0x00BE),
            ("🪟", "Mission", 0x00BF),
            ("🚀", "Launch",  0x00C0),
        ];
        let basic_app_keys: &[(&str, u16)] = &[
            ("Exec",   0x0074),
            ("Help",   0x0075),
            ("Select", 0x0077),
            ("Stop",   0x0078),
            ("Again",  0x0079),
            ("Undo",   0x007A),
            ("Cut",    0x007B),
            ("Copy",   0x007C),
            ("Paste",  0x007D),
            ("Find",   0x007E),
        ];

        ui.add_space(10.0);
        ui.label(RichText::new("Media, Apps, System").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (icon, text, value) in media_keys {
                let resp = ui.add_sized(Vec2::new(56.0, 42.0), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let visuals = ui.style().interact(&resp);
                let icon_color = visuals.fg_stroke.color.gamma_multiply(0.6);
                let icon_galley = ui.painter().layout_no_wrap(
                    (*icon).to_owned(),
                    egui::FontId::proportional(10.5),
                    icon_color,
                );
                let text_galley = ui.painter().layout_no_wrap(
                    (*text).to_owned(),
                    egui::FontId::proportional(10.5),
                    visuals.fg_stroke.color,
                );
                let line_spacing = 1.0;
                let icon_width = icon_galley.size().x;
                let icon_height = icon_galley.size().y;
                let text_width = text_galley.size().x;
                let total_height = icon_height + line_spacing + text_galley.size().y;
                let icon_pos = egui::pos2(
                    resp.rect.center().x - icon_width / 2.0,
                    resp.rect.center().y - total_height / 2.0,
                );
                ui.painter().galley(icon_pos, icon_galley, icon_color);
                let text_pos = egui::pos2(
                    resp.rect.center().x - text_width / 2.0,
                    icon_pos.y + icon_height + line_spacing,
                );
                ui.painter().galley(text_pos, text_galley, visuals.fg_stroke.color);
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(crate::keycode::keycode_tooltip(*value, &[], &self.layer_names));
            }
        });

        ui.add_space(10.0);
        ui.label(RichText::new("Basic app / edit keys").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (label, value) in basic_app_keys {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(11.0))
                    .min_size(Vec2::new(56.0, 42.0)));
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(crate::keycode::keycode_tooltip(*value, &[], &self.layer_names));
            }
        });

        ui.add_space(10.0);
        ui.label(RichText::new("Extra function keys").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (label, value) in extra_fn_keys {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(11.0))
                    .min_size(Vec2::new(56.0, 42.0)));
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(format!("Function key {}", label));
            }
        });

        ui.add_space(10.0);
        ui.label(RichText::new("OS shortcuts").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let os_shortcuts: &[(&str, &str, u16, &str)] = &[
            ("Win/Linux", "Prev Word", 0x0100 | 0x0050, "Ctrl + Left Arrow"),
            ("Win/Linux", "Next Word", 0x0100 | 0x004F, "Ctrl + Right Arrow"),
            ("Win/Linux", "Prev App",  0x0600 | 0x002B, "Shift + Alt + Tab"),
            ("Win/Linux", "Next App",  0x0400 | 0x002B, "Alt + Tab"),
            ("macOS",     "Prev Word", 0x0400 | 0x0050, "Option + Left Arrow"),
            ("macOS",     "Next Word", 0x0400 | 0x004F, "Option + Right Arrow"),
            ("macOS",     "Prev App",  0x0A00 | 0x002B, "Shift + Command + Tab"),
            ("macOS",     "Next App",  0x0800 | 0x002B, "Command + Tab"),
        ];
        ui.horizontal_wrapped(|ui| {
            let os_text_color = if ui.visuals().dark_mode {
                Color32::from_gray(105)
            } else {
                Color32::from_gray(145)
            };
            for (os, text, value, tip) in os_shortcuts {
                let resp = ui.add_sized(Vec2::new(78.0, 44.0), egui::Button::new(""));
                let visuals = ui.style().interact(&resp);
                let painter = ui.painter();
                let os_galley = painter.layout_no_wrap(
                    (*os).to_owned(),
                    egui::FontId::proportional(9.5),
                    os_text_color,
                );
                let text_galley = painter.layout_no_wrap(
                    (*text).to_owned(),
                    egui::FontId::proportional(10.5),
                    visuals.fg_stroke.color,
                );
                let line_spacing = 1.0;
                let os_size = os_galley.size();
                let text_size = text_galley.size();
                let total_height = os_size.y + line_spacing + text_size.y;
                let os_pos = egui::pos2(
                    resp.rect.center().x - os_size.x / 2.0,
                    resp.rect.center().y - total_height / 2.0,
                );
                painter.galley(os_pos, os_galley, os_text_color);
                let text_pos = egui::pos2(
                    resp.rect.center().x - text_size.x / 2.0,
                    os_pos.y + os_size.y + line_spacing,
                );
                painter.galley(text_pos, text_galley, visuals.fg_stroke.color);
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(*tip);
            }
        });

        ui.add_space(10.0);
        ui.label(RichText::new("Numpad").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let num_text_color = if ui.visuals().dark_mode {
                Color32::from_gray(105)
            } else {
                Color32::from_gray(145)
            };
            for kc in crate::keycode::KEYCODES.iter().filter(|kc| matches!(kc.category, crate::keycode::KeycodeCategory::Numpad)) {
                let display = match kc.name {
                    "KC_NUMLOCK" => "Lock",
                    "KC_KP_SLASH" => "÷",
                    "KC_KP_ASTERISK" => "×",
                    "KC_KP_MINUS" => "−",
                    "KC_KP_PLUS" => "+",
                    "KC_KP_ENTER" => "Enter",
                    "KC_KP_1" => "1",
                    "KC_KP_2" => "2",
                    "KC_KP_3" => "3",
                    "KC_KP_4" => "4",
                    "KC_KP_5" => "5",
                    "KC_KP_6" => "6",
                    "KC_KP_7" => "7",
                    "KC_KP_8" => "8",
                    "KC_KP_9" => "9",
                    "KC_KP_0" => "0",
                    "KC_KP_DOT" => ".",
                    "KC_KP_COMMA" => ",",
                    "KC_KP_EQUAL" => "=",
                    _ => kc.label
                        .strip_prefix("Num ")
                        .or_else(|| kc.label.strip_prefix("Numpad "))
                        .or_else(|| kc.label.strip_prefix("Num"))
                        .unwrap_or(kc.label),
                };
                let font_size = if display.len() > 2 { 10.5 } else { 13.0 };
                let mut resp = ui.add_sized(Vec2::new(56.0, 42.0), egui::Button::new(""));
                let rect = resp.rect;
                let painter = ui.painter();
                let main_color = if resp.hovered() {
                    ui.visuals().widgets.hovered.fg_stroke.color
                } else {
                    ui.visuals().widgets.inactive.fg_stroke.color
                };
                painter.text(
                    egui::pos2(rect.center().x, rect.center().y - 6.0),
                    egui::Align2::CENTER_CENTER,
                    "Num",
                    egui::FontId::proportional(9.5),
                    num_text_color,
                );
                painter.text(
                    egui::pos2(rect.center().x, rect.center().y + 6.0),
                    egui::Align2::CENTER_CENTER,
                    display,
                    egui::FontId::proportional(font_size),
                    main_color,
                );
                if resp.clicked() { self.result = Some(kc.value); self.open = false; }
                resp = resp.on_hover_text(crate::keycode::keycode_tooltip(kc.value, &[], &self.layer_names));
                let _ = resp;
            }
        });

        ui.add_space(10.0);
        ui.label(RichText::new("Magic").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let magic_keys: &[u16] = &[
            0x7000, 0x7001, 0x7002, 0x7004, 0x7003,
            0x7020, 0x7021, 0x7022,
            0x7017, 0x7018, 0x7019, 0x701A, 0x701B, 0x701C, 0x701D,
            0x7005, 0x7006, 0x7007, 0x7008, 0x7014, 0x7015, 0x7016,
            0x700A, 0x7009, 0x700B,
            0x700C, 0x700D,
            0x700E, 0x700F, 0x7010,
            0x7011, 0x7012, 0x7013,
            0x701E, 0x701F,
        ];
        ui.horizontal_wrapped(|ui| {
            let magic_top_color = if ui.visuals().dark_mode {
                Color32::from_gray(105)
            } else {
                Color32::from_gray(145)
            };
            for value in magic_keys {
                let label = crate::keycode::keycode_label(*value);
                let mut parts = label.splitn(2, '\n');
                let top = parts.next().unwrap_or("");
                let bottom = parts.next().unwrap_or("");
                let mut resp = ui.add_sized(Vec2::new(76.0, 44.0), egui::Button::new(""));
                let rect = resp.rect;
                let painter = ui.painter();
                let main_color = if resp.hovered() {
                    ui.visuals().widgets.hovered.fg_stroke.color
                } else {
                    ui.visuals().widgets.inactive.fg_stroke.color
                };
                let top_font = if top.chars().count() > 10 { 8.6 } else { 9.2 };
                let bottom_font = if bottom.chars().count() > 8 { 9.4 } else { 10.2 };
                if !top.is_empty() {
                    painter.text(
                        egui::pos2(rect.center().x, rect.center().y - 6.5),
                        egui::Align2::CENTER_CENTER,
                        top,
                        egui::FontId::proportional(top_font),
                        magic_top_color,
                    );
                }
                painter.text(
                    egui::pos2(rect.center().x, rect.center().y + 6.5),
                    egui::Align2::CENTER_CENTER,
                    if bottom.is_empty() { top } else { bottom },
                    egui::FontId::proportional(bottom_font),
                    main_color,
                );
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp = resp.on_hover_text(crate::keycode::keycode_tooltip(*value, &[], &self.layer_names));
                let _ = resp;
            }
        });

        ui.add_space(10.0);
        ui.label(RichText::new("Space Cadet").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let space_cadet_keys: &[(&str, &str, u16, &str)] = &[
            ("LCtrl", "(",     0x7C18, "Left Control when held, ( when tapped"),
            ("RCtrl", ")",     0x7C19, "Right Control when held, ) when tapped"),
            ("LShift", "(",    0x7C1A, "Left Shift when held, ( when tapped"),
            ("RShift", ")",    0x7C1B, "Right Shift when held, ) when tapped"),
            ("LAlt", "(",      0x7C1C, "Left Alt when held, ( when tapped"),
            ("RAlt", ")",      0x7C1D, "Right Alt when held, ) when tapped"),
            ("RShift", "Enter",0x7C1E, "Right Shift when held, Enter when tapped"),
        ];
        ui.horizontal_wrapped(|ui| {
            let cadet_top_color = if ui.visuals().dark_mode {
                Color32::from_gray(105)
            } else {
                Color32::from_gray(145)
            };
            for (top, bottom, value, tip) in space_cadet_keys {
                let mut resp = ui.add_sized(Vec2::new(72.0, 44.0), egui::Button::new(""));
                let rect = resp.rect;
                let painter = ui.painter();
                let main_color = if resp.hovered() {
                    ui.visuals().widgets.hovered.fg_stroke.color
                } else {
                    ui.visuals().widgets.inactive.fg_stroke.color
                };
                let top_font = if top.chars().count() > 6 { 8.7 } else { 9.3 };
                let bottom_font = if bottom.chars().count() > 5 { 9.4 } else { 10.6 };
                painter.text(
                    egui::pos2(rect.center().x, rect.center().y - 6.5),
                    egui::Align2::CENTER_CENTER,
                    *top,
                    egui::FontId::proportional(top_font),
                    cadet_top_color,
                );
                painter.text(
                    egui::pos2(rect.center().x, rect.center().y + 6.5),
                    egui::Align2::CENTER_CENTER,
                    *bottom,
                    egui::FontId::proportional(bottom_font),
                    main_color,
                );
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp = resp.on_hover_text(*tip);
                let _ = resp;
            }
        });

        ui.add_space(10.0);
        ui.label(RichText::new("International").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let international_keys: &[(&str, &str, u16, &str)] = &[
            ("JIS", "\\ _",   0x0087, "JIS \\ and _"),
            ("JIS", "Kana",   0x0088, "JIS Katakana/Hiragana"),
            ("JIS", "¥ |",    0x0089, "JIS ¥ and |"),
            ("JIS", "Henkan", 0x008A, "JIS Henkan"),
            ("JIS", "Muhenk", 0x008B, "JIS Muhenkan"),
            ("JIS", "Num ,",  0x008C, "JIS Numpad ,"),
            ("Hangul", "Eng",   0x0090, "Hangul/English"),
            ("Hangul", "Hanja", 0x0091, "Hanja"),
            ("JIS", "Katak",  0x0092, "JIS Katakana"),
            ("JIS", "Hirag",  0x0093, "JIS Hiragana"),
            ("JIS", "ZenHan", 0x0094, "JIS Zenkaku/Hankaku"),
        ];
        ui.horizontal_wrapped(|ui| {
            let intl_top_color = if ui.visuals().dark_mode {
                Color32::from_gray(105)
            } else {
                Color32::from_gray(145)
            };
            for (top, bottom, value, tip) in international_keys {
                let mut resp = ui.add_sized(Vec2::new(72.0, 44.0), egui::Button::new(""));
                let rect = resp.rect;
                let painter = ui.painter();
                let main_color = if resp.hovered() {
                    ui.visuals().widgets.hovered.fg_stroke.color
                } else {
                    ui.visuals().widgets.inactive.fg_stroke.color
                };
                let top_font = if top.chars().count() > 6 { 8.5 } else { 9.2 };
                let bottom_font = if bottom.chars().count() > 6 { 9.0 } else { 10.2 };
                painter.text(
                    egui::pos2(rect.center().x, rect.center().y - 6.5),
                    egui::Align2::CENTER_CENTER,
                    *top,
                    egui::FontId::proportional(top_font),
                    intl_top_color,
                );
                painter.text(
                    egui::pos2(rect.center().x, rect.center().y + 6.5),
                    egui::Align2::CENTER_CENTER,
                    *bottom,
                    egui::FontId::proportional(bottom_font),
                    main_color,
                );
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp = resp.on_hover_text(*tip);
                let _ = resp;
            }
        });
    }

fn show_zmk(&mut self, ctx: &egui::Context) {
        // Physical key capture for ZMK
        let captured = ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    if *key == Key::Escape {
                        return Some(None); // close without assigning
                    }
                    if modifiers.is_none() {
                        if let Some(usage) = egui_key_to_zmk_usage(*key) {
                            return Some(Some(usage));
                        }
                    }
                }
            }
            None
        });
        if let Some(opt) = captured {
            if let Some(usage) = opt {
                if let Some(beh) = self.zmk_find_behavior("Key Press") {
                    let id = beh.id;
                    self.zmk_assign(id, usage, 0);
                }
            } else {
                self.open = false;
            }
            return;
        }

        let mut still_open = true;
        egui::Window::new("Binding")
            .open(&mut still_open)
            .collapsible(false)
            .resizable(false)
            .min_size(Vec2::new(KEY_PICKER_POPUP_WIDTH, KEY_PICKER_POPUP_HEIGHT))
            .default_size([KEY_PICKER_POPUP_WIDTH, KEY_PICKER_POPUP_HEIGHT])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                apply_picker_button_visuals(ui);
                ui.label(RichText::new("Press a key on your keyboard, or pick below")
                    .size(11.0).color(Color32::from_gray(140)));
                ui.add_space(4.0);

                // Tab bar
                ui.horizontal_wrapped(|ui| {
                    for tab in KeycodeTab::ZMK_TABS {
                        let active = self.selected_tab == *tab;
                        let text = if active {
                            RichText::new(tab.label())
                                .size(12.0)
                                .color(ui.visuals().widgets.hovered.fg_stroke.color)
                        } else {
                            RichText::new(tab.label()).size(12.0)
                        };
                        let btn = egui::Button::new(text)
                            .fill(if active { Color32::from_rgb(184, 92, 112) } else { Color32::TRANSPARENT });
                        if ui.add(btn).clicked() { self.selected_tab = *tab; }
                    }
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.selected_tab {
                        KeycodeTab::Layers      => self.show_zmk_layers(ui),
                        KeycodeTab::Modifiers   => self.show_zmk_modifiers(ui),
                        KeycodeTab::Special     => self.show_zmk_special(ui),
                        KeycodeTab::ZmkAdvanced => self.show_zmk_advanced(ui),
                        _                       => self.show_zmk_keys(ui),
                    }
                });
            });
        if !still_open { self.open = false; }
    }

    /// Generic ZMK keys tab — shows Key Press behavior with HID usages from KEYCODES table
    fn show_zmk_keys(&mut self, ui: &mut egui::Ui) {
        let beh_id = match self.zmk_find_behavior("Key Press") {
            Some(b) => b.id,
            None => {
                ui.label("Key Press behavior not found on device");
                return;
            }
        };
        let key_choices: Vec<&'static crate::keycode::Keycode> = KEYCODES.iter()
            .filter(|kc| self.selected_tab.vial_matches(kc))
            .filter(|kc| kc.value < 0x0200)
            .collect();
        if let Some(value) = show_grouped_popup_key_buttons(ui, key_choices, &self.layer_names, true) {
            let id = beh_id;
            let zmk_usage = 0x0007_0000u32 | value as u32;
            self.zmk_assign(id, zmk_usage, 0);
        }
    }

    fn show_zmk_layers(&mut self, ui: &mut egui::Ui) {
        let ops: &[(&str, &str, &str)] = &[
            ("Momentary Layer", "Hold\n(MO)",      "Hold to activate, release to return"),
            ("Toggle Layer",    "Toggle\n(TG)",    "Tap to toggle on/off"),
            ("To Layer",        "Switch\n(TO)",    "Switch and stay on this layer"),
            ("Sticky Layer",    "One-Shot\n(SL)",  "Active for next keypress only"),
            ("Layer-Tap",       "Tap\n(LT)",       "Hold=layer, tap=key"),
        ];
        let op_ids: Vec<(&str, &str, &str, Option<u32>)> = ops.iter()
            .map(|(name, header, hint)| {
                let id = self.zmk_find_behavior(name).map(|b| b.id);
                (*name, *header, *hint, id)
            })
            .collect();

        let layer_count = self.zmk_layer_count.max(1) as u16;
        let col_w = 80.0_f32;
        let row_h = 32.0_f32;
        let dark = ui.visuals().dark_mode;

        egui::Grid::new("zmk_layers_grid").spacing([4.0, 2.0]).show(ui, |ui| {
            ui.label("");
            for (_, header, hint, _) in &op_ids {
                ui.add(egui::Label::new(RichText::new(*header).size(10.0).strong())
                    .sense(egui::Sense::hover())).on_hover_text(*hint);
            }
            ui.end_row();

            for n in 0..layer_count {
                let raw = self.layer_names.get(n as usize).cloned().unwrap_or(n.to_string());
                let is_named = !raw.is_empty() && raw != n.to_string();
                let row_bg = if n % 2 == 0 {
                    if dark { Color32::from_rgba_premultiplied(255,255,255,6) }
                    else    { Color32::from_rgba_premultiplied(0,0,0,8) }
                } else { Color32::TRANSPARENT };
                let label_color = if is_named {
                    Color32::from_rgb(184, 92, 112)
                } else if dark { Color32::from_gray(110) } else { Color32::from_gray(160) };
                ui.label(RichText::new(if is_named { raw } else { format!("Layer {}", n) })
                    .size(11.5).color(label_color).strong());

                for (_, header, _, id) in &op_ids {
                    let op_short = header.split('\n').last().unwrap_or("")
                        .trim_matches(|c| c == '(' || c == ')');
                    let btn_text = format!("{}({})", op_short, n);
                    let enabled = id.is_some();
                    let btn_color = if enabled {
                        if is_named {
                            if dark { Color32::from_gray(220) } else { Color32::from_gray(30) }
                        } else {
                            if dark { Color32::from_gray(100) } else { Color32::from_gray(160) }
                        }
                    } else { Color32::from_gray(80) };
                    let fill = if is_named && enabled {
                        if dark { Color32::from_rgb(38, 43, 88) } else { Color32::from_rgb(224, 227, 249) }
                    } else { row_bg };
                    let resp = ui.add(
                        egui::Button::new(RichText::new(&btn_text).size(10.5).color(btn_color))
                            .fill(fill).min_size(Vec2::new(col_w, row_h))
                    );
                    if resp.clicked() {
                        if let Some(beh_id) = id {
                            self.zmk_assign(*beh_id, n as u32, 0);
                        }
                    }
                }
                ui.end_row();
            }
        });
    }

    fn show_zmk_modifiers(&mut self, ui: &mut egui::Ui) {
        let lgui = gui_label(false);
        let rgui = gui_label(true);

        let key_press_id = self.zmk_find_behavior("Key Press").map(|b| b.id);
        let sticky_id = self.zmk_find_behavior("Sticky Key").map(|b| b.id);

        // Modifier HID usages
        let mods: &[(&str, u32, &str)] = &[
            ("Ctrl",          0x000700E0, "Left Control"),
            ("Shift",         0x000700E1, "Left Shift"),
            ("Alt",           0x000700E2, "Left Alt"),
            ("Ctrl",          0x000700E4, "Right Control"),
            ("Shift",         0x000700E5, "Right Shift"),
            ("Alt",           0x000700E6, "Right Alt / AltGr"),
        ];

        ui.label(RichText::new("Hold modifiers (Key Press)").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let lgui_usage = 0x000700E3u32;
        let rgui_usage = 0x000700E7u32;
        ui.horizontal_wrapped(|ui| {
            for (label, usage, tip) in mods {
                if let Some(id) = key_press_id {
                    let resp = ui.add(egui::Button::new(RichText::new(*label).size(11.0))
                        .min_size(Vec2::new(80.0, 38.0)));
                    if resp.clicked() { self.zmk_assign(id, *usage, 0); }
                    resp.on_hover_text(*tip);
                }
            }
            if let Some(id) = key_press_id {
                let resp = ui.add(egui::Button::new(RichText::new(lgui).size(11.0))
                    .min_size(Vec2::new(80.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, lgui_usage, 0); }
                resp.on_hover_text(format!("Left {}", lgui));
                let resp = ui.add(egui::Button::new(RichText::new(rgui).size(11.0))
                    .min_size(Vec2::new(80.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, rgui_usage, 0); }
                resp.on_hover_text(format!("Right {}", rgui));
            }
        });

        ui.separator();
        ui.label(RichText::new("One-Shot / Sticky Key — active for next keypress only").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let sk_mods: &[(&str, u32, &str)] = &[
                ("SK\nLCtrl",   0x000700E0, "One-Shot Left Ctrl"),
                ("SK\nLShift",  0x000700E1, "One-Shot Left Shift — capitalise next letter"),
                ("SK\nLAlt",    0x000700E2, "One-Shot Left Alt"),
                ("SK\nRCtrl",   0x000700E4, "One-Shot Right Ctrl"),
                ("SK\nRShift",  0x000700E5, "One-Shot Right Shift"),
                ("SK\nRAlt",    0x000700E6, "One-Shot Right Alt / AltGr"),
            ];
            for (label, usage, tip) in sk_mods {
                if let Some(id) = sticky_id {
                    let parts: Vec<&str> = label.splitn(2, '\n').collect();
                    let btn_label = if parts.len() == 2 { format!("{} {}", parts[0], parts[1]) } else { label.to_string() };
                    let resp = ui.add(egui::Button::new(RichText::new(&btn_label).size(10.5))
                        .min_size(Vec2::new(80.0, 38.0)));
                    if resp.clicked() { self.zmk_assign(id, *usage, 0); }
                    resp.on_hover_text(*tip);
                }
            }
            if let Some(id) = sticky_id {
                let resp = ui.add(egui::Button::new(RichText::new(format!("SK {}", lgui)).size(10.5))
                    .min_size(Vec2::new(80.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, lgui_usage, 0); }
                resp.on_hover_text(format!("One-Shot Left {}", lgui));
                let resp = ui.add(egui::Button::new(RichText::new(format!("SK {}", rgui)).size(10.5))
                    .min_size(Vec2::new(80.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, rgui_usage, 0); }
                resp.on_hover_text(format!("One-Shot Right {}", rgui));
            }
        });
    }

    fn show_zmk_special(&mut self, ui: &mut egui::Ui) {
        // TRNS / None
        let transparent_id = self.zmk_find_behavior("Transparent").map(|b| b.id);
        let none_id = self.zmk_find_behavior("None").map(|b| b.id);
        let boot_id = self.zmk_find_behavior("Bootloader").map(|b| b.id);
        let reset_id = self.zmk_find_behavior("Reset").map(|b| b.id);
        let caps_word_id = self.zmk_find_behavior("Caps Word").map(|b| b.id);
        let gesc_id = self.zmk_find_behavior("Grave/Escape").map(|b| b.id);
        let unlock_id = self.zmk_find_behavior("Studio Unlock").map(|b| b.id);
        let bt_id = self.zmk_find_behavior("Bluetooth").map(|b| b.id);
        let out_id = self.zmk_find_behavior("Output Selection").map(|b| b.id);

        ui.label(RichText::new("Basic").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if let Some(id) = transparent_id {
                let resp = ui.add(egui::Button::new(RichText::new("▽ Inherit").size(11.0))
                    .min_size(Vec2::new(64.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, 0, 0); }
                resp.on_hover_text("Inherit the key from the layer below");
            }
            if let Some(id) = none_id {
                let resp = ui.add(egui::Button::new(RichText::new("✕ None").size(11.0))
                    .min_size(Vec2::new(64.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, 0, 0); }
                resp.on_hover_text("No operation — key does nothing");
            }
            if let Some(id) = caps_word_id {
                let resp = ui.add(egui::Button::new(RichText::new("CapsWrd").size(11.0))
                    .min_size(Vec2::new(64.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, 0, 0); }
                resp.on_hover_text("Caps Word — capitalise next word, then auto-disable");
            }
            if let Some(id) = gesc_id {
                let resp = ui.add(egui::Button::new(RichText::new("~\nEsc").size(10.5))
                    .min_size(Vec2::new(56.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, 0, 0); }
                resp.on_hover_text("Grave/Escape — ` when Shift held, Esc otherwise");
            }
            if let Some(id) = unlock_id {
                let resp = ui.add(egui::Button::new(RichText::new("Unlock\nStudio").size(10.0))
                    .min_size(Vec2::new(64.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, 0, 0); }
                resp.on_hover_text("Studio Unlock — tap to allow ZMK Studio to edit the keymap");
            }
        });

        ui.separator();
        ui.label(RichText::new("Firmware").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if let Some(id) = boot_id {
                let resp = ui.add(egui::Button::new(RichText::new("⚡\nBoot").size(10.5))
                    .min_size(Vec2::new(56.0, 42.0)));
                if resp.clicked() { self.zmk_assign(id, 0, 0); }
                resp.on_hover_text("Bootloader — put keyboard into flash mode");
            }
            if let Some(id) = reset_id {
                let resp = ui.add(egui::Button::new(RichText::new("Reset").size(11.0))
                    .min_size(Vec2::new(56.0, 42.0)));
                if resp.clicked() { self.zmk_assign(id, 0, 0); }
                resp.on_hover_text("Reset firmware");
            }
        });

        if let Some(id) = bt_id {
            ui.separator();
            ui.label(RichText::new("Bluetooth").size(11.0).color(Color32::from_gray(150)));
            ui.add_space(4.0);
            let bt_actions: &[(&str, u32, u32, &str)] = &[
                ("BT\nCLR",   0, 0, "Clear current BT profile pairing"),
                ("BT\nCLR ALL", 1, 0, "Clear ALL BT profiles"),
                ("BT\nNext",  2, 0, "Switch to next BT profile"),
                ("BT\nPrev",  3, 0, "Switch to previous BT profile"),
                ("BT\nSEL 0", 4, 0, "Select BT profile 0"),
                ("BT\nSEL 1", 4, 1, "Select BT profile 1"),
                ("BT\nSEL 2", 4, 2, "Select BT profile 2"),
                ("BT\nSEL 3", 4, 3, "Select BT profile 3"),
                ("BT\nSEL 4", 4, 4, "Select BT profile 4"),
            ];
            ui.horizontal_wrapped(|ui| {
                for (label, p1, p2, tip) in bt_actions {
                    let parts: Vec<&str> = label.splitn(2, '\n').collect();
                    let btn_label = if parts.len() == 2 { format!("{} {}", parts[0], parts[1]) } else { label.to_string() };
                    let resp = ui.add(egui::Button::new(RichText::new(&btn_label).size(10.5))
                        .min_size(Vec2::new(64.0, 38.0)));
                    if resp.clicked() { self.zmk_assign(id, *p1, *p2); }
                    resp.on_hover_text(*tip);
                }
            });
        }

        if let Some(id) = out_id {
            ui.separator();
            ui.label(RichText::new("Output").size(11.0).color(Color32::from_gray(150)));
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                let resp = ui.add(egui::Button::new(RichText::new("Out\nUSB").size(10.5))
                    .min_size(Vec2::new(60.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, 0, 0); }
                resp.on_hover_text("Output: USB");
                let resp = ui.add(egui::Button::new(RichText::new("Out\nBLE").size(10.5))
                    .min_size(Vec2::new(60.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, 1, 0); }
                resp.on_hover_text("Output: Bluetooth");
            });
        }
    }

    fn show_zmk_advanced(&mut self, ui: &mut egui::Ui) {
        // All behaviors not shown in other tabs
        let covered: &[&str] = &[
            "Key Press", "Sticky Key", "Momentary Layer", "Toggle Layer",
            "To Layer", "Sticky Layer", "Layer-Tap",
            "Transparent", "None", "Bootloader", "Reset",
            "Caps Word", "Grave/Escape", "Studio Unlock",
            "Bluetooth", "Output Selection",
        ];

        let behaviors: Vec<(u32, String)> = self.zmk_behaviors.iter()
            .filter(|b| !covered.contains(&b.display_name.as_str()))
            .map(|b| (b.id, b.display_name.clone()))
            .collect();

        if behaviors.is_empty() {
            ui.label(RichText::new("No additional behaviors found on this device")
                .size(11.0).color(Color32::from_gray(150)));
            return;
        }

        ui.label(RichText::new("Other behaviors available on this device:").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (id, name) in &behaviors {
                let resp = ui.add(egui::Button::new(RichText::new(name).size(11.0))
                    .min_size(Vec2::new(72.0, 38.0)));
                if resp.clicked() {
                    self.zmk_result = Some(ZmkBinding { behavior_id: *id as i32, param1: 0, param2: 0 });
                    self.open = false;
                }
                resp.on_hover_text(format!("Behavior: {} (id={})", name, id));
            }
        });
    }
}
