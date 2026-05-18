use super::*;

pub(super) fn popup_group_i18n_key(title: &str) -> Option<&'static str> {
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

pub(super) fn popup_group_title(
    language: crate::i18n::Language,
    title: &'static str,
) -> &'static str {
    popup_group_i18n_key(title)
        .map(|key| tr_picker(language, key))
        .unwrap_or_else(|| crate::i18n::tr_catalog(language, title))
}

pub(super) fn popup_key_group_title(kc: &crate::keycode::Keycode) -> &'static str {
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

pub(super) fn is_f13_to_f24(value: u16) -> bool {
    (0x0068..=0x0073).contains(&value)
}

pub(super) fn popup_key_button_label(
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

pub(super) fn picker_shifted_number_label(
    value: u16,
    key_legend_layout: KeyLegendLayout,
) -> Option<String> {
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

pub(super) fn show_grouped_popup_key_buttons(
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

pub(super) fn show_grouped_popup_choice_buttons(
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
