/// Keycode picker modal — supports both Vial (QMK keycodes) and ZMK (behaviors).

use crate::firmware::FirmwareProtocol;
use crate::keycode::{gui_label, gui_sym, keycode_tooltip, KeycodeCategory, KEYCODES};
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
    pub custom_keycodes: Vec<(String, String, u16)>,
    pub layer_names: Vec<String>,
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
    pub tap_dance_editor_open: Option<u8>,
    pub tap_dance_dirty: bool,
    /// Which field is being edited: (td_idx, field: 0=tap,1=hold,2=dtap,3=taphold)
    pub td_key_pick: Option<(usize, u8)>,
    pub macro_editor_open: Option<u8>,
    /// Macro editor text buffers (one per macro)
    pub macro_texts: Vec<String>,
    /// Macro actions for editor UI
    pub macro_actions: Vec<Vec<MacroAction>>,
    /// Flag: macro texts changed, need to write to device
    pub macros_dirty: bool,
    /// Pending action type for "add action" dropdown
    macro_add_action_open: bool,
    /// Undo stack for macro editor: (macro_idx, previous_actions)
    macro_undo_stack: Vec<(usize, Vec<MacroAction>)>,
    /// Macro key picker: (macro_idx, action_idx) being edited
    macro_key_pick: Option<(usize, usize)>,
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
        KeycodeTab::Function,
        KeycodeTab::Navigation,
        KeycodeTab::Modifiers,
        KeycodeTab::Layers,
        KeycodeTab::Media,
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
            KeycodeTab::Media       => "Media & Mouse",
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
            KeycodeTab::Function   => matches!(kc.category, KeycodeCategory::Function),
            KeycodeTab::Navigation => matches!(kc.category, KeycodeCategory::Navigation),
            KeycodeTab::Modifiers  => matches!(kc.category, KeycodeCategory::Modifier),
            KeycodeTab::Layers     => matches!(kc.category, KeycodeCategory::Layer),
            KeycodeTab::Media      => matches!(kc.category, KeycodeCategory::Media | KeycodeCategory::Mouse),
            KeycodeTab::Mouse      => matches!(kc.category, KeycodeCategory::Mouse),
            KeycodeTab::Numpad     => matches!(kc.category, KeycodeCategory::Numpad),
            KeycodeTab::Special    => matches!(kc.category, KeycodeCategory::Special),
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
            layer_names: (0..16).map(|i| i.to_string()).collect(),
            listening: false,
            firmware: FirmwareProtocol::Vial,
            zmk_behaviors: vec![],
            zmk_result: None,
            zmk_selected_behavior: None,
            zmk_layer_count: 4,
            vial_quantum_pending_mod: None,
            vial_quantum_pending_mt: None,
            vial_layer_pending: None,
            macro_editor_open: None,
            macro_count: 16,
            tap_dance_entries: vec![],
            tap_dance_editor_open: None,
            tap_dance_dirty: false,
            td_key_pick: None,
            macro_texts: vec![String::new(); 16],
            macro_actions: vec![vec![]; 16],
            macro_add_action_open: false,
            macro_undo_stack: Vec::new(),
            macro_key_pick: None,
            macros_dirty: false,
        }
    }
}

