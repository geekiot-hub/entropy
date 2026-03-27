/// Keycode picker modal — organized by category

use crate::keycode::{KeycodeCategory, KEYCODES};
use egui::{Color32, Key, RichText, Vec2};

pub struct KeycodePicker {
    pub open: bool,
    pub selected_tab: KeycodeTab,
    pub search_query: String,
    pub result: Option<u16>,
    pub custom_keycodes: Vec<(String, String, u16)>,
    pub layer_names: Vec<String>,
    pub listening: bool,
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
    Custom,
}

impl KeycodeTab {
    pub const ALL: &'static [KeycodeTab] = &[
        KeycodeTab::Basic,
        KeycodeTab::Symbols,
        KeycodeTab::Function,
        KeycodeTab::Navigation,
        KeycodeTab::Modifiers,
        KeycodeTab::Layers,
        KeycodeTab::Media,
        KeycodeTab::Mouse,
        KeycodeTab::Numpad,
        KeycodeTab::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            KeycodeTab::Basic      => "Basic",
            KeycodeTab::Symbols    => "Symbols",
            KeycodeTab::Function   => "F1-F24",
            KeycodeTab::Navigation => "Nav",
            KeycodeTab::Modifiers  => "Mods",
            KeycodeTab::Layers     => "Layers",
            KeycodeTab::Media      => "Media",
            KeycodeTab::Mouse      => "Mouse",
            KeycodeTab::Numpad     => "Numpad",
            KeycodeTab::Custom     => "Custom",
        }
    }

    fn matches(&self, kc: &crate::keycode::Keycode) -> bool {
        match self {
            KeycodeTab::Basic => matches!(kc.category, KeycodeCategory::Basic)
                && !is_symbol(kc.value),
            KeycodeTab::Symbols => matches!(kc.category, KeycodeCategory::Basic)
                && is_symbol(kc.value),
            KeycodeTab::Function   => matches!(kc.category, KeycodeCategory::Function),
            KeycodeTab::Navigation => matches!(kc.category, KeycodeCategory::Navigation),
            KeycodeTab::Modifiers  => matches!(kc.category, KeycodeCategory::Modifier),
            KeycodeTab::Layers     => matches!(kc.category, KeycodeCategory::Layer),
            KeycodeTab::Media      => matches!(kc.category, KeycodeCategory::Media),
            KeycodeTab::Mouse      => matches!(kc.category, KeycodeCategory::Mouse),
            KeycodeTab::Numpad     => matches!(kc.category, KeycodeCategory::Numpad),
            KeycodeTab::Custom     => false, // handled separately
        }
    }
}

