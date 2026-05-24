/// Keycode picker modal for Vial/QMK keycodes.
use crate::keycode::{
    gui_label, gui_mod_name, gui_sym, key_label_font_sizes, keycode_label_with_names_and_layout,
    keycode_tooltip, modifier_label_from_bits, KeyLegendLayout, KeycodeCategory, KEYCODES,
};
use crate::popup_state::{PopupKey, PopupState};
use egui::{Color32, Key, RichText, Vec2};

#[path = "keycode_picker_keyboard.rs"]
mod keycode_picker_keyboard;
pub use keycode_picker_keyboard::egui_key_to_qmk;
#[path = "keycode_picker_model.rs"]
mod keycode_picker_model;
pub use keycode_picker_model::{BasicPickerLayout, KeycodeTab};
#[path = "keycode_picker_ui.rs"]
mod keycode_picker_ui;
use keycode_picker_ui::*;
#[path = "keycode_picker_popups.rs"]
mod keycode_picker_popups;
use keycode_picker_popups::*;
#[path = "keycode_picker_basic.rs"]
mod keycode_picker_basic;
#[path = "keycode_picker_lighting_quantum.rs"]
mod keycode_picker_lighting_quantum;
#[path = "keycode_picker_macro.rs"]
mod keycode_picker_macro;
#[path = "keycode_picker_special.rs"]
mod keycode_picker_special;
#[path = "keycode_picker_tabs.rs"]
mod keycode_picker_tabs;
#[path = "keycode_picker_tap_dance.rs"]
mod keycode_picker_tap_dance;
#[path = "keycode_picker_tap_dance_picker.rs"]
mod keycode_picker_tap_dance_picker;

fn plain_modifier_tooltip(mod_name: &str) -> String {
    format!(
        "Use {mod_name} by itself as a held modifier\nLeft click assigns Left {mod_name}\nRight click assigns Right {mod_name}"
    )
}

fn one_sided_modifier_tooltip(mod_name: &str, side: &str) -> String {
    format!("Use {side} {mod_name} by itself as a held modifier")
}

fn mod_combo_tooltip(mod_name: &str, has_right_side: bool) -> String {
    if has_right_side {
        format!(
            "Hold {mod_name} together with another key\nLeft click starts a Left {mod_name}+key binding\nRight click starts a Right {mod_name}+key binding\nThen choose the key part"
        )
    } else {
        format!("Hold {mod_name} together with another key\nClick to choose the key part")
    }
}

fn mod_tap_tooltip(mod_name: &str, has_right_side: bool) -> String {
    if has_right_side {
        format!(
            "Dual-role key: hold for {mod_name}, tap for another key\nLeft click uses Left {mod_name}\nRight click uses Right {mod_name}\nThen choose the tap key"
        )
    } else {
        format!(
            "Dual-role key: hold for {mod_name}, tap for another key\nClick to choose the tap key"
        )
    }
}

fn one_shot_modifier_tooltip(mod_name: &str, has_right_side: bool) -> String {
    if has_right_side {
        format!(
            "Applies {mod_name} to the next keypress only\nLeft click assigns One-Shot Left {mod_name}\nRight click assigns One-Shot Right {mod_name}"
        )
    } else {
        format!("Applies {mod_name} to the next keypress only")
    }
}

fn picker_ok_label(language: crate::i18n::Language) -> &'static str {
    match language {
        crate::i18n::Language::Russian => "Ок",
        crate::i18n::Language::English => "OK",
    }
}

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
    pub basic_layout: BasicPickerLayout,
    pub search_query: String,
    pub result: Option<u16>,
    pub custom_keycodes: Vec<(String, String, String, u16)>,
    pub supports_rgb: bool,
    pub supports_macro: bool,
    pub supports_tap_dance: bool,
    pub supports_mouse_keys: bool,
    pub supports_combo: bool,
    pub supports_auto_shift: bool,
    pub supports_caps_word: bool,
    pub supports_repeat_key: bool,
    pub supports_layer_lock: bool,
    pub supports_persistent_default_layer: bool,
    pub supports_bluetooth_custom_keycodes: bool,
    pub layer_names: Vec<String>,
    pub layer_count: usize,
    pub layer_has_content: Vec<bool>,
    pub listening: bool,
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
    pub language: crate::i18n::Language,
    pub key_legend_layout: KeyLegendLayout,
    pub show_shifted_number_symbols: bool,
}

