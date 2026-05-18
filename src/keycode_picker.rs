/// Keycode picker modal for Vial/QMK keycodes.

fn inactive_picker_entry_text(dark: bool) -> egui::Color32 {
    if dark {
        egui::Color32::from_gray(55)
    } else {
        egui::Color32::from_gray(175)
    }
}

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

fn popup_group_i18n_key(title: &str) -> Option<&'static str> {
    match title {
        "Letters" => Some("key_picker.group_letters"),
        "Numbers" => Some("key_picker.group_numbers"),
        "Symbols" => Some("key_picker.group_symbols"),
        "Editing" => Some("key_picker.group_editing"),
        "Navigation" => Some("key_picker.group_navigation"),
        "Function keys" => Some("key_picker.group_function_keys"),
        "Modifiers" => Some("key_picker.group_modifiers"),
        "Other keys" => Some("key_picker.group_other_keys"),
        _ => None,
    }
}

fn popup_group_title(language: crate::i18n::Language, title: &'static str) -> &'static str {
    popup_group_i18n_key(title)
        .map(|key| tr_picker(language, key))
        .unwrap_or_else(|| crate::i18n::tr_catalog(language, title))
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

fn apply_picker_button_visuals(ui: &mut egui::Ui) {
    let dark_mode = ui.visuals().dark_mode;
    let visuals = ui.visuals_mut();
    let key_fill = if dark_mode {
        Color32::from_rgb(48, 48, 52)
    } else {
        Color32::from_rgb(255, 255, 255)
    };
    visuals.widgets.inactive.bg_fill = key_fill;
    visuals.widgets.inactive.weak_bg_fill = key_fill;
    let picker_hover_fill = crate::ui_style::hover_fill(dark_mode);
    visuals.widgets.hovered.bg_fill = picker_hover_fill;
    visuals.widgets.hovered.weak_bg_fill = picker_hover_fill;
    visuals.widgets.active.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.active.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.open.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.open.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.open.corner_radius = egui::CornerRadius::same(6);
    if dark_mode {
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(54, 54, 58));
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(54, 54, 58));
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, crate::ui_style::accent());
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, crate::ui_style::accent());
    } else {
        visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0, Color32::from_rgb(230, 230, 233));
        visuals.widgets.hovered.bg_stroke =
            egui::Stroke::new(1.0, Color32::from_rgb(230, 230, 233));
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, crate::ui_style::accent());
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, crate::ui_style::accent());
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

fn is_f13_to_f24(value: u16) -> bool {
    (0x0068..=0x0073).contains(&value)
}

fn popup_key_button_label(
    kc: &crate::keycode::Keycode,
    layer_names: &[String],
    friendly_mods: bool,
    key_legend_layout: KeyLegendLayout,
) -> String {
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
    keycode_label_with_names_and_layout(kc.value, &[], layer_names, key_legend_layout)
}

fn picker_shifted_number_label(value: u16, key_legend_layout: KeyLegendLayout) -> Option<String> {
    let (digit, english, russian) = match value {
        0x001E => ("1", "!", "!"),
        0x001F => ("2", "@", "\""),
        0x0020 => ("3", "#", "№"),
        0x0021 => ("4", "$", ";"),
        0x0022 => ("5", "%", "%"),
        0x0023 => ("6", "^", ":"),
        0x0024 => ("7", "&", "?"),
        0x0025 => ("8", "*", "*"),
        0x0026 => ("9", "(", "("),
        0x0027 => ("0", ")", ")"),
        _ => return None,
    };
    let shifted = match key_legend_layout {
        KeyLegendLayout::English => english.to_string(),
        KeyLegendLayout::Russian if english == russian => english.to_string(),
        KeyLegendLayout::Russian => format!("{}  {}", english, russian),
        KeyLegendLayout::RussianPrimary if english == russian => russian.to_string(),
        KeyLegendLayout::RussianPrimary => format!("{}  {}", russian, english),
    };
    Some(format!("{}\n{}", shifted, digit))
}

fn popup_key_button_size(ui: &egui::Ui, _label: &str) -> Vec2 {
    responsive_picker_key_size(ui.ctx())
}

