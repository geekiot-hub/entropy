#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    English,
    Russian,
}

impl Language {
    pub const ALL: [Language; 2] = [Language::English, Language::Russian];

    pub fn native_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Russian => "Русский",
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        default_language()
    }
}

pub fn default_language() -> Language {
    Language::English
}

const EN_CATALOG: &str = include_str!("../i18n/en.toml");
const RU_CATALOG: &str = include_str!("../i18n/ru.toml");

pub fn tr_catalog(language: Language, key: &'static str) -> &'static str {
    let translated = match language {
        Language::English => catalog_lookup(EN_CATALOG, key),
        Language::Russian => catalog_lookup(RU_CATALOG, key),
    };

    translated
        .or_else(|| catalog_lookup(EN_CATALOG, key))
        .unwrap_or(key)
}

pub fn tr_catalog_format(language: Language, key: &'static str, vars: &[(&str, &str)]) -> String {
    let mut text = tr_catalog(language, key).to_owned();
    for (name, value) in vars {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

fn tr_catalog_string(language: Language, key: &'static str) -> String {
    let translated = match language {
        Language::English => catalog_lookup_owned(EN_CATALOG, key),
        Language::Russian => catalog_lookup_owned(RU_CATALOG, key),
    };

    translated
        .or_else(|| catalog_lookup_owned(EN_CATALOG, key))
        .unwrap_or_else(|| key.to_owned())
}

fn catalog_lookup(catalog: &'static str, key: &str) -> Option<&'static str> {
    let (wanted_section, wanted_name) = key.rsplit_once('.')?;
    let mut section = "";

    for raw_line in catalog.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim();
            continue;
        }

        if section != wanted_section {
            continue;
        }

        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        if name.trim() != wanted_name {
            continue;
        }

        let value = value.trim();
        if value.len() >= 2
            && ((value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\'')))
        {
            return Some(&value[1..value.len() - 1]);
        }
    }

    None
}

fn catalog_lookup_owned(catalog: &'static str, key: &str) -> Option<String> {
    let (wanted_section, wanted_name) = key.rsplit_once('.')?;
    let mut section = "";

    for raw_line in catalog.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim();
            continue;
        }

        if section != wanted_section {
            continue;
        }

        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        if name.trim() != wanted_name {
            continue;
        }

        return parse_catalog_value(value.trim());
    }

    None
}

fn parse_catalog_value(value: &str) -> Option<String> {
    if value.len() < 2 {
        return None;
    }

    if value.starts_with('"') && value.ends_with('"') {
        let inner = &value[1..value.len() - 1];
        let mut out = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(ch) = chars.next() {
            if ch != '\\' {
                out.push(ch);
                continue;
            }

            match chars.next()? {
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                other => out.push(other),
            }
        }
        return Some(out);
    }

    if value.starts_with('\'') && value.ends_with('\'') {
        return Some(value[1..value.len() - 1].to_owned());
    }

    None
}

fn ru_smart_symbol_name(name: &str) -> &str {
    match name {
        "Left brace" => "левая фигурная скобка",
        "Right brace" => "правая фигурная скобка",
        "Left bracket" => "левая квадратная скобка",
        "Right bracket" => "правая квадратная скобка",
        "Left parenthesis" => "левая круглая скобка",
        "Right parenthesis" => "правая круглая скобка",
        "Less-than" => "знак «меньше»",
        "Greater-than" => "знак «больше»",
        "Number sign" => "решётка",
        "At sign" => "собака",
        "Numero sign" => "знак номера",
        "Ruble sign" => "знак рубля",
        "Exclamation mark" => "восклицательный знак",
        "Quotation mark" => "двойная кавычка",
        "Dollar sign" => "знак доллара",
        "Percent sign" => "знак процента",
        "Ampersand" => "амперсанд",
        "Apostrophe" => "апостроф",
        "Asterisk" => "звёздочка",
        "Plus sign" => "плюс",
        "Equals sign" => "знак равенства",
        "Question mark" => "вопросительный знак",
        "Vertical bar" => "вертикальная черта",
        "Backslash" => "обратный слэш",
        "Left guillemet" => "левая ёлочка",
        "Right guillemet" => "правая ёлочка",
        "Euro sign" => "знак евро",
        "Em dash" => "длинное тире",
        "En dash" => "среднее тире",
        "Bullet" => "маркер списка",
        "Multiplication sign" => "знак умножения",
        "Plus-minus sign" => "плюс-минус",
        "Not equal sign" => "знак неравенства",
        "Almost equal sign" => "знак примерного равенства",
        "Check mark" => "галочка",
        "Section sign" => "знак параграфа",
        "Full stop" => "точка",
        "Comma" => "запятая",
        "Semicolon" => "точка с запятой",
        "Colon" => "двоеточие",
        "Slash" => "слэш",
        "Grave accent" => "гравис",
        "Caret" => "карет",
        "Cyrillic be" | "Cyrillic Be" => "кириллическая Б",
        "Cyrillic yu" | "Cyrillic Yu" => "кириллическая Ю",
        "Cyrillic zhe" | "Cyrillic Zhe" => "кириллическая Ж",
        "Cyrillic e" | "Cyrillic E" => "кириллическая Э",
        "Cyrillic ha" | "Cyrillic Ha" => "кириллическая Х",
        "Cyrillic hard sign" | "Cyrillic Hard Sign" => "кириллический твёрдый знак",
        "Cyrillic yo" | "Cyrillic Yo" => "кириллическая Ё",
        "Degree sign" => "знак градуса",
        "Per mille sign" => "промилле",
        "Prime" => "штрих",
        "Double prime" => "двойной штрих",
        "Left single quotation mark" => "левая одинарная кавычка",
        "Right single quotation mark" => "правая одинарная кавычка",
        "Double low quotation mark" => "нижняя двойная кавычка",
        "Left double quotation mark" => "левая двойная кавычка",
        "Right double quotation mark" => "правая двойная кавычка",
        "Trade mark sign" => "знак торговой марки",
        "Tilde" => "тильда",
        "Underscore" => "нижнее подчёркивание",
        _ => name,
    }
}

fn ru_modifier_name(name: &str) -> String {
    name.replace("Left ", "левый ").replace("Right ", "правый ")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    MainTabLayout,
    MainTabAdvanced,
    MainTabConfig,
    NoDevicesFound,
    LockAction,
    UnlockAction,
    ComboTitle,
    AutoShiftTitle,
    KeyOverridesTitle,
    MouseKeysTitle,
    MatrixTesterTitle,
    MatrixTesterDescription,
    UniversalSymbolsSetupTitle,
    UniversalSymbolsTitle,
    RgbTitle,
    LayerLedsTitle,
    EncodersTitle,
    DisplayPresetsTitle,
    TouchpadTitle,
    LiveFeaturesTitle,
    MagicTitle,
    TapHoldOneShotTitle,
    AltRepeatTitle,
    AltRepeatDescription,
    AltRepeatUnavailable,
    GraveEscapeTitle,
    RgbUnavailableTooltip,
    AppSettingsTitle,
    AppSettingsDescription,
    LanguageLabel,
    LanguageTooltip,
    CloseToTrayLabel,
    CloseToTrayTooltip,
    ShiftedNumberSymbolsLabel,
    ShiftedNumberSymbolsTooltip,
    LayerHoverPreviewLabel,
    LayerHoverPreviewTooltip,
    EncoderHoverZoomLabel,
    EncoderHoverZoomTooltip,
    AccentColorLabel,
    AccentColorTooltip,
    MouseKeysDescription,
    MouseKeysUnavailable,
    MouseKeysEnableHint,
    RgbDescription,
    RgbConnect,
    KeyOverridesDescription,
    ComboDescription,
    EncodersDescription,
    EncodersUnavailable,
    DisplayPresetsDescription,
    DisplayPresetsUnavailable,
    DisplayPresetsConnect,
    AutoShiftDescription,
    AutoShiftUnavailable,
    AutoShiftEnableHint,
    KeyboardLocked,
    AutoShiftUnlockHint,
    LayerLedsDescription,
    LayerLedsUnavailable,
    LayerLedsEnableHint,
    LayerLedsConnect,
    GraveEscapeDescription,
    GraveEscapeUnavailable,
    GraveEscapeEnableHint,
    GraveEscapeConnect,
    MagicDescription,
    MagicUnavailable,
    MagicEnableHint,
    MagicConnect,
    TapHoldOneShotDescription,
    TapHoldOneShotUnavailable,
    QmkSettingsEnableHint,
    TapHoldOneShotConnect,
    LiveFeaturesDescription,
    LiveFeaturesInactive,
    LiveFeaturesSelectHint,
    LiveFeaturesReadyNote,
    TouchpadDescription,
    TouchpadUnavailable,
    TouchpadEnableHint,
    TouchpadConnect,
}
impl Key {
    fn i18n_key(self) -> &'static str {
        match self {
            Key::MainTabLayout => "ui.main_tab_layout",
            Key::MainTabAdvanced => "ui.main_tab_advanced",
            Key::MainTabConfig => "ui.main_tab_config",
            Key::NoDevicesFound => "ui.no_devices_found",
            Key::LockAction => "ui.lock_action",
            Key::UnlockAction => "ui.unlock_action",
            Key::ComboTitle => "ui.combo_title",
            Key::AutoShiftTitle => "ui.auto_shift_title",
            Key::KeyOverridesTitle => "ui.key_overrides_title",
            Key::MouseKeysTitle => "ui.mouse_keys_title",
            Key::MatrixTesterTitle => "ui.matrix_tester_title",
            Key::MatrixTesterDescription => "ui.matrix_tester_description",
            Key::UniversalSymbolsSetupTitle => "ui.universal_symbols_setup_title",
            Key::UniversalSymbolsTitle => "ui.universal_symbols_title",
            Key::RgbTitle => "ui.rgb_title",
            Key::LayerLedsTitle => "ui.layer_leds_title",
            Key::EncodersTitle => "ui.encoders_title",
            Key::DisplayPresetsTitle => "ui.display_presets_title",
            Key::TouchpadTitle => "ui.touchpad_title",
            Key::LiveFeaturesTitle => "ui.live_features_title",
            Key::MagicTitle => "ui.magic_title",
            Key::TapHoldOneShotTitle => "ui.tap_hold_one_shot_title",
            Key::AltRepeatTitle => "ui.alt_repeat_title",
            Key::AltRepeatDescription => "ui.alt_repeat_description",
            Key::AltRepeatUnavailable => "ui.alt_repeat_unavailable",
            Key::GraveEscapeTitle => "ui.grave_escape_title",
            Key::RgbUnavailableTooltip => "ui.rgb_unavailable_tooltip",
            Key::AppSettingsTitle => "ui.app_settings_title",
            Key::AppSettingsDescription => "ui.app_settings_description",
            Key::LanguageLabel => "ui.language_label",
            Key::LanguageTooltip => "ui.language_tooltip",
            Key::CloseToTrayLabel => "ui.close_to_tray_label",
            Key::CloseToTrayTooltip => "ui.close_to_tray_tooltip",
            Key::ShiftedNumberSymbolsLabel => "ui.shifted_number_symbols_label",
            Key::ShiftedNumberSymbolsTooltip => "ui.shifted_number_symbols_tooltip",
            Key::LayerHoverPreviewLabel => "ui.layer_hover_preview_label",
            Key::LayerHoverPreviewTooltip => "ui.layer_hover_preview_tooltip",
            Key::EncoderHoverZoomLabel => "ui.encoder_hover_zoom_label",
            Key::EncoderHoverZoomTooltip => "ui.encoder_hover_zoom_tooltip",
            Key::AccentColorLabel => "ui.accent_color_label",
            Key::AccentColorTooltip => "ui.accent_color_tooltip",
            Key::MouseKeysDescription => "ui.mouse_keys_description",
            Key::MouseKeysUnavailable => "ui.mouse_keys_unavailable",
            Key::MouseKeysEnableHint => "ui.mouse_keys_enable_hint",
            Key::RgbDescription => "ui.rgb_description",
            Key::RgbConnect => "ui.rgb_connect",
            Key::KeyOverridesDescription => "ui.key_overrides_description",
            Key::ComboDescription => "ui.combo_description",
            Key::EncodersDescription => "ui.encoders_description",
            Key::EncodersUnavailable => "ui.encoders_unavailable",
            Key::DisplayPresetsDescription => "ui.display_presets_description",
            Key::DisplayPresetsUnavailable => "ui.display_presets_unavailable",
            Key::DisplayPresetsConnect => "ui.display_presets_connect",
            Key::AutoShiftDescription => "ui.auto_shift_description",
            Key::AutoShiftUnavailable => "ui.auto_shift_unavailable",
            Key::AutoShiftEnableHint => "ui.auto_shift_enable_hint",
            Key::KeyboardLocked => "ui.keyboard_locked",
            Key::AutoShiftUnlockHint => "ui.auto_shift_unlock_hint",
            Key::LayerLedsDescription => "ui.layer_leds_description",
            Key::LayerLedsUnavailable => "ui.layer_leds_unavailable",
            Key::LayerLedsEnableHint => "ui.layer_leds_enable_hint",
            Key::LayerLedsConnect => "ui.layer_leds_connect",
            Key::GraveEscapeDescription => "ui.grave_escape_description",
            Key::GraveEscapeUnavailable => "ui.grave_escape_unavailable",
            Key::GraveEscapeEnableHint => "ui.grave_escape_enable_hint",
            Key::GraveEscapeConnect => "ui.grave_escape_connect",
            Key::MagicDescription => "ui.magic_description",
            Key::MagicUnavailable => "ui.magic_unavailable",
            Key::MagicEnableHint => "ui.magic_enable_hint",
            Key::MagicConnect => "ui.magic_connect",
            Key::TapHoldOneShotDescription => "ui.tap_hold_one_shot_description",
            Key::TapHoldOneShotUnavailable => "ui.tap_hold_one_shot_unavailable",
            Key::QmkSettingsEnableHint => "ui.qmk_settings_enable_hint",
            Key::TapHoldOneShotConnect => "ui.tap_hold_one_shot_connect",
            Key::LiveFeaturesDescription => "ui.live_features_description",
            Key::LiveFeaturesInactive => "ui.live_features_inactive",
            Key::LiveFeaturesSelectHint => "ui.live_features_select_hint",
            Key::LiveFeaturesReadyNote => "ui.live_features_ready_note",
            Key::TouchpadDescription => "ui.touchpad_description",
            Key::TouchpadUnavailable => "ui.touchpad_unavailable",
            Key::TouchpadEnableHint => "ui.touchpad_enable_hint",
            Key::TouchpadConnect => "ui.touchpad_connect",
        }
    }
}

