/// Keycode picker modal — organized by category

use crate::keycode::{keycode_tooltip, KeycodeCategory, KEYCODES};
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
                    // Escape always closes picker without assigning anything
                    if *key == Key::Escape {
                        self.open = false;
                        self.listening = false;
                        return;
                    }
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
                    KeycodeTab::Layers     => "MO — hold to activate, release to return  •  TG — tap to toggle on/off  •  OSL — active for next keypress only  •  TT — hold=MO, tap=toggle  •  TO — switch and stay  •  DF — set as permanent base layer",
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
                            let layer_name = |n: u16| -> String {
                                match self.layer_names.get(n as usize) {
                                    Some(s) if !s.is_empty() && s != &n.to_string()
                                        => format!("{}. {}", n, s),
                                    _ => n.to_string(),
                                }
                            };
                            let active_layers: u16 = 16;

                            // ops: (base value, column header, tooltip desc)
                            let ops: &[(u16, &str, &str)] = &[
                                (0x5220, "Hold\n(MO)",      "Hold to activate, release to return"),
                                (0x5260, "Toggle\n(TG)",    "Tap to toggle on/off"),
                                (0x5280, "One-Shot\n(OSL)", "Active for next keypress only"),
                                (0x52C0, "Tap-Tog\n(TT)",   "Hold = MO, tap repeatedly = toggle"),
                                (0x5200, "Switch\n(TO)",    "Switch and stay on this layer"),
                                (0x5240, "Default\n(DF)",   "Set as permanent base layer"),
                            ];

                            // Grid: columns = ops, rows = layers
                            let col_w = 80.0_f32;
                            let row_h = 32.0_f32;
                            let dark = ui.visuals().dark_mode;

                            egui::Grid::new("layers_grid")
                                .spacing([4.0, 2.0])
                                .show(ui, |ui| {
                                    // Header row
                                    ui.label(""); // row label column
                                    for (_, header, hint) in ops {
                                        ui.add(egui::Label::new(
                                            RichText::new(*header).size(10.0).strong()
                                        ).sense(egui::Sense::hover()))
                                        .on_hover_text(*hint);
                                    }
                                    ui.end_row();

                                    // One row per layer
                                    for n in 0..active_layers {
                                        let raw = self.layer_names.get(n as usize).cloned().unwrap_or(n.to_string());
                                        let is_named = !raw.is_empty() && raw != n.to_string();

                                        // Alternating row background
                                        let row_bg = if n % 2 == 0 {
                                            if dark { Color32::from_rgba_premultiplied(255,255,255,6) }
                                            else     { Color32::from_rgba_premultiplied(0,0,0,8) }
                                        } else {
                                            Color32::TRANSPARENT
                                        };

                                        // Row label — just the name (number is shown on the buttons)
                                        let label_text = if is_named {
                                            raw.clone()
                                        } else {
                                            format!("Layer {}", n)
                                        };
                                        let label_color = if is_named {
                                            Color32::from_rgb(91, 104, 223)
                                        } else if dark {
                                            Color32::from_gray(110)
                                        } else {
                                            Color32::from_gray(160)
                                        };
                                        ui.label(RichText::new(&label_text).size(11.5).color(label_color)
                                            .strong());

                                        for (base, header, _) in ops {
                                            let value = base + n;
                                            let tip = keycode_tooltip(value, &[], &self.layer_names);
                                            // Extract op name: "Hold\n(MO)" → "MO"
                                            let op_short = header.split('\n').last()
                                                .unwrap_or("")
                                                .trim_matches(|c| c == '(' || c == ')');
                                            let btn_text = format!("{}({})", op_short, n);
                                            let btn_color = if is_named {
                                                if dark { Color32::from_gray(220) } else { Color32::from_gray(30) }
                                            } else {
                                                if dark { Color32::from_gray(80) } else { Color32::from_gray(190) }
                                            };
                                            let fill = if is_named {
                                                if dark { Color32::from_rgb(38, 43, 88) } else { Color32::from_rgb(224, 227, 249) }
                                            } else {
                                                row_bg
                                            };
                                            let resp = ui.add(
                                                egui::Button::new(
                                                    RichText::new(&btn_text).size(10.5).color(btn_color)
                                                )
                                                .fill(fill)
                                                .min_size(Vec2::new(col_w, row_h))
                                            );
                                            if resp.clicked() { self.result = Some(value); self.open = false; }
                                            resp.on_hover_text(tip);
                                        }
                                        ui.end_row();
                                    }
                                });
                        } else if self.selected_tab != KeycodeTab::Custom {
                            let custom_pairs: Vec<(String, String)> = self.custom_keycodes.iter()
                                .map(|(n, l, _)| (n.clone(), l.clone()))
                                .collect();
                            for kc in KEYCODES.iter() {
                                if !self.selected_tab.matches(kc) { continue; }
                                let tip = keycode_tooltip(kc.value, &custom_pairs, &self.layer_names);
                                let resp = ui.add(
                                    egui::Button::new(RichText::new(kc.label).size(11.0))
                                        .min_size(Vec2::new(52.0, 38.0)),
                                );
                                if resp.clicked() {
                                    self.result = Some(kc.value);
                                    self.open = false;
                                }
                                resp.on_hover_text(tip);
                            }
                        } else {
                            for (name, label, value) in &self.custom_keycodes {
                                if label.is_empty() { continue; }
                                let tip = format!("Custom key: {} ({})", label, name);
                                let resp = ui.add(
                                    egui::Button::new(RichText::new(label).size(11.0))
                                        .min_size(Vec2::new(52.0, 38.0)),
                                );
                                if resp.clicked() {
                                    self.result = Some(*value);
                                    self.open = false;
                                }
                                resp.on_hover_text(tip);
                            }
                        }
                    });
                });
            });

        if !still_open { self.open = false; }
    }
}
