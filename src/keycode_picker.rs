/// Keycode picker modal — supports both Vial (QMK keycodes) and ZMK (behaviors).

use crate::firmware::FirmwareProtocol;
use crate::keycode::{gui_label, gui_sym, keycode_tooltip, KeycodeCategory, KEYCODES};
use crate::zmk::{BehaviorInfo, ZmkBinding};
use egui::{Color32, Key, RichText, Vec2};

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
        KeycodeTab::Mouse,
        KeycodeTab::Numpad,
        KeycodeTab::Special,
        KeycodeTab::Rgb,
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
            KeycodeTab::Media       => "Media",
            KeycodeTab::Mouse       => "Mouse",
            KeycodeTab::Numpad      => "Numpad",
            KeycodeTab::Special     => "Special",
            KeycodeTab::Rgb         => "RGB",
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
            KeycodeTab::Media      => matches!(kc.category, KeycodeCategory::Media),
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

        match self.firmware {
            FirmwareProtocol::Vial => self.show_vial(ctx),
            FirmwareProtocol::Zmk  => self.show_zmk(ctx),
        }
    }

    // ─────────────────────────── VIAL PICKER ────────────────────────────────

    fn show_vial(&mut self, ctx: &egui::Context) {
        // Physical key capture
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

        let mut still_open = true;
        egui::Window::new("Keycode")
            .open(&mut still_open)
            .collapsible(false)
            .resizable(true)
            .min_size(Vec2::new(640.0, 420.0))
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
                        KeycodeTab::Special  => self.show_vial_special(ui),
                        KeycodeTab::Custom   => self.show_vial_custom(ui),
                        _ => self.show_vial_generic(ui),
                    }
                });
            });

        if !still_open { self.open = false; }

        // Second-step picker for pending mod/MT
        let pending = self.vial_quantum_pending_mod.or(self.vial_quantum_pending_mt);
        let is_mt = self.vial_quantum_pending_mod.is_none() && self.vial_quantum_pending_mt.is_some();
        if let Some(base) = pending {
            let title = if is_mt { "Pick tap key (hold = modifier)" } else { "Pick key for modifier combo" };
            let mut pending_open = true;
            egui::Window::new(title)
                .open(&mut pending_open)
                .collapsible(false)
                .resizable(false)
                .min_size(Vec2::new(480.0, 200.0))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Press a key, or click below. Esc to cancel.").size(11.0).color(Color32::from_gray(140)));
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
                            resp.on_hover_text(kc.name);
                        }
                    });
                });
            if !pending_open {
                self.vial_quantum_pending_mod = None;
                self.vial_quantum_pending_mt = None;
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
            (0x5220, "Hold\n(MO)",      "Hold to activate, release to return"),
            (0x5260, "Toggle\n(TG)",    "Tap to toggle on/off"),
            (0x5280, "One-Shot\n(OSL)", "Active for next keypress only"),
            (0x52C0, "Tap-Tog\n(TT)",   "Hold = MO, tap = toggle"),
            (0x5200, "Switch\n(TO)",    "Switch and stay on this layer"),
            (0x5240, "Default\n(DF)",   "Set as permanent base layer"),
        ];
        let col_w = 80.0_f32;
        let row_h = 32.0_f32;
        let dark = ui.visuals().dark_mode;

        egui::Grid::new("vial_layers_grid").spacing([4.0, 2.0]).show(ui, |ui| {
            ui.label("");
            for (_, header, hint) in ops {
                ui.add(egui::Label::new(RichText::new(*header).size(10.0).strong())
                    .sense(egui::Sense::hover())).on_hover_text(*hint);
            }
            ui.end_row();

            for n in 0u16..16 {
                let raw = self.layer_names.get(n as usize).cloned().unwrap_or(n.to_string());
                let is_named = !raw.is_empty() && raw != n.to_string();
                let row_bg = if n % 2 == 0 {
                    if dark { Color32::from_rgba_premultiplied(255,255,255,6) }
                    else    { Color32::from_rgba_premultiplied(0,0,0,8) }
                } else { Color32::TRANSPARENT };

                let label_color = if is_named {
                    Color32::from_rgb(91, 104, 223)
                } else if dark { Color32::from_gray(110) } else { Color32::from_gray(160) };
                ui.label(RichText::new(if is_named { raw.clone() } else { format!("Layer {}", n) })
                    .size(11.5).color(label_color).strong());

                for (base, header, _) in ops {
                    let value = base + n;
                    let tip = keycode_tooltip(value, &[], &self.layer_names);
                    let op_short = header.split('\n').last().unwrap_or("")
                        .trim_matches(|c| c == '(' || c == ')');
                    let btn_text = format!("{}({})", op_short, n);
                    let btn_color = if is_named {
                        if dark { Color32::from_gray(220) } else { Color32::from_gray(30) }
                    } else {
                        if dark { Color32::from_gray(80) } else { Color32::from_gray(190) }
                    };
                    let fill = if is_named {
                        if dark { Color32::from_rgb(38, 43, 88) } else { Color32::from_rgb(224, 227, 249) }
                    } else { row_bg };
                    let resp = ui.add(
                        egui::Button::new(RichText::new(&btn_text).size(10.5).color(btn_color))
                            .fill(fill).min_size(Vec2::new(col_w, row_h))
                    );
                    if resp.clicked() { self.result = Some(value); self.open = false; }
                    resp.on_hover_text(tip);
                }
                ui.end_row();
            }
        });
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

    fn show_vial_rgb(&mut self, ui: &mut egui::Ui) {
        let rgb_keys: &[(&str, u16, &str)] = &[
            ("RGB\nTog",  0x7A00, "RGB Toggle on/off"),
            ("RGB\nMod+", 0x7A01, "RGB Next mode"),
            ("RGB\nMod-", 0x7A02, "RGB Previous mode"),
            ("RGB\nHue+", 0x7A03, "RGB Hue increase"),
            ("RGB\nHue-", 0x7A04, "RGB Hue decrease"),
            ("RGB\nSat+", 0x7A05, "RGB Saturation increase"),
            ("RGB\nSat-", 0x7A06, "RGB Saturation decrease"),
            ("RGB\nBri+", 0x7A07, "RGB Brightness increase"),
            ("RGB\nBri-", 0x7A08, "RGB Brightness decrease"),
            ("RGB\nSpd+", 0x7A09, "RGB Speed increase"),
            ("RGB\nSpd-", 0x7A0A, "RGB Speed decrease"),
        ];
        ui.label(RichText::new("RGB lighting controls").size(11.0).color(Color32::from_gray(150)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in rgb_keys {
                let resp = ui.add(egui::Button::new(RichText::new(*label).size(10.5))
                    .min_size(Vec2::new(56.0, 42.0)));
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