pub fn tr(language: Language, key: Key) -> &'static str {
    tr_catalog(language, key.i18n_key())
}

fn legacy_static_key(text: &str) -> Option<&'static str> {
    match text {
        "Effect" => Some("legacy_static.effect"),
        "Color" => Some("legacy_static.color"),
        "Speed" => Some("legacy_static.speed"),
        "Brightness" => Some("legacy_static.brightness"),
        "Key Picker" => Some("legacy_static.key_picker"),
        "Pick key" => Some("legacy_static.pick_key"),
        "Press a key on your keyboard, or pick below" => Some("legacy_static.press_a_key_on_your_keyboard_or_pick_below"),
        "Press a key on your keyboard, or click below" => Some("legacy_static.press_a_key_on_your_keyboard_or_click_below"),
        "Press a key on your keyboard, or click below (Esc to cancel)" => Some("legacy_static.press_a_key_on_your_keyboard_or_click_below_esc_to_cancel"),
        "Best for normal keys, navigation, media and special actions" => Some("legacy_static.best_for_normal_keys_navigation_media_and_special_actions"),
        "None (clear)" => Some("legacy_static.none_clear"),
        "Pick layer" => Some("legacy_static.pick_layer"),
        "Choose which layer (Esc to cancel)" => Some("legacy_static.choose_which_layer_esc_to_cancel"),
        "Pick tap key (hold = modifier)" => Some("legacy_static.pick_tap_key_hold_modifier"),
        "Pick key for modifier combo" => Some("legacy_static.pick_key_for_modifier_combo"),
        "Basic" => Some("legacy_static.basic"),
        "Symbols" => Some("legacy_static.symbols"),
        "Mods" => Some("legacy_static.mods"),
        "Special" => Some("legacy_static.special"),
        "Macros" => Some("legacy_static.macros"),
        "Tap Dance" => Some("legacy_static.tap_dance"),
        "Custom" => Some("legacy_static.custom"),
        "Layers" => Some("legacy_static.layers"),
        "Letters" => Some("legacy_static.letters"),
        "Numbers" => Some("legacy_static.numbers"),
        "Editing" => Some("legacy_static.editing"),
        "Navigation" => Some("legacy_static.navigation"),
        "Function keys" => Some("legacy_static.function_keys"),
        "Modifiers" => Some("legacy_static.modifiers"),
        "Other keys" => Some("legacy_static.other_keys"),
        "Special QMK keys" => Some("legacy_static.special_qmk_keys"),
        "Mouse" => Some("legacy_static.mouse"),
        "Media, Apps, System" => Some("legacy_static.media_apps_system"),
        "OS / edit shortcuts" => Some("legacy_static.os_edit_shortcuts"),
        "Numpad" => Some("legacy_static.numpad"),
        "Space Cadet" => Some("legacy_static.space_cadet"),
        "International" => Some("legacy_static.international"),
        "Backlight" => Some("legacy_static.backlight"),
        "RGB Underglow" => Some("legacy_static.rgb_underglow"),
        "RGB Matrix Modes" => Some("legacy_static.rgb_matrix_modes"),
        "RGB Matrix Controls" => Some("legacy_static.rgb_matrix_controls"),
        "Hold to activate, release to return" => Some("legacy_static.hold_to_activate_release_to_return"),
        "Toggle layer on/off" => Some("legacy_static.toggle_layer_on_off"),
        "Tap to toggle on/off" => Some("legacy_static.tap_to_toggle_on_off"),
        "Active for next keypress only" => Some("legacy_static.active_for_next_keypress_only"),
        "Hold = MO, tap = toggle" => Some("legacy_static.hold_mo_tap_toggle"),
        "Tap multiple times to toggle layer" => Some("legacy_static.tap_multiple_times_to_toggle_layer"),
        "Switch and stay on this layer" => Some("legacy_static.switch_and_stay_on_this_layer"),
        "Set as permanent base layer" => Some("legacy_static.set_as_permanent_base_layer"),
        "Hold = activate layer, tap = keycode (set key via right-click afterwards)" => Some("legacy_static.hold_activate_layer_tap_keycode_set_key_via_right_click_afterwards"),
        "Toggle backlight on/off" => Some("legacy_static.toggle_backlight_on_off"),
        "Cycle through backlight brightness levels" => Some("legacy_static.cycle_through_backlight_brightness_levels"),
        "Toggle breathing effect on/off" => Some("legacy_static.toggle_breathing_effect_on_off"),
        "Turn backlight on" => Some("legacy_static.turn_backlight_on"),
        "Turn backlight off" => Some("legacy_static.turn_backlight_off"),
        "Decrease backlight brightness" => Some("legacy_static.decrease_backlight_brightness"),
        "Increase backlight brightness" => Some("legacy_static.increase_backlight_brightness"),
        "Toggle RGB lighting on/off" => Some("legacy_static.toggle_rgb_lighting_on_off"),
        "Switch to previous RGB animation mode" => Some("legacy_static.switch_to_previous_rgb_animation_mode"),
        "Switch to next RGB animation mode" => Some("legacy_static.switch_to_next_rgb_animation_mode"),
        "Decrease color hue" => Some("legacy_static.decrease_color_hue"),
        "Increase color hue" => Some("legacy_static.increase_color_hue"),
        "Decrease color saturation" => Some("legacy_static.decrease_color_saturation"),
        "Increase color saturation" => Some("legacy_static.increase_color_saturation"),
        "Decrease brightness" => Some("legacy_static.decrease_brightness"),
        "Increase brightness" => Some("legacy_static.increase_brightness"),
        "Decrease animation speed" => Some("legacy_static.decrease_animation_speed"),
        "Increase animation speed" => Some("legacy_static.increase_animation_speed"),
        "Previous RGB effect" => Some("legacy_static.previous_rgb_effect"),
        "Next RGB effect" => Some("legacy_static.next_rgb_effect"),
        "Turn RGB Matrix on" => Some("legacy_static.turn_rgb_matrix_on"),
        "Turn RGB Matrix off" => Some("legacy_static.turn_rgb_matrix_off"),
        "Toggle RGB Matrix on/off" => Some("legacy_static.toggle_rgb_matrix_on_off"),
        "Previous RGB Matrix animation" => Some("legacy_static.previous_rgb_matrix_animation"),
        "Next RGB Matrix animation" => Some("legacy_static.next_rgb_matrix_animation"),
        "Decrease RGB Matrix hue" => Some("legacy_static.decrease_rgb_matrix_hue"),
        "Increase RGB Matrix hue" => Some("legacy_static.increase_rgb_matrix_hue"),
        "Decrease RGB Matrix saturation" => Some("legacy_static.decrease_rgb_matrix_saturation"),
        "Increase RGB Matrix saturation" => Some("legacy_static.increase_rgb_matrix_saturation"),
        "Decrease RGB Matrix brightness" => Some("legacy_static.decrease_rgb_matrix_brightness"),
        "Increase RGB Matrix brightness" => Some("legacy_static.increase_rgb_matrix_brightness"),
        "Decrease RGB Matrix animation speed" => Some("legacy_static.decrease_rgb_matrix_animation_speed"),
        "Increase RGB Matrix animation speed" => Some("legacy_static.increase_rgb_matrix_animation_speed"),
        "Rose" => Some("legacy_static.rose"),
        "Violet" => Some("legacy_static.violet"),
        "Blue" => Some("legacy_static.blue"),
        "Amber" => Some("legacy_static.amber"),
        "Copper" => Some("legacy_static.copper"),
        "Teal" => Some("legacy_static.teal"),
        "White" => Some("legacy_static.white"),
        "Red" => Some("legacy_static.red"),
        "Orange" => Some("legacy_static.orange"),
        "Goldenrod" => Some("legacy_static.goldenrod"),
        "Gold" => Some("legacy_static.gold"),
        "Yellow" => Some("legacy_static.yellow"),
        "Chartreuse" => Some("legacy_static.chartreuse"),
        "Lime" => Some("legacy_static.lime"),
        "Green" => Some("legacy_static.green"),
        "Spring Green" => Some("legacy_static.spring_green"),
        "Turquoise" => Some("legacy_static.turquoise"),
        "Cyan" => Some("legacy_static.cyan"),
        "Azure" => Some("legacy_static.azure"),
        "Sky" => Some("legacy_static.sky"),
        "Indigo" => Some("legacy_static.indigo"),
        "Purple" => Some("legacy_static.purple"),
        "Magenta" => Some("legacy_static.magenta"),
        "Pink" => Some("legacy_static.pink"),
        "Coral" => Some("legacy_static.coral"),
        "Salmon" => Some("legacy_static.salmon"),
        "Warm White" => Some("legacy_static.warm_white"),
        "Trigger" => Some("legacy_static.trigger"),
        "Replacement" => Some("legacy_static.replacement"),
        "Suppressed mods" => Some("legacy_static.suppressed_mods"),
        "Trigger mods" => Some("legacy_static.trigger_mods"),
        "Negative mods" => Some("legacy_static.negative_mods"),
        "Enable on layers" => Some("legacy_static.enable_on_layers"),
        "Pick trigger" => Some("legacy_static.pick_trigger"),
        "Pick replacement" => Some("legacy_static.pick_replacement"),
        "None" => Some("legacy_static.none"),
        "All mods" => Some("legacy_static.all_mods"),
        "No layers" => Some("legacy_static.no_layers"),
        "All layers" => Some("legacy_static.all_layers"),
        "Enable all" => Some("legacy_static.enable_all"),
        "Disable all" => Some("legacy_static.disable_all"),
        "Trigger press" => Some("legacy_static.trigger_press"),
        "Required mod press" => Some("legacy_static.required_mod_press"),
        "Blocked mod release" => Some("legacy_static.blocked_mod_release"),
        "Any one mod" => Some("legacy_static.any_one_mod"),
        "No re-send" => Some("legacy_static.no_re_send"),
        "Stay active" => Some("legacy_static.stay_active"),
        "Input keys" => Some("legacy_static.input_keys"),
        "Output key" => Some("legacy_static.output_key"),
        "Select Key Override slot" => Some("legacy_static.select_key_override_slot"),
        "Local name for this Key Override slot" => Some("legacy_static.local_name_for_this_key_override_slot"),
        "Original key that can be overridden" => Some("legacy_static.original_key_that_can_be_overridden"),
        "Keycode sent while override conditions match" => Some("legacy_static.keycode_sent_while_override_conditions_match"),
        "Modifiers hidden while the replacement is active" => Some("legacy_static.modifiers_hidden_while_the_replacement_is_active"),
        "Modifiers required for this override" => Some("legacy_static.modifiers_required_for_this_override"),
        "Modifiers that block this override" => Some("legacy_static.modifiers_that_block_this_override"),
        "Layers where this override can activate" => Some("legacy_static.layers_where_this_override_can_activate"),
        "Activate when the trigger key is pressed" => Some("legacy_static.activate_when_the_trigger_key_is_pressed"),
        "Activate when a required modifier is pressed" => Some("legacy_static.activate_when_a_required_modifier_is_pressed"),
        "Activate when a blocking modifier is released" => Some("legacy_static.activate_when_a_blocking_modifier_is_released"),
        "Any one trigger modifier is enough" => Some("legacy_static.any_one_trigger_modifier_is_enough"),
        "Do not resend the trigger after override ends" => Some("legacy_static.do_not_resend_the_trigger_after_override_ends"),
        "Stay active when another key is pressed" => Some("legacy_static.stay_active_when_another_key_is_pressed"),
        "Select Combo slot" => Some("legacy_static.select_combo_slot"),
        "Local name for this combo slot" => Some("legacy_static.local_name_for_this_combo_slot"),
        "Keys that must be pressed together" => Some("legacy_static.keys_that_must_be_pressed_together"),
        "Keycode sent when the combo activates" => Some("legacy_static.keycode_sent_when_the_combo_activates"),
        "Maximum time between combo key presses" => Some("legacy_static.maximum_time_between_combo_key_presses"),
        "Press 2-4 keys" => Some("legacy_static.press_2_4_keys"),
        "Record 2-4 keys" => Some("legacy_static.record_2_4_keys"),
        "Pick output" => Some("legacy_static.pick_output"),
        "Hold actions are limited to left/right modifiers and layers" => Some("legacy_static.hold_actions_are_limited_to_left_right_modifiers_and_layers"),
        "Tap-then-hold actions are limited to left/right modifiers and layers" => Some("legacy_static.tap_then_hold_actions_are_limited_to_left_right_modifiers_and_layers"),
        "Left Control" => Some("legacy_static.left_control"),
        "Right Control" => Some("legacy_static.right_control"),
        "Left Shift" => Some("legacy_static.left_shift"),
        "Right Shift" => Some("legacy_static.right_shift"),
        "Left Alt" => Some("legacy_static.left_alt"),
        "Right Alt" => Some("legacy_static.right_alt"),
        "Clear all" => Some("legacy_static.clear_all"),
        "↩ Undo" => Some("legacy_static.undo_undo"),
        "Undo last change" => Some("legacy_static.undo_last_change"),
        "Remove all actions from this macro" => Some("legacy_static.remove_all_actions_from_this_macro"),
        "Select a tap dance tab above to edit" => Some("legacy_static.select_a_tap_dance_tab_above_to_edit"),
        "Key sent on single tap" => Some("legacy_static.key_sent_on_single_tap"),
        "Key sent when held" => Some("legacy_static.key_sent_when_held"),
        "Key sent on double tap" => Some("legacy_static.key_sent_on_double_tap"),
        "Key sent on tap then hold" => Some("legacy_static.key_sent_on_tap_then_hold"),
        "Best for a second tap action, usually another normal key or command" => Some("legacy_static.best_for_a_second_tap_action_usually_another_normal_key_or_command"),
        "Basic keys — standard keyboard layout" => Some("legacy_static.basic_keys_standard_keyboard_layout"),
        "Universal symbols — same output in any language" => Some("legacy_static.universal_symbols_same_output_in_any_language"),
        "Layout symbols — follow the active keyboard language" => Some("legacy_static.layout_symbols_follow_the_active_keyboard_language"),
        "Extra universal symbols — typography and math" => Some("legacy_static.extra_universal_symbols_typography_and_math"),
        "Custom keycodes — defined by this keyboard" => Some("legacy_static.custom_keycodes_defined_by_this_keyboard"),
        "Layers: choose a layer action, then pick the target layer" => Some("legacy_static.layers_choose_a_layer_action_then_pick_the_target_layer"),
        "Plain modifiers" => Some("legacy_static.plain_modifiers"),
        "Mod+Key — always sends modifier+key together" => Some("legacy_static.mod_plus_key_always_sends_modifier_plus_key_together"),
        "Mod-Tap — hold for modifier, tap for regular key" => Some("legacy_static.mod_tap_hold_for_modifier_tap_for_regular_key"),
        "One-Shot Mod — active for next keypress only" => Some("legacy_static.one_shot_mod_active_for_next_keypress_only"),
        "Choose the key to pair with the modifier" => Some("legacy_static.choose_the_key_to_pair_with_the_modifier"),
        "This key will always be sent together with the selected modifier" => Some("legacy_static.this_key_will_always_be_sent_together_with_the_selected_modifier"),
        "Choose the tap key" => Some("legacy_static.choose_the_tap_key"),
        "Hold will send the modifier; tap will send the key you pick" => Some("legacy_static.hold_will_send_the_modifier_tap_will_send_the_key_you_pick"),
        "✕ Cancel" => Some("legacy_static.cancel_cancel"),
        "Mod+Key — pick modifier, then key" => Some("legacy_static.mod_plus_key_pick_modifier_then_key"),
        "Mod-Tap — pick modifier, then tap key" => Some("legacy_static.mod_tap_pick_modifier_then_tap_key"),
        "Choose macro" => Some("legacy_static.choose_macro"),
        "Select a macro above to edit" => Some("legacy_static.select_a_macro_above_to_edit"),
        "Macro name" => Some("legacy_static.macro_name"),
        "Move up" => Some("legacy_static.move_up"),
        "Move down" => Some("legacy_static.move_down"),
        "Text" => Some("legacy_static.text"),
        "Types text characters one by one" => Some("legacy_static.types_text_characters_one_by_one"),
        "Tap" => Some("legacy_static.tap"),
        "Press and release a key" => Some("legacy_static.press_and_release_a_key"),
        "Down" => Some("legacy_static.down"),
        "Press a key (hold until Up)" => Some("legacy_static.press_a_key_hold_until_up"),
        "Up" => Some("legacy_static.up"),
        "Release a previously pressed key" => Some("legacy_static.release_a_previously_pressed_key"),
        "Delay" => Some("legacy_static.delay"),
        "Wait before next action" => Some("legacy_static.wait_before_next_action"),
        "Type text here" => Some("legacy_static.type_text_here"),
        "Characters to type when this macro runs" => Some("legacy_static.characters_to_type_when_this_macro_runs"),
        "Click to change key — press and release this key" => Some("legacy_static.click_to_change_key_press_and_release_this_key"),
        "Click to change key — holds down until Up" => Some("legacy_static.click_to_change_key_holds_down_until_up"),
        "Click to change key — releases this key" => Some("legacy_static.click_to_change_key_releases_this_key"),
        "Delay is in milliseconds" => Some("legacy_static.delay_is_in_milliseconds"),
        "Remove this action" => Some("legacy_static.remove_this_action"),
        "+ Text" => Some("legacy_static.plus_text"),
        "Type characters" => Some("legacy_static.type_characters"),
        "+ Tap" => Some("legacy_static.plus_tap"),
        "+ Down" => Some("legacy_static.plus_down"),
        "Hold a key" => Some("legacy_static.hold_a_key"),
        "+ Up" => Some("legacy_static.plus_up"),
        "Release a key" => Some("legacy_static.release_a_key"),
        "+ Delay" => Some("legacy_static.plus_delay"),
        "Pause in milliseconds" => Some("legacy_static.pause_in_milliseconds"),
        "No Tap Dance slots available on this keyboard" => Some("legacy_static.no_tap_dance_slots_available_on_this_keyboard"),
        "Choose tap dance" => Some("legacy_static.choose_tap_dance"),
        "TD name" => Some("legacy_static.td_name"),
        "On Tap" => Some("legacy_static.on_tap"),
        "On Hold" => Some("legacy_static.on_hold"),
        "On Double Tap" => Some("legacy_static.on_double_tap"),
        "On Tap + Hold" => Some("legacy_static.on_tap_plus_hold"),
        "Click to assign a key" => Some("legacy_static.click_to_assign_a_key"),
        "Tapping Term" => Some("legacy_static.tapping_term"),
        "Time in milliseconds to distinguish tap from hold (default: 200)" => Some("legacy_static.time_in_milliseconds_to_distinguish_tap_from_hold_default_200"),
        "Tapping term is in milliseconds" => Some("legacy_static.tapping_term_is_in_milliseconds"),
        "Clear all actions for this tap dance" => Some("legacy_static.clear_all_actions_for_this_tap_dance"),
        "Undo last tap dance change" => Some("legacy_static.undo_last_tap_dance_change"),
        "Tap Dance Editor" => Some("legacy_static.tap_dance_editor"),
        "Keyboard is locked, unlock it to use Matrix Tester" => Some("legacy_static.keyboard_is_locked_unlock_it_to_use_matrix_tester"),
        "Click to reset Matrix Tester" => Some("legacy_static.click_to_reset_matrix_tester"),
        "Matrix Tester is currently available only for Vial keyboards" => Some("legacy_static.matrix_tester_is_currently_available_only_for_vial_keyboards"),
        "Connect a Vial keyboard to start live switch testing" => Some("legacy_static.connect_a_vial_keyboard_to_start_live_switch_testing"),
        "Click Tested to reset progress" => Some("legacy_static.click_tested_to_reset_progress"),
        "Tested" => Some("legacy_static.tested"),
        "Toggle firmware layout/display option" => Some("legacy_static.toggle_firmware_layout_display_option"),
        "Choose firmware preset" => Some("legacy_static.choose_firmware_preset"),
        "Connect a Vial keyboard to edit Auto Shift settings" => Some("legacy_static.connect_a_vial_keyboard_to_edit_auto_shift_settings"),
        "Enable" => Some("legacy_static.enable"),
        "Turn Auto Shift on or off" => Some("legacy_static.turn_auto_shift_on_or_off"),
        "Enable for modifiers" => Some("legacy_static.enable_for_modifiers"),
        "Allow Auto Shift behavior on modifier keys" => Some("legacy_static.allow_auto_shift_behavior_on_modifier_keys"),
        "No special keys" => Some("legacy_static.no_special_keys"),
        "Do not Auto Shift special keys such as Enter, Esc, Tab or Backspace" => Some("legacy_static.do_not_auto_shift_special_keys_such_as_enter_esc_tab_or_backspace"),
        "No numeric keys" => Some("legacy_static.no_numeric_keys"),
        "Do not Auto Shift number keys" => Some("legacy_static.do_not_auto_shift_number_keys"),
        "No alpha keys" => Some("legacy_static.no_alpha_keys"),
        "Do not Auto Shift letter keys" => Some("legacy_static.do_not_auto_shift_letter_keys"),
        "Enable keyrepeat" => Some("legacy_static.enable_keyrepeat"),
        "Allow held Auto Shift keys to repeat" => Some("legacy_static.allow_held_auto_shift_keys_to_repeat"),
        "Stop repeat after timeout" => Some("legacy_static.stop_repeat_after_timeout"),
        "Disable key repeat after the Auto Shift timeout is exceeded" => Some("legacy_static.disable_key_repeat_after_the_auto_shift_timeout_is_exceeded"),
        "Timeout" => Some("legacy_static.timeout"),
        "Hold time before Auto Shift sends the shifted key" => Some("legacy_static.hold_time_before_auto_shift_sends_the_shifted_key"),
        "Timeout is in milliseconds" => Some("legacy_static.timeout_is_in_milliseconds"),
        "Light" => Some("legacy_static.light"),
        "Dark" => Some("legacy_static.dark"),
        "☀ Light" => Some("legacy_static.light_light"),
        "🌙 Dark" => Some("legacy_static.dark_dark"),
        "🔓 Unlock Keyboard" => Some("legacy_static.unlock_unlock_keyboard"),
        "Unlock Keyboard" => Some("legacy_static.unlock_keyboard"),
        "Macros saved" => Some("legacy_static.macros_saved"),
        "Combos saved" => Some("legacy_static.combos_saved"),
        "Combo timeout saved" => Some("legacy_static.combo_timeout_saved"),
        "Entry" => Some("legacy_static.entry"),
        "Select Alt Repeat slot" => Some("legacy_static.select_alt_repeat_slot"),
        "Name" => Some("legacy_static.name"),
        "Local name for this slot" => Some("legacy_static.local_name_for_this_slot"),
        "Stored locally in Entropy" => Some("legacy_static.stored_locally_in_entropy"),
        "Last key" => Some("legacy_static.last_key"),
        "Key that triggers alternate repeat behavior" => Some("legacy_static.key_that_triggers_alternate_repeat_behavior"),
        "Alt key" => Some("legacy_static.alt_key"),
        "Key repeated when alternate repeat activates" => Some("legacy_static.key_repeated_when_alternate_repeat_activates"),
        "Ctrl mods" => Some("legacy_static.ctrl_mods"),
        "Shift mods" => Some("legacy_static.shift_mods"),
        "Alt mods" => Some("legacy_static.alt_mods"),
        "Allowed Ctrl modifiers" => Some("legacy_static.allowed_ctrl_modifiers"),
        "Allowed Shift modifiers" => Some("legacy_static.allowed_shift_modifiers"),
        "Allowed Alt modifiers" => Some("legacy_static.allowed_alt_modifiers"),
        "Allowed OS modifiers" => Some("legacy_static.allowed_os_modifiers"),
        "Right-side modifier" => Some("legacy_static.right_side_modifier"),
        "Left-side modifier" => Some("legacy_static.left_side_modifier"),
        "Default alt key" => Some("legacy_static.default_alt_key"),
        "Use this alt key by default" => Some("legacy_static.use_this_alt_key_by_default"),
        "Bidirectional" => Some("legacy_static.bidirectional"),
        "Allow both keys to alternate each other" => Some("legacy_static.allow_both_keys_to_alternate_each_other"),
        "Ignore handedness" => Some("legacy_static.ignore_handedness"),
        "Treat left and right modifiers as equivalent" => Some("legacy_static.treat_left_and_right_modifiers_as_equivalent"),
        "Clear" => Some("legacy_static.clear"),
        "Undo" => Some("legacy_static.undo"),
        "↶ Undo" => Some("legacy_static.undo_undo_6840d526"),
        "LED brightness" => Some("legacy_static.led_brightness"),
        "Global LED brightness for layer color lighting" => Some("legacy_static.global_led_brightness_for_layer_color_lighting"),
        "LED timeout" => Some("legacy_static.led_timeout"),
        "Minutes before LEDs turn off automatically, 0 disables timeout" => Some("legacy_static.minutes_before_leds_turn_off_automatically_0_disables_timeout"),
        "Off" => Some("legacy_static.off"),
        "Alt forces Esc" => Some("legacy_static.alt_forces_esc"),
        "When Alt is held, Grave Escape sends Esc instead of ` or ~" => Some("legacy_static.when_alt_is_held_grave_escape_sends_esc_instead_of_grave_or_tilde"),
        "Control forces Esc" => Some("legacy_static.control_forces_esc"),
        "When Control is held, Grave Escape sends Esc instead of ` or ~" => Some("legacy_static.when_control_is_held_grave_escape_sends_esc_instead_of_grave_or_tilde"),
        "Shift forces Esc" => Some("legacy_static.shift_forces_esc"),
        "When Shift is held, Grave Escape sends Esc instead of ` or ~" => Some("legacy_static.when_shift_is_held_grave_escape_sends_esc_instead_of_grave_or_tilde"),
        "Swap Caps Lock and Left Control" => Some("legacy_static.swap_caps_lock_and_left_control"),
        "Caps Lock sends Left Control and Left Control sends Caps Lock" => Some("legacy_static.caps_lock_sends_left_control_and_left_control_sends_caps_lock"),
        "Treat Caps Lock as Control" => Some("legacy_static.treat_caps_lock_as_control"),
        "Caps Lock sends Control without swapping Left Control" => Some("legacy_static.caps_lock_sends_control_without_swapping_left_control"),
        "Disable OS keys" => Some("legacy_static.disable_os_keys"),
        "Ignore both OS keys while this option is enabled" => Some("legacy_static.ignore_both_os_keys_while_this_option_is_enabled"),
        "Swap ` and Escape" => Some("legacy_static.swap_grave_and_escape"),
        "Grave sends Escape and Escape sends Grave" => Some("legacy_static.grave_sends_escape_and_escape_sends_grave"),
        "Swap \\ and Backspace" => Some("legacy_static.swap_backslash_and_backspace"),
        "Backslash sends Backspace and Backspace sends Backslash" => Some("legacy_static.backslash_sends_backspace_and_backspace_sends_backslash"),
        "Enable N-key rollover" => Some("legacy_static.enable_n_key_rollover"),
        "Allow more simultaneous key presses when the keyboard supports it" => Some("legacy_static.allow_more_simultaneous_key_presses_when_the_keyboard_supports_it"),
        "Tapping term" => Some("legacy_static.tapping_term_cbc17e7a"),
        "Global tap-vs-hold decision window for dual-role keys" => Some("legacy_static.global_tap_vs_hold_decision_window_for_dual_role_keys"),
        "Permissive hold" => Some("legacy_static.permissive_hold"),
        "Nested taps choose hold for Mod-Tap and Layer-Tap keys" => Some("legacy_static.nested_taps_choose_hold_for_mod_tap_and_layer_tap_keys"),
        "Hold on other key" => Some("legacy_static.hold_on_other_key"),
        "Pressing another key immediately chooses hold for dual-role keys" => Some("legacy_static.pressing_another_key_immediately_chooses_hold_for_dual_role_keys"),
        "Retro tapping" => Some("legacy_static.retro_tapping"),
        "A held-and-released-alone dual-role key still sends its tap action" => Some("legacy_static.a_held_and_released_alone_dual_role_key_still_sends_its_tap_action"),
        "Chordal hold" => Some("legacy_static.chordal_hold"),
        "Same-hand chords prefer tap to reduce home-row mod accidents" => Some("legacy_static.same_hand_chords_prefer_tap_to_reduce_home_row_mod_accidents"),
        "Quick tap term" => Some("legacy_static.quick_tap_term"),
        "Tap-then-hold repeat window for dual-role key tap actions" => Some("legacy_static.tap_then_hold_repeat_window_for_dual_role_key_tap_actions"),
        "Tap code delay" => Some("legacy_static.tap_code_delay"),
        "Delay between register and unregister in tap_code" => Some("legacy_static.delay_between_register_and_unregister_in_tap_code"),
        "Tap hold caps delay" => Some("legacy_static.tap_hold_caps_delay"),
        "Extra delay for LT/MT keys whose tap action is Caps Lock" => Some("legacy_static.extra_delay_for_lt_mt_keys_whose_tap_action_is_caps_lock"),
        "Tapping toggle" => Some("legacy_static.tapping_toggle"),
        "Number of taps needed for TT layer toggle" => Some("legacy_static.number_of_taps_needed_for_tt_layer_toggle"),
        "Flow tap" => Some("legacy_static.flow_tap"),
        "Fast typing timeout that forces MT/LT keys to tap" => Some("legacy_static.fast_typing_timeout_that_forces_mt_lt_keys_to_tap"),
        "One Shot Keys" => Some("legacy_static.one_shot_keys"),
        "One-shot tap toggle" => Some("legacy_static.one_shot_tap_toggle"),
        "Tap this many times to keep a one-shot key held until tapped again" => Some("legacy_static.tap_this_many_times_to_keep_a_one_shot_key_held_until_tapped_again"),
        "One-shot timeout" => Some("legacy_static.one_shot_timeout"),
        "How long one-shot state waits before it is released" => Some("legacy_static.how_long_one_shot_state_waits_before_it_is_released"),
        "Value is in milliseconds" => Some("legacy_static.value_is_in_milliseconds"),
        "DPI" => Some("legacy_static.dpi"),
        "Touchpad pointer resolution in dots per inch" => Some("legacy_static.touchpad_pointer_resolution_in_dots_per_inch"),
        "Sniper sens" => Some("legacy_static.sniper_sens"),
        "Sniper divisor: lower is faster, higher is more precise" => Some("legacy_static.sniper_divisor_lower_is_faster_higher_is_more_precise"),
        "Scroll sens" => Some("legacy_static.scroll_sens"),
        "Scroll divisor: lower is faster, higher is smoother" => Some("legacy_static.scroll_divisor_lower_is_faster_higher_is_smoother"),
        "Text sens" => Some("legacy_static.text_sens"),
        "Text mode divisor: lower is faster, higher is slower" => Some("legacy_static.text_mode_divisor_lower_is_faster_higher_is_slower"),
        "Invert scroll" => Some("legacy_static.invert_scroll"),
        "Reverse the touchpad scroll direction" => Some("legacy_static.reverse_the_touchpad_scroll_direction"),
        "Acceleration" => Some("legacy_static.acceleration"),
        "Use firmware pointer acceleration for touchpad movement" => Some("legacy_static.use_firmware_pointer_acceleration_for_touchpad_movement"),
        "Sticky mode" => Some("legacy_static.sticky_mode"),
        "Keep the selected touchpad mode active until another mode is selected" => Some("legacy_static.keep_the_selected_touchpad_mode_active_until_another_mode_is_selected"),
        "Auto layer enable" => Some("legacy_static.auto_layer_enable"),
        "Automatically switch to the selected layer while the touchpad is active" => Some("legacy_static.automatically_switch_to_the_selected_layer_while_the_touchpad_is_activ"),
        "Auto layer" => Some("legacy_static.auto_layer"),
        "Layer selected automatically while the touchpad is active" => Some("legacy_static.layer_selected_automatically_while_the_touchpad_is_active"),
        "Entropy background" => Some("legacy_static.entropy_background"),
        "Keep Entropy running in the background for live firmware data" => Some("legacy_static.keep_entropy_running_in_the_background_for_live_firmware_data"),
        "Time sync" => Some("legacy_static.time_sync"),
        "Uses the local system clock" => Some("legacy_static.uses_the_local_system_clock"),
        "Volume sync" => Some("legacy_static.volume_sync"),
        "native Windows audio" => Some("legacy_static.native_windows_audio"),
        "Uses the Windows default output device" => Some("legacy_static.uses_the_windows_default_output_device"),
        "Uses PipeWire default sink volume" => Some("legacy_static.uses_pipewire_default_sink_volume"),
        "Uses PulseAudio/PipeWire Pulse default sink volume" => Some("legacy_static.uses_pulseaudio_pipewire_pulse_default_sink_volume"),
        "missing wpctl/pactl" => Some("legacy_static.missing_wpctl_pactl"),
        "Install wireplumber or pulseaudio-utils/pavucontrol package for volume sync" => Some("legacy_static.install_wireplumber_or_pulseaudio_utils_pavucontrol_package_for_volume"),
        "Uses macOS system output volume" => Some("legacy_static.uses_macos_system_output_volume"),
        "unsupported OS" => Some("legacy_static.unsupported_os"),
        "Volume sync is implemented for Windows, Linux and macOS" => Some("legacy_static.volume_sync_is_implemented_for_windows_linux_and_macos"),
        "native Windows media session" => Some("legacy_static.native_windows_media_session"),
        "Uses Windows global media session metadata" => Some("legacy_static.uses_windows_global_media_session_metadata"),
        "Uses MPRIS metadata from the active player" => Some("legacy_static.uses_mpris_metadata_from_the_active_player"),
        "missing playerctl" => Some("legacy_static.missing_playerctl"),
        "Install playerctl and use an MPRIS-compatible player for media info" => Some("legacy_static.install_playerctl_and_use_an_mpris_compatible_player_for_media_info"),
        "Spotify / Music via AppleScript" => Some("legacy_static.spotify_music_via_applescript"),
        "macOS may ask for Automation permission for Entropy, System Events, Spotify or Music" => Some("legacy_static.macos_may_ask_for_automation_permission_for_entropy_system_events_spot"),
        "Media sync is implemented for Windows, Linux and macOS" => Some("legacy_static.media_sync_is_implemented_for_windows_linux_and_macos"),
        "Media info" => Some("legacy_static.media_info"),
        "ready" => Some("legacy_static.ready"),
        "needs setup" => Some("legacy_static.needs_setup"),
        "active" => Some("legacy_static.active"),
        "starting" => Some("legacy_static.starting"),
        _ => None,
    }
}

