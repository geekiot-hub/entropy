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
#[path = "keycode_picker_lighting_quantum.rs"]
mod keycode_picker_lighting_quantum;
#[path = "keycode_picker_macro.rs"]
mod keycode_picker_macro;
#[path = "keycode_picker_tap_dance.rs"]
mod keycode_picker_tap_dance;

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
                    for n in 0u16..self.layer_names.len().max(4) as u16 {
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
        let right_nav_extra_gap = if col >= 16 && matches!(row, 1 | 2) {
            14.0
        } else {
            0.0
        };
        let y = origin.y + row as f32 * (cell_h + gap) + right_nav_extra_gap;
        let width = span as f32 * cell_w + span.saturating_sub(1) as f32 * gap;
        let rect = egui::Rect::from_min_size(egui::pos2(x, y), Vec2::new(width, cell_h));
        let resp = picker_keycap_button_in_rect(ui, rect, label, true, false);
        if resp.clicked() {
            self.assign_keycode_value(value);
        }
        if resp.hovered() {
            resp.on_hover_text(crate::i18n::tr_text(
                self.language,
                &keycode_tooltip(value, &[], &self.layer_names),
            ));
        }
    }

    fn show_vial_basic(&mut self, ui: &mut egui::Ui) {
        const COLS: usize = 16;
        const ROWS: usize = 6;

        let scale = responsive_picker_element_scale(ui.ctx());
        let cell_w = 54.0 * scale;
        let cell_h = 54.0 * scale;
        let gap = 3.0 * scale;
        let width = COLS as f32 * cell_w + (COLS.saturating_sub(1)) as f32 * gap;
        let height = ROWS as f32 * cell_h + (ROWS.saturating_sub(1)) as f32 * gap;
        let available_width = ui.available_width();
        let x_offset = ((available_width - width).max(0.0) * 0.5).floor();

        ui.horizontal(|ui| {
            if x_offset > 0.0 {
                ui.add_space(x_offset);
            }
            ui.allocate_ui_with_layout(
                Vec2::new(width, 32.0 * scale),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(
                        RichText::new(tr_picker(self.language, "key_picker.section_basic"))
                            .size(11.0 * scale)
                            .color(Color32::from_gray(150)),
                    );
                    let dropdown_width = 126.0 * scale;
                    let spacer = (ui.available_width() - dropdown_width).max(0.0);
                    if spacer > 0.0 {
                        ui.add_space(spacer);
                    }
                    let dropdown_id = ui.make_persistent_id("basic_layout_dropdown");
                    let dropdown_resp = crate::ui_style::modern_dropdown_button(
                        ui,
                        dropdown_id,
                        self.basic_layout.label(),
                        ui.visuals().text_color(),
                        dropdown_width,
                    );
                    egui::popup_below_widget(
                        ui,
                        dropdown_id,
                        &dropdown_resp,
                        egui::PopupCloseBehavior::CloseOnClickOutside,
                        |ui| {
                            ui.set_min_width(dropdown_width);
                            ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                            for layout in BasicPickerLayout::ALL {
                                let selected = self.basic_layout == layout;
                                let (option_rect, option_resp) = ui.allocate_exact_size(
                                    Vec2::new(dropdown_width, 28.0 * scale),
                                    egui::Sense::click(),
                                );
                                if option_resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                let fill = if selected {
                                    if ui.visuals().dark_mode {
                                        Color32::from_rgb(58, 58, 61)
                                    } else {
                                        Color32::from_rgb(236, 236, 238)
                                    }
                                } else if option_resp.hovered() {
                                    crate::ui_style::hover_fill(ui.visuals().dark_mode)
                                } else {
                                    Color32::TRANSPARENT
                                };
                                ui.painter().rect_filled(option_rect, 7.0, fill);
                                ui.painter().text(
                                    option_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    layout.label(),
                                    egui::FontId::proportional(12.0 * scale),
                                    if selected {
                                        ui.visuals().text_color()
                                    } else {
                                        Color32::from_gray(150)
                                    },
                                );
                                if option_resp.clicked() {
                                    self.basic_layout = layout;
                                    ui.memory_mut(|m| m.close_popup());
                                }
                            }
                        },
                    );
                },
            );
        });
        ui.add_space(4.0);

        let keys: &[(usize, usize, usize, &str, u16)] = &[
            (0, 0, 1, "Esc", 0x0029),
            (0, 1, 1, "F1", 0x003A),
            (0, 2, 1, "F2", 0x003B),
            (0, 3, 1, "F3", 0x003C),
            (0, 4, 1, "F4", 0x003D),
            (0, 5, 1, "F5", 0x003E),
            (0, 6, 1, "F6", 0x003F),
            (0, 7, 1, "F7", 0x0040),
            (0, 8, 1, "F8", 0x0041),
            (0, 9, 1, "F9", 0x0042),
            (0, 10, 1, "F10", 0x0043),
            (0, 11, 1, "F11", 0x0044),
            (0, 12, 1, "F12", 0x0045),
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
            (1, 13, 1, "Backspace", 0x002A),
            (1, 14, 1, "Insert", 0x0049),
            (1, 15, 1, "Delete", 0x004C),
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
            (3, 0, 2, "Caps\nLock", 0x0039),
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
            (5, 2, 1, gui_label(false), 0x00E3),
            (5, 3, 1, "Alt", 0x00E2),
            (5, 4, 4, "Space", 0x002C),
            (5, 8, 1, "Alt", 0x00E6),
            (5, 9, 1, "Menu", 0x0065),
            (5, 10, 1, "Ctrl", 0x00E4),
            (0, 13, 1, "Print\nScreen", 0x0046),
            (0, 14, 1, "Scroll\nLock", 0x0047),
            (0, 15, 1, "Pause", 0x0048),
            (2, 15, 1, "Home", 0x004A),
            (3, 15, 1, "End", 0x004D),
            (4, 15, 1, "Page\nUp", 0x004B),
            (5, 15, 1, "Page\nDown", 0x004E),
            (5, 11, 1, "←", 0x0050),
            (5, 12, 1, "↑", 0x0052),
            (5, 13, 1, "↓", 0x0051),
            (5, 14, 1, "→", 0x004F),
        ];

        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(available_width, height), egui::Sense::hover());
        let origin = egui::pos2(rect.min.x + x_offset, rect.min.y);
        for &(row, col, span, fallback_label, value) in keys {
            let assigned_value = self.basic_layout.map_value(value);
            let display_label = if self.key_legend_layout != KeyLegendLayout::English {
                if self.show_shifted_number_symbols {
                    if let Some(label) =
                        picker_shifted_number_label(assigned_value, self.key_legend_layout)
                    {
                        label
                    } else {
                        crate::keycode::find_keycode(assigned_value)
                            .map(|_| {
                                keycode_label_with_names_and_layout(
                                    assigned_value,
                                    &[],
                                    &self.layer_names,
                                    self.key_legend_layout,
                                )
                            })
                            .unwrap_or_else(|| fallback_label.to_string())
                    }
                } else {
                    crate::keycode::find_keycode(assigned_value)
                        .map(|_| {
                            keycode_label_with_names_and_layout(
                                assigned_value,
                                &[],
                                &self.layer_names,
                                self.key_legend_layout,
                            )
                        })
                        .unwrap_or_else(|| fallback_label.to_string())
                }
            } else {
                if self.show_shifted_number_symbols {
                    match assigned_value {
                        0x0035 => "~\n`".to_string(),
                        0x001E => "!\n1".to_string(),
                        0x001F => "@\n2".to_string(),
                        0x0020 => "#\n3".to_string(),
                        0x0021 => "$\n4".to_string(),
                        0x0022 => "%\n5".to_string(),
                        0x0023 => "^\n6".to_string(),
                        0x0024 => "&\n7".to_string(),
                        0x0025 => "*\n8".to_string(),
                        0x0026 => "(\n9".to_string(),
                        0x0027 => ")\n0".to_string(),
                        0x002D => "_\n-".to_string(),
                        0x002E => "+\n=".to_string(),
                        _ => crate::keycode::find_keycode(assigned_value)
                            .map(|_| {
                                keycode_label_with_names_and_layout(
                                    assigned_value,
                                    &[],
                                    &self.layer_names,
                                    self.key_legend_layout,
                                )
                            })
                            .unwrap_or_else(|| fallback_label.to_string()),
                    }
                } else {
                    crate::keycode::find_keycode(assigned_value)
                        .map(|_| {
                            keycode_label_with_names_and_layout(
                                assigned_value,
                                &[],
                                &self.layer_names,
                                self.key_legend_layout,
                            )
                        })
                        .unwrap_or_else(|| fallback_label.to_string())
                }
            };
            self.basic_key_button_at(
                ui,
                origin,
                cell_w,
                cell_h,
                gap,
                row,
                col,
                span,
                &display_label,
                assigned_value,
            );
        }
    }

    fn vial_tab_supported(&self, tab: KeycodeTab) -> bool {
        match tab {
            KeycodeTab::Rgb => self.supports_rgb,
            KeycodeTab::Macro => self.supports_macro,
            KeycodeTab::TapDance => self.supports_tap_dance,
            KeycodeTab::Custom => !self.custom_keycodes.is_empty(),
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
            KeycodeTab::Custom => self.show_vial_custom(ui),
            _ => self.show_vial_generic(ui),
        }
    }

    fn show_vial_symbols(&mut self, ui: &mut egui::Ui) {
        let custom_pairs: Vec<crate::keyboard::CustomKeycode> = self
            .custom_keycodes
            .iter()
            .map(|(name, label, title, _)| crate::keyboard::CustomKeycode {
                name: name.clone(),
                label: label.clone(),
                title: title.clone(),
            })
            .collect();

        let main_symbol_order = [
            '.', ',', ';', ':', '!', '?', '/', '`', '~', '\'', '"', '(', ')', '[', ']', '{', '}',
            '<', '>', '+', '*', '=', '#', '@', '$', '%', '^', '&', '|', '\\', '_',
        ];
        let extra_symbol_order = [
            '₽', '€', '«', '»', '‘', '’', '„', '“', '”', '—', '–', '•', '×', '±', '≠', '≈', '✓',
            '§', '°', '‰', '′', '″', '™', '№',
        ];

        ui.label(
            RichText::new(tr_picker(
                self.language,
                "key_picker.section_universal_symbols",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        if let Some(hint) = crate::smart_input::universal_output_setup_hint() {
            ui.add_space(3.0);
            ui.label(
                RichText::new(crate::i18n::tr_text(self.language, hint))
                    .size(10.0)
                    .color(Color32::from_gray(120)),
            );
        }
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for wanted in main_symbol_order {
                let Some(smart) = crate::smart_input::SMART_SYMBOLS
                    .iter()
                    .copied()
                    .find(|smart| smart.symbol == wanted)
                else {
                    continue;
                };
                let label = smart.symbol.to_string();
                let tip = format!(
                    "Universal symbol: {} — types {} consistently regardless of the active keyboard language",
                    smart.name, smart.symbol
                );
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                Self::paint_compact_picker_label(ui, &resp, &label);
                if resp.clicked() {
                    self.assign_keycode_value(smart.trigger_keycode);
                }
                resp.on_hover_text(crate::i18n::tr_text(self.language, &tip));
            }
        });

        ui.add_space(10.0);
        ui.label(
            RichText::new(tr_picker(
                self.language,
                "key_picker.section_layout_symbols",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for kc in KEYCODES.iter() {
                if !self.selected_tab.vial_matches(kc) || !self.vial_keycode_supported(kc) {
                    continue;
                }
                let label = keycode_label_with_names_and_layout(
                    kc.value,
                    &custom_pairs,
                    &self.layer_names,
                    self.key_legend_layout,
                );
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                Self::paint_compact_picker_label(ui, &resp, &label);
                if resp.clicked() {
                    self.assign_keycode_value(kc.value);
                }
                if resp.hovered() {
                    resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &self.picker_keycode_tooltip(kc.value, &custom_pairs),
                    ));
                }
            }
        });

        ui.add_space(10.0);
        ui.label(
            RichText::new(tr_picker(
                self.language,
                "key_picker.section_extra_universal_symbols",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for wanted in extra_symbol_order {
                let Some(smart) = crate::smart_input::SMART_SYMBOLS
                    .iter()
                    .copied()
                    .find(|smart| smart.symbol == wanted)
                else {
                    continue;
                };
                let label = smart.symbol.to_string();
                let tip = format!(
                    "Universal symbol: {} — types {} consistently regardless of the active keyboard language",
                    smart.name, smart.symbol
                );
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                Self::paint_compact_picker_label(ui, &resp, &label);
                if resp.clicked() {
                    self.assign_keycode_value(smart.trigger_keycode);
                }
                resp.on_hover_text(crate::i18n::tr_text(self.language, &tip));
            }
        });
    }

    fn show_vial_generic(&mut self, ui: &mut egui::Ui) {
        let custom_pairs: Vec<crate::keyboard::CustomKeycode> = self
            .custom_keycodes
            .iter()
            .map(|(name, label, title, _)| crate::keyboard::CustomKeycode {
                name: name.clone(),
                label: label.clone(),
                title: title.clone(),
            })
            .collect();
        ui.horizontal_wrapped(|ui| {
            for kc in KEYCODES.iter() {
                if !self.selected_tab.vial_matches(kc) || !self.vial_keycode_supported(kc) {
                    continue;
                }
                let label = keycode_label_with_names_and_layout(
                    kc.value,
                    &custom_pairs,
                    &self.layer_names,
                    self.key_legend_layout,
                );
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                Self::paint_compact_picker_label(ui, &resp, &label);
                if resp.clicked() {
                    self.assign_keycode_value(kc.value);
                }
                if resp.hovered() {
                    resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &self.picker_keycode_tooltip(kc.value, &custom_pairs),
                    ));
                }
            }
        });
    }

    fn show_vial_custom(&mut self, ui: &mut egui::Ui) {
        let custom_keycodes = self.custom_keycodes.clone();
        ui.label(
            RichText::new(tr_picker(
                self.language,
                "key_picker.section_custom_keycodes",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (name, label, title, value) in custom_keycodes {
                if label.is_empty() {
                    continue;
                }
                let tip = if title.trim().is_empty() {
                    name.as_str()
                } else {
                    title.as_str()
                };
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                Self::paint_compact_picker_label(ui, &resp, &label);
                if resp.clicked() {
                    self.assign_keycode_value(value);
                }
                resp.on_hover_text(crate::i18n::tr_text(self.language, &tip));
            }
        });
    }

    fn show_vial_layers(&mut self, ui: &mut egui::Ui) {
        let ops: &[(u16, &str, &str)] = &[
            (0x5220, "Layer\nMO", "Hold to activate, release to return"),
            (0x5260, "Layer\nTG", "Tap to toggle on/off"),
            (0x5280, "Layer\nOSL", "Active for next keypress only"),
            (0x52C0, "Layer\nTT", "Hold = MO, tap = toggle"),
            (0x5200, "Layer\nTO", "Switch and stay on this layer"),
            (0x5240, "Layer\nDF", "Set as permanent base layer"),
        ];

        ui.label(
            RichText::new(tr_picker(self.language, "key_picker.section_layers"))
                .size(11.0)
                .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (base, label, hint) in ops {
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                Self::paint_compact_picker_label(ui, &resp, label);
                if resp.clicked() {
                    self.vial_layer_pending = Some(*base);
                }
                resp.on_hover_text(crate::i18n::tr_catalog(self.language, hint));
            }
            let lt_resp = ui
                .add(egui::Button::new("").min_size(Self::picker_key_size(ui.ctx())))
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            Self::paint_compact_picker_label(ui, &lt_resp, "Layer\nLT");
            if lt_resp.clicked() {
                self.vial_layer_pending = Some(0x4000);
            }
            lt_resp.on_hover_text(crate::i18n::tr_catalog(self.language, "key_picker_text.hold_activate_layer_tap_keycode_set_key_via_right_click_afterwards"));
        });
    }

    fn show_vial_modifiers(&mut self, ui: &mut egui::Ui) {
        let gui = gui_label(false);
        let lgui = gui_label(false);

        ui.label(
            RichText::new(tr_picker(
                self.language,
                "key_picker.section_plain_modifiers",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let plain: Vec<(String, u16, u16, String)> = vec![
            ("Ctrl".into(), 0x00E0, 0x00E4, "Ctrl".into()),
            ("Shift".into(), 0x00E1, 0x00E5, "Shift".into()),
            ("Alt".into(), 0x00E2, 0x00E6, "Alt".into()),
            (gui.into(), 0x00E3, 0x00E7, lgui.to_string()),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, left_value, right_value, mod_name) in &plain {
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                Self::paint_compact_picker_label(ui, &resp, label);
                if resp.clicked_by(egui::PointerButton::Primary) {
                    self.assign_keycode_value(*left_value);
                }
                if resp.clicked_by(egui::PointerButton::Secondary) {
                    self.assign_keycode_value(*right_value);
                }
                resp.on_hover_text(crate::i18n::tr_text(
                    self.language,
                    &plain_modifier_tooltip(mod_name),
                ));
            }
        });

        ui.add_space(10.0);
        self.show_vial_layers(ui);

        ui.add_space(10.0);
        ui.label(
            RichText::new(tr_picker(self.language, "key_picker.section_mod_key"))
                .size(11.0)
                .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let mk: Vec<(String, u16, Option<u16>, String)> = vec![
            (
                picker_mod_key_label(0x0100),
                0x0100,
                Some(0x1100),
                "Ctrl".into(),
            ),
            (
                picker_mod_key_label(0x0200),
                0x0200,
                Some(0x1200),
                "Shift".into(),
            ),
            (
                picker_mod_key_label(0x0400),
                0x0400,
                Some(0x1400),
                "Alt".into(),
            ),
            (
                picker_mod_key_label(0x0800),
                0x0800,
                Some(0x1800),
                lgui.to_string(),
            ),
            (
                picker_mod_key_label(0x0300),
                0x0300,
                None,
                "Ctrl+Shift".into(),
            ),
            (
                picker_mod_key_label(0x0500),
                0x0500,
                None,
                "Ctrl+Alt".into(),
            ),
            (
                picker_mod_key_label(0x0600),
                0x0600,
                None,
                "Shift+Alt (LSA)".into(),
            ),
            (
                picker_mod_key_label(0x0700),
                0x0700,
                None,
                "Ctrl+Shift+Alt".into(),
            ),
            (
                picker_mod_key_label(0x0A00),
                0x0A00,
                None,
                format!("{}+Shift", lgui),
            ),
            (
                picker_mod_key_label(0x0F00),
                0x0F00,
                None,
                format!("Ctrl+Shift+Alt+{}", gui_mod_name()),
            ),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, left_value, right_value, mod_name) in &mk {
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                Self::paint_compact_picker_label(ui, &resp, label);
                if resp.clicked_by(egui::PointerButton::Primary) {
                    self.vial_quantum_pending_mod = Some(*left_value);
                }
                if let Some(right_value) = right_value {
                    if resp.clicked_by(egui::PointerButton::Secondary) {
                        self.vial_quantum_pending_mod = Some(*right_value);
                    }
                    resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &mod_combo_tooltip(mod_name, true),
                    ));
                } else {
                    resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &mod_combo_tooltip(mod_name, false),
                    ));
                }
            }
        });

        ui.add_space(10.0);
        ui.label(
            RichText::new(tr_picker(self.language, "key_picker.section_mod_tap"))
                .size(11.0)
                .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let mut mt: Vec<(String, u16, Option<u16>, String)> = vec![
            (
                picker_mod_tap_label(0x2100),
                0x2100,
                Some(0x3100),
                "Ctrl".into(),
            ),
            (
                picker_mod_tap_label(0x2200),
                0x2200,
                Some(0x3200),
                "Shift".into(),
            ),
            (
                picker_mod_tap_label(0x2400),
                0x2400,
                Some(0x3400),
                "Alt".into(),
            ),
            (
                picker_mod_tap_label(0x2800),
                0x2800,
                Some(0x3800),
                lgui.to_string(),
            ),
            (
                picker_mod_tap_label(0x2300),
                0x2300,
                None,
                "Ctrl+Shift".into(),
            ),
            (
                picker_mod_tap_label(0x2500),
                0x2500,
                None,
                "Ctrl+Alt".into(),
            ),
            (
                picker_mod_tap_label(0x2600),
                0x2600,
                None,
                "Shift+Alt (LSA)".into(),
            ),
            (
                picker_mod_tap_label(0x2700),
                0x2700,
                None,
                "Meh (Ctrl+Shift+Alt)".into(),
            ),
            (
                picker_mod_tap_label(0x2F00),
                0x2F00,
                None,
                format!("Hyper (Ctrl+Shift+Alt+{})", gui_mod_name()),
            ),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, left_value, right_value, mod_name) in &mt {
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                Self::paint_compact_picker_label(ui, &resp, label);
                if resp.clicked_by(egui::PointerButton::Primary) {
                    self.vial_quantum_pending_mt = Some(*left_value);
                }
                if let Some(right_value) = right_value {
                    if resp.clicked_by(egui::PointerButton::Secondary) {
                        self.vial_quantum_pending_mt = Some(*right_value);
                    }
                    resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &mod_tap_tooltip(mod_name, true),
                    ));
                } else {
                    resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &mod_tap_tooltip(mod_name, false),
                    ));
                }
            }
        });

        ui.add_space(10.0);
        ui.label(
            RichText::new(tr_picker(self.language, "key_picker.section_one_shot_mod"))
                .size(11.0)
                .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let mut osm: Vec<(String, u16, Option<u16>, String)> = vec![
            ("OSM\nCtrl".into(), 0x52A1, Some(0x52B1), "Ctrl".into()),
            ("OSM\nShift".into(), 0x52A2, Some(0x52B2), "Shift".into()),
            ("OSM\nAlt".into(), 0x52A4, Some(0x52B4), "Alt".into()),
            (
                format!("OSM\n{lgui}"),
                0x52A8,
                Some(0x52B8),
                lgui.to_string(),
            ),
            (
                "OSM\nMeh".into(),
                0x52A7,
                None,
                "Meh (Ctrl+Shift+Alt)".into(),
            ),
            (
                "OSM\nHyper".into(),
                0x52AF,
                None,
                format!("Hyper (Ctrl+Shift+Alt+{})", gui_mod_name()),
            ),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, left_value, right_value, mod_name) in &osm {
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                Self::paint_compact_picker_label(ui, &resp, label);
                if resp.clicked_by(egui::PointerButton::Primary) {
                    self.assign_keycode_value(*left_value);
                }
                if let Some(right_value) = right_value {
                    if resp.clicked_by(egui::PointerButton::Secondary) {
                        self.assign_keycode_value(*right_value);
                    }
                    resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &one_shot_modifier_tooltip(mod_name, true),
                    ));
                } else {
                    resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &one_shot_modifier_tooltip(mod_name, false),
                    ));
                }
            }
        });
    }

    fn picker_value_supported(&self, value: u16) -> bool {
        let _ = value;
        true
    }

    fn picker_key_size(ctx: &egui::Context) -> Vec2 {
        responsive_picker_key_size(ctx)
    }

    fn key_grid_width(ui: &egui::Ui, cols: usize, spacing: f32) -> f32 {
        let key_size = Self::picker_key_size(ui.ctx());
        key_size.x * cols as f32 + spacing * cols.saturating_sub(1) as f32
    }

    fn slot_grid_width(cols: usize, spacing: f32) -> f32 {
        48.0 * cols as f32 + spacing * cols.saturating_sub(1) as f32
    }

    fn tab_content_width(&self, ui: &egui::Ui) -> f32 {
        let spacing = ui.spacing().item_spacing.x;
        let width = match self.selected_tab {
            KeycodeTab::Symbols | KeycodeTab::Special | KeycodeTab::Rgb | KeycodeTab::Custom => {
                Self::key_grid_width(ui, 13, spacing)
            }
            KeycodeTab::Modifiers => Self::key_grid_width(ui, 13, spacing),
            KeycodeTab::Macro | KeycodeTab::TapDance => Self::slot_grid_width(16, 4.0),
            _ => 840.0,
        };
        width.min(ui.available_width())
    }

    fn paint_compact_picker_label(ui: &egui::Ui, resp: &egui::Response, label: &str) {
        let visuals = ui.style().interact(resp);
        let painter = ui.painter();
        let dark = ui.visuals().dark_mode;
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
        let label_scale = (resp.rect.height() / 54.0).clamp(1.0, 1.22);
        let (top_size, bottom_size) = key_label_font_sizes(label);
        let top_size = top_size.map(|size| size * label_scale);
        let bottom_size = bottom_size * label_scale;
        let top_color = if dark {
            Color32::from_rgb(130, 130, 145)
        } else {
            Color32::from_rgb(130, 130, 150)
        };
        let main_color = if resp.enabled() {
            if dark {
                Color32::from_rgb(239, 233, 232)
            } else {
                Color32::from_rgb(26, 26, 30)
            }
        } else {
            visuals.fg_stroke.color
        };

        if let Some(top_str) = top {
            let center = resp.rect.center();
            painter.text(
                egui::pos2(center.x, center.y - 7.0 * label_scale),
                egui::Align2::CENTER_CENTER,
                top_str,
                egui::FontId::proportional(top_size.unwrap_or(9.0)),
                top_color,
            );
            painter.text(
                egui::pos2(center.x, center.y + 6.0 * label_scale),
                egui::Align2::CENTER_CENTER,
                bottom,
                egui::FontId::proportional(bottom_size),
                main_color,
            );
        } else {
            let font_size = if bottom == "↵" {
                16.0 * label_scale
            } else {
                bottom_size
            };
            painter.text(
                resp.rect.center(),
                egui::Align2::CENTER_CENTER,
                bottom,
                egui::FontId::proportional(font_size),
                main_color,
            );
        }
    }

    fn show_vial_special(&mut self, ui: &mut egui::Ui) {
        let special_keys: Vec<(String, u16, String)> = vec![
            (
                "✕
None"
                    .into(),
                0x0000,
                "KC_NO — disables this key completely, it sends nothing when pressed".into(),
            ),
            (
                "▽
Inherit"
                    .into(),
                0x0001,
                "KC_TRNS — inherits the key from the layer below".into(),
            ),
            (
                "Esc
~"
                .into(),
                0x7C16,
                format!(
                    "Grave/Escape — sends Esc normally, ` when Shift or {} is held",
                    gui_mod_name()
                ),
            ),
            (
                "⚡
Boot"
                    .into(),
                0x7C00,
                "QK_BOOT — put keyboard into flash mode".into(),
            ),
            (
                "🐛
Debug"
                    .into(),
                0x7C02,
                "DB_TOGG — toggle debug mode".into(),
            ),
            (
                "🔒
Lock"
                    .into(),
                0x7800,
                "QK_LOCK — hold to lock remaining keys until pressed again".into(),
            ),
            (
                "Auto
Shift"
                    .into(),
                0x7C15,
                "Toggles the state of the Auto Shift feature".into(),
            ),
            (
                "Combo
Toggle"
                    .into(),
                0x7C52,
                "Toggles Combo feature on and off".into(),
            ),
            (
                "Caps
Word"
                    .into(),
                0x7C73,
                "Capitalizes until end of current word".into(),
            ),
            (
                "Repeat".into(),
                0x7C79,
                "Repeats the last pressed key".into(),
            ),
            (
                "Alt
Repeat"
                    .into(),
                0x7C7A,
                "Alt repeats the last pressed key".into(),
            ),
        ];

        let special_title =
            crate::i18n::tr_catalog(self.language, "key_picker_text.special_qmk_keys");
        ui.label(
            RichText::new(special_title)
                .size(11.0)
                .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in &special_keys {
                if !self.picker_value_supported(*value) {
                    continue;
                }
                if let Some(kc) = crate::keycode::KEYCODES
                    .iter()
                    .find(|kc| kc.value == *value)
                {
                    if !self.vial_keycode_supported(kc) {
                        continue;
                    }
                }
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                Self::paint_compact_picker_label(ui, &resp, label);
                if resp.clicked() {
                    self.assign_keycode_value(*value);
                }
                resp.on_hover_text(crate::i18n::tr_text(self.language, &tip));
            }
        });

        let mouse_values: Vec<u16> = crate::keycode::KEYCODES
            .iter()
            .filter(|kc| matches!(kc.category, crate::keycode::KeycodeCategory::Mouse))
            .map(|kc| kc.value)
            .filter(|value| self.picker_value_supported(*value))
            .collect();

        if !self.supports_mouse_keys {
            return;
        }

        if !mouse_values.is_empty() {
            ui.add_space(10.0);
            ui.label(
                RichText::new(crate::i18n::tr_catalog(
                    self.language,
                    "key_picker_text.mouse",
                ))
                .size(11.0)
                .color(Color32::from_gray(150)),
            );
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for value in &mouse_values {
                    let resp = ui
                        .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    let label = keycode_label_with_names_and_layout(
                        *value,
                        &[],
                        &self.layer_names,
                        self.key_legend_layout,
                    );
                    Self::paint_compact_picker_label(ui, &resp, &label);
                    if resp.clicked() {
                        self.assign_keycode_value(*value);
                    }
                    if resp.hovered() {
                        resp.on_hover_text(crate::i18n::tr_text(
                            self.language,
                            &self.picker_keycode_tooltip(*value, &[]),
                        ));
                    }
                }
            });
        }

        let media_keys: &[(&str, &str, u16)] = &[
            ("⏻", "Power", 0x00A5),
            ("🌙", "Sleep", 0x00A6),
            ("☀", "Wake", 0x00A7),
            ("🔇", "Mute", 0x00A8),
            ("🔉", "Vol-", 0x00AA),
            ("🔊", "Vol+", 0x00A9),
            ("⏮", "Prev", 0x00AC),
            ("⏭", "Next", 0x00AB),
            ("⏹", "Stop", 0x00AD),
            ("⏯", "Play", 0x00AE),
            ("🎵", "Media", 0x00AF),
            ("⏏", "Eject", 0x00B0),
            ("✉", "Mail", 0x00B1),
            ("∑", "Calc", 0x00B2),
            ("📁", "Files", 0x00B3),
            ("🔍", "Search", 0x00B4),
            ("🌐", "Home", 0x00B5),
            ("←", "Back", 0x00B6),
            ("→", "Fwd", 0x00B7),
            ("⏹", "Web", 0x00B8),
            ("↻", "Reload", 0x00B9),
            ("★", "Favs", 0x00BA),
            ("⏪", "Rewind", 0x00BC),
            ("⏩", "Fast+", 0x00BB),
            ("🔅", "Bright-", 0x00BE),
            ("🔆", "Bright+", 0x00BD),
            ("🪟", "Mission", 0x00BF),
            ("🚀", "Launch", 0x00C0),
        ];
        ui.add_space(10.0);
        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.media_apps_system",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (_, _, value) in media_keys {
                if !self.picker_value_supported(*value) {
                    continue;
                }
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let label = keycode_label_with_names_and_layout(
                    *value,
                    &[],
                    &self.layer_names,
                    self.key_legend_layout,
                );
                Self::paint_compact_picker_label(ui, &resp, &label);
                if resp.clicked() {
                    self.assign_keycode_value(*value);
                }
                if resp.hovered() {
                    resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &self.picker_keycode_tooltip(*value, &[]),
                    ));
                }
            }
        });

        ui.add_space(10.0);
        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.os_edit_shortcuts",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let mut os_shortcuts: Vec<(&str, &str, u16, &str)> = Vec::new();
        #[cfg(target_os = "macos")]
        {
            os_shortcuts.extend_from_slice(&[
                ("macOS", "Undo", 0x0800 | 0x001D, "Command + Z"),
                ("macOS", "Redo", 0x0A00 | 0x001D, "Command + Shift + Z"),
                ("macOS", "Cut", 0x0800 | 0x001B, "Command + X"),
                ("macOS", "Copy", 0x0800 | 0x0006, "Command + C"),
                ("macOS", "Paste", 0x0800 | 0x0019, "Command + V"),
                ("macOS", "Find", 0x0800 | 0x0009, "Command + F"),
                (
                    "macOS",
                    "Prev\nWord",
                    0x0400 | 0x0050,
                    "Option + Left Arrow",
                ),
                (
                    "macOS",
                    "Next\nWord",
                    0x0400 | 0x004F,
                    "Option + Right Arrow",
                ),
                (
                    "macOS",
                    "Prev\nApp",
                    0x0A00 | 0x002B,
                    "Shift + Command + Tab",
                ),
                ("macOS", "Next\nApp", 0x0800 | 0x002B, "Command + Tab"),
            ]);
        }
        #[cfg(target_os = "windows")]
        {
            os_shortcuts.extend_from_slice(&[
                ("Windows", "Undo", 0x0100 | 0x001D, "Ctrl + Z"),
                ("Windows", "Redo", 0x0100 | 0x001C, "Ctrl + Y"),
                ("Windows", "Cut", 0x0100 | 0x001B, "Ctrl + X"),
                ("Windows", "Copy", 0x0100 | 0x0006, "Ctrl + C"),
                ("Windows", "Paste", 0x0100 | 0x0019, "Ctrl + V"),
                ("Windows", "Find", 0x0100 | 0x0009, "Ctrl + F"),
                (
                    "Windows",
                    "Prev\nWord",
                    0x0100 | 0x0050,
                    "Ctrl + Left Arrow",
                ),
                (
                    "Windows",
                    "Next\nWord",
                    0x0100 | 0x004F,
                    "Ctrl + Right Arrow",
                ),
                ("Windows", "Prev\nApp", 0x0600 | 0x002B, "Shift + Alt + Tab"),
                ("Windows", "Next\nApp", 0x0400 | 0x002B, "Alt + Tab"),
            ]);
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            os_shortcuts.extend_from_slice(&[
                ("Linux", "Undo", 0x0100 | 0x001D, "Ctrl + Z"),
                ("Linux", "Redo", 0x0100 | 0x001C, "Ctrl + Y"),
                ("Linux", "Cut", 0x0100 | 0x001B, "Ctrl + X"),
                ("Linux", "Copy", 0x0100 | 0x0006, "Ctrl + C"),
                ("Linux", "Paste", 0x0100 | 0x0019, "Ctrl + V"),
                ("Linux", "Find", 0x0100 | 0x0009, "Ctrl + F"),
                ("Linux", "Prev\nWord", 0x0100 | 0x0050, "Ctrl + Left Arrow"),
                ("Linux", "Next\nWord", 0x0100 | 0x004F, "Ctrl + Right Arrow"),
                ("Linux", "Prev\nApp", 0x0600 | 0x002B, "Shift + Alt + Tab"),
                ("Linux", "Next\nApp", 0x0400 | 0x002B, "Alt + Tab"),
            ]);
        }
        ui.horizontal_wrapped(|ui| {
            for (_os, text, value, tip) in os_shortcuts {
                if !self.picker_value_supported(value) {
                    continue;
                }
                let resp = ui.add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""));
                Self::paint_compact_picker_label(ui, &resp, text);
                if resp.clicked() {
                    self.assign_keycode_value(value);
                }
                resp.on_hover_text(crate::i18n::tr_text(self.language, &tip));
            }
        });

        ui.add_space(10.0);
        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.numpad",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let num_text_color = if ui.visuals().dark_mode {
                Color32::from_gray(105)
            } else {
                Color32::from_gray(145)
            };
            for kc in crate::keycode::KEYCODES
                .iter()
                .filter(|kc| matches!(kc.category, crate::keycode::KeycodeCategory::Numpad))
            {
                if !self.picker_value_supported(kc.value) {
                    continue;
                }
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
                    _ => kc
                        .label
                        .strip_prefix("Num ")
                        .or_else(|| kc.label.strip_prefix("Numpad "))
                        .or_else(|| kc.label.strip_prefix("Num"))
                        .unwrap_or(kc.label),
                };
                let font_size = if display.chars().count() > 2 {
                    10.5
                } else {
                    13.0
                };
                let mut resp = ui.add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""));
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
                if resp.clicked() {
                    self.assign_keycode_value(kc.value);
                }
                if resp.hovered() {
                    resp = resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &self.picker_keycode_tooltip(kc.value, &[]),
                    ));
                    let _ = resp;
                }
            }
        });

        let magic_keys: &[u16] = &[
            0x7000, 0x7001, 0x7002, 0x7004, 0x7003, 0x7020, 0x7021, 0x7022, 0x7017, 0x7018, 0x7019,
            0x701A, 0x701B, 0x701C, 0x701D, 0x7005, 0x7006, 0x7007, 0x7008, 0x7014, 0x7015, 0x7016,
            0x700A, 0x7009, 0x700B, 0x700C, 0x700D, 0x700E, 0x700F, 0x7010, 0x7011, 0x7012, 0x7013,
            0x701E, 0x701F,
        ];
        let visible_magic_keys: Vec<u16> = magic_keys
            .iter()
            .copied()
            .filter(|value| self.picker_value_supported(*value))
            .collect();
        if !visible_magic_keys.is_empty() {
            ui.add_space(10.0);
            ui.label(
                RichText::new(crate::i18n::tr_catalog(self.language, "ui.magic_title"))
                    .size(11.0)
                    .color(Color32::from_gray(150)),
            );
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                let magic_top_color = if ui.visuals().dark_mode {
                    Color32::from_gray(105)
                } else {
                    Color32::from_gray(145)
                };
                for value in &visible_magic_keys {
                    let label = crate::keycode::keycode_label(*value);
                    let mut parts = label.splitn(2, '\n');
                    let top = parts.next().unwrap_or("");
                    let bottom = parts.next().unwrap_or("");
                    let mut resp =
                        ui.add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""));
                    let rect = resp.rect;
                    let painter = ui.painter();
                    let main_color = if resp.hovered() {
                        ui.visuals().widgets.hovered.fg_stroke.color
                    } else {
                        ui.visuals().widgets.inactive.fg_stroke.color
                    };
                    let top_font = if top.chars().count() > 10 { 8.6 } else { 9.2 };
                    let bottom_font = if bottom.chars().count() > 8 {
                        9.4
                    } else {
                        10.2
                    };
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
                    if resp.clicked() {
                        self.assign_keycode_value(*value);
                    }
                    if resp.hovered() {
                        resp = resp.on_hover_text(crate::i18n::tr_text(
                            self.language,
                            &self.picker_keycode_tooltip(*value, &[]),
                        ));
                        let _ = resp;
                    }
                }
            });
        }

        ui.add_space(10.0);
        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.space_cadet",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let space_cadet_keys: &[(&str, &str, u16, &str)] = &[
            (
                "LCtrl",
                "(",
                0x7C18,
                "Left Control when held, ( when tapped",
            ),
            (
                "RCtrl",
                ")",
                0x7C19,
                "Right Control when held, ) when tapped",
            ),
            ("LShift", "(", 0x7C1A, "Left Shift when held, ( when tapped"),
            (
                "RShift",
                ")",
                0x7C1B,
                "Right Shift when held, ) when tapped",
            ),
            ("LAlt", "(", 0x7C1C, "Left Alt when held, ( when tapped"),
            ("RAlt", ")", 0x7C1D, "Right Alt when held, ) when tapped"),
            (
                "RShift",
                "Enter",
                0x7C1E,
                "Right Shift when held, Enter when tapped",
            ),
        ];
        ui.horizontal_wrapped(|ui| {
            let cadet_top_color = if ui.visuals().dark_mode {
                Color32::from_gray(105)
            } else {
                Color32::from_gray(145)
            };
            for (top, bottom, value, tip) in space_cadet_keys {
                if !self.picker_value_supported(*value) {
                    continue;
                }
                let mut resp = ui.add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""));
                let rect = resp.rect;
                let painter = ui.painter();
                let main_color = if resp.hovered() {
                    ui.visuals().widgets.hovered.fg_stroke.color
                } else {
                    ui.visuals().widgets.inactive.fg_stroke.color
                };
                let top_font = if top.chars().count() > 6 { 8.7 } else { 9.3 };
                let bottom_font = if bottom.chars().count() > 5 {
                    9.4
                } else {
                    10.6
                };
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
                if resp.clicked() {
                    self.assign_keycode_value(*value);
                }
                resp = resp.on_hover_text(crate::i18n::tr_text(self.language, &tip));
                let _ = resp;
            }
        });

        ui.add_space(10.0);
        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.international",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let international_keys: &[(&str, &str, u16, &str)] = &[
            ("Universal", "б", 0x0500 | 0x0068, "Universal Cyrillic б — types б consistently regardless of the active keyboard language; hold Shift for Б"),
            ("Universal", "ю", 0x0500 | 0x0069, "Universal Cyrillic ю — types ю consistently regardless of the active keyboard language; hold Shift for Ю"),
            ("Universal", "ж", 0x0500 | 0x006A, "Universal Cyrillic ж — types ж consistently regardless of the active keyboard language; hold Shift for Ж"),
            ("Universal", "э", 0x0500 | 0x006B, "Universal Cyrillic э — types э consistently regardless of the active keyboard language; hold Shift for Э"),
            ("Universal", "х", 0x0500 | 0x006C, "Universal Cyrillic х — types х consistently regardless of the active keyboard language; hold Shift for Х"),
            ("Universal", "ъ", 0x0500 | 0x006D, "Universal Cyrillic ъ — types ъ consistently regardless of the active keyboard language; hold Shift for Ъ"),
            ("Universal", "ё", 0x0500 | 0x006E, "Universal Cyrillic ё — types ё consistently regardless of the active keyboard language; hold Shift for Ё"),
            ("JIS", "\\ _", 0x0087, "JIS \\ and _"),
            ("JIS", "Kana", 0x0088, "JIS Katakana/Hiragana"),
            ("JIS", "¥ |", 0x0089, "JIS ¥ and |"),
            ("JIS", "Henkan", 0x008A, "JIS Henkan"),
            ("JIS", "Muhenk", 0x008B, "JIS Muhenkan"),
            ("JIS", "Num ,", 0x008C, "JIS Numpad ,"),
            ("Hangul", "Eng", 0x0090, "Hangul/English"),
            ("Hangul", "Hanja", 0x0091, "Hanja"),
            ("JIS", "Katak", 0x0092, "JIS Katakana"),
            ("JIS", "Hirag", 0x0093, "JIS Hiragana"),
            ("JIS", "ZenHan", 0x0094, "JIS Zenkaku/Hankaku"),
        ];
        ui.horizontal_wrapped(|ui| {
            let intl_top_color = if ui.visuals().dark_mode {
                Color32::from_gray(105)
            } else {
                Color32::from_gray(145)
            };
            for (top, bottom, value, tip) in international_keys {
                let mut resp = ui.add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""));
                let rect = resp.rect;
                let painter = ui.painter();
                let main_color = if resp.hovered() {
                    ui.visuals().widgets.hovered.fg_stroke.color
                } else {
                    ui.visuals().widgets.inactive.fg_stroke.color
                };
                let top_font = if top.chars().count() > 6 { 8.5 } else { 9.2 };
                let bottom_font = if bottom.chars().count() > 6 {
                    9.0
                } else {
                    10.2
                };
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
                if resp.clicked() {
                    self.assign_keycode_value(*value);
                }
                resp = resp.on_hover_text(crate::i18n::tr_text(self.language, &tip));
                let _ = resp;
            }
        });
    }
}