fn picker_keycap_button_in_rect(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    label: &str,
    enabled: bool,
    active: bool,
) -> egui::Response {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let resp = ui.allocate_rect(rect, sense);
    let dark = ui.visuals().dark_mode;
    let hovered = enabled && resp.hovered();
    let pressed = enabled && resp.is_pointer_button_down_on();
    let stroke = crate::ui_style::modal_outline_stroke(dark);
    let fill = if active {
        crate::ui_style::accent()
    } else if pressed {
        if dark {
            Color32::from_rgb(56, 56, 59)
        } else {
            Color32::from_rgb(232, 232, 235)
        }
    } else if hovered {
        crate::ui_style::hover_fill(dark)
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect(rect, 9.0, fill, stroke, egui::StrokeKind::Inside);

    let text_color = if active {
        Color32::WHITE
    } else if enabled {
        ui.visuals().text_color()
    } else {
        crate::ui_style::muted_text(dark)
    };
    let label_scale = (rect.height() / 54.0).clamp(1.0, 1.22);
    let (top_size, bottom_size) = key_label_font_sizes(label);
    let top_size = top_size.map(|size| size * label_scale);
    let bottom_size = bottom_size * label_scale;
    if let Some((top, bottom)) = label.split_once('\n') {
        let top_color = text_color.gamma_multiply(0.75);
        let top_galley = ui.painter().layout_no_wrap(
            top.to_owned(),
            egui::FontId::proportional(top_size.unwrap_or(9.0)),
            top_color,
        );
        let bottom_galley = ui.painter().layout_no_wrap(
            bottom.to_owned(),
            egui::FontId::proportional(bottom_size),
            text_color,
        );
        ui.painter().galley(
            egui::pos2(
                rect.center().x - top_galley.size().x / 2.0,
                rect.center().y - 7.0 * label_scale - top_galley.size().y / 2.0,
            ),
            top_galley,
            top_color,
        );
        ui.painter().galley(
            egui::pos2(
                rect.center().x - bottom_galley.size().x / 2.0,
                rect.center().y + 6.0 * label_scale - bottom_galley.size().y / 2.0,
            ),
            bottom_galley,
            text_color,
        );
    } else {
        let galley = ui.painter().layout_no_wrap(
            label.to_owned(),
            egui::FontId::proportional(bottom_size),
            text_color,
        );
        ui.painter().galley(
            egui::pos2(
                rect.center().x - galley.size().x / 2.0,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            text_color,
        );
    }
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

fn picker_keycap_button(
    ui: &mut egui::Ui,
    label: &str,
    size: Vec2,
    enabled: bool,
    active: bool,
) -> egui::Response {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    picker_keycap_button_in_rect(ui, rect, label, enabled, active)
}

const KEY_PICKER_POPUP_WIDTH: f32 = 760.0;
const KEY_PICKER_POPUP_HEIGHT: f32 = 560.0;
const KEY_PICKER_SCROLL_HEIGHT: f32 = 430.0;
const KEY_PICKER_MAIN_WIDTH: f32 = 920.0;
const KEY_PICKER_MAIN_HEIGHT: f32 = 560.0;
const KEY_PICKER_MAIN_CONTENT_HEIGHT: f32 = 455.0;

fn responsive_window_size(ctx: &egui::Context, base: Vec2, max: Vec2) -> Vec2 {
    let screen = ctx.screen_rect().size();
    Vec2::new(
        base.x.max((screen.x * 0.82).min(max.x)),
        base.y.max((screen.y * 0.78).min(max.y)),
    )
}

fn key_picker_main_size(ctx: &egui::Context) -> Vec2 {
    responsive_window_size(
        ctx,
        Vec2::new(KEY_PICKER_MAIN_WIDTH, KEY_PICKER_MAIN_HEIGHT),
        Vec2::new(1_260.0, 820.0),
    )
}

fn key_picker_main_content_height(picker_size: Vec2) -> f32 {
    (picker_size.y - 105.0).clamp(KEY_PICKER_MAIN_CONTENT_HEIGHT, 700.0)
}

fn key_picker_popup_size(ctx: &egui::Context) -> Vec2 {
    responsive_window_size(
        ctx,
        Vec2::new(KEY_PICKER_POPUP_WIDTH, KEY_PICKER_POPUP_HEIGHT),
        Vec2::new(1_050.0, 760.0),
    )
}

fn key_picker_popup_scroll_height(popup_size: Vec2) -> f32 {
    (popup_size.y - 130.0).clamp(KEY_PICKER_SCROLL_HEIGHT, 620.0)
}

fn responsive_picker_element_scale(ctx: &egui::Context) -> f32 {
    crate::ui_style::ResponsiveMetrics::from_ctx(ctx).scale
}

fn responsive_picker_key_size(ctx: &egui::Context) -> Vec2 {
    Vec2::splat(54.0 * responsive_picker_element_scale(ctx))
}

fn picker_scaled_size(ctx: &egui::Context, width: f32, height: f32) -> Vec2 {
    crate::ui_style::ResponsiveMetrics::from_ctx(ctx).size(width, height)
}

fn picker_paint_centered_label(
    ui: &egui::Ui,
    rect: egui::Rect,
    label: &str,
    font_size: f32,
    color: Color32,
) {
    let lines: Vec<&str> = label.split('\n').collect();
    if lines.len() <= 1 {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(font_size),
            color,
        );
        return;
    }

    let line_h = font_size + 2.0;
    let total_h = line_h * lines.len() as f32;
    let start_y = rect.center().y - total_h * 0.5 + line_h * 0.5;
    for (idx, line) in lines.iter().enumerate() {
        ui.painter().text(
            egui::pos2(rect.center().x, start_y + idx as f32 * line_h),
            egui::Align2::CENTER_CENTER,
            *line,
            egui::FontId::proportional(font_size),
            color,
        );
    }
}

fn picker_button(
    ui: &mut egui::Ui,
    label: &str,
    size: Vec2,
    enabled: bool,
    active: bool,
) -> egui::Response {
    let sense = if enabled {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let dark = ui.visuals().dark_mode;
    let hovered = enabled && resp.hovered();
    let pressed = enabled && resp.is_pointer_button_down_on();
    let fill = if active {
        crate::ui_style::accent()
    } else if pressed {
        if dark {
            Color32::from_rgb(56, 56, 59)
        } else {
            Color32::from_rgb(232, 232, 235)
        }
    } else if hovered {
        crate::ui_style::hover_fill(dark)
    } else {
        crate::ui_style::surface_fill(dark)
    };
    ui.painter().rect(
        rect,
        9.0,
        fill,
        crate::ui_style::modal_outline_stroke(dark),
        egui::StrokeKind::Inside,
    );
    let color = if active {
        Color32::WHITE
    } else if enabled {
        ui.visuals().text_color()
    } else {
        crate::ui_style::muted_text(dark)
    };
    let label_scale = (size.y / 54.0).clamp(1.0, 1.22);
    picker_paint_centered_label(ui, rect, label, 12.0 * label_scale, color);
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

fn picker_tab_width(label: &str) -> f32 {
    (label.chars().count() as f32 * 7.0 + 24.0).clamp(52.0, 132.0)
}

fn picker_tab_button(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
    picker_button(
        ui,
        label,
        Vec2::new(picker_tab_width(label), 30.0),
        true,
        active,
    )
}

fn picker_slot_button(
    ui: &mut egui::Ui,
    id_text: &str,
    display_name: &str,
    active: bool,
    has_content: bool,
) -> egui::Response {
    let scale = responsive_picker_element_scale(ui.ctx());
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(48.0 * scale, 30.0 * scale), egui::Sense::click());
    let dark = ui.visuals().dark_mode;
    let hovered = resp.hovered();
    let fill = if active {
        crate::ui_style::accent()
    } else if hovered {
        crate::ui_style::hover_fill(dark)
    } else {
        crate::ui_style::surface_fill(dark)
    };
    ui.painter().rect(
        rect,
        8.0,
        fill,
        crate::ui_style::modal_outline_stroke(dark),
        egui::StrokeKind::Inside,
    );
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let text_color = if active {
        Color32::WHITE
    } else if has_content {
        ui.visuals().text_color()
    } else {
        inactive_picker_entry_text(dark)
    };
    if display_name != id_text {
        ui.painter().text(
            egui::pos2(rect.center().x, rect.top() + 9.0 * scale),
            egui::Align2::CENTER_CENTER,
            id_text,
            egui::FontId::proportional(10.5 * scale),
            text_color,
        );
        ui.painter().text(
            egui::pos2(rect.center().x, rect.bottom() - 8.5 * scale),
            egui::Align2::CENTER_CENTER,
            display_name,
            egui::FontId::proportional(10.5 * scale),
            text_color,
        );
    } else {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            id_text,
            egui::FontId::proportional(12.0 * scale),
            text_color,
        );
    }
    resp
}

fn show_grouped_popup_key_buttons(
    ui: &mut egui::Ui,
    keys: Vec<&'static crate::keycode::Keycode>,
    layer_names: &[String],
    friendly_mods: bool,
    language: crate::i18n::Language,
    key_legend_layout: KeyLegendLayout,
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
        ui.label(
            RichText::new(popup_group_title(language, title))
                .size(11.0)
                .color(Color32::from_gray(150))
                .strong(),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for kc in &group {
                let label =
                    popup_key_button_label(kc, layer_names, friendly_mods, key_legend_layout);
                let size = popup_key_button_size(ui, &label);
                let resp = picker_keycap_button(ui, &label, size, true, false);
                if resp.clicked() {
                    selected = Some(kc.value);
                }
                if resp.hovered() {
                    resp.on_hover_text(crate::i18n::tr_text(
                        language,
                        &keycode_tooltip(kc.value, &[], layer_names),
                    ));
                }
            }
        });
        ui.add_space(8.0);
    }

    selected
}

