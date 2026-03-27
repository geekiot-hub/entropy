/// ORYX-style keycode picker modal.

use crate::keycode::{KeycodeCategory, KEYCODES};
use egui::{Color32, Key, RichText, Vec2};

pub struct KeycodePicker {
    pub open: bool,
    pub selected_tab: KeycodeTab,
    pub search_query: String,
    pub result: Option<u16>,
    /// Custom keycodes: (name, label, value starting at 0x7E40)
    pub custom_keycodes: Vec<(String, String, u16)>,
    /// Listen mode: waiting for physical key press
    pub listening: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeycodeTab {
    Basic,
    Modifiers,
    Function,
    Navigation,
    Numpad,
    Media,
    Mouse,
    Layers,
    Special,
    Custom,
}

impl KeycodeTab {
    pub const ALL: &'static [KeycodeTab] = &[
        KeycodeTab::Basic,
        KeycodeTab::Modifiers,
        KeycodeTab::Function,
        KeycodeTab::Navigation,
        KeycodeTab::Numpad,
        KeycodeTab::Media,
        KeycodeTab::Mouse,
        KeycodeTab::Layers,
        KeycodeTab::Special,
        KeycodeTab::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            KeycodeTab::Basic      => "Basic",
            KeycodeTab::Modifiers  => "Mods",
            KeycodeTab::Function   => "Fn",
            KeycodeTab::Navigation => "Nav",
            KeycodeTab::Numpad     => "Num",
            KeycodeTab::Media      => "Media",
            KeycodeTab::Mouse      => "Mouse",
            KeycodeTab::Layers     => "Layers",
            KeycodeTab::Special    => "Special",
            KeycodeTab::Custom     => "Custom",
        }
    }

    fn matches_category(self, cat: &KeycodeCategory) -> bool {
        matches!(
            (self, cat),
            (KeycodeTab::Basic,      KeycodeCategory::Basic)
            | (KeycodeTab::Modifiers,  KeycodeCategory::Modifier)
            | (KeycodeTab::Function,   KeycodeCategory::Function)
            | (KeycodeTab::Navigation, KeycodeCategory::Navigation)
            | (KeycodeTab::Numpad,     KeycodeCategory::Numpad)
            | (KeycodeTab::Media,      KeycodeCategory::Media)
            | (KeycodeTab::Mouse,      KeycodeCategory::Mouse)
            | (KeycodeTab::Layers,     KeycodeCategory::Layer)
            | (KeycodeTab::Special,    KeycodeCategory::Special)
        )
    }
}