pub fn tr_static(language: Language, text: &'static str) -> &'static str {
    legacy_static_key(text)
        .map(|key| tr_catalog(language, key))
        .unwrap_or(text)
}

fn legacy_text_key(text: &str) -> Option<&'static str> {
    match text {
        "No extra setup is required on Windows" => Some("legacy_text.no_extra_setup_is_required_on_windows"),
        "Keep Entropy running while using Universal Symbols" => Some("legacy_text.keep_entropy_running_while_using_universal_symbols"),
        "Assign keys from Symbols → Universal symbols in the key picker" => Some("legacy_text.assign_keys_from_symbols_to_universal_symbols_in_the_key_picker"),
        "Open Privacy & Security" => Some("legacy_text.open_privacy_and_security"),
        "Allow Entropy in Accessibility" => Some("legacy_text.allow_entropy_in_accessibility"),
        "If prompted, allow Entropy in Input Monitoring too" => Some("legacy_text.if_prompted_allow_entropy_in_input_monitoring_too"),
        "Restart Entropy after changing permissions" => Some("legacy_text.restart_entropy_after_changing_permissions"),
        "X11: install xdotool and keep Entropy running" => Some("legacy_text.x11_install_xdotool_and_keep_entropy_running"),
        "Wayland + IBus: install Entropy Universal Symbols and select it as an input source" => Some("legacy_text.wayland_plus_ibus_install_entropy_universal_symbols_and_select_it_as_a"),
        "Wayland + Fcitx5: install the addon, restart Fcitx5, and enable Entropy Universal Symbols" => Some("legacy_text.wayland_plus_fcitx5_install_the_addon_restart_fcitx5_and_enable_entrop"),
        "Universal Symbols are not supported on this OS yet" => Some("legacy_text.universal_symbols_are_not_supported_on_this_os_yet"),
        "Open Config → Universal Symbols to finish permissions setup" => Some("legacy_text.open_config_to_universal_symbols_to_finish_permissions_setup"),
        "Open Config → Universal Symbols to finish Linux setup" => Some("legacy_text.open_config_to_universal_symbols_to_finish_linux_setup"),
        "Disabled" => Some("legacy_text.disabled"),
        "OLED master" => Some("legacy_text.oled_master"),
        "OLED slave" => Some("legacy_text.oled_slave"),
        "Clock" => Some("legacy_text.clock"),
        "Volume" => Some("legacy_text.volume"),
        "Media" => Some("legacy_text.media"),
        "default" => Some("legacy_text.default"),
        "Unknown" => Some("legacy_text.unknown"),
        "Universal output backend: Windows native" => Some("legacy_text.universal_output_backend_windows_native"),
        "Universal output backend: macOS native — requires Accessibility/Input Monitoring permission" => Some("legacy_text.universal_output_backend_macos_native_requires_accessibility_input_mon"),
        "Universal output backend: unsupported on this OS" => Some("legacy_text.universal_output_backend_unsupported_on_this_os"),
        "Esc" => Some("legacy_text.esc"),
        "Escape" => Some("legacy_text.escape"),
        "Backspace" => Some("legacy_text.backspace"),
        "Insert" => Some("legacy_text.insert"),
        "Delete" => Some("legacy_text.delete"),
        "Caps Lock" => Some("legacy_text.caps_lock"),
        "Print Screen" => Some("legacy_text.print_screen"),
        "Scroll Lock" => Some("legacy_text.scroll_lock"),
        "Page Up" => Some("legacy_text.page_up"),
        "Page Down" => Some("legacy_text.page_down"),
        "Space" => Some("legacy_text.space"),
        "Menu" => Some("legacy_text.menu"),
        "Pause" => Some("legacy_text.pause"),
        "Home" => Some("legacy_text.home"),
        "End" => Some("legacy_text.end"),
        r#"Left
Ctrl"# => Some("legacy_text.left_ctrl"),
        r#"Right
Ctrl"# => Some("legacy_text.right_ctrl"),
        r#"Left
Shift"# => Some("legacy_text.left_shift"),
        r#"Right
Shift"# => Some("legacy_text.right_shift"),
        r#"Left
Alt"# => Some("legacy_text.left_alt"),
        r#"Right
Alt"# => Some("legacy_text.right_alt"),
        "No key — this key does nothing" => Some("legacy_text.no_key_this_key_does_nothing"),
        "Transparent — uses the key assigned on the layer below" => Some("legacy_text.transparent_uses_the_key_assigned_on_the_layer_below"),
        "Enter — confirm / new line" => Some("legacy_text.enter_confirm_new_line"),
        "Escape — cancel / close" => Some("legacy_text.escape_cancel_close"),
        "Backspace — delete character before cursor" => Some("legacy_text.backspace_delete_character_before_cursor"),
        "Tab — indent / move focus forward" => Some("legacy_text.tab_indent_move_focus_forward"),
        "Caps Lock — toggle uppercase input" => Some("legacy_text.caps_lock_toggle_uppercase_input"),
        "Menu key — open right-click context menu" => Some("legacy_text.menu_key_open_right_click_context_menu"),
        "Minus — type -, Shift gives underscore (_)" => Some("legacy_text.minus_type_shift_gives_underscore"),
        "Equals — type =, Shift gives plus (+)" => Some("legacy_text.equals_type_shift_gives_plus_plus"),
        "Left bracket — type [, Shift gives left brace ({)" => Some("legacy_text.left_bracket_type_shift_gives_left_brace"),
        "Right bracket — type ], Shift gives right brace (})" => Some("legacy_text.right_bracket_type_shift_gives_right_brace"),
        r#"Backslash — type \, Shift gives pipe (|)"# => Some("legacy_text.backslash_type_backslash_shift_gives_pipe"),
        "Non-US hash key — type #, Shift gives tilde (~)" => Some("legacy_text.non_us_hash_key_type_shift_gives_tilde_tilde"),
        "Semicolon key — tap for semicolon (;), Shift gives colon (:)" => Some("legacy_text.semicolon_key_tap_for_semicolon_shift_gives_colon"),
        r#"Quote — type apostrophe ('), Shift gives double quote (")"# => Some("legacy_text.quote_type_apostrophe_shift_gives_double_quote"),
        "Grave accent — type `, Shift gives tilde (~)" => Some("legacy_text.grave_accent_type_grave_shift_gives_tilde_tilde"),
        "Comma — type comma (,), Shift gives less-than (<)" => Some("legacy_text.comma_type_comma_shift_gives_less_than"),
        "Period — type dot (.), Shift gives greater-than (>)" => Some("legacy_text.period_type_dot_shift_gives_greater_than"),
        "Slash — type /, Shift gives question mark (?)" => Some("legacy_text.slash_type_shift_gives_question_mark"),
        r#"Non-US backslash key — type \, Shift gives pipe (|)"# => Some("legacy_text.non_us_backslash_key_type_backslash_shift_gives_pipe"),
        "Home — jump to beginning of line" => Some("legacy_text.home_jump_to_beginning_of_line"),
        "End — jump to end of line" => Some("legacy_text.end_jump_to_end_of_line"),
        "Page Up — scroll up one page" => Some("legacy_text.page_up_scroll_up_one_page"),
        "Page Down — scroll down one page" => Some("legacy_text.page_down_scroll_down_one_page"),
        "Insert — toggle insert/overwrite mode" => Some("legacy_text.insert_toggle_insert_overwrite_mode"),
        "Delete — delete character after cursor" => Some("legacy_text.delete_delete_character_after_cursor"),
        "Print Screen — take a screenshot" => Some("legacy_text.print_screen_take_a_screenshot"),
        "Pause / Break" => Some("legacy_text.pause_break"),
        "Execute — run the currently selected action or file" => Some("legacy_text.execute_run_the_currently_selected_action_or_file"),
        "Help — open help for the current app or context" => Some("legacy_text.help_open_help_for_the_current_app_or_context"),
        "Select — select the current item" => Some("legacy_text.select_select_the_current_item"),
        "Stop — cancel the current action or loading" => Some("legacy_text.stop_cancel_the_current_action_or_loading"),
        "Again — repeat the previous action" => Some("legacy_text.again_repeat_the_previous_action"),
        "Undo — revert the last action" => Some("legacy_text.undo_revert_the_last_action"),
        "Cut — remove selection and copy it to clipboard" => Some("legacy_text.cut_remove_selection_and_copy_it_to_clipboard"),
        "Copy — copy selection to clipboard" => Some("legacy_text.copy_copy_selection_to_clipboard"),
        "Paste — insert clipboard contents" => Some("legacy_text.paste_insert_clipboard_contents"),
        "Find — search in the current document or view" => Some("legacy_text.find_search_in_the_current_document_or_view"),
        "Mouse acceleration 0 — slowest cursor speed profile" => Some("legacy_text.mouse_acceleration_0_slowest_cursor_speed_profile"),
        "Mouse acceleration 1 — medium cursor speed profile" => Some("legacy_text.mouse_acceleration_1_medium_cursor_speed_profile"),
        "Mouse acceleration 2 — fastest cursor speed profile" => Some("legacy_text.mouse_acceleration_2_fastest_cursor_speed_profile"),
        r#"JIS \ and _"# => Some("legacy_text.jis_backslash_and"),
        "JIS Katakana/Hiragana" => Some("legacy_text.jis_katakana_hiragana"),
        "JIS ¥ and |" => Some("legacy_text.jis_and"),
        "JIS Henkan" => Some("legacy_text.jis_henkan"),
        "JIS Muhenkan" => Some("legacy_text.jis_muhenkan"),
        "JIS Numpad ," => Some("legacy_text.jis_numpad"),
        "Hangul/English" => Some("legacy_text.hangul_english"),
        "Hanja" => Some("legacy_text.hanja"),
        "JIS Katakana" => Some("legacy_text.jis_katakana"),
        "JIS Hiragana" => Some("legacy_text.jis_hiragana"),
        "JIS Zenkaku/Hankaku" => Some("legacy_text.jis_zenkaku_hankaku"),
        "Compile firmware" => Some("legacy_text.compile_firmware"),
        "RGB lighting — toggle on/off" => Some("legacy_text.rgb_lighting_toggle_on_off"),
        "RGB lighting — next animation mode" => Some("legacy_text.rgb_lighting_next_animation_mode"),
        "RGB lighting — previous animation mode" => Some("legacy_text.rgb_lighting_previous_animation_mode"),
        "RGB lighting — hue +" => Some("legacy_text.rgb_lighting_hue_plus"),
        "RGB lighting — hue −" => Some("legacy_text.rgb_lighting_hue"),
        "RGB lighting — saturation +" => Some("legacy_text.rgb_lighting_saturation_plus"),
        "RGB lighting — saturation −" => Some("legacy_text.rgb_lighting_saturation"),
        "RGB lighting — brightness +" => Some("legacy_text.rgb_lighting_brightness_plus"),
        "RGB lighting — brightness −" => Some("legacy_text.rgb_lighting_brightness"),
        "RGB lighting — animation speed +" => Some("legacy_text.rgb_lighting_animation_speed_plus"),
        "RGB lighting — animation speed −" => Some("legacy_text.rgb_lighting_animation_speed"),
        r#"✕
None"# => Some("legacy_text.cancel_none"),
        r#"▽
Inherit"# => Some("legacy_text.inherit_inherit"),
        r#"🔒
Lock"# => Some("legacy_text.lock_lock"),
        r#"Combo
Toggle"# => Some("legacy_text.combo_toggle"),
        "KC_NO — disables this key completely, it sends nothing when pressed" => Some("legacy_text.kc_no_disables_this_key_completely_it_sends_nothing_when_pressed"),
        "KC_TRNS — inherits the key from the layer below" => Some("legacy_text.kc_trns_inherits_the_key_from_the_layer_below"),
        "Bootloader — put keyboard into flash mode" => Some("legacy_text.bootloader_put_keyboard_into_flash_mode"),
        "Debug toggle — enable/disable debug output" => Some("legacy_text.debug_toggle_enable_disable_debug_output"),
        "Lock — lock a key in pressed state until pressed again" => Some("legacy_text.lock_lock_a_key_in_pressed_state_until_pressed_again"),
        "Toggles the state of the Auto Shift feature" => Some("legacy_text.toggles_the_state_of_the_auto_shift_feature"),
        "Toggles Combo feature on and off" => Some("legacy_text.toggles_combo_feature_on_and_off"),
        "Capitalizes until end of current word" => Some("legacy_text.capitalizes_until_end_of_current_word"),
        "Repeats the last pressed key" => Some("legacy_text.repeats_the_last_pressed_key"),
        "Alt repeats the last pressed key" => Some("legacy_text.alt_repeats_the_last_pressed_key"),
        r#"Mouse
Up"# => Some("legacy_text.mouse_up"),
        r#"Mouse
Down"# => Some("legacy_text.mouse_down"),
        r#"Mouse
Left"# => Some("legacy_text.mouse_left"),
        r#"Mouse
Right"# => Some("legacy_text.mouse_right"),
        r#"Scroll
Up"# => Some("legacy_text.scroll_up"),
        r#"Scroll
Down"# => Some("legacy_text.scroll_down"),
        r#"Scroll
Left"# => Some("legacy_text.scroll_left"),
        r#"Scroll
Right"# => Some("legacy_text.scroll_right"),
        r#"Accel
0"# => Some("legacy_text.accel_0"),
        r#"Accel
1"# => Some("legacy_text.accel_1"),
        r#"Accel
2"# => Some("legacy_text.accel_2"),
        r#"⏻
Power"# => Some("legacy_text.power_power"),
        r#"🌙
Sleep"# => Some("legacy_text.sleep_sleep"),
        r#"☀
Wake"# => Some("legacy_text.sun_wake"),
        r#"🔇
Mute"# => Some("legacy_text.mute_mute"),
        r#"🔉
Vol-"# => Some("legacy_text.vol_down_vol"),
        r#"🔊
Vol+"# => Some("legacy_text.vol_up_vol_plus"),
        r#"⏮
Prev"# => Some("legacy_text.previous_prev"),
        r#"⏭
Next"# => Some("legacy_text.next_next"),
        r#"⏹
Stop"# => Some("legacy_text.stop_stop"),
        r#"🎵
Media"# => Some("legacy_text.media_media"),
        r#"✉
Mail"# => Some("legacy_text.mail_mail"),
        r#"🖩
Calc"# => Some("legacy_text.calc_calc"),
        r#"💻
Files"# => Some("legacy_text.files_files"),
        r#"🔍
Search"# => Some("legacy_text.search_search"),
        r#"⬅
Back"# => Some("legacy_text.back_back"),
        r#"➡
Forward"# => Some("legacy_text.forward_forward"),
        r#"↻
Refresh"# => Some("legacy_text.refresh_refresh"),
        r#"★
Favs"# => Some("legacy_text.favs_favs"),
        r#"⏩
Fwd"# => Some("legacy_text.fast_forward_fwd"),
        r#"⏪
Rew"# => Some("legacy_text.rewind_rew"),
        r#"☀+
Bright"# => Some("legacy_text.sun_plus_bright"),
        r#"☀-
Bright"# => Some("legacy_text.sun_bright"),
        r#"Ctrl
View"# => Some("legacy_text.ctrl_view"),
        r#"Launch
Pad"# => Some("legacy_text.launch_pad"),
        "Mute / Unmute audio" => Some("legacy_text.mute_unmute_audio"),
        "Volume Up" => Some("legacy_text.volume_up"),
        "Volume Down" => Some("legacy_text.volume_down"),
        "Next Track" => Some("legacy_text.next_track"),
        "Previous Track" => Some("legacy_text.previous_track"),
        "Stop playback" => Some("legacy_text.stop_playback"),
        "Play / Pause" => Some("legacy_text.play_pause"),
        "Open media player" => Some("legacy_text.open_media_player"),
        "Open email client" => Some("legacy_text.open_email_client"),
        "Open calculator" => Some("legacy_text.open_calculator"),
        "Open My Computer / file manager" => Some("legacy_text.open_my_computer_file_manager"),
        "Browser search" => Some("legacy_text.browser_search"),
        "Browser home page" => Some("legacy_text.browser_home_page"),
        "Browser back" => Some("legacy_text.browser_back"),
        "Browser forward" => Some("legacy_text.browser_forward"),
        "Browser stop loading" => Some("legacy_text.browser_stop_loading"),
        "Browser refresh" => Some("legacy_text.browser_refresh"),
        "Browser favourites" => Some("legacy_text.browser_favourites"),
        "Sleep — put computer to sleep" => Some("legacy_text.sleep_put_computer_to_sleep"),
        "Wake — wake computer from sleep" => Some("legacy_text.wake_wake_computer_from_sleep"),
        "Brightness Up" => Some("legacy_text.brightness_up"),
        "Brightness Down" => Some("legacy_text.brightness_down"),
        "Power — system power button" => Some("legacy_text.power_system_power_button"),
        "Eject — eject removable media" => Some("legacy_text.eject_eject_removable_media"),
        "Fast Forward — jump forward in media" => Some("legacy_text.fast_forward_jump_forward_in_media"),
        "Rewind — jump backward in media" => Some("legacy_text.rewind_jump_backward_in_media"),
        "Mission Control / Task View — show open windows and spaces" => Some("legacy_text.mission_control_task_view_show_open_windows_and_spaces"),
        "Launchpad / app launcher" => Some("legacy_text.launchpad_app_launcher"),
        "Undo" => Some("legacy_text.undo"),
        "On" => Some("legacy_text.on"),
        "Off" => Some("legacy_text.off"),
        r#"⏯
Play"# => Some("legacy_text.play"),
        r#"⏏
Eject"# => Some("legacy_text.eject"),
        r#"🏠
Home"# => Some("legacy_text.home_e3170d03"),
        "Redo" => Some("legacy_text.redo"),
        "Cut" => Some("legacy_text.cut"),
        "Copy" => Some("legacy_text.copy"),
        "Paste" => Some("legacy_text.paste"),
        "Find" => Some("legacy_text.find"),
        r#"Prev
Word"# => Some("legacy_text.prev_word"),
        r#"Next
Word"# => Some("legacy_text.next_word"),
        r#"Prev
App"# => Some("legacy_text.prev_app"),
        r#"Next
App"# => Some("legacy_text.next_app"),
        "Lock" => Some("legacy_text.lock"),
        "Swap" => Some("legacy_text.swap"),
        "Restore" => Some("legacy_text.restore"),
        "Toggle" => Some("legacy_text.toggle"),
        "as Caps" => Some("legacy_text.as_caps"),
        "as Ctrl" => Some("legacy_text.as_ctrl"),
        "Left" => Some("legacy_text.left"),
        "Right" => Some("legacy_text.right"),
        "Num Lock — toggle numpad number input" => Some("legacy_text.num_lock_toggle_numpad_number_input"),
        "Numpad ÷ (divide)" => Some("legacy_text.numpad_divide"),
        "Numpad × (multiply)" => Some("legacy_text.numpad_multiply"),
        "Numpad − (minus)" => Some("legacy_text.numpad_minus"),
        "Numpad + (plus)" => Some("legacy_text.numpad_plus_plus"),
        "Numpad Enter" => Some("legacy_text.numpad_enter"),
        "Numpad . (decimal point)" => Some("legacy_text.numpad_decimal_point"),
        "Numpad = (equals)" => Some("legacy_text.numpad_equals"),
        "Numpad , (comma)" => Some("legacy_text.numpad_comma"),
        "Left Control when held, ( when tapped" => Some("legacy_text.left_control_when_held_when_tapped"),
        "Right Control when held, ) when tapped" => Some("legacy_text.right_control_when_held_when_tapped"),
        "Left Shift when held, ( when tapped" => Some("legacy_text.left_shift_when_held_when_tapped"),
        "Right Shift when held, ) when tapped" => Some("legacy_text.right_shift_when_held_when_tapped"),
        "Left Alt when held, ( when tapped" => Some("legacy_text.left_alt_when_held_when_tapped"),
        "Right Alt when held, ) when tapped" => Some("legacy_text.right_alt_when_held_when_tapped"),
        "Right Shift when held, Enter when tapped" => Some("legacy_text.right_shift_when_held_enter_when_tapped"),
        "Left Ctrl — modifier key (hold to activate shortcuts)" => Some("legacy_text.left_ctrl_modifier_key_hold_to_activate_shortcuts"),
        "Right Ctrl — modifier key (hold to activate shortcuts)" => Some("legacy_text.right_ctrl_modifier_key_hold_to_activate_shortcuts"),
        "Left Shift — hold to type uppercase / shifted symbols" => Some("legacy_text.left_shift_hold_to_type_uppercase_shifted_symbols"),
        "Right Shift — hold to type uppercase / shifted symbols" => Some("legacy_text.right_shift_hold_to_type_uppercase_shifted_symbols"),
        "Left Alt — modifier key (hold to activate shortcuts)" => Some("legacy_text.left_alt_modifier_key_hold_to_activate_shortcuts"),
        "Right Alt / AltGr — access special characters" => Some("legacy_text.right_alt_altgr_access_special_characters"),
        "Plain modifier — hold for left/right side, tap nothing" => Some("legacy_text.plain_modifier_hold_for_left_right_side_tap_nothing"),
        "Arrow Up" => Some("legacy_text.arrow_up"),
        "Arrow Down" => Some("legacy_text.arrow_down"),
        "Arrow Left" => Some("legacy_text.arrow_left"),
        "Arrow Right" => Some("legacy_text.arrow_right"),
        "Left Control" => Some("legacy_text.left_control"),
        "Right Control" => Some("legacy_text.right_control"),
        "Left Shift" => Some("legacy_text.left_shift_494064e7"),
        "Right Shift" => Some("legacy_text.right_shift_26a36a6d"),
        "Left Alt" => Some("legacy_text.left_alt_c689a8e4"),
        "Right Alt" => Some("legacy_text.right_alt_cdd21f78"),
        "Swap Caps Lock and Left Control" => Some("legacy_text.swap_caps_lock_and_left_control"),
        "Unswap Caps Lock and Left Control" => Some("legacy_text.unswap_caps_lock_and_left_control"),
        "Toggle Caps Lock and Left Control swap" => Some("legacy_text.toggle_caps_lock_and_left_control_swap"),
        "Stop treating Caps Lock as Control" => Some("legacy_text.stop_treating_caps_lock_as_control"),
        "Treat Caps Lock as Control" => Some("legacy_text.treat_caps_lock_as_control"),
        "Swap ` and Escape" => Some("legacy_text.swap_grave_and_escape"),
        "Unswap ` and Escape" => Some("legacy_text.unswap_grave_and_escape"),
        r#"Swap \ and Backspace"# => Some("legacy_text.swap_backslash_and_backspace"),
        r#"Unswap \ and Backspace"# => Some("legacy_text.unswap_backslash_and_backspace"),
        r#"Toggle \ and Backspace swap state"# => Some("legacy_text.toggle_backslash_and_backspace_swap_state"),
        "Enable N-key rollover" => Some("legacy_text.enable_n_key_rollover"),
        "Disable N-key rollover" => Some("legacy_text.disable_n_key_rollover"),
        "Toggle N-key rollover" => Some("legacy_text.toggle_n_key_rollover"),
        "Set the master half of a split keyboard as the left hand (for EE_HANDS)" => Some("legacy_text.set_the_master_half_of_a_split_keyboard_as_the_left_hand_for_ee_hands"),
        "Set the master half of a split keyboard as the right hand (for EE_HANDS)" => Some("legacy_text.set_the_master_half_of_a_split_keyboard_as_the_right_hand_for_ee_hands"),
        "Swap Caps Lock and Escape" => Some("legacy_text.swap_caps_lock_and_escape"),
        "Unswap Caps Lock and Escape" => Some("legacy_text.unswap_caps_lock_and_escape"),
        "Toggle Caps Lock and Escape swap" => Some("legacy_text.toggle_caps_lock_and_escape_swap"),
        _ => None,
    }
}