fn show_grouped_popup_choice_buttons(
    ui: &mut egui::Ui,
    groups: Vec<(&'static str, Vec<(u16, String, String)>)>,
    language: crate::i18n::Language,
) -> Option<u16> {
    let mut selected = None;

    for (title, choices) in groups {
        if choices.is_empty() {
            continue;
        }
        ui.add_space(2.0);
        ui.label(
            RichText::new(popup_group_title(language, title))
                .size(11.0)
                .color(Color32::from_gray(150))
                .strong(),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (value, label, tooltip) in choices {
                let size = popup_key_button_size(ui, &label);
                let resp = picker_keycap_button(ui, &label, size, true, false);
                if resp.clicked() {
                    selected = Some(value);
                }
                if resp.hovered() {
                    resp.on_hover_text(crate::i18n::tr_text(language, &tooltip));
                }
            }
        });
        ui.add_space(8.0);
    }

    selected
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

    fn show_vial_quantum(&mut self, ui: &mut egui::Ui) {
        // Pending mod+key selection
        if let Some(base) = self.vial_quantum_pending_mod {
            ui.label(
                RichText::new(tr_picker(self.language, "key_picker.pending_mod_title"))
                    .size(11.5)
                    .strong(),
            );
            ui.label(
                RichText::new(tr_picker(self.language, "key_picker.pending_mod_hint"))
                    .size(10.5)
                    .color(Color32::from_gray(150)),
            );
            ui.add_space(4.0);
            if ui
                .button(tr_picker(self.language, "key_picker.cancel"))
                .clicked()
            {
                self.vial_quantum_pending_mod = None;
            }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for kc in KEYCODES.iter() {
                    if !matches!(
                        kc.category,
                        KeycodeCategory::Basic
                            | KeycodeCategory::Function
                            | KeycodeCategory::Navigation
                    ) {
                        continue;
                    }
                    if kc.value >= 0x0200 || is_f13_to_f24(kc.value) {
                        continue;
                    }
                    let label = keycode_label_with_names_and_layout(
                        kc.value,
                        &[],
                        &self.layer_names,
                        self.key_legend_layout,
                    );
                    let resp = picker_keycap_button(
                        ui,
                        &label,
                        Self::picker_key_size(ui.ctx()),
                        true,
                        false,
                    );
                    if resp.clicked() {
                        self.finish_quantum_pending_key(base, kc.value, false);
                    }
                    resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &keycode_tooltip(kc.value, &[], &self.layer_names),
                    ));
                }
            });
            return;
        }
        if let Some(base) = self.vial_quantum_pending_mt {
            ui.label(
                RichText::new(tr_picker(self.language, "key_picker.pending_tap_title"))
                    .size(11.5)
                    .strong(),
            );
            ui.label(
                RichText::new(tr_picker(self.language, "key_picker.pending_tap_hint"))
                    .size(10.5)
                    .color(Color32::from_gray(150)),
            );
            ui.add_space(4.0);
            if ui
                .button(tr_picker(self.language, "key_picker.cancel"))
                .clicked()
            {
                self.vial_quantum_pending_mt = None;
            }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for kc in KEYCODES.iter() {
                    if !matches!(
                        kc.category,
                        KeycodeCategory::Basic
                            | KeycodeCategory::Function
                            | KeycodeCategory::Navigation
                    ) {
                        continue;
                    }
                    if kc.value >= 0x0200 || is_f13_to_f24(kc.value) {
                        continue;
                    }
                    let label = keycode_label_with_names_and_layout(
                        kc.value,
                        &[],
                        &self.layer_names,
                        self.key_legend_layout,
                    );
                    let resp = picker_keycap_button(
                        ui,
                        &label,
                        Self::picker_key_size(ui.ctx()),
                        true,
                        false,
                    );
                    if resp.clicked() {
                        self.finish_quantum_pending_key(base, kc.value, true);
                    }
                    resp.on_hover_text(crate::i18n::tr_text(
                        self.language,
                        &keycode_tooltip(kc.value, &[], &self.layer_names),
                    ));
                }
            });
            return;
        }

        let gui = gui_sym();
        let lgui = gui_label(false);

        // Mod+Key section
        ui.label(
            RichText::new(tr_picker(
                self.language,
                "key_picker.section_quantum_mod_key",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let mod_bases: Vec<(String, u16, String)> = vec![
            (
                "Ctrl+key".into(),
                0x0100,
                "Hold Left Ctrl together with the key you choose next".into(),
            ),
            (
                "Shift+key".into(),
                0x0200,
                "Hold Left Shift together with the key you choose next".into(),
            ),
            (
                "Alt+key".into(),
                0x0400,
                "Hold Left Alt together with the key you choose next".into(),
            ),
            (
                format!("{}+key", gui),
                0x0800,
                format!("Hold Left {lgui} together with the key you choose next"),
            ),
            (
                "C+S+key".into(),
                0x0300,
                "Hold Ctrl+Shift together with the key you choose next".into(),
            ),
            (
                "C+A+key".into(),
                0x0500,
                "Hold Ctrl+Alt together with the key you choose next".into(),
            ),
            (
                "S+A+key".into(),
                0x0600,
                "Hold Shift+Alt together with the key you choose next".into(),
            ),
            (
                "Meh+key".into(),
                0x0700,
                "Hold Ctrl+Shift+Alt together with the key you choose next".into(),
            ),
            (
                "Hyper+key".into(),
                0x0F00,
                format!(
                    "Hold Ctrl+Shift+Alt+{} together with the key you choose next",
                    gui_mod_name()
                ),
            ),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, base, tip) in &mod_bases {
                let resp = ui
                    .add(
                        egui::Button::new(RichText::new(label.as_str()).size(10.5))
                            .min_size(Self::picker_key_size(ui.ctx())),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    self.vial_quantum_pending_mod = Some(*base);
                }
                resp.on_hover_text(crate::i18n::tr_text(self.language, &tip));
            }
        });

        ui.separator();
        ui.label(
            RichText::new(tr_picker(
                self.language,
                "key_picker.section_quantum_mod_tap",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let mt_bases: Vec<(String, u16, String)> = vec![
            (
                "MT Ctrl".into(),
                0x2100,
                "Dual-role key: hold for Left Ctrl, tap for the key you choose next".into(),
            ),
            (
                "MT Shift".into(),
                0x2200,
                "Dual-role key: hold for Left Shift, tap for the key you choose next".into(),
            ),
            (
                "MT Alt".into(),
                0x2400,
                "Dual-role key: hold for Left Alt, tap for the key you choose next".into(),
            ),
            (
                format!("MT {}", lgui),
                0x2800,
                format!("Dual-role key: hold for Left {lgui}, tap for the key you choose next"),
            ),
            (
                "MT Meh".into(),
                0x2700,
                "Dual-role key: hold for Meh, tap for the key you choose next".into(),
            ),
            (
                "MT Hyper".into(),
                0x2F00,
                "Dual-role key: hold for Hyper, tap for the key you choose next".into(),
            ),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, base, tip) in &mt_bases {
                let resp = ui
                    .add(
                        egui::Button::new(RichText::new(label.as_str()).size(10.5))
                            .min_size(Self::picker_key_size(ui.ctx())),
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    self.vial_quantum_pending_mt = Some(*base);
                }
                resp.on_hover_text(crate::i18n::tr_text(self.language, &tip));
            }
        });
    }

    fn show_macro_editor_contents(
        &mut self,
        ui: &mut egui::Ui,
        raw_n: u8,
        grid_id: &'static str,
        _add_action_id: &'static str,
        _footer_text: &'static str,
    ) -> u8 {
        let mut selected_macro = raw_n;
        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.choose_macro",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        egui::Frame::NONE.show(ui, |ui| {
            let slot_scroll_height = 86.0 * responsive_picker_element_scale(ui.ctx());
            ui.set_max_height(slot_scroll_height);
            egui::ScrollArea::vertical()
                .max_height(slot_scroll_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new(grid_id)
                        .num_columns(16)
                        .spacing([4.0, 4.0])
                        .show(ui, |ui| {
                            for i in 0..128u8 {
                                let is_active = i == selected_macro;
                                let has_content = self.macro_has_content(i as usize);
                                let display_name = self.macro_display_name(i as usize);
                                let id_text = format!("M{}", i);
                                let mut resp = picker_slot_button(
                                    ui,
                                    &id_text,
                                    &display_name,
                                    is_active,
                                    has_content,
                                );
                                if display_name != id_text {
                                    resp = resp.on_hover_text(display_name.clone());
                                }
                                if resp.clicked() {
                                    self.ensure_macro_meta_len(i as usize);
                                    selected_macro = i;
                                }
                                if (i + 1) % 16 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        });
        ui.add_space(crate::ui_style::modal_space_sm());

        if selected_macro == 254 {
            ui.label(
                RichText::new(crate::i18n::tr_catalog(
                    self.language,
                    "macro_editor.select_a_macro_above_to_edit",
                ))
                .size(16.0)
                .color(Color32::from_gray(140)),
            );
            return selected_macro;
        }

        let n = selected_macro as usize;
        self.ensure_macro_meta_len(n);

        let scale = responsive_picker_element_scale(ui.ctx());
        let macro_font_size = 14.0 * scale;
        ui.add_space(4.0 * scale);
        if let Some(name) = self.macro_names.get_mut(n) {
            let resp = crate::ui_style::modern_text_field_sized(
                ui,
                ui.make_persistent_id(("macro_name", grid_id, n)),
                name,
                124.0 * scale,
                32.0 * scale,
                crate::i18n::tr_catalog(self.language, "macro_editor.macro_name"),
                7,
                egui::Align::Center,
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
                    let arrow_size = picker_scaled_size(ui.ctx(), 28.0, 28.0);
                    let up_resp = picker_button(ui, "↑", arrow_size, i > 0, false).on_hover_text(
                        crate::i18n::tr_catalog(self.language, "macro_editor.move_up"),
                    );
                    let down_resp = picker_button(ui, "↓", arrow_size, i + 1 < action_count, false)
                        .on_hover_text(crate::i18n::tr_catalog(
                            self.language,
                            "macro_editor.move_down",
                        ));
                    if up_resp.clicked() && i > 0 {
                        move_up = Some(i);
                    }
                    if down_resp.clicked() && i + 1 < action_count {
                        move_down = Some(i);
                    }

                    let (type_label, type_color, tooltip) = match action {
                        MacroAction::Text(_) => (
                            crate::i18n::tr_catalog(self.language, "macro_editor.text"),
                            crate::ui_style::accent(),
                            crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.types_text_characters_one_by_one",
                            ),
                        ),
                        MacroAction::Tap(_) => (
                            crate::i18n::tr_catalog(self.language, "macro_editor.tap"),
                            crate::ui_style::accent(),
                            crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.press_and_release_a_key",
                            ),
                        ),
                        MacroAction::Down(_) => (
                            crate::i18n::tr_catalog(self.language, "macro_editor.down"),
                            Color32::from_rgb(200, 150, 50),
                            crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.press_a_key_hold_until_up",
                            ),
                        ),
                        MacroAction::Up(_) => (
                            crate::i18n::tr_catalog(self.language, "macro_editor.up"),
                            Color32::from_rgb(132, 150, 178),
                            crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.release_a_previously_pressed_key",
                            ),
                        ),
                        MacroAction::Delay(_) => (
                            crate::i18n::tr_catalog(self.language, "macro_editor.delay"),
                            Color32::from_gray(150),
                            crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.wait_before_next_action",
                            ),
                        ),
                    };
                    ui.allocate_ui(picker_scaled_size(ui.ctx(), 55.0, 30.0), |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(type_label)
                                    .size(macro_font_size)
                                    .color(type_color)
                                    .strong(),
                            )
                            .sense(egui::Sense::hover()),
                        )
                        .on_hover_text(tooltip);
                    });

                    match action {
                        MacroAction::Text(text) => {
                            let text_w = (avail_w - 220.0 * scale).max(150.0 * scale);
                            crate::ui_style::modern_text_field_sized(
                                ui,
                                ui.make_persistent_id(("macro_text_action", grid_id, n, i)),
                                text,
                                text_w,
                                32.0 * scale,
                                crate::i18n::tr_catalog(
                                    self.language,
                                    "macro_editor.type_text_here",
                                ),
                                256,
                                egui::Align::Min,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.characters_to_type_when_this_macro_runs",
                            ));
                        }
                        MacroAction::Tap(kc) => {
                            let label = keycode_label_with_names_and_layout(
                                *kc as u16,
                                &[],
                                &self.layer_names,
                                self.key_legend_layout,
                            );
                            if picker_button(
                                ui,
                                &label,
                                picker_scaled_size(ui.ctx(), 100.0, 30.0),
                                true,
                                false,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.click_to_change_key_press_and_release_this_key",
                            ))
                            .clicked()
                            {
                                self.macro_key_pick = Some((n, i));
                            }
                        }
                        MacroAction::Down(kc) => {
                            let label = keycode_label_with_names_and_layout(
                                *kc as u16,
                                &[],
                                &self.layer_names,
                                self.key_legend_layout,
                            );
                            if picker_button(
                                ui,
                                &label,
                                picker_scaled_size(ui.ctx(), 100.0, 30.0),
                                true,
                                false,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.click_to_change_key_holds_down_until_up",
                            ))
                            .clicked()
                            {
                                self.macro_key_pick = Some((n, i));
                            }
                        }
                        MacroAction::Up(kc) => {
                            let label = keycode_label_with_names_and_layout(
                                *kc as u16,
                                &[],
                                &self.layer_names,
                                self.key_legend_layout,
                            );
                            if picker_button(
                                ui,
                                &label,
                                picker_scaled_size(ui.ctx(), 100.0, 30.0),
                                true,
                                false,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.click_to_change_key_releases_this_key",
                            ))
                            .clicked()
                            {
                                self.macro_key_pick = Some((n, i));
                            }
                        }
                        MacroAction::Delay(ms) => {
                            let mut ms_str = ms.to_string();
                            if crate::ui_style::modern_text_field_sized(
                                ui,
                                ui.make_persistent_id(("macro_delay", grid_id, n, i)),
                                &mut ms_str,
                                80.0 * scale,
                                32.0 * scale,
                                "",
                                5,
                                egui::Align::Center,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.delay_is_in_milliseconds",
                            ))
                            .changed()
                            {
                                if let Ok(v) = ms_str.parse::<u16>() {
                                    *ms = v;
                                }
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if picker_button(
                            ui,
                            "✕",
                            picker_scaled_size(ui.ctx(), 30.0, 30.0),
                            true,
                            false,
                        )
                        .on_hover_text(crate::i18n::tr_catalog(
                            self.language,
                            "macro_editor.remove_this_action",
                        ))
                        .clicked()
                        {
                            remove_idx = Some(i);
                        }
                    });
                });
                ui.add_space(2.0);
            }
        }
        if let Some(idx) = remove_idx {
            if idx < self.macro_actions[n].len() {
                self.macro_undo_stack
                    .push((n, self.macro_actions[n].clone()));
                self.macro_actions[n].remove(idx);
                if let Some((mn, ai)) = self.macro_key_pick {
                    if mn == n && ai >= idx {
                        self.macro_key_pick = None;
                    }
                }
            }
        }
        if let Some(idx) = move_up {
            if idx > 0 {
                self.macro_actions[n].swap(idx, idx - 1);
            }
        }
        if let Some(idx) = move_down {
            if idx + 1 < self.macro_actions[n].len() {
                self.macro_actions[n].swap(idx, idx + 1);
            }
        }

        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "macro_editor.plus_text"),
                picker_scaled_size(ui.ctx(), 72.0, 30.0),
                true,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.type_characters",
            ))
            .clicked()
            {
                self.macro_actions[n].push(MacroAction::Text(String::new()));
            }
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "macro_editor.plus_tap"),
                picker_scaled_size(ui.ctx(), 66.0, 30.0),
                true,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.press_and_release_a_key",
            ))
            .clicked()
            {
                self.macro_actions[n].push(MacroAction::Tap(0x04));
                self.macro_key_pick = Some((n, self.macro_actions[n].len() - 1));
            }
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "macro_editor.plus_down"),
                picker_scaled_size(ui.ctx(), 80.0, 30.0),
                true,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.hold_a_key",
            ))
            .clicked()
            {
                self.macro_actions[n].push(MacroAction::Down(0x04));
                self.macro_key_pick = Some((n, self.macro_actions[n].len() - 1));
            }
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "macro_editor.plus_up"),
                picker_scaled_size(ui.ctx(), 64.0, 30.0),
                true,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.release_a_key",
            ))
            .clicked()
            {
                self.macro_actions[n].push(MacroAction::Up(0x04));
                self.macro_key_pick = Some((n, self.macro_actions[n].len() - 1));
            }
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "macro_editor.plus_delay"),
                picker_scaled_size(ui.ctx(), 82.0, 30.0),
                true,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.pause_in_milliseconds",
            ))
            .clicked()
            {
                self.macro_actions[n].push(MacroAction::Delay(100));
            }
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let can_clear_macro = self.macro_has_content(n)
                || self
                    .macro_names
                    .get(n)
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "key_picker_text.clear_all"),
                picker_scaled_size(ui.ctx(), 86.0, 30.0),
                can_clear_macro,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.remove_all_actions_from_this_macro",
            ))
            .clicked()
            {
                self.macro_undo_stack
                    .push((n, self.macro_actions[n].clone()));
                self.macro_actions[n].clear();
                if n < self.macro_texts.len() {
                    self.macro_texts[n].clear();
                }
                if n < self.macro_names.len() {
                    self.macro_names[n].clear();
                }
            }
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "key_picker_text.undo_undo"),
                picker_scaled_size(ui.ctx(), 78.0, 30.0),
                !self.macro_undo_stack.is_empty(),
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.undo_last_change",
            ))
            .clicked()
            {
                if let Some((idx, prev)) = self.macro_undo_stack.pop() {
                    if idx < self.macro_actions.len() {
                        self.macro_actions[idx] = prev;
                    }
                }
            }
        });

        selected_macro
    }

    fn show_vial_tap_dance(&mut self, ui: &mut egui::Ui) {
        if self.tap_dance_entries.is_empty() {
            self.tap_dance_editor_open = None;
            ui.label(
                RichText::new(crate::i18n::tr_catalog(
                    self.language,
                    "tap_dance_editor.no_tap_dance_slots_available_on_this_keyboard",
                ))
                .size(16.0)
                .color(Color32::from_gray(140)),
            );
            return;
        }

        let selected = match self.tap_dance_editor_open {
            Some(n) if (n as usize) < self.tap_dance_entries.len() => n,
            _ => 0,
        };
        self.tap_dance_editor_open = Some(selected);
        self.ensure_tap_dance_name_len(selected as usize);

        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "tap_dance_editor.choose_tap_dance",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        egui::Frame::NONE.show(ui, |ui| {
            let slot_scroll_height = 86.0 * responsive_picker_element_scale(ui.ctx());
            ui.set_max_height(slot_scroll_height);
            egui::ScrollArea::vertical()
                .max_height(slot_scroll_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("tap_dance_grid_inline")
                        .num_columns(16)
                        .spacing([4.0, 4.0])
                        .show(ui, |ui| {
                            for n in 0..self.tap_dance_entries.len() as u8 {
                                self.ensure_tap_dance_name_len(n as usize);
                                let is_active = n == selected;
                                let display_name = self.tap_dance_display_name(n as usize);
                                let id_text = format!("TD{}", n);
                                let has_content = {
                                    let td = &self.tap_dance_entries[n as usize];
                                    td.on_tap != 0
                                        || td.on_hold != 0
                                        || td.on_double_tap != 0
                                        || td.on_tap_hold != 0
                                        || td.tapping_term != 200
                                };
                                let mut resp = picker_slot_button(
                                    ui,
                                    &id_text,
                                    &display_name,
                                    is_active,
                                    has_content,
                                );
                                if display_name != id_text {
                                    resp = resp.on_hover_text(display_name.clone());
                                }
                                if resp.clicked() {
                                    self.tap_dance_editor_open = Some(n);
                                }
                                if (n + 1) % 16 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        });
        ui.add_space(crate::ui_style::modal_space_sm());

        let n = self.tap_dance_editor_open.unwrap_or(0) as usize;
        self.ensure_tap_dance_name_len(n);
        let scale = responsive_picker_element_scale(ui.ctx());
        let td_font_size = 14.0 * scale;
        ui.add_space(4.0 * scale);
        let prev_name = self.tap_dance_names.get(n).cloned().unwrap_or_default();
        let mut edited_name = prev_name.clone();
        let resp = crate::ui_style::modern_text_field_sized(
            ui,
            ui.make_persistent_id(("tap_dance_name", n)),
            &mut edited_name,
            124.0 * scale,
            32.0 * scale,
            crate::i18n::tr_catalog(self.language, "tap_dance_editor.td_name"),
            7,
            egui::Align::Center,
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
            (
                crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap"),
                crate::i18n::tr_catalog(self.language, "key_picker_text.key_sent_on_single_tap"),
                0u8,
            ),
            (
                crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_hold"),
                crate::i18n::tr_catalog(self.language, "key_picker_text.key_sent_when_held"),
                1,
            ),
            (
                crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_double_tap"),
                crate::i18n::tr_catalog(self.language, "key_picker_text.key_sent_on_double_tap"),
                2,
            ),
            (
                crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap_plus_hold"),
                crate::i18n::tr_catalog(self.language, "key_picker_text.key_sent_on_tap_then_hold"),
                3,
            ),
        ];

        egui::Grid::new("td_fields_inline")
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                for (label, tooltip, field_id) in &fields {
                    ui.add(
                        egui::Label::new(RichText::new(*label).size(td_font_size).strong())
                            .sense(egui::Sense::hover()),
                    )
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
                        crate::keycode::keycode_label_with_names_and_layout(kc, &[], &self.layer_names, self.key_legend_layout)
                    };
                    if picker_button(ui, &kc_label, Vec2::new(120.0, 30.0), true, false)
                        .on_hover_text(if kc == 0 {
                            crate::i18n::tr_catalog(
                                self.language,
                                "tap_dance_editor.click_to_assign_a_key",
                            )
                            .to_string()
                        } else {
                            crate::i18n::tr_text(
                                self.language,
                                &keycode_tooltip(kc, &[], &self.layer_names),
                            )
                        })
                        .clicked()
                    {
                        self.td_key_pick = Some((n, *field_id));
                    }
                    ui.end_row();
                }

                ui.add(
                    egui::Label::new(
                        RichText::new(crate::i18n::tr_catalog(
                            self.language,
                            "tap_dance_editor.tapping_term",
                        ))
                        .size(td_font_size)
                        .strong(),
                    )
                    .sense(egui::Sense::hover()),
                )
                .on_hover_text(crate::i18n::tr_catalog(
                    self.language,
                    "tap_dance_editor.time_in_milliseconds_to_distinguish_tap_from_hold_default_200",
                ));
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let prev_term = self.tap_dance_entries[n].tapping_term;
                    let mut term_str = prev_term.to_string();
                    if crate::ui_style::modern_text_field_sized(
                        ui,
                        ui.make_persistent_id(("tap_dance_term", n)),
                        &mut term_str,
                        76.0 * scale,
                        32.0 * scale,
                        "",
                        5,
                        egui::Align::Center,
                    )
                    .on_hover_text(crate::i18n::tr_catalog(
                        self.language,
                        "tap_dance_editor.tapping_term_is_in_milliseconds",
                    ))
                    .changed()
                    {
                        if let Ok(v) = term_str.parse::<u16>() {
                            let v = v.clamp(10, 3000);
                            if v != prev_term {
                                self.push_tap_dance_undo(n);
                                self.tap_dance_entries[n].tapping_term = v;
                                self.tap_dance_dirty = true;
                            }
                        }
                    }
                });
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let can_clear_tap_dance = self
                .tap_dance_entries
                .get(n)
                .map(|td| {
                    td.on_tap != 0
                        || td.on_hold != 0
                        || td.on_double_tap != 0
                        || td.on_tap_hold != 0
                        || td.tapping_term != 200
                })
                .unwrap_or(false)
                || self
                    .tap_dance_names
                    .get(n)
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "key_picker_text.clear_all"),
                picker_scaled_size(ui.ctx(), 86.0, 30.0),
                can_clear_tap_dance,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "tap_dance_editor.clear_all_actions_for_this_tap_dance",
            ))
            .clicked()
            {
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
            let can_undo_current = self
                .tap_dance_undo_stack
                .iter()
                .any(|(idx, _, _)| *idx == n);
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "key_picker_text.undo_undo"),
                picker_scaled_size(ui.ctx(), 78.0, 30.0),
                can_undo_current,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "tap_dance_editor.undo_last_tap_dance_change",
            ))
            .clicked()
            {
                if let Some(pos) = self
                    .tap_dance_undo_stack
                    .iter()
                    .rposition(|(idx, _, _)| *idx == n)
                {
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
        });
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
            crate::i18n::tr_catalog(self.language, "tap_dance_editor.tap_dance_editor"),
            self.popup_state.id(PopupKey::TapDanceEditorWindow),
            &mut still_open,
            responsive_window_size(ctx, Vec2::new(680.0, 480.0), Vec2::new(980.0, 720.0)),
        )
        .show(ctx, |ui| {
            // Tabs
            ui.horizontal_wrapped(|ui| {
                for n in 0..self.tap_dance_entries.len() as u8 {
                    let is_active = n == active_td;
                    let label = format!("TD{}", n);
                    let btn =
                        egui::Button::new(RichText::new(&label).size(14.0).color(if is_active {
                            Color32::WHITE
                        } else {
                            Color32::from_gray(100)
                        }))
                        .fill(if is_active {
                            crate::ui_style::accent()
                        } else {
                            Color32::TRANSPARENT
                        })
                        .min_size(crate::ui_style::modal_tab_button_size());
                    if ui.add(btn).clicked() {
                        self.tap_dance_editor_open = Some(n);
                    }
                }
            });
            ui.separator();

            if self.tap_dance_entries.is_empty() {
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(
                        self.language,
                        "tap_dance_editor.no_tap_dance_slots_available_on_this_keyboard",
                    ))
                    .size(16.0)
                    .color(Color32::from_gray(140)),
                );
                return;
            }

            if active_td == 255 || active_td as usize >= self.tap_dance_entries.len() {
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(
                        self.language,
                        "key_picker_text.select_a_tap_dance_tab_above_to_edit",
                    ))
                    .size(16.0)
                    .color(Color32::from_gray(140)),
                );
                return;
            }

            let scale = responsive_picker_element_scale(ui.ctx());
            let n = active_td as usize;
            ui.label(
                RichText::new(format!("TD{}", n))
                    .size(18.0 * scale)
                    .strong(),
            );
            ui.add_space(8.0 * scale);

            let fields = [
                (
                    crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap"),
                    crate::i18n::tr_catalog(
                        self.language,
                        "key_picker_text.key_sent_on_single_tap",
                    ),
                    0u8,
                ),
                (
                    crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_hold"),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.key_sent_when_held"),
                    1,
                ),
                (
                    crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_double_tap"),
                    crate::i18n::tr_catalog(
                        self.language,
                        "key_picker_text.key_sent_on_double_tap",
                    ),
                    2,
                ),
                (
                    crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap_plus_hold"),
                    crate::i18n::tr_catalog(
                        self.language,
                        "key_picker_text.key_sent_on_tap_then_hold",
                    ),
                    3,
                ),
            ];

            egui::Grid::new("td_fields")
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    for (label, tooltip, field_id) in &fields {
                        ui.add(
                            egui::Label::new(RichText::new(*label).size(15.0 * scale).strong())
                                .sense(egui::Sense::hover()),
                        )
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
                            crate::keycode::keycode_label_with_names_and_layout(kc, &[], &self.layer_names, self.key_legend_layout)
                        };
                        if ui
                            .add(
                                egui::Button::new(RichText::new(&kc_label).size(16.0))
                                    .min_size(picker_scaled_size(ui.ctx(), 132.0, 30.0)),
                            )
                            .on_hover_text(if kc == 0 {
                                crate::i18n::tr_catalog(
                                    self.language,
                                    "tap_dance_editor.click_to_assign_a_key",
                                )
                                .to_string()
                            } else {
                                crate::i18n::tr_text(
                                    self.language,
                                    &keycode_tooltip(kc, &[], &self.layer_names),
                                )
                            })
                            .clicked()
                        {
                            self.td_key_pick = Some((n, *field_id));
                        }
                        ui.end_row();
                    }

                    // Tapping term
                    ui.add(
                        egui::Label::new(
                            RichText::new(crate::i18n::tr_catalog(
                                self.language,
                                "tap_dance_editor.tapping_term",
                            ))
                            .size(15.0 * scale)
                            .strong(),
                        )
                        .sense(egui::Sense::hover()),
                    )
                    .on_hover_text(crate::i18n::tr_catalog(
                        self.language,
                        "tap_dance_editor.time_in_milliseconds_to_distinguish_tap_from_hold_default_200",
                    ));
                    let mut term_str = self.tap_dance_entries[n].tapping_term.to_string();
                    ui.horizontal(|ui| {
                        if crate::ui_style::modern_text_field_sized(
                            ui,
                            ui.make_persistent_id(("tap_dance_legacy_term", n)),
                            &mut term_str,
                            80.0 * scale,
                            32.0 * scale,
                            "",
                            5,
                            egui::Align::Center,
                        )
                        .on_hover_text(crate::i18n::tr_catalog(
                            self.language,
                            "tap_dance_editor.tapping_term_is_in_milliseconds",
                        ))
                        .changed()
                        {
                            if let Ok(v) = term_str.parse::<u16>() {
                                self.tap_dance_entries[n].tapping_term = v;
                            }
                        }
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
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
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
            0 => crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap"),
            1 => crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_hold"),
            2 => crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_double_tap"),
            3 => crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap_plus_hold"),
            _ => "?",
        };
        let helper_text = match field {
            0 => crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.best_for_normal_keys_navigation_media_and_special_actions",
            ),
            1 => crate::i18n::tr_catalog(self.language, "key_picker_text.hold_actions_are_limited_to_left_right_modifiers_and_layers"),
            2 => crate::i18n::tr_catalog(self.language, "key_picker_text.best_for_a_second_tap_action_usually_another_normal_key_or_command"),
            3 => crate::i18n::tr_catalog(self.language, "key_picker_text.tap_then_hold_actions_are_limited_to_left_right_modifiers_and_layers"),
            _ => tr_picker(self.language, "key_picker.press_key_or_click_cancel"),
        };
        let td_choices: Vec<(u16, String, String)> = if matches!(field, 1 | 3) {
            let gui = gui_label(false).to_string();
            let mut out: Vec<(u16, String, String)> = vec![
                (
                    0x00E0,
                    "Left\nCtrl".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.left_control").into(),
                ),
                (
                    0x00E4,
                    "Right\nCtrl".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.right_control").into(),
                ),
                (
                    0x00E1,
                    "Left\nShift".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.left_shift").into(),
                ),
                (
                    0x00E5,
                    "Right\nShift".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.right_shift").into(),
                ),
                (
                    0x00E2,
                    "Left\nAlt".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.left_alt").into(),
                ),
                (
                    0x00E6,
                    "Right\nAlt".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.right_alt").into(),
                ),
                (
                    0x00E3,
                    format!("Left\n{}", gui),
                    crate::i18n::tr_text(self.language, &format!("Left {}", gui)),
                ),
                (
                    0x00E7,
                    format!("Right\n{}", gui),
                    crate::i18n::tr_text(self.language, &format!("Right {}", gui)),
                ),
            ];
            out.extend(
                self.tap_dance_layer_choices()
                    .into_iter()
                    .map(|(value, _label)| {
                        let layer = (value & 0x1F) as usize;
                        let layer_name = self
                            .layer_names
                            .get(layer)
                            .cloned()
                            .unwrap_or_else(|| layer.to_string());
                        (
                            value,
                            format!("MO({})\n{}", layer, layer_name),
                            crate::i18n::tr_text(
                                self.language,
                                &format!("Momentarily activate layer {} while held", layer_name),
                            ),
                        )
                    }),
            );
            out
        } else {
            KEYCODES
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
                        && !is_f13_to_f24(kc.value)
                })
                .map(|kc| {
                    (
                        kc.value,
                        keycode_label_with_names_and_layout(
                            kc.value,
                            &[],
                            &self.layer_names,
                            self.key_legend_layout,
                        ),
                        keycode_tooltip(kc.value, &[], &self.layer_names),
                    )
                })
                .collect()
        };
        let mut still_open = true;
        let popup_size = key_picker_popup_size(ctx);
        let window_title = crate::i18n::tr_catalog_format(
            self.language,
            "key_picker.pick_key_for",
            &[("field", field_name)],
        );
        crate::ui_style::centered_modal_window(
            ctx,
            &window_title,
            self.popup_state.id(PopupKey::TdKeyPickWindow),
            &mut still_open,
            popup_size,
        )
        .show(ctx, |ui| {
            apply_picker_button_visuals(ui);
            crate::ui_style::modal_intro(
                ui,
                tr_picker(self.language, "key_picker.press_key_or_click_cancel"),
            );
            crate::ui_style::modal_hint(ui, helper_text);
            ui.add_space(crate::ui_style::modal_space_xs());
            if picker_button(
                ui,
                tr_picker(self.language, "key_picker.none_clear"),
                crate::ui_style::modal_action_button_size(),
                true,
                false,
            )
            .clicked()
            {
                self.set_tap_dance_field(td_idx, field, 0);
                self.td_key_pick = None;
            }
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .max_height(key_picker_popup_scroll_height(popup_size))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if matches!(field, 1 | 3) {
                        let modifier_choices: Vec<(u16, String, String)> =
                            td_choices.iter().take(8).cloned().collect();
                        let layer_choices: Vec<(u16, String, String)> =
                            td_choices.iter().skip(8).cloned().collect();
                        let groups =
                            vec![("Modifiers", modifier_choices), ("Layers", layer_choices)];
                        if let Some(value) =
                            show_grouped_popup_choice_buttons(ui, groups, self.language)
                        {
                            self.set_tap_dance_field(td_idx, field, value);
                            self.td_key_pick = None;
                        }
                    } else {
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
                                    && !is_f13_to_f24(kc.value)
                            })
                            .collect();
                        if let Some(value) = show_grouped_popup_key_buttons(
                            ui,
                            key_choices,
                            &self.layer_names,
                            false,
                            self.language,
                            self.key_legend_layout,
                        ) {
                            self.set_tap_dance_field(td_idx, field, value);
                            self.td_key_pick = None;
                        }
                    }
                });
        });
        if !still_open {
            self.td_key_pick = None;
        }
    }

    fn macro_has_content(&self, n: usize) -> bool {
        self.macro_actions
            .get(n)
            .map(|a| !a.is_empty())
            .unwrap_or(false)
            || self
                .macro_texts
                .get(n)
                .map(|s| !s.is_empty())
                .unwrap_or(false)
    }

    fn ensure_macro_meta_len(&mut self, n: usize) {
        while self.macro_texts.len() <= n {
            self.macro_texts.push(String::new());
        }
        while self.macro_names.len() <= n {
            self.macro_names.push(String::new());
        }
        while self.macro_actions.len() <= n {
            self.macro_actions.push(vec![]);
        }
    }

    fn macro_display_name(&self, n: usize) -> String {
        match self.macro_names.get(n) {
            Some(name) if !name.trim().is_empty() => name.clone(),
            _ => format!("M{}", n),
        }
    }

    fn macro_custom_name(&self, n: usize) -> Option<String> {
        self.macro_names
            .get(n)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn ensure_tap_dance_name_len(&mut self, n: usize) {
        while self.tap_dance_names.len() <= n {
            self.tap_dance_names.push(String::new());
        }
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
                let name = self
                    .layer_names
                    .get(layer)
                    .cloned()
                    .unwrap_or_else(|| layer.to_string());
                (0x5220 | layer as u16, format!("MO({})", name))
            })
            .collect()
    }

    fn set_tap_dance_field(&mut self, n: usize, field: u8, value: u16) {
        if n >= self.tap_dance_entries.len() {
            return;
        }
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
        if current == value {
            return;
        }
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
        while self.macro_texts.len() <= n {
            self.macro_texts.push(String::new());
        }
        while self.macro_actions.len() <= n {
            self.macro_actions.push(vec![]);
        }
        let mut encoded = Vec::new();
        for action in &self.macro_actions[n] {
            match action {
                MacroAction::Text(s) => encoded.extend_from_slice(s.as_bytes()),
                MacroAction::Tap(kc) => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(*kc);
                }
                MacroAction::Down(kc) => {
                    encoded.push(1);
                    encoded.push(2);
                    encoded.push(*kc);
                }
                MacroAction::Up(kc) => {
                    encoded.push(1);
                    encoded.push(3);
                    encoded.push(*kc);
                }
                MacroAction::Delay(ms) => {
                    let hi = (*ms / 255 + 1) as u8;
                    let lo = (*ms % 255 + 1) as u8;
                    encoded.push(1);
                    encoded.push(4);
                    encoded.push(lo);
                    encoded.push(hi);
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
        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.backlight",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let bl_keys: &[(&str, u16, &str)] = &[
            ("Toggle", 0x7800, "Toggle backlight on/off"),
            ("Cycle", 0x7801, "Cycle through backlight brightness levels"),
            ("Breathing", 0x7802, "Toggle breathing effect on/off"),
            ("On", 0x7805, "Turn backlight on"),
            ("Off", 0x7806, "Turn backlight off"),
            ("Brightness -", 0x7804, "Decrease backlight brightness"),
            ("Brightness +", 0x7803, "Increase backlight brightness"),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in bl_keys {
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let display_label = picker_action_label(label);
                Self::paint_compact_picker_label(ui, &resp, &display_label);
                if resp.clicked() {
                    self.assign_keycode_value(*value);
                }
                resp.on_hover_text(crate::i18n::tr_catalog(self.language, tip));
            }
        });

        ui.add_space(10.0);
        // RGB Underglow (QMK rgblight)
        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.rgb_underglow",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let rgb_keys: &[(&str, u16, &str)] = &[
            ("Toggle", 0x7A00, "Toggle RGB lighting on/off"),
            ("Prev Mode", 0x7A02, "Switch to previous RGB animation mode"),
            ("Next Mode", 0x7A01, "Switch to next RGB animation mode"),
            ("Hue -", 0x7A04, "Decrease color hue"),
            ("Hue +", 0x7A03, "Increase color hue"),
            ("Saturation -", 0x7A06, "Decrease color saturation"),
            ("Saturation +", 0x7A05, "Increase color saturation"),
            ("Brightness -", 0x7A08, "Decrease brightness"),
            ("Brightness +", 0x7A07, "Increase brightness"),
            ("Speed -", 0x7A0A, "Decrease animation speed"),
            ("Speed +", 0x7A09, "Increase animation speed"),
            ("Effect -", 0x7A0C, "Previous RGB effect"),
            ("Effect +", 0x7A0B, "Next RGB effect"),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in rgb_keys {
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let display_label = picker_action_label(label);
                Self::paint_compact_picker_label(ui, &resp, &display_label);
                if resp.clicked() {
                    self.assign_keycode_value(*value);
                }
                resp.on_hover_text(crate::i18n::tr_catalog(self.language, tip));
            }
        });

        ui.add_space(10.0);
        // RGB Matrix modes
        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.rgb_matrix_modes",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let rgbm_keys: &[(&str, u16, &str)] = &[
            ("Plain", 0x7A0D, "RGB Matrix: solid color, no animation"),
            (
                "Breathe",
                0x7A0E,
                "RGB Matrix: breathing effect — smooth brightness fade",
            ),
            (
                "Rainbow",
                0x7A0F,
                "RGB Matrix: rainbow gradient across all keys",
            ),
            ("Swirl", 0x7A10, "RGB Matrix: swirling rainbow pattern"),
            (
                "Snake",
                0x7A11,
                "RGB Matrix: snake animation moving across keys",
            ),
            ("Knight", 0x7A12, "RGB Matrix: Knight Rider scanning effect"),
            (
                "Xmas",
                0x7A13,
                "RGB Matrix: alternating red and green like Christmas lights",
            ),
            ("Gradient", 0x7A14, "RGB Matrix: static gradient effect"),
            (
                "Test",
                0x7A15,
                "RGB Matrix: test mode — cycles through R, G, B",
            ),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in rgbm_keys {
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let display_label = picker_action_label(label);
                Self::paint_compact_picker_label(ui, &resp, &display_label);
                if resp.clicked() {
                    self.assign_keycode_value(*value);
                }
                resp.on_hover_text(crate::i18n::tr_catalog(self.language, tip));
            }
        });

        ui.add_space(10.0);
        // RGB Matrix controls
        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.rgb_matrix_controls",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        let rgbm_ctrl: &[(&str, u16, &str)] = &[
            ("On", 0x7A16, "Turn RGB Matrix on"),
            ("Off", 0x7A17, "Turn RGB Matrix off"),
            ("Toggle", 0x7A18, "Toggle RGB Matrix on/off"),
            ("Previous", 0x7A1A, "Previous RGB Matrix animation"),
            ("Next", 0x7A19, "Next RGB Matrix animation"),
            ("Hue -", 0x7A1C, "Decrease RGB Matrix hue"),
            ("Hue +", 0x7A1B, "Increase RGB Matrix hue"),
            ("Saturation -", 0x7A1E, "Decrease RGB Matrix saturation"),
            ("Saturation +", 0x7A1D, "Increase RGB Matrix saturation"),
            ("Brightness -", 0x7A20, "Decrease RGB Matrix brightness"),
            ("Brightness +", 0x7A1F, "Increase RGB Matrix brightness"),
            ("Speed -", 0x7A22, "Decrease RGB Matrix animation speed"),
            ("Speed +", 0x7A21, "Increase RGB Matrix animation speed"),
        ];
        ui.horizontal_wrapped(|ui| {
            for (label, value, tip) in rgbm_ctrl {
                let resp = ui
                    .add_sized(Self::picker_key_size(ui.ctx()), egui::Button::new(""))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let display_label = picker_action_label(label);
                Self::paint_compact_picker_label(ui, &resp, &display_label);
                if resp.clicked() {
                    self.assign_keycode_value(*value);
                }
                resp.on_hover_text(crate::i18n::tr_catalog(self.language, tip));
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