fn tr_picker(language: crate::i18n::Language, key: &'static str) -> &'static str {
    crate::i18n::tr_catalog(language, key)
}

fn picker_tab_label(language: crate::i18n::Language, tab: KeycodeTab) -> &'static str {
    tr_picker(language, tab.i18n_key())
}

fn picker_mod_key_label(base: u16) -> String {
    format!("{}/key", modifier_label_from_bits(base >> 8))
}

fn picker_mod_tap_label(base: u16) -> String {
    format!("Hold {}/key", modifier_label_from_bits((base >> 8) & 0x1F))
}

fn is_bluetooth_custom_keycode(name: &str, label: &str, title: &str) -> bool {
    let upper_name = name.trim().to_ascii_uppercase();
    let lower_text = format!("{} {}", label, title).to_ascii_lowercase();
    let mentions_wireless = lower_text.contains("bluetooth") || lower_text.contains("ble");
    let bt_channel = upper_name
        .strip_prefix("BT")
        .map(|suffix| !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false);
    bt_channel
        || matches!(
            upper_name.as_str(),
            "NEXT_BT" | "PREV_BT" | "CLR_BT" | "CLEAR_BT"
        )
        || upper_name.contains("_BT")
        || upper_name.contains("BT_")
        || mentions_wireless
        || (upper_name == "SWITCH" && lower_text.contains("usb") && lower_text.contains("output"))
}

fn picker_action_label(label: &str) -> String {
    match label {
        "Brightness -" => "Bright\n-".to_string(),
        "Brightness +" => "Bright\n+".to_string(),
        "Saturation -" => "Sat\n-".to_string(),
        "Saturation +" => "Sat\n+".to_string(),
        "Hue -" => "Hue\n-".to_string(),
        "Hue +" => "Hue\n+".to_string(),
        "Speed -" => "Speed\n-".to_string(),
        "Speed +" => "Speed\n+".to_string(),
        "Effect -" => "Effect\n-".to_string(),
        "Effect +" => "Effect\n+".to_string(),
        "Prev Mode" => "Mode\nPrev".to_string(),
        "Next Mode" => "Mode\nNext".to_string(),
        other => other.to_string(),
    }
}

impl Default for KeycodePicker {
    fn default() -> Self {
        Self {
            open: false,
            selected_tab: KeycodeTab::Basic,
            basic_layout: BasicPickerLayout::Qwerty,
            search_query: String::new(),
            result: None,
            custom_keycodes: vec![],
            supports_rgb: true,
            supports_macro: true,
            supports_tap_dance: true,
            supports_mouse_keys: true,
            supports_combo: true,
            supports_auto_shift: true,
            supports_caps_word: true,
            supports_repeat_key: true,
            supports_layer_lock: true,
            supports_persistent_default_layer: true,
            supports_bluetooth_custom_keycodes: false,
            layer_names: (0..16).map(|i| i.to_string()).collect(),
            layer_count: 4,
            layer_has_content: vec![true; 16],
            listening: false,
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
            language: crate::i18n::default_language(),
            key_legend_layout: KeyLegendLayout::default(),
            show_shifted_number_symbols: true,
        }
    }
}

impl KeycodePicker {
    fn picker_keycode_tooltip(
        &self,
        value: u16,
        custom_pairs: &[crate::keyboard::CustomKeycode],
    ) -> String {
        keycode_tooltip(value, custom_pairs, &self.layer_names)
    }

    fn assign_keycode_value(&mut self, value: u16) {
        self.result = Some(value);
        self.open = false;
    }