impl Default for KeycodePicker {
    fn default() -> Self {
        Self {
            open: false,
            selected_tab: KeycodeTab::Basic,
            search_query: String::new(),
            result: None,
            custom_keycodes: vec![],
            listening: false,
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
        Key::Enter => 0x28,
        Key::Escape => 0x29,
        Key::Backspace => 0x2A,
        Key::Tab => 0x2B,
        Key::Space => 0x2C,
        Key::Minus => 0x2D,
        Key::Equals => 0x2E,
        Key::OpenBracket => 0x2F,
        Key::CloseBracket => 0x30,
        Key::Backslash => 0x31,
        Key::Semicolon => 0x33,
        Key::Quote => 0x34,
        Key::Backtick => 0x35,
        Key::Comma => 0x36,
        Key::Period => 0x37,
        Key::Slash => 0x38,
        Key::F1 => 0x3A, Key::F2 => 0x3B, Key::F3 => 0x3C, Key::F4 => 0x3D,
        Key::F5 => 0x3E, Key::F6 => 0x3F, Key::F7 => 0x40, Key::F8 => 0x41,
        Key::F9 => 0x42, Key::F10 => 0x43, Key::F11 => 0x44, Key::F12 => 0x45,
        Key::Insert => 0x49, Key::Home => 0x4A, Key::PageUp => 0x4B,
        Key::Delete => 0x4C, Key::End => 0x4D, Key::PageDown => 0x4E,
        Key::ArrowRight => 0x4F, Key::ArrowLeft => 0x50,
        Key::ArrowDown => 0x51, Key::ArrowUp => 0x52,
        _ => return None,
    };

    // Apply modifiers
    let mut mod_mask: u16 = 0;
    if mods.ctrl  { mod_mask |= 0x0100; }
    if mods.shift { mod_mask |= 0x0200; }
    if mods.alt   { mod_mask |= 0x0400; }
    if mods.mac_cmd || mods.command { mod_mask |= 0x0800; }

    if mod_mask != 0 {
        Some(mod_mask | base)
    } else {
        Some(base)
    }
}

impl KeycodePicker {
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        // Always capture physical key presses when picker is open
        // (unless search bar is focused — text input takes priority)
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    // Skip if typing in search box (modifier-free letters)
                    if self.search_query.is_empty() || modifiers.any() {
                        if let Some(qmk) = egui_key_to_qmk(*key, *modifiers) {
                            self.result = Some(qmk);
                            self.open = false;
                            self.listening = false;
                        }
                    }
                }
            }
        });

        let mut still_open = true;
        egui::Window::new("Pick Keycode")
            .open(&mut still_open)
            .collapsible(false)
            .resizable(true)
            .min_size(Vec2::new(600.0, 420.0))
            .show(ctx, |ui| {
                // Search bar + hint
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut self.search_query);
                    if ui.button("x").clicked() {
                        self.search_query.clear();
                    }
                    ui.separator();
                    ui.label(RichText::new("or just press a key").size(11.0).color(Color32::from_gray(140)));
                });
                ui.separator();

                // Tabs
                ui.horizontal_wrapped(|ui| {
                    for tab in KeycodeTab::ALL {
                        // Hide Custom tab if no custom keycodes
                        if *tab == KeycodeTab::Custom && self.custom_keycodes.is_empty() {
                            continue;
                        }
                        let active = self.selected_tab == *tab;
                        let btn = egui::Button::new(tab.label()).fill(
                            if active { Color32::from_rgb(80, 120, 200) } else { Color32::TRANSPARENT },
                        );
                        if ui.add(btn).clicked() {
                            self.selected_tab = *tab;
                        }
                    }
                });
                ui.separator();

                let search_lower = self.search_query.to_lowercase();
                let searching = !search_lower.is_empty();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        // Standard keycodes
                        if self.selected_tab != KeycodeTab::Custom {
                            for kc in KEYCODES.iter() {
                                let matches = if searching {
                                    kc.name.to_lowercase().contains(&search_lower)
                                        || kc.label.to_lowercase().contains(&search_lower)
                                } else {
                                    self.selected_tab.matches_category(&kc.category)
                                };
                                if !matches { continue; }

                                let resp = ui.add(
                                    egui::Button::new(RichText::new(kc.label).size(11.0))
                                        .min_size(Vec2::new(52.0, 36.0)),
                                );
                                if resp.clicked() {
                                    self.result = Some(kc.value);
                                    self.open = false;
                                }
                                resp.on_hover_text(format!("{} (0x{:04X})", kc.name, kc.value));
                            }
                        }

                        // Custom keycodes
                        if self.selected_tab == KeycodeTab::Custom || searching {
                            for (name, label, value) in &self.custom_keycodes {
                                if label.is_empty() { continue; }
                                if searching && !name.to_lowercase().contains(&search_lower)
                                    && !label.to_lowercase().contains(&search_lower) {
                                    continue;
                                }

                                let resp = ui.add(
                                    egui::Button::new(RichText::new(label).size(11.0))
                                        .min_size(Vec2::new(52.0, 36.0)),
                                );
                                if resp.clicked() {
                                    self.result = Some(*value);
                                    self.open = false;
                                }
                                resp.on_hover_text(format!("{} (0x{:04X})", name, value));
                            }
                        }
                    });
                });
            });

        if !still_open {
            self.open = false;
        }
    }
}