fn egui_key_to_qmk(key: Key, mods: egui::Modifiers) -> Option<u16> {
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

impl KeycodePicker {
    fn zmk_find_behavior<'a>(&'a self, name: &str) -> Option<&'a BehaviorInfo> {
        self.zmk_behaviors.iter().find(|b| b.display_name == name)
    }

    fn zmk_assign(&mut self, behavior_id: u32, param1: u32, param2: u32) {
        self.zmk_result = Some(ZmkBinding { behavior_id: behavior_id as i32, param1, param2 });
        self.open = false;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open { return; }

        // If pending mod/MT — show only the minimal second picker, not the full picker
        let has_pending = self.vial_quantum_pending_mod.is_some() || self.vial_quantum_pending_mt.is_some() || self.vial_layer_pending.is_some() || self.macro_editor_open == Some(255);
        if has_pending && self.firmware == FirmwareProtocol::Vial {
            self.show_pending_picker(ctx);
            return;
        }

        // Macro key picker (sub-window of macro editor)
        if let Some((macro_idx, action_idx)) = self.macro_key_pick {
            let mut pick_open = true;
            egui::Window::new("Pick key")
                .open(&mut pick_open)
                .collapsible(false)
                .resizable(false)
                .min_size(Vec2::new(400.0, 200.0))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Press a key on your keyboard, or click below")
                        .size(11.0).color(Color32::from_gray(140)));
                    ui.add_space(4.0);
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
                    ui.horizontal_wrapped(|ui| {
                        for kc in KEYCODES.iter() {
                            if !matches!(kc.category, KeycodeCategory::Basic | KeycodeCategory::Function | KeycodeCategory::Navigation) { continue; }
                            if kc.value == 0 || kc.value >= 0x0100 { continue; }
                            let resp = ui.add(egui::Button::new(RichText::new(kc.label).size(10.0))
                                .min_size(Vec2::new(36.0, 28.0)));
                            if resp.clicked() {
                                if let Some(action) = self.macro_actions.get_mut(macro_idx).and_then(|a| a.get_mut(action_idx)) {
                                    match action {
                                        MacroAction::Tap(k) => *k = kc.value as u8,
                                        MacroAction::Down(k) => *k = kc.value as u8,
                                        MacroAction::Up(k) => *k = kc.value as u8,
                                        _ => {}
                                    }
                                }
                                self.macro_key_pick = None;
                            }
                            resp.on_hover_text(keycode_tooltip(kc.value, &[], &self.layer_names));
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

        // Tap dance editor
        if let Some(td_n) = self.tap_dance_editor_open {
            if (td_n as usize) < self.tap_dance_entries.len() {
                self.show_tap_dance_editor(ctx, td_n);
                return;
            }
        }

        // Macro editor window
        if let Some(macro_n) = self.macro_editor_open {
            if (macro_n as usize) < self.macro_count || macro_n == 254 {
                self.show_macro_editor(ctx, if macro_n == 254 { 254 } else { macro_n });
                return;
            }
        }

        match self.firmware {
            FirmwareProtocol::Vial => self.show_vial(ctx),
            FirmwareProtocol::Zmk  => self.show_zmk(ctx),
        }
    }

    // ─────────────────────────── VIAL PICKER ────────────────────────────────

    fn show_vial(&mut self, ctx: &egui::Context) {
        // Physical key capture (disabled when macro editor is open)
        if self.macro_editor_open.is_none() || self.macro_editor_open == Some(255) {
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    if *key == Key::Escape {
                        if self.vial_quantum_pending_mod.is_some() || self.vial_quantum_pending_mt.is_some() {
                            self.vial_quantum_pending_mod = None;
                            self.vial_quantum_pending_mt = None;
                        } else {
                            self.open = false;
                        }
                        return;
                    }
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
        } // end physical key capture guard

        let mut still_open = true;
        egui::Window::new("Keycode")
            .open(&mut still_open)
            .collapsible(false)
            .resizable(true)
            .min_size(Vec2::new(640.0, 420.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(RichText::new("Press a key on your keyboard, or pick below")
                    .size(11.0).color(Color32::from_gray(140)));
                ui.add_space(4.0);

                // Tab bar
                ui.horizontal_wrapped(|ui| {
                    for tab in KeycodeTab::VIAL_TABS {
                        if *tab == KeycodeTab::Custom && self.custom_keycodes.is_empty() { continue; }
                        let active = self.selected_tab == *tab;
                        let btn = egui::Button::new(RichText::new(tab.label()).size(12.0))
                            .fill(if active { Color32::from_rgb(91, 104, 223) } else { Color32::TRANSPARENT });
                        if ui.add(btn).clicked() {
                            self.selected_tab = *tab;
                            self.vial_quantum_pending_mod = None;
                            self.vial_quantum_pending_mt = None;
                            self.vial_layer_pending = None;
                        }
                    }
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    match self.selected_tab {
                        KeycodeTab::Layers   => self.show_vial_layers(ui),
                        KeycodeTab::Modifiers => self.show_vial_modifiers(ui),
                        KeycodeTab::Quantum  => self.show_vial_quantum(ui),
                        KeycodeTab::Rgb      => self.show_vial_rgb(ui),
                        KeycodeTab::Macro    => self.show_vial_macros(ui),
                        KeycodeTab::TapDance => self.show_vial_tap_dance(ui),
                        KeycodeTab::Special  => self.show_vial_special(ui),
                        KeycodeTab::Custom   => self.show_vial_custom(ui),
                        _ => self.show_vial_generic(ui),
                    }
                });
            });

        if !still_open { self.open = false; }
    }

    fn show_pending_picker(&mut self, ctx: &egui::Context) {
        // Macro number picker
        if self.macro_editor_open == Some(255) {
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key { key, pressed: true, .. } = event {
                        if *key == egui::Key::Escape {
                            self.macro_editor_open = None;
                            self.open = false;
                            return;
                        }
                    }
                }
            });
            let mut still_open = true;
            let resp_win = egui::Window::new("Pick macro")
                .open(&mut still_open)
                .collapsible(false)
                .resizable(false)
                .min_size(Vec2::new(350.0, 100.0))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Choose macro number. Esc to cancel.")
                        .size(11.0).color(Color32::from_gray(140)));
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        for n in 0..self.macro_count as u8 {
                            let has_content = self.macro_texts.get(n as usize).map(|s| !s.is_empty()).unwrap_or(false);
                            let label = format!("M{}", n);
                            let resp = ui.add(egui::Button::new(
                                RichText::new(&label).size(11.0)
                                    .color(if has_content { Color32::from_gray(220) } else { Color32::from_gray(100) })
                            ).min_size(Vec2::new(40.0, 32.0)));
                            if resp.clicked() {
                                // Don't set result yet — editor will set it on close
                                self.macro_editor_open = Some(n);
                            }
                            let tip = if has_content {
                                format!("Macro {} — has content", n)
                            } else {
                                format!("Macro {} — empty", n)
                            };
                            resp.on_hover_text(tip);
                        }
                    });
                });
            // If a macro was selected, don't close
            if self.macro_editor_open != Some(255) {
                return; // editor will take over next frame
            }
            let clicked_outside = ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary))
                && resp_win.as_ref().map(|r| !r.response.hovered()).unwrap_or(false);
            if !still_open || clicked_outside {
                self.macro_editor_open = None;
                self.open = false;
            }
            return;
        }

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
            let resp_win = egui::Window::new("Pick layer")
                .open(&mut still_open)
                .collapsible(false)
                .resizable(false)
                .min_size(Vec2::new(300.0, 100.0))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Choose which layer. Esc to cancel.")
                        .size(11.0).color(Color32::from_gray(140)));
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        for n in 0u16..self.layer_names.len().max(4) as u16 {
                            let raw = self.layer_names.get(n as usize).cloned().unwrap_or(n.to_string());
                            let label = if !raw.is_empty() && raw != n.to_string() {
                                format!("{}: {}", n, raw)
                            } else {
                                format!("Layer {}", n)
                            };
                            let resp = ui.add(egui::Button::new(RichText::new(&label).size(11.0))
                                .min_size(Vec2::new(70.0, 32.0)));
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
            let clicked_outside = ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary))
                && resp_win.as_ref().map(|r| !r.response.hovered()).unwrap_or(false);
            if !still_open || clicked_outside {
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
            let resp_win = egui::Window::new(title)
                .open(&mut still_open)
                .collapsible(false)
                .resizable(false)
                .min_size(Vec2::new(480.0, 200.0))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Press a key on your keyboard, or click below. Esc to cancel.")
                        .size(11.0).color(Color32::from_gray(140)));
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        for kc in KEYCODES.iter() {
                            if !matches!(kc.category, KeycodeCategory::Basic | KeycodeCategory::Function | KeycodeCategory::Navigation) { continue; }
                            if kc.value == 0 || kc.value >= 0x0100 { continue; }
                            let resp = ui.add(egui::Button::new(RichText::new(kc.label).size(11.0))
                                .min_size(Vec2::new(40.0, 32.0)));
                            if resp.clicked() {
                                self.result = Some(base | kc.value);
                                self.vial_quantum_pending_mod = None;
                                self.vial_quantum_pending_mt = None;
                                self.open = false;
                            }
                            resp.on_hover_text(keycode_tooltip(kc.value, &[], &self.layer_names));
                        }
                    });
                });
            // Only check clicked_outside with primary button (not secondary which opened this)
            let clicked_outside = ctx.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary))
                && resp_win.as_ref().map(|r| !r.response.hovered()).unwrap_or(false);
            if !still_open || clicked_outside {
                self.vial_quantum_pending_mod = None;
                self.vial_quantum_pending_mt = None;
                self.open = false;
            }
        }
    }

    fn show_vial_generic(&mut self, ui: &mut egui::Ui) {
        let custom_pairs: Vec<(String, String)> = self.custom_keycodes.iter()
            .map(|(n, l, _)| (n.clone(), l.clone()))
            .collect();
        ui.horizontal_wrapped(|ui| {
            for kc in KEYCODES.iter() {
                if !self.selected_tab.vial_matches(kc) { continue; }
                let tip = keycode_tooltip(kc.value, &custom_pairs, &self.layer_names);
                let resp = ui.add(
                    egui::Button::new(RichText::new(kc.label).size(11.0))
                        .min_size(Vec2::new(52.0, 38.0)),
                );
                if resp.clicked() { self.result = Some(kc.value); self.open = false; }
                resp.on_hover_text(tip);
            }
        });
    }

    fn show_vial_custom(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for (name, label, value) in &self.custom_keycodes {
                if label.is_empty() { continue; }
                let tip = format!("Custom: {} ({})", label, name);
                let resp = ui.add(
                    egui::Button::new(RichText::new(label).size(11.0))
                        .min_size(Vec2::new(52.0, 38.0)),
                );
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(tip);
            }
        });
    }

    fn show_vial_layers(&mut self, ui: &mut egui::Ui) {
        let ops: &[(u16, &str, &str)] = &[
            (0x5220, "MO — Momentary",   "Hold to activate, release to return"),
            (0x5260, "TG — Toggle",      "Tap to toggle on/off"),
            (0x5280, "OSL — One-Shot",   "Active for next keypress only"),
            (0x52C0, "TT — Tap-Toggle",  "Hold = MO, tap = toggle"),
            (0x5200, "TO — Switch",      "Switch and stay on this layer"),
            (0x5240, "DF — Default",     "Set as permanent base layer"),
        ];

        ui.label(RichText::new("Pick layer action, then choose which layer").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for (base, label, hint) in ops {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(11.0))
                    .min_size(Vec2::new(120.0, 38.0)));
                if resp.clicked() {
                    self.vial_layer_pending = Some(*base);
                }
                resp.on_hover_text(*hint);
            }
        });

        // LT (Layer-Tap) — separate since base is 0x4000 + layer<<8
        ui.add_space(6.0);
        let lt_resp = ui.add(egui::Button::new(RichText::new("LT — Layer-Tap").size(11.0))
            .min_size(Vec2::new(120.0, 38.0)));
        if lt_resp.clicked() {
            self.vial_layer_pending = Some(0x4000);
        }
        lt_resp.on_hover_text("Hold = activate layer, tap = keycode (set key via right-click afterwards)");
    }

    fn show_vial_modifiers(&mut self, ui: &mut egui::Ui) {
        let gui = gui_sym();
        let lgui = gui_label(false);
        let rgui = gui_label(true);

        ui.label(RichText::new("Plain modifiers").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let plain: Vec<(String, u16, String)> = vec![
            ("⌃ LCtrl".into(),       0x00E0, "Left Control".into()),
            ("⇧ LShift".into(),      0x00E1, "Left Shift".into()),
            ("⌥ LAlt".into(),        0x00E2, "Left Alt".into()),
            (format!("  L{}", lgui), 0x00E3, format!("Left {}", lgui)),
            ("⌃ RCtrl".into(),       0x00E4, "Right Control".into()),
            ("⇧ RShift".into(),      0x00E5, "Right Shift".into()),
            ("⌥ RAlt".into(),        0x00E6, "Right Alt / AltGr".into()),
            (format!("  R{}", rgui), 0x00E7, format!("Right {}", rgui)),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in &plain {
                let resp = ui.add(egui::Button::new(RichText::new(label.as_str()).size(11.0))
                    .min_size(Vec2::new(80.0, 38.0)));
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(tip.as_str());
            }
        });

        ui.separator();
        ui.label(RichText::new("Mod-Tap — hold for modifier, tap for regular key").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(2.0);
        ui.label(RichText::new("Pick modifier below; tap key is set separately.").size(10.0).color(Color32::from_gray(120)));
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
            ("MT RCtrl".into(),        0x3100, "Mod-Tap: hold=RCtrl".into()),
            ("MT RShift".into(),       0x3200, "Mod-Tap: hold=RShift".into()),
            ("MT RAlt".into(),         0x3400, "Mod-Tap: hold=RAlt".into()),
            (format!("MT R{}", rgui),  0x3800, format!("Mod-Tap: hold=R{}", rgui)),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in &mt {
                let resp = ui.add(egui::Button::new(RichText::new(label.as_str()).size(10.5))
                    .min_size(Vec2::new(80.0, 38.0)));
                if resp.clicked() {
                    self.vial_quantum_pending_mt = Some(*value);
                }
                resp.on_hover_text(tip.as_str());
            }
        });

        ui.separator();
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
                    .min_size(Vec2::new(90.0, 38.0)));
                if resp.clicked() {
                    self.vial_quantum_pending_mod = Some(*value);
                }
                resp.on_hover_text(tip.as_str());
            }
        });

        ui.separator();
        ui.label(RichText::new("One-Shot Mod — active for next keypress only").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let osm: Vec<(String, u16, String)> = vec![
            ("OSM LCtrl".into(),        0x52A2, "One-Shot Left Ctrl".into()),
            ("OSM LShift".into(),       0x52A1, "One-Shot Left Shift — tap to capitalise next letter".into()),
            ("OSM LAlt".into(),         0x52A4, "One-Shot Left Alt".into()),
            (format!("OSM L{}", lgui),  0x52A8, format!("One-Shot Left {}", lgui)),
            ("OSM RCtrl".into(),        0x52B2, "One-Shot Right Ctrl".into()),
            ("OSM RShift".into(),       0x52B1, "One-Shot Right Shift".into()),
            ("OSM RAlt".into(),         0x52B4, "One-Shot Right Alt / AltGr".into()),
            (format!("OSM R{}", rgui),  0x52B8, format!("One-Shot Right {}", rgui)),
            ("OSM Meh".into(),          0x52A7, "One-Shot Meh (Ctrl+Shift+Alt)".into()),
            ("OSM Hyper".into(),        0x52AF, "One-Shot Hyper (Ctrl+Shift+Alt+Win)".into()),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in &osm {
                let resp = ui.add(egui::Button::new(RichText::new(label.as_str()).size(10.5))
                    .min_size(Vec2::new(80.0, 38.0)));
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(tip.as_str());
            }
        });
    }

    fn show_vial_quantum(&mut self, ui: &mut egui::Ui) {
        // Pending mod+key selection
        if let Some(base) = self.vial_quantum_pending_mod {
            ui.label(RichText::new("Now pick the KEY to add the modifier to:").size(11.5).strong());
            ui.label(RichText::new("Click any key below to create the combo, or Escape to cancel.").size(10.5).color(Color32::from_gray(150)));
            ui.add_space(4.0);
            if ui.button("✕ Cancel").clicked() { self.vial_quantum_pending_mod = None; }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for kc in KEYCODES.iter() {
                    if !matches!(kc.category, KeycodeCategory::Basic | KeycodeCategory::Function | KeycodeCategory::Navigation) { continue; }
                    if kc.value >= 0x0200 { continue; }
                    let resp = ui.add(egui::Button::new(RichText::new(kc.label).size(11.0))
                        .min_size(Vec2::new(44.0, 34.0)));
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
                        .min_size(Vec2::new(44.0, 34.0)));
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
                    .min_size(Vec2::new(72.0, 36.0)));
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
                    .min_size(Vec2::new(72.0, 36.0)));
                if resp.clicked() { self.vial_quantum_pending_mt = Some(*base); }
                resp.on_hover_text(tip.as_str());
            }
        });
    }

    fn show_macro_editor(&mut self, ctx: &egui::Context, _active_macro: u8) {
        let active_macro = self.macro_editor_open.unwrap_or(254);
        // Esc to close
        if ctx.input(|i| i.key_pressed(Key::Escape)) && self.macro_key_pick.is_none() {
            if (active_macro as usize) < self.macro_count {
                let n = active_macro as usize;
                self.encode_macro(n);
                self.result = Some(0x7700 + active_macro as u16);
                self.macros_dirty = true;
            }
            self.macro_editor_open = None;
            self.open = false;
            return;
        }
        let mut still_open = true;
        egui::Window::new("Macro Editor")
            .open(&mut still_open)
            .collapsible(false)
            .resizable(true)
            .min_size(Vec2::new(700.0, 450.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // Tabs for each macro (scrollable, 2 rows visible)
                // Clickable macro buttons
                ui.label("Switch macro:");
                ui.horizontal_wrapped(|ui| {
                    // Allow wrapping without constraining clickable areas
                    for tab_n in 0..128u8 {
                        let is_active = tab_n == active_macro as u8;
                        let has_content = self.macro_texts.get(tab_n as usize).map(|s| !s.is_empty()).unwrap_or(false);
                        let label = format!("M{}", tab_n);
                        let btn = egui::Button::new(RichText::new(&label).size(10.0)
                            .color(if is_active { Color32::WHITE } else if has_content { Color32::from_gray(200) } else { Color32::from_gray(100) }))
                            .fill(if is_active { Color32::from_rgb(91, 104, 223) } else { Color32::TRANSPARENT })
                            .min_size(Vec2::new(34.0, 22.0));
                        let resp = ui.add(btn);
                        resp.on_hover_text(format!("Edit macro {}", tab_n));
                        if resp.clicked() {
                            while self.macro_texts.len() <= tab_n as usize { self.macro_texts.push(String::new()); }
                            while self.macro_actions.len() <= tab_n as usize { self.macro_actions.push(vec![]); }
                            self.macro_editor_open = Some(tab_n);
                        }
                    }
                });
                ui.separator();

                let raw_n = self.macro_editor_open.unwrap_or(254);
                if raw_n == 254 {
                    ui.label(RichText::new("Select a macro tab above to edit").size(16.0).color(Color32::from_gray(140)));
                    return;
                }
                let n = raw_n as usize;
                while self.macro_actions.len() <= n { self.macro_actions.push(vec![]); }

                ui.label(RichText::new(format!("Macro M{}", n)).size(18.0).strong());
                ui.add_space(4.0);

                // List of actions
                let mut remove_idx = None;
                let mut move_up: Option<usize> = None;
                let mut move_down: Option<usize> = None;
                let avail_w = ui.available_width();
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    let action_count = self.macro_actions[n].len();
                    for (i, action) in self.macro_actions[n].iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            // Move buttons
                            let arrow_size = Vec2::new(26.0, 26.0);
                            let up_btn = egui::Button::new(RichText::new("↑").size(14.0))
                                .min_size(arrow_size)
                                .sense(if i > 0 { egui::Sense::click() } else { egui::Sense::hover() });
                            let down_btn = egui::Button::new(RichText::new("↓").size(14.0))
                                .min_size(arrow_size)
                                .sense(if i + 1 < action_count { egui::Sense::click() } else { egui::Sense::hover() });
                            if ui.add(up_btn).on_hover_text("Move up").clicked() && i > 0 { move_up = Some(i); }
                            if ui.add(down_btn).on_hover_text("Move down").clicked() && i + 1 < action_count { move_down = Some(i); }

                            // Type label — fixed width
                            let (type_label, type_color, tooltip) = match action {
                                MacroAction::Text(_) => ("Text", Color32::from_rgb(91, 104, 223), "Types text characters one by one"),
                                MacroAction::Tap(_) => ("Tap", Color32::from_rgb(91, 104, 223), "Press and release a key"),
                                MacroAction::Down(_) => ("Down", Color32::from_rgb(200, 150, 50), "Press a key (hold until Up)"),
                                MacroAction::Up(_) => ("Up", Color32::from_rgb(150, 200, 50), "Release a previously pressed key"),
                                MacroAction::Delay(_) => ("Delay", Color32::from_gray(150), "Wait before next action"),
                            };
                            ui.allocate_ui(Vec2::new(55.0, 30.0), |ui| {
                                ui.add(egui::Label::new(RichText::new(type_label).size(15.0).color(type_color).strong())
                                    .sense(egui::Sense::hover()))
                                    .on_hover_text(tooltip);
                            });

                            // Value
                            match action {
                                MacroAction::Text(text) => {
                                    let text_w = (avail_w - 220.0).max(150.0);
                                    ui.add_sized(Vec2::new(text_w, 30.0),
                                        egui::TextEdit::singleline(text)
                                        .hint_text("Type text here...")
                                        .font(egui::FontId::monospace(14.0)))
                                        .on_hover_text("Characters to type when this macro runs");
                                }
                                MacroAction::Tap(kc) => {
                                    let label = crate::keycode::KEYCODES.iter().find(|k| k.value == *kc as u16).map(|k| k.label).unwrap_or("?");
                                    if ui.add(egui::Button::new(RichText::new(label).size(18.0)).min_size(Vec2::new(100.0, 30.0)))
                                        .on_hover_text("Click to change key — press and release this key")
                                        .clicked() { self.macro_key_pick = Some((n, i)); }
                                }
                                MacroAction::Down(kc) => {
                                    let label = crate::keycode::KEYCODES.iter().find(|k| k.value == *kc as u16).map(|k| k.label).unwrap_or("?");
                                    if ui.add(egui::Button::new(RichText::new(label).size(18.0)).min_size(Vec2::new(100.0, 30.0)))
                                        .on_hover_text("Click to change key — holds down until Up")
                                        .clicked() { self.macro_key_pick = Some((n, i)); }
                                }
                                MacroAction::Up(kc) => {
                                    let label = crate::keycode::KEYCODES.iter().find(|k| k.value == *kc as u16).map(|k| k.label).unwrap_or("?");
                                    if ui.add(egui::Button::new(RichText::new(label).size(18.0)).min_size(Vec2::new(100.0, 30.0)))
                                        .on_hover_text("Click to change key — releases this key")
                                        .clicked() { self.macro_key_pick = Some((n, i)); }
                                }
                                MacroAction::Delay(ms) => {
                                    let mut ms_str = ms.to_string();
                                    if ui.add_sized(Vec2::new(80.0, 30.0),
                                        egui::TextEdit::singleline(&mut ms_str)
                                        .font(egui::FontId::monospace(14.0))).changed() {
                                        if let Ok(v) = ms_str.parse::<u16>() { *ms = v; }
                                    }
                                    ui.label(RichText::new("ms").size(14.0)).on_hover_text("Milliseconds to wait");
                                }
                            }

                            // Spacer to push ✕ to the right
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(egui::Button::new(RichText::new("✕").size(14.0))
                                    .min_size(Vec2::new(30.0, 30.0)))
                                    .on_hover_text("Remove this action")
                                    .clicked() {
                                    remove_idx = Some(i);
                                }
                            });
                        });
                        ui.add_space(2.0);
                    }
                });
                if let Some(idx) = remove_idx {
                    if idx < self.macro_actions[n].len() {
                        self.macro_undo_stack.push((n, self.macro_actions[n].clone()));
                        self.macro_actions[n].remove(idx);
                        // Reset key pick if it pointed at removed or later action
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

                // Add action button with dropdown
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("add_action")
                        .selected_text(RichText::new("+ Add action").size(14.0))
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

                    if ui.add(egui::Button::new("Clear all"))
                        .on_hover_text("Remove all actions from this macro")
                        .clicked() {
                        self.macro_undo_stack.push((n, self.macro_actions[n].clone()));
                        self.macro_actions[n].clear();
                    }
                    if !self.macro_undo_stack.is_empty() {
                        if ui.add(egui::Button::new("↩ Undo"))
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
                ui.label(RichText::new("Saved to keyboard when you close this editor")
                    .size(10.0).color(Color32::from_gray(120)));
            });

        if !still_open {
            let n = self.macro_editor_open.unwrap_or(0) as usize;
            self.encode_macro(n);
            self.result = Some(0x7700 + n as u16);
            self.macro_editor_open = None;
            self.macros_dirty = true;
            self.open = false;
        }
    }

    fn show_vial_tap_dance(&mut self, _ui: &mut egui::Ui) {
        self.tap_dance_editor_open = Some(255); // 255 = open editor, no selection
    }

    fn show_tap_dance_editor(&mut self, ctx: &egui::Context, active_td: u8) {
        if ctx.input(|i| i.key_pressed(Key::Escape)) && self.td_key_pick.is_none() {
            self.tap_dance_editor_open = None;
            self.tap_dance_dirty = true;
            if active_td < self.tap_dance_entries.len() as u8 {
                self.result = Some(0x7C00 + active_td as u16); // TD keycode
            }
            self.open = false;
            return;
        }

        let mut still_open = true;
        egui::Window::new("Tap Dance Editor")
            .open(&mut still_open)
            .collapsible(false)
            .resizable(true)
            .min_size(Vec2::new(600.0, 350.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // Tabs
                ui.horizontal_wrapped(|ui| {
                    for n in 0..self.tap_dance_entries.len() as u8 {
                        let is_active = n == active_td;
                        let label = format!("TD{}", n);
                        let btn = egui::Button::new(
                            RichText::new(&label).size(14.0)
                                .color(if is_active { Color32::WHITE } else { Color32::from_gray(100) })
                        ).fill(if is_active { Color32::from_rgb(91, 104, 223) } else { Color32::TRANSPARENT })
                         .min_size(Vec2::new(42.0, 30.0));
                        if ui.add(btn).clicked() {
                            self.tap_dance_editor_open = Some(n);
                        }
                    }
                });
                ui.separator();

                if active_td == 255 || active_td as usize >= self.tap_dance_entries.len() {
                    ui.label(RichText::new("Select a tap dance tab above to edit").size(16.0).color(Color32::from_gray(140)));
                    return;
                }

                let n = active_td as usize;
                ui.label(RichText::new(format!("Tap Dance TD{}", n)).size(18.0).strong());
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
                            .min_size(Vec2::new(120.0, 36.0)))
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
                        if ui.add_sized(Vec2::new(80.0, 30.0),
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
                self.result = Some(0x7C00 + active_td as u16);
            }
            self.tap_dance_editor_open = None;
            self.tap_dance_dirty = true;
            self.open = false;
        }
    }

    fn show_td_key_picker(&mut self, ctx: &egui::Context, td_idx: usize, field: u8) {
        // Esc to cancel
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.td_key_pick = None;
            return;
        }
        // Physical key capture
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    if let Some(qmk) = egui_key_to_qmk(*key, *modifiers) {
                        if qmk > 0 && qmk < 0x0100 {
                            if let Some(td) = self.tap_dance_entries.get_mut(td_idx) {
                                match field {
                                    0 => td.on_tap = qmk,
                                    1 => td.on_hold = qmk,
                                    2 => td.on_double_tap = qmk,
                                    3 => td.on_tap_hold = qmk,
                                    _ => {}
                                }
                            }
                            self.td_key_pick = None;
                        }
                    }
                }
            }
        });

        let field_name = match field {
            0 => "On Tap", 1 => "On Hold", 2 => "On Double Tap", 3 => "On Tap+Hold", _ => "?"
        };
        let mut still_open = true;
        egui::Window::new(format!("Pick key for {}", field_name))
            .open(&mut still_open)
            .collapsible(false)
            .resizable(false)
            .min_size(Vec2::new(400.0, 200.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(RichText::new("Press a key on your keyboard, or click below. Esc to cancel.")
                    .size(11.0).color(Color32::from_gray(140)));
                ui.add_space(4.0);
                // None button
                if ui.add(egui::Button::new(RichText::new("None (clear)").size(12.0))
                    .min_size(Vec2::new(100.0, 28.0))).clicked() {
                    if let Some(td) = self.tap_dance_entries.get_mut(td_idx) {
                        match field { 0 => td.on_tap = 0, 1 => td.on_hold = 0, 2 => td.on_double_tap = 0, 3 => td.on_tap_hold = 0, _ => {} }
                    }
                    self.td_key_pick = None;
                }
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    for kc in KEYCODES.iter() {
                        if !matches!(kc.category, KeycodeCategory::Basic | KeycodeCategory::Function | KeycodeCategory::Navigation) { continue; }
                        if kc.value == 0 || kc.value >= 0x0100 { continue; }
                        let resp = ui.add(egui::Button::new(RichText::new(kc.label).size(10.0))
                            .min_size(Vec2::new(36.0, 28.0)));
                        if resp.clicked() {
                            if let Some(td) = self.tap_dance_entries.get_mut(td_idx) {
                                match field { 0 => td.on_tap = kc.value, 1 => td.on_hold = kc.value, 2 => td.on_double_tap = kc.value, 3 => td.on_tap_hold = kc.value, _ => {} }
                            }
                            self.td_key_pick = None;
                        }
                        resp.on_hover_text(keycode_tooltip(kc.value, &[], &self.layer_names));
                    }
                });
            });
        if !still_open { self.td_key_pick = None; }
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

    fn show_vial_macros(&mut self, _ui: &mut egui::Ui) {
        // Open macro editor without selecting a specific macro
        self.macro_editor_open = Some(254); // 254 = editor open, no macro selected
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
            ("Brightness +", 0x7803, "Increase backlight brightness"),
            ("Brightness -", 0x7804, "Decrease backlight brightness"),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in bl_keys {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(10.5))
                    .min_size(Vec2::new(80.0, 36.0)));
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(*tip);
            }
        });

        ui.separator();
        // RGB Underglow (QMK rgblight)
        ui.label(RichText::new("RGB Underglow").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let rgb_keys: &[(&str, u16, &str)] = &[
            ("Toggle",       0x7A00, "Toggle RGB lighting on/off"),
            ("Next Mode",    0x7A01, "Switch to next RGB animation mode"),
            ("Prev Mode",    0x7A02, "Switch to previous RGB animation mode"),
            ("Hue +",        0x7A03, "Increase color hue"),
            ("Hue -",        0x7A04, "Decrease color hue"),
            ("Saturation +", 0x7A05, "Increase color saturation"),
            ("Saturation -", 0x7A06, "Decrease color saturation"),
            ("Brightness +", 0x7A07, "Increase brightness"),
            ("Brightness -", 0x7A08, "Decrease brightness"),
            ("Speed +",      0x7A09, "Increase animation speed"),
            ("Speed -",      0x7A0A, "Decrease animation speed"),
            ("Effect +",     0x7A0B, "Next RGB effect"),
            ("Effect -",     0x7A0C, "Previous RGB effect"),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in rgb_keys {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(10.5))
                    .min_size(Vec2::new(80.0, 36.0)));
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(*tip);
            }
        });

        ui.separator();
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
                    .min_size(Vec2::new(80.0, 36.0)));
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(*tip);
            }
        });

        ui.separator();
        // RGB Matrix controls
        ui.label(RichText::new("RGB Matrix Controls").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        let rgbm_ctrl: &[(&str, u16, &str)] = &[
            ("On",           0x7A16, "Turn RGB Matrix on"),
            ("Off",          0x7A17, "Turn RGB Matrix off"),
            ("Toggle",       0x7A18, "Toggle RGB Matrix on/off"),
            ("Next",         0x7A19, "Next RGB Matrix animation"),
            ("Previous",     0x7A1A, "Previous RGB Matrix animation"),
            ("Hue +",        0x7A1B, "Increase RGB Matrix hue"),
            ("Hue -",        0x7A1C, "Decrease RGB Matrix hue"),
            ("Saturation +", 0x7A1D, "Increase RGB Matrix saturation"),
            ("Saturation -", 0x7A1E, "Decrease RGB Matrix saturation"),
            ("Brightness +", 0x7A1F, "Increase RGB Matrix brightness"),
            ("Brightness -", 0x7A20, "Decrease RGB Matrix brightness"),
            ("Speed +",      0x7A21, "Increase RGB Matrix animation speed"),
            ("Speed -",      0x7A22, "Decrease RGB Matrix animation speed"),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in rgbm_ctrl {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(10.5))
                    .min_size(Vec2::new(80.0, 36.0)));
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(*tip);
            }
        });
    }

    fn show_vial_special(&mut self, ui: &mut egui::Ui) {
        let special_keys: &[(&str, u16, &str)] = &[
            ("~\nEsc",    0x7C16, "Grave/Escape — sends Esc normally, ` when Shift/GUI held"),
            ("⚡\nBoot",  0x7C77, "QK_BOOT — put keyboard into flash mode"),
            ("🐛\nDbg",   0x7C00, "DB_TOGG — toggle debug mode"),
            ("🔒\nLock",  0x7800, "QK_LOCK — hold to lock remaining keys until pressed again"),
            ("LSPO",      0x7C1A, "Left Shift when held, ( when tapped"),
            ("RSPC",      0x7C1B, "Right Shift when held, ) when tapped"),
            ("LCPO",      0x7C18, "Left Ctrl when held, ( when tapped"),
            ("RCPC",      0x7C19, "Right Ctrl when held, ) when tapped"),
            ("SftEnt",    0x7C1E, "Shift when held, Enter when tapped"),
            ("GEsc",      0x7C16, "Grave/Escape dual-function key"),
        ];
        ui.label(RichText::new("Special QMK keys").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in special_keys {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(10.5))
                    .min_size(Vec2::new(56.0, 42.0)));
                if resp.clicked() { self.result = Some(*value); self.open = false; }
                resp.on_hover_text(*tip);
            }
        });
    }

    // ─────────────────────────── ZMK PICKER ─────────────────────────────────

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
            .resizable(true)
            .min_size(Vec2::new(640.0, 420.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(RichText::new("Press a key on your keyboard, or pick below")
                    .size(11.0).color(Color32::from_gray(140)));
                ui.add_space(4.0);

                // Tab bar
                ui.horizontal_wrapped(|ui| {
                    for tab in KeycodeTab::ZMK_TABS {
                        let active = self.selected_tab == *tab;
                        let btn = egui::Button::new(RichText::new(tab.label()).size(12.0))
                            .fill(if active { Color32::from_rgb(91, 104, 223) } else { Color32::TRANSPARENT });
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
                ui.label("Key Press behavior not found on device.");
                return;
            }
        };
        ui.horizontal_wrapped(|ui| {
            for kc in KEYCODES.iter() {
                if !self.selected_tab.vial_matches(kc) { continue; }
                // Skip shifted symbols — ZMK uses raw HID usages
                if kc.value >= 0x0200 { continue; }
                let zmk_usage = 0x0007_0000u32 | kc.value as u32;
                let resp = ui.add(egui::Button::new(RichText::new(kc.label).size(11.0))
                    .min_size(Vec2::new(52.0, 38.0)));
                if resp.clicked() {
                    let id = beh_id;
                    self.zmk_assign(id, zmk_usage, 0);
                }
                resp.on_hover_text(keycode_tooltip(kc.value, &[], &self.layer_names));
            }
        });
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
                    Color32::from_rgb(91, 104, 223)
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
            ("⌃ LCtrl",       0x000700E0, "Left Control"),
            ("⇧ LShift",      0x000700E1, "Left Shift"),
            ("⌥ LAlt",        0x000700E2, "Left Alt"),
            ("⌃ RCtrl",       0x000700E4, "Right Control"),
            ("⇧ RShift",      0x000700E5, "Right Shift"),
            ("⌥ RAlt",        0x000700E6, "Right Alt / AltGr"),
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
                let resp = ui.add(egui::Button::new(RichText::new(format!("L{}", lgui)).size(11.0))
                    .min_size(Vec2::new(80.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, lgui_usage, 0); }
                resp.on_hover_text(format!("Left {}", lgui));
                let resp = ui.add(egui::Button::new(RichText::new(format!("R{}", rgui)).size(11.0))
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
                let resp = ui.add(egui::Button::new(RichText::new(format!("SK L{}", lgui)).size(10.5))
                    .min_size(Vec2::new(80.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, lgui_usage, 0); }
                resp.on_hover_text(format!("One-Shot Left {}", lgui));
                let resp = ui.add(egui::Button::new(RichText::new(format!("SK R{}", rgui)).size(10.5))
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
                let resp = ui.add(egui::Button::new(RichText::new("▽ TRNS").size(11.0))
                    .min_size(Vec2::new(64.0, 38.0)));
                if resp.clicked() { self.zmk_assign(id, 0, 0); }
                resp.on_hover_text("Transparent — fall through to layer below");
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
            ui.label(RichText::new("No additional behaviors found on this device.")
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