/// Symbols: punctuation + shifted symbols
fn is_symbol(value: u16) -> bool {
    matches!(value,
        0x002D..=0x0038 | // - = [ ] \ ; ' ` , . /
        0x0032 |          // NONUS_HASH
        0x0064 |          // NONUS_BSLASH
        0x021E..=0x0238   // ! @ # $ % ^ & * ( ) _ + { } | : " ~ < > ?
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

impl KeycodePicker {
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open { return; }

        // Capture physical key presses
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
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
        egui::Window::new("Keycode")
            .open(&mut still_open)
            .collapsible(false)
            .resizable(true)
            .min_size(Vec2::new(580.0, 400.0))
            .show(ctx, |ui| {
                // Hint
                ui.label(
                    RichText::new("Press a key on your keyboard, or pick below")
                        .size(11.0)
                        .color(Color32::from_gray(140))
                );
                ui.add_space(4.0);

                // Tab bar
                ui.horizontal_wrapped(|ui| {
                    for tab in KeycodeTab::ALL {
                        if *tab == KeycodeTab::Custom && self.custom_keycodes.is_empty() {
                            continue;
                        }
                        let active = self.selected_tab == *tab;
                        let btn = egui::Button::new(
                            RichText::new(tab.label()).size(12.0)
                        ).fill(
                            if active { Color32::from_rgb(91, 104, 223) } else { Color32::TRANSPARENT }
                        );
                        if ui.add(btn).clicked() {
                            self.selected_tab = *tab;
                        }
                    }
                });
                ui.separator();

                // Description for current tab
                let desc = match self.selected_tab {
                    KeycodeTab::Basic      => "Letters, numbers, Enter, Esc, Tab, Space, Backspace, Caps Lock",
                    KeycodeTab::Symbols    => "Punctuation and symbols: - = [ ] \\ ; ' ` , . /  and  ! @ # $ % ^ & * ( )",
                    KeycodeTab::Function   => "Function keys F1–F24",
                    KeycodeTab::Navigation => "Arrows, Home, End, Page Up/Down, Insert, Delete, Print Screen",
                    KeycodeTab::Modifiers  => "Ctrl, Shift, Alt, Gui (Win/Cmd), one-shot modifiers (OSM)",
                    KeycodeTab::Layers     => "Layer operations: MO (momentary), TG (toggle), TO, OSL, TT, DF",
                    KeycodeTab::Media      => "Volume, brightness, play/pause, media controls, browser keys",
                    KeycodeTab::Mouse      => "Mouse movement, buttons MB1–MB5, scroll wheel",
                    KeycodeTab::Numpad     => "Numpad 0–9, operators, Enter, dot, comma",
                    KeycodeTab::Custom     => "Ergohaven custom keycodes (RuEn, trackball modes, etc.)",
                };
                ui.label(RichText::new(desc).size(10.5).color(Color32::from_gray(150)));
                ui.add_space(6.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        if self.selected_tab == KeycodeTab::Layers {
                            // Dynamic layer keys with names
                            let layer_name = |n: u16| -> String {
                                match self.layer_names.get(n as usize) {
                                    Some(s) if !s.is_empty() && s != &n.to_string()
                                        => format!("{}. {}", n, s),
                                    _ => n.to_string(),
                                }
                            };
                            let ops: &[(&str, u16, &str)] = &[
                                ("MO", 0x5220, "Momentary"),
                                ("TG", 0x5260, "Toggle"),
                                ("TO", 0x5210, "Turn On"),
                                ("OSL", 0x5280, "One-Shot"),
                                ("TT", 0x52C0, "Tap-Toggle"),
                                ("DF", 0x5240, "Default"),
                            ];
                            for (op, base, desc) in ops {
                                ui.label(RichText::new(format!("── {} ({}) ──", op, desc)).size(10.0).color(Color32::from_gray(150)));
                                ui.end_row();
                                for n in 0..16u16 {
                                    let value = base + n;
                                    let lname = layer_name(n);
                                    let label = format!("{}\n{}", op, lname);
                                    let resp = ui.add(
                                        egui::Button::new(RichText::new(&label).size(10.0))
                                            .min_size(Vec2::new(52.0, 38.0)),
                                    );
                                    if resp.clicked() { self.result = Some(value); self.open = false; }
                                    resp.on_hover_text(format!("{}({}) — 0x{:04X}", op, n, value));
                                }
                                ui.add_space(4.0);
                            }
                        } else if self.selected_tab != KeycodeTab::Custom {
                            for kc in KEYCODES.iter() {
                                if !self.selected_tab.matches(kc) { continue; }
                                let resp = ui.add(
                                    egui::Button::new(RichText::new(kc.label).size(11.0))
                                        .min_size(Vec2::new(52.0, 38.0)),
                                );
                                if resp.clicked() {
                                    self.result = Some(kc.value);
                                    self.open = false;
                                }
                                resp.on_hover_text(format!("{}\n0x{:04X}", kc.name, kc.value));
                            }
                        } else {
                            for (name, label, value) in &self.custom_keycodes {
                                if label.is_empty() { continue; }
                                let resp = ui.add(
                                    egui::Button::new(RichText::new(label).size(11.0))
                                        .min_size(Vec2::new(52.0, 38.0)),
                                );
                                if resp.clicked() {
                                    self.result = Some(*value);
                                    self.open = false;
                                }
                                resp.on_hover_text(format!("{}\n0x{:04X}", name, value));
                            }
                        }
                    });
                });
            });

        if !still_open { self.open = false; }
    }
}