pub fn tr_text(language: Language, text: &str) -> String {
    if let Some(key) = legacy_text_key(text) {
        return tr_catalog_string(language, key);
    }

    if !matches!(language, Language::Russian) {
        return text.to_owned();
    }

    match text {
        other if other.starts_with("Grave/Escape — sends Esc normally") => other
            .replace(
                "Grave/Escape — sends Esc normally, ` when Shift or",
                "Grave/Escape — обычно отправляет Esc, ` при удержании Shift или",
            )
            .replace("is held", ""),
        other if other.starts_with("Mouse cursor — move ") => other
            .replace("Mouse cursor — move up", "Курсор мыши — вверх")
            .replace("Mouse cursor — move down", "Курсор мыши — вниз")
            .replace("Mouse cursor — move left", "Курсор мыши — влево")
            .replace("Mouse cursor — move right", "Курсор мыши — вправо"),
        other if other.starts_with("Mouse button ") => other
            .replace("Mouse button", "Кнопка мыши")
            .replace("left click", "левый клик")
            .replace("right click", "правый клик")
            .replace("middle click", "средний клик")
            .replace("back", "назад")
            .replace("forward", "вперёд"),
        other if other.starts_with("Mouse wheel — scroll ") => other
            .replace("Mouse wheel — scroll up", "Колесо мыши — вверх")
            .replace("Mouse wheel — scroll down", "Колесо мыши — вниз")
            .replace("Mouse wheel — scroll left", "Колесо мыши — влево")
            .replace("Mouse wheel — scroll right", "Колесо мыши — вправо"),
        other if other.starts_with("Browser ") => other
            .replace("Browser search", "Поиск в браузере")
            .replace("Browser home page", "Домашняя страница браузера")
            .replace("Browser back", "Браузер назад")
            .replace("Browser forward", "Браузер вперёд")
            .replace("Browser stop loading", "Остановить загрузку в браузере")
            .replace("Browser refresh", "Обновить страницу")
            .replace("Browser favourites", "Избранное браузера"),
        other if other.starts_with("Left Cmd, ") => other.replace(
            "Left Cmd, macOS modifier key and app shortcuts",
            "Левый Cmd — модификатор macOS и сочетания приложений",
        ),
        other if other.starts_with("Right Cmd, ") => other.replace(
            "Right Cmd, macOS modifier key and app shortcuts",
            "Правый Cmd — модификатор macOS и сочетания приложений",
        ),
        other if other.starts_with("Left Win, ") => other.replace(
            "Left Win, Windows modifier key and OS shortcuts",
            "Левый Win — модификатор Windows и системные сочетания",
        ),
        other if other.starts_with("Right Win, ") => other.replace(
            "Right Win, Windows modifier key and OS shortcuts",
            "Правый Win — модификатор Windows и системные сочетания",
        ),
        other if other.starts_with("Left Super, ") => other.replace(
            "Left Super, desktop modifier key and OS shortcuts",
            "Левый Super — модификатор рабочего стола и системные сочетания",
        ),
        other if other.starts_with("Right Super, ") => other.replace(
            "Right Super, desktop modifier key and OS shortcuts",
            "Правый Super — модификатор рабочего стола и системные сочетания",
        ),
        other if other.starts_with("Use ") && other.contains(" by itself as a held modifier") => {
            let modifier = other
                .strip_prefix("Use ")
                .and_then(|s| s.split(" by itself as a held modifier").next())
                .unwrap_or("");
            if other.contains("Left click assigns Left ") {
                format!(
                    "Использовать {} как обычный удерживаемый модификатор\nЛевый клик: левая клавиша {}\nПравый клик: правая клавиша {}",
                    ru_modifier_name(modifier),
                    modifier,
                    modifier
                )
            } else {
                format!(
                    "Использовать {} как обычный удерживаемый модификатор",
                    ru_modifier_name(modifier)
                )
            }
        }
        other if other.starts_with("Hold ") && other.contains(" together with another key") => {
            let modifier = other
                .strip_prefix("Hold ")
                .and_then(|s| s.split(" together with another key").next())
                .unwrap_or("");
            if other.contains("Left click starts a Left ") {
                format!(
                    "Модификатор + клавиша: удерживать {} вместе с выбранной клавишей\nЛевый клик: левый {}\nПравый клик: правый {}\nЗатем выберите клавишу",
                    ru_modifier_name(modifier),
                    modifier,
                    modifier
                )
            } else {
                format!(
                    "Модификатор + клавиша: удерживать {} вместе с выбранной клавишей\nКликните, чтобы выбрать клавишу",
                    ru_modifier_name(modifier)
                )
            }
        }
        other if other.starts_with("Hold ") && other.contains(" with the key you choose next") => {
            other
                .replace("Hold ", "Удерживать ")
                .replace("Left ", "левый ")
                .replace("Right ", "правый ")
                .replace(
                    " together with the key you choose next",
                    " вместе со следующей выбранной клавишей",
                )
        }
        other if other.starts_with("Dual-role key: hold for ") => {
            let modifier = other
                .strip_prefix("Dual-role key: hold for ")
                .and_then(|s| s.split(',').next())
                .unwrap_or("");
            let tap_text = if other.contains("tap for the key you choose next") {
                "следующая выбранная клавиша"
            } else {
                "другая клавиша"
            };
            if other.contains("Left click uses Left ") {
                format!(
                    "Двойная роль: удержание = {}, нажатие = {}\nЛевый клик: левый {}\nПравый клик: правый {}\nЗатем выберите клавишу для нажатия",
                    ru_modifier_name(modifier),
                    tap_text,
                    modifier,
                    modifier
                )
            } else {
                format!(
                    "Двойная роль: удержание = {}, нажатие = {}\nКликните, чтобы выбрать клавишу для нажатия",
                    ru_modifier_name(modifier),
                    tap_text
                )
            }
        }
        other if other.starts_with("Applies ") && other.contains(" to the next keypress only") => {
            let modifier = other
                .strip_prefix("Applies ")
                .and_then(|s| s.split(" to the next keypress only").next())
                .unwrap_or("");
            if other.contains("Left click assigns One-Shot Left ") {
                format!(
                    "One-Shot модификатор: применит {} только к следующему нажатию\nЛевый клик: левый {}\nПравый клик: правый {}",
                    ru_modifier_name(modifier),
                    modifier,
                    modifier
                )
            } else {
                format!(
                    "One-Shot модификатор: применит {} только к следующему нажатию",
                    ru_modifier_name(modifier)
                )
            }
        }
        other if other.starts_with("Universal symbol: ") && other.contains(" — types ") => {
            let rest = other.strip_prefix("Universal symbol: ").unwrap_or(other);
            let (name, rest) = rest.split_once(" — types ").unwrap_or((rest, ""));
            let symbol = rest
                .split(" consistently regardless of the active keyboard language")
                .next()
                .unwrap_or(rest);
            format!(
                "Универсальный символ: {} — вводит {} одинаково при любом активном языке клавиатуры",
                ru_smart_symbol_name(name),
                symbol
            )
        }
        other if other.starts_with("One-Shot ") => other
            .replace("One-Shot ", "One-Shot ")
            .replace(
                " — active for the next keypress only",
                " — активен только для следующего нажатия",
            )
            .replace(" — applies ", " — применяет ")
            .replace(" to the next keypress only", " только к следующему нажатию")
            .replace(" — activates ", " — активирует ")
            .replace(
                " for the very next keypress only",
                " только для следующего нажатия",
            ),
        other if other.starts_with("RGB Matrix: solid color") => {
            "RGB Matrix: сплошной цвет без анимации".to_owned()
        }
        other if other.starts_with("RGB Matrix: breathing effect") => {
            "RGB Matrix: breathing-эффект с плавным изменением яркости".to_owned()
        }
        other if other.starts_with("RGB Matrix: rainbow gradient") => {
            "RGB Matrix: радужный градиент по всем клавишам".to_owned()
        }
        other if other.starts_with("RGB Matrix: swirling rainbow") => {
            "RGB Matrix: вращающийся радужный паттерн".to_owned()
        }
        other if other.starts_with("RGB Matrix: snake animation") => {
            "RGB Matrix: анимация змейки по клавишам".to_owned()
        }
        other if other.starts_with("RGB Matrix: Knight Rider") => {
            "RGB Matrix: сканирующий эффект Knight Rider".to_owned()
        }
        other if other.starts_with("RGB Matrix: alternating red and green") => {
            "RGB Matrix: чередование красного и зелёного как рождественская подсветка".to_owned()
        }
        other if other.starts_with("RGB Matrix: static gradient") => {
            "RGB Matrix: статичный градиент".to_owned()
        }
        other if other.starts_with("RGB Matrix: test mode") => {
            "RGB Matrix: тестовый режим, циклически R/G/B".to_owned()
        }
        other if other.starts_with("Swap Left Alt and ") => {
            other.replace("Swap Left Alt and ", "Поменять левый Alt и ")
        }
        other if other.starts_with("Unswap Left Alt and ") => {
            other.replace("Unswap Left Alt and ", "Вернуть левый Alt и ")
        }
        other if other.starts_with("Swap Right Alt and ") => {
            other.replace("Swap Right Alt and ", "Поменять правый Alt и ")
        }
        other if other.starts_with("Unswap Right Alt and ") => {
            other.replace("Unswap Right Alt and ", "Вернуть правый Alt и ")
        }
        other if other.starts_with("Enable the ") && other.ends_with(" keys") => other
            .replace("Enable the ", "Включить клавиши ")
            .replace(" keys", ""),
        other if other.starts_with("Disable the ") && other.ends_with(" keys") => other
            .replace("Disable the ", "Отключить клавиши ")
            .replace(" keys", ""),
        other if other.starts_with("Toggles the status of the ") => other
            .replace(
                "Toggles the status of the ",
                "Переключить состояние клавиш ",
            )
            .replace(" keys", ""),
        other if other.starts_with("Swap Alt and ") && other.ends_with(" on both sides") => other
            .replace("Swap Alt and ", "Поменять Alt и ")
            .replace(" on both sides", " с обеих сторон"),
        other if other.starts_with("Unswap Alt and ") && other.ends_with(" on both sides") => other
            .replace("Unswap Alt and ", "Вернуть Alt и ")
            .replace(" on both sides", " с обеих сторон"),
        other if other.starts_with("Toggle Alt and ") && other.ends_with(" swap on both sides") => {
            other
                .replace("Toggle Alt and ", "Переключить обмен Alt и ")
                .replace(" swap on both sides", " с обеих сторон")
        }
        other if other.starts_with("Swap Left Control and ") => {
            other.replace("Swap Left Control and ", "Поменять левый Control и ")
        }
        other if other.starts_with("Unswap Left Control and ") => {
            other.replace("Unswap Left Control and ", "Вернуть левый Control и ")
        }
        other if other.starts_with("Swap Right Control and ") => {
            other.replace("Swap Right Control and ", "Поменять правый Control и ")
        }
        other if other.starts_with("Unswap Right Control and ") => {
            other.replace("Unswap Right Control and ", "Вернуть правый Control и ")
        }
        other if other.starts_with("Swap Control and ") && other.ends_with(" on both sides") => {
            other
                .replace("Swap Control and ", "Поменять Control и ")
                .replace(" on both sides", " с обеих сторон")
        }
        other if other.starts_with("Unswap Control and ") && other.ends_with(" on both sides") => {
            other
                .replace("Unswap Control and ", "Вернуть Control и ")
                .replace(" on both sides", " с обеих сторон")
        }
        other
            if other.starts_with("Toggle Control and ")
                && other.ends_with(" swap on both sides") =>
        {
            other
                .replace("Toggle Control and ", "Переключить обмен Control и ")
                .replace(" swap on both sides", " с обеих сторон")
        }
        other
            if other.starts_with("Layer ") && other[6..].chars().all(|ch| ch.is_ascii_digit()) =>
        {
            other.replace("Layer ", "Слой ")
        }
        other if other.starts_with("Pick key for ") => {
            other.replace("Pick key for ", "Выбрать клавишу для ")
        }
        other if other.starts_with("Momentarily activate layer ") => other
            .replace(
                "Momentarily activate layer ",
                "Моментально активировать слой ",
            )
            .replace(" while held", " при удержании"),
        other if other.starts_with("Key: ") => other
            .replace("Key: ", "Клавиша: ")
            .replace("Left Ctrl", "левый Ctrl")
            .replace("Right Ctrl", "правый Ctrl")
            .replace("Left Shift", "левый Shift")
            .replace("Right Shift", "правый Shift")
            .replace("Left Alt", "левый Alt")
            .replace("Right Alt", "правый Alt")
            .replace("Left Cmd", "левый Cmd")
            .replace("Right Cmd", "правый Cmd")
            .replace("Left Win", "левый Win")
            .replace("Right Win", "правый Win")
            .replace("Left Super", "левый Super")
            .replace("Right Super", "правый Super"),
        other if other.ends_with(" function key") => {
            other.replace(" function key", " — функциональная клавиша")
        }
        other if other.starts_with("Numpad ") => other.replace("Numpad ", "Нампад "),
        other if other.starts_with("Shortcut: ") => other
            .replace("Shortcut: ", "Сочетание: ")
            .replace("Right ", "Правый "),
        other
            if other.starts_with("Macro ")
                && other.contains(" — sends a sequence of keystrokes") =>
        {
            other.replace("Macro ", "Макрос ").replace(
                " — sends a sequence of keystrokes",
                " — отправляет последовательность нажатий",
            )
        }
        other
            if other.starts_with("Tap Dance ")
                && other.contains(" — different actions on tap, hold, double tap") =>
        {
            other.replace(
                " — different actions on tap, hold, double tap",
                " — разные действия на tap, hold и double tap",
            )
        }
        other if other.contains(" — macro ") => other.replace(" — macro ", " — макрос "),
        other if other.contains(" — tap dance ") => {
            other.replace(" — tap dance ", " — Tap Dance ")
        }
        other if other.starts_with("MO(") => other
            .replace(" — activate layer ", " — активировать слой ")
            .replace(" — activate ", " — активировать ")
            .replace(
                " while held, return when released",
                " при удержании, вернуть при отпускании",
            ),
        other if other.starts_with("TO(") => other
            .replace(" — switch to layer ", " — переключиться на слой ")
            .replace(" — switch to ", " — переключиться на ")
            .replace(" and stay there", " и остаться там"),
        other if other.starts_with("TG(") => other
            .replace(" — toggle layer ", " — переключить слой ")
            .replace(" — toggle ", " — переключить ")
            .replace(" on/off", " вкл/выкл"),
        other if other.starts_with("DF(") || other.starts_with("PDF(") => other
            .replace(" — set ", " — сделать ")
            .replace(" — permanently set ", " — постоянно сделать ")
            .replace(" as the default base layer", " базовым слоем по умолчанию")
            .replace(" as the default layer", " слоем по умолчанию"),
        other if other.starts_with("OSL(") => other
            .replace(" — activate layer ", " — активировать слой ")
            .replace(" — activate ", " — активировать ")
            .replace(" for next keypress only", " только для следующего нажатия"),
        other if other.starts_with("TT(") => other
            .replace(" — tap to toggle layer ", " — tap переключает слой ")
            .replace(" — tap to toggle ", " — tap переключает ")
            .replace(
                ", hold to activate while held",
                ", hold активирует при удержании",
            ),
        other if other.starts_with("Layer Tap — tap for ") => other
            .replace("Layer Tap — tap for ", "Layer Tap — tap для ")
            .replace(", hold to activate layer ", ", hold активирует слой ")
            .replace(", hold to activate ", ", hold активирует "),
        other if other.starts_with("LM(") => other
            .replace(" — activate layer ", " — активировать слой ")
            .replace(" with ", " с ")
            .replace(
                " held while key is pressed",
                " при удержании во время нажатия клавиши",
            ),
        other if other.starts_with("Unknown layer op ") => {
            other.replace("Unknown layer op", "Неизвестная операция слоя")
        }
        other if other.starts_with("Custom key ") => {
            other.replace("Custom key", "Пользовательская клавиша")
        }
        other if other.starts_with("Unknown keycode ") => {
            other.replace("Unknown keycode", "Неизвестный keycode")
        }
        other if other.starts_with("Mod Tap — tap for ") => other
            .replace("Mod Tap — tap for ", "Mod Tap — tap для ")
            .replace(", hold for Left ", ", hold для левого ")
            .replace(", hold for Right ", ", hold для правого ")
            .replace(", hold for ", ", hold для "),
        other if other.starts_with("Universal Cyrillic ") => other
            .replace("Universal Cyrillic", "Universal Cyrillic")
            .replace("types", "вводит")
            .replace(
                "consistently regardless of the active keyboard language",
                "одинаково независимо от активного языка клавиатуры",
            )
            .replace("hold Shift for", "удерживайте Shift для"),
        other
            if other
                .starts_with("Universal output backend: Wayland via IBus/Fcitx5 input method") =>
        {
            other.replacen(
                "Universal output backend: Wayland via IBus/Fcitx5 input method",
                "Бэкенд универсального вывода: Wayland через IBus/Fcitx5",
                1,
            )
        }
        other if other.starts_with("Universal output backend: Linux X11 native") => other.replacen(
            "Universal output backend: Linux X11 native; Wayland uses IBus/Fcitx5",
            "Бэкенд универсального вывода: Linux X11 native; Wayland использует IBus/Fcitx5",
            1,
        ),
        other
            if other
                .starts_with("Universal output backend: Linux; use IBus/Fcitx5 for Wayland") =>
        {
            other.replacen(
                "Universal output backend: Linux; use IBus/Fcitx5 for Wayland",
                "Бэкенд универсального вывода: Linux; для Wayland используйте IBus/Fcitx5",
                1,
            )
        }
        other => other.to_owned(),
    }
}