    fn finish_quantum_pending_key(&mut self, base: u16, key_value: u16, is_mt: bool) {
        let _ = is_mt;
        self.result = Some(base | key_value);
        self.vial_quantum_pending_mod = None;
        self.vial_quantum_pending_mt = None;
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

        self.popup_state
            .begin_frame(PopupKey::PickerWindow, self.open);
        self.popup_state
            .begin_frame(PopupKey::MacroKeyPickWindow, macro_key_pick_open);
        self.popup_state
            .begin_frame(PopupKey::PickLayerWindow, layer_pick_open);
        self.popup_state
            .begin_frame(PopupKey::PendingKeyPickWindow, pending_key_pick_open);
        self.popup_state
            .begin_frame(PopupKey::TapDanceEditorWindow, tap_dance_editor_open);
        self.popup_state
            .begin_frame(PopupKey::TdKeyPickWindow, td_key_pick_open);

        if !self.open {
            return;
        }

        // If pending mod/MT — show only the minimal second picker, not the full picker
        let has_pending = self.vial_quantum_pending_mod.is_some()
            || self.vial_quantum_pending_mt.is_some()
            || self.vial_layer_pending.is_some();
        if has_pending {
            self.show_pending_picker(ctx);
            return;
        }

        // Macro key picker (sub-window of macro editor)
        if let Some((macro_idx, action_idx)) = self.macro_key_pick {
            let mut pick_open = true;
            let popup_size = key_picker_popup_size(ctx);
            crate::ui_style::centered_modal_window(
                ctx,
                tr_picker(self.language, "key_picker.pick_key_title"),
                self.popup_state.id(PopupKey::MacroKeyPickWindow),
                &mut pick_open,
                popup_size,
            )
            .show(ctx, |ui| {
                apply_picker_button_visuals(ui);
                crate::ui_style::modal_intro(
                    ui,
                    tr_picker(self.language, "key_picker.press_key_or_click"),
                );
                crate::ui_style::modal_hint(
                    ui,
                    tr_picker(self.language, "key_picker.best_for_normal"),
                );
                ui.add_space(crate::ui_style::modal_space_xs());
                // Physical key capture
                ctx.input(|i| {
                    for event in &i.events {
                        if let egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            ..
                        } = event
                        {
                            if *key == Key::Escape {
                                self.macro_key_pick = None;
                                return;
                            }
                            if let Some(qmk) = egui_key_to_qmk(*key, *modifiers) {
                                if qmk > 0 && qmk < 0x0100 {
                                    if let Some(action) = self
                                        .macro_actions
                                        .get_mut(macro_idx)
                                        .and_then(|a| a.get_mut(action_idx))
                                    {
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
                if picker_button(
                    ui,
                    tr_picker(self.language, "key_picker.none_clear"),
                    crate::ui_style::modal_action_button_size(),
                    true,
                    false,
                )
                .clicked()
                {
                    if let Some(action) = self
                        .macro_actions
                        .get_mut(macro_idx)
                        .and_then(|a| a.get_mut(action_idx))
                    {
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
                let key_choices: Vec<&'static crate::keycode::Keycode> = KEYCODES
                    .iter()
                    .filter(|kc| {
                        kc.value != 0
                            && kc.value != 0x0001
                            && !kc.name.starts_with("RGB_")
                            && matches!(
                                kc.category,
                                KeycodeCategory::Basic
                                    | KeycodeCategory::Function
                                    | KeycodeCategory::Navigation
                                    | KeycodeCategory::Media
                                    | KeycodeCategory::Special
                            )
                            && kc.value < 0x0100
                            && !is_f13_to_f24(kc.value)
                    })
                    .collect();
                egui::ScrollArea::vertical()
                    .max_height(key_picker_popup_scroll_height(popup_size))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Some(value) = show_grouped_popup_key_buttons(
                            ui,
                            key_choices,
                            &self.layer_names,
                            false,
                            self.language,
                            self.key_legend_layout,
                        ) {
                            if let Some(action) = self
                                .macro_actions
                                .get_mut(macro_idx)
                                .and_then(|a| a.get_mut(action_idx))
                            {
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

        self.show_vial(ctx);
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
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        // Physical key capture only when no pending mod (avoid accidental assignment)
                        if self.vial_quantum_pending_mod.is_none()
                            && self.vial_quantum_pending_mt.is_none()
                        {
                            if self.search_query.is_empty() || modifiers.any() {
                                if let Some(qmk) = egui_key_to_qmk(*key, *modifiers) {
                                    self.assign_keycode_value(qmk);
                                }
                            }
                        } else {
                            // Pending mod: only accept basic keys (no mods pressed)
                            if !modifiers.any() {
                                if let Some(qmk) = egui_key_to_qmk(*key, *modifiers) {
                                    if qmk > 0 && qmk < 0x0100 {
                                        let base = self
                                            .vial_quantum_pending_mod
                                            .or(self.vial_quantum_pending_mt)
                                            .unwrap_or(0);
                                        let is_mt = self.vial_quantum_pending_mod.is_none()
                                            && self.vial_quantum_pending_mt.is_some();
                                        self.finish_quantum_pending_key(base, qmk, is_mt);
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }

        let mut still_open = true;
        let picker_size = key_picker_main_size(ctx);
        crate::ui_style::centered_modal_window(
            ctx,
            tr_picker(self.language, "key_picker.title"),
            self.popup_state.id(PopupKey::PickerWindow),
            &mut still_open,
            picker_size,
        )
        .show(ctx, |ui| {
            apply_picker_button_visuals(ui);
            ui.vertical_centered(|ui| {
                crate::ui_style::modal_intro(
                    ui,
                    tr_picker(self.language, "key_picker.press_key_or_pick"),
                );
            });
            ui.add_space(4.0);

            if !self.vial_tab_supported(self.selected_tab) {
                self.selected_tab = KeycodeTab::Basic;
            }

            // Tab bar
            let tabs = KeycodeTab::VIAL_TABS;
            let visible_tabs: Vec<KeycodeTab> = tabs
                .iter()
                .copied()
                .filter(|tab| self.vial_tab_supported(*tab))
                .collect();
            let tab_spacing = 6.0;
            let tab_bar_width: f32 = visible_tabs
                .iter()
                .map(|tab| picker_tab_width(picker_tab_label(self.language, *tab)))
                .sum::<f32>()
                + tab_spacing * visible_tabs.len().saturating_sub(1) as f32;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(tab_spacing, 6.0);
                let x_offset = ((ui.available_width() - tab_bar_width).max(0.0) * 0.5).floor();
                if x_offset > 0.0 {
                    ui.add_space(x_offset);
                }
                for tab in &visible_tabs {
                    let active = self.selected_tab == *tab;
                    let tab_label = picker_tab_label(self.language, *tab);
                    if picker_tab_button(ui, tab_label, active).clicked() {
                        self.selected_tab = *tab;
                        self.vial_quantum_pending_mod = None;
                        self.vial_quantum_pending_mt = None;
                        self.vial_layer_pending = None;
                    }
                }
            });
            ui.add_space(crate::ui_style::modal_space_sm());

            let content_height = key_picker_main_content_height(picker_size);
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
                                    ui.add_space(28.0);
                                    self.show_vial_tab_content(ui);
                                } else {
                                    let centered_width = self.tab_content_width(ui);
                                    let x_offset =
                                        ((ui.available_width() - centered_width).max(0.0) * 0.5)
                                            .floor();
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
                    if let egui::Event::Key {
                        key, pressed: true, ..
                    } = event
                    {
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
                tr_picker(self.language, "key_picker.pick_layer_title"),
                self.popup_state.id(PopupKey::PickLayerWindow),
                &mut still_open,
                Vec2::new(300.0, 120.0),
            )
            .show(ctx, |ui| {
                apply_picker_button_visuals(ui);
                crate::ui_style::modal_intro(
                    ui,
                    tr_picker(self.language, "key_picker.pick_layer_intro"),
                );
                ui.add_space(crate::ui_style::modal_space_sm());
                ui.horizontal_wrapped(|ui| {
                    for n in 0u16..self.layer_count.max(1) as u16 {
                        let raw = self
                            .layer_names
                            .get(n as usize)
                            .cloned()
                            .unwrap_or(n.to_string());
                        let label = if !raw.is_empty() && raw != n.to_string() {
                            format!("{}: {}", n, raw)
                        } else {
                            format!("Layer {}", n)
                        };
                        let resp = picker_button(
                            ui,
                            &label,
                            crate::ui_style::modal_small_button_size(84.0),
                            true,
                            false,
                        );
                        let resp = resp.on_hover_text(crate::i18n::tr_text(self.language, &label));
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

        let pending = self
            .vial_quantum_pending_mod
            .or(self.vial_quantum_pending_mt);
        let is_mt =
            self.vial_quantum_pending_mod.is_none() && self.vial_quantum_pending_mt.is_some();
        // Physical key capture for pending
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
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
                                    self.finish_quantum_pending_key(base, qmk, is_mt);
                                }
                            }
                        }
                    }
                }
            }
        });

        if let Some(base) = pending {
            let title = if is_mt {
                tr_picker(self.language, "key_picker.pick_tap_key_title")
            } else {
                tr_picker(self.language, "key_picker.pick_modifier_combo_title")
            };
            let mut still_open = true;
            let popup_size = key_picker_popup_size(ctx);
            let resp_win = crate::ui_style::centered_modal_window(
                ctx,
                title,
                self.popup_state.id(PopupKey::PendingKeyPickWindow),
                &mut still_open,
                popup_size,
            )
            .show(ctx, |ui| {
                apply_picker_button_visuals(ui);
                crate::ui_style::modal_intro(
                    ui,
                    tr_picker(self.language, "key_picker.press_key_or_click_cancel"),
                );
                ui.add_space(crate::ui_style::modal_space_sm());
                let key_choices: Vec<&'static crate::keycode::Keycode> = KEYCODES
                    .iter()
                    .filter(|kc| {
                        matches!(
                            kc.category,
                            KeycodeCategory::Basic
                                | KeycodeCategory::Function
                                | KeycodeCategory::Navigation
                        )
                    })
                    .filter(|kc| kc.value != 0 && kc.value < 0x0100 && !is_f13_to_f24(kc.value))
                    .collect();
                egui::ScrollArea::vertical()
                    .max_height(key_picker_popup_scroll_height(popup_size))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Some(value) = show_grouped_popup_key_buttons(
                            ui,
                            key_choices,
                            &self.layer_names,
                            false,
                            self.language,
                            self.key_legend_layout,
                        ) {
                            self.finish_quantum_pending_key(base, value, is_mt);
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

    fn vial_tab_supported(&self, tab: KeycodeTab) -> bool {
        match tab {
            KeycodeTab::Rgb => self.supports_rgb,
            KeycodeTab::Macro => self.supports_macro,
            KeycodeTab::TapDance => self.supports_tap_dance,
            KeycodeTab::Bluetooth => {
                self.supports_bluetooth_custom_keycodes
                    && self.custom_keycodes.iter().any(|(name, label, title, _)| {
                        is_bluetooth_custom_keycode(name, label, title)
                    })
            }
            KeycodeTab::Custom => self.custom_keycodes.iter().any(|(name, label, title, _)| {
                !self.supports_bluetooth_custom_keycodes
                    || !is_bluetooth_custom_keycode(name, label, title)
            }),
            _ => true,
        }
    }

    fn vial_keycode_supported(&self, kc: &crate::keycode::Keycode) -> bool {
        match kc.name {
            "QK_CAPS_WORD_TOGGLE" => self.supports_caps_word,
            "QK_REPEAT_KEY" | "QK_ALT_REPEAT_KEY" => self.supports_repeat_key,
            "CMB_TOG" => self.supports_combo,
            "KC_ASTG" => self.supports_auto_shift,
            "QK_LAYER_LOCK" => self.supports_layer_lock,
            name if name.starts_with("RGB_") => self.supports_rgb,
            name if name.starts_with("BL_") => false,
            _ => true,
        }
    }

    fn show_vial_tab_content(&mut self, ui: &mut egui::Ui) {
        match self.selected_tab {
            KeycodeTab::Basic => self.show_vial_basic(ui),
            KeycodeTab::Symbols => self.show_vial_symbols(ui),
            KeycodeTab::Layers => self.show_vial_layers(ui),
            KeycodeTab::Modifiers => self.show_vial_modifiers(ui),
            KeycodeTab::Quantum => self.show_vial_quantum(ui),
            KeycodeTab::Rgb => self.show_vial_rgb(ui),
            KeycodeTab::Macro => self.show_vial_macros(ui),
            KeycodeTab::TapDance => self.show_vial_tap_dance(ui),
            KeycodeTab::Special => self.show_vial_special(ui),
            KeycodeTab::Bluetooth => self.show_vial_bluetooth(ui),
            KeycodeTab::Custom => self.show_vial_custom(ui),
            _ => self.show_vial_generic(ui),
        }
    }
}
