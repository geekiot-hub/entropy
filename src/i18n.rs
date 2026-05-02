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
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            return Some(&value[1..value.len() - 1]);
        }
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

pub fn tr_static(language: Language, text: &'static str) -> &'static str {
    if !matches!(language, Language::Russian) {
        return text;
    }

    match text {
        "Open Privacy Settings" => "Открыть настройки приватности",
        "Effect" => "Эффект",
        "Color" => "Цвет",
        "Speed" => "Скорость",
        "Brightness" => "Яркость",
        "Key Picker" => "Пикер клавиш",
        "Pick key" => "Выбрать клавишу",
        "Press a key on your keyboard, or pick below" => {
            "Нажмите клавишу на клавиатуре или выберите ниже"
        }
        "Press a key on your keyboard, or click below" => {
            "Нажмите клавишу на клавиатуре или выберите ниже"
        }
        "Press a key on your keyboard, or click below (Esc to cancel)" => {
            "Нажмите клавишу на клавиатуре или выберите ниже (Esc — отмена)"
        }
        "Best for normal keys, navigation, media and special actions" => {
            "Лучше всего для обычных клавиш, навигации, медиа и спецдействий"
        }
        "None (clear)" => "Нет (очистить)",
        "Pick layer" => "Выбрать слой",
        "Choose which layer (Esc to cancel)" => "Выберите слой (Esc — отмена)",
        "Pick tap key (hold = modifier)" => "Выбрать tap-клавишу (hold = модификатор)",
        "Pick key for modifier combo" => "Выбрать клавишу для комбо с модификатором",
        "Basic" => "Базовые",
        "Symbols" => "Символы",
        "Mods" => "Моды",
        "Special" => "Спец",
        "Macros" => "Макросы",
        "Tap Dance" => "Tap Dance",
        "Custom" => "Кастом",
        "Layers" => "Слои",
        "Letters" => "Буквы",
        "Numbers" => "Цифры",
        "Editing" => "Редактирование",
        "Navigation" => "Навигация",
        "Function keys" => "Функциональные клавиши",
        "Modifiers" => "Модификаторы",
        "Other keys" => "Другие клавиши",
        "Special QMK keys" => "Специальные QMK-клавиши",
        "Mouse" => "Мышь",
        "Media, Apps, System" => "Медиа, приложения, система",
        "OS / edit shortcuts" => "ОС / редактирование",
        "Numpad" => "Нампад",
        "Space Cadet" => "Space Cadet",
        "International" => "Международные",
        "Backlight" => "Подсветка",
        "RGB Underglow" => "RGB Underglow",
        "RGB Matrix Modes" => "Режимы RGB Matrix",
        "RGB Matrix Controls" => "Управление RGB Matrix",
        "Hold to activate, release to return" => {
            "Удержание активирует слой, отпускание возвращает назад"
        }
        "Toggle layer on/off" => "Включить или выключить слой",
        "Tap to toggle on/off" => "Нажатие включает или выключает слой",
        "Active for next keypress only" => "Активен только для следующего нажатия",
        "Hold = MO, tap = toggle" => "Удержание = временно активировать, нажатие = переключить",
        "Tap multiple times to toggle layer" => "Нажмите несколько раз, чтобы переключить слой",
        "Switch and stay on this layer" => "Переключиться и остаться на этом слое",
        "Set as permanent base layer" => "Сделать постоянным базовым слоем",
        "Hold = activate layer, tap = keycode (set key via right-click afterwards)" => {
            "Hold активирует слой, tap отправляет keycode; keycode задаётся потом через правый клик"
        }
        "Toggle backlight on/off" => "Включить или выключить подсветку",
        "Cycle through backlight brightness levels" => "Переключать уровни яркости подсветки",
        "Toggle breathing effect on/off" => "Включить или выключить breathing-эффект",
        "Turn backlight on" => "Включить подсветку",
        "Turn backlight off" => "Выключить подсветку",
        "Decrease backlight brightness" => "Уменьшить яркость подсветки",
        "Increase backlight brightness" => "Увеличить яркость подсветки",
        "Toggle RGB lighting on/off" => "Включить или выключить RGB-подсветку",
        "Switch to previous RGB animation mode" => "Предыдущий режим RGB-анимации",
        "Switch to next RGB animation mode" => "Следующий режим RGB-анимации",
        "Decrease color hue" => "Уменьшить оттенок",
        "Increase color hue" => "Увеличить оттенок",
        "Decrease color saturation" => "Уменьшить насыщенность",
        "Increase color saturation" => "Увеличить насыщенность",
        "Decrease brightness" => "Уменьшить яркость",
        "Increase brightness" => "Увеличить яркость",
        "Decrease animation speed" => "Уменьшить скорость анимации",
        "Increase animation speed" => "Увеличить скорость анимации",
        "Previous RGB effect" => "Предыдущий RGB-эффект",
        "Next RGB effect" => "Следующий RGB-эффект",
        "Turn RGB Matrix on" => "Включить RGB Matrix",
        "Turn RGB Matrix off" => "Выключить RGB Matrix",
        "Toggle RGB Matrix on/off" => "Включить или выключить RGB Matrix",
        "Previous RGB Matrix animation" => "Предыдущая RGB Matrix анимация",
        "Next RGB Matrix animation" => "Следующая RGB Matrix анимация",
        "Decrease RGB Matrix hue" => "Уменьшить оттенок RGB Matrix",
        "Increase RGB Matrix hue" => "Увеличить оттенок RGB Matrix",
        "Decrease RGB Matrix saturation" => "Уменьшить насыщенность RGB Matrix",
        "Increase RGB Matrix saturation" => "Увеличить насыщенность RGB Matrix",
        "Decrease RGB Matrix brightness" => "Уменьшить яркость RGB Matrix",
        "Increase RGB Matrix brightness" => "Увеличить яркость RGB Matrix",
        "Decrease RGB Matrix animation speed" => "Уменьшить скорость RGB Matrix анимации",
        "Increase RGB Matrix animation speed" => "Увеличить скорость RGB Matrix анимации",
        "Rose" => "Розовый",
        "Violet" => "Фиолетовый",
        "Blue" => "Синий",
        "Amber" => "Янтарный",
        "Copper" => "Медный",
        "Teal" => "Бирюзовый",
        "White" => "Белый",
        "Red" => "Красный",
        "Orange" => "Оранжевый",
        "Goldenrod" => "Золотистый",
        "Gold" => "Золотой",
        "Yellow" => "Жёлтый",
        "Chartreuse" => "Шартрез",
        "Lime" => "Лайм",
        "Green" => "Зелёный",
        "Spring Green" => "Весенне-зелёный",
        "Turquoise" => "Турквиз",
        "Cyan" => "Циан",
        "Azure" => "Лазурный",
        "Sky" => "Небесный",
        "Indigo" => "Индиго",
        "Purple" => "Пурпурный",
        "Magenta" => "Маджента",
        "Pink" => "Розовый",
        "Coral" => "Коралловый",
        "Salmon" => "Лососевый",
        "Warm White" => "Тёплый белый",
        "Trigger" => "Триггер",
        "Replacement" => "Замена",
        "Suppressed mods" => "Подавляемые модификаторы",
        "Trigger mods" => "Триггер-модификаторы",
        "Negative mods" => "Блокирующие модификаторы",
        "Enable on layers" => "Активно на слоях",
        "Pick trigger" => "Выбрать триггер",
        "Pick replacement" => "Выбрать замену",
        "None" => "Нет",
        "All mods" => "Все модификаторы",
        "No layers" => "Нет слоёв",
        "All layers" => "Все слои",
        "Enable all" => "Включить всё",
        "Disable all" => "Отключить всё",
        "Trigger press" => "Нажатие триггера",
        "Required mod press" => "Нажатие требуемого модификатора",
        "Blocked mod release" => "Отпускание блокирующего модификатора",
        "Any one mod" => "Любой один модификатор",
        "No re-send" => "Не отправлять повторно",
        "Stay active" => "Оставаться активным",
        "Input keys" => "Входные клавиши",
        "Output key" => "Выходная клавиша",
        "Select Key Override slot" => "Выберите слот Key Override",
        "Local name for this Key Override slot" => "Локальное имя этого слота Key Override",
        "Original key that can be overridden" => "Исходная клавиша, которую можно заменить",
        "Keycode sent while override conditions match" => {
            "Keycode, отправляемый при выполнении условий override"
        }
        "Modifiers hidden while the replacement is active" => {
            "Модификаторы, скрываемые пока замена активна"
        }
        "Modifiers required for this override" => "Модификаторы, обязательные для этого override",
        "Modifiers that block this override" => "Модификаторы, блокирующие этот override",
        "Layers where this override can activate" => {
            "Слои, на которых этот override может сработать"
        }
        "Activate when the trigger key is pressed" => "Активировать при нажатии клавиши-триггера",
        "Activate when a required modifier is pressed" => {
            "Активировать при нажатии требуемого модификатора"
        }
        "Activate when a blocking modifier is released" => {
            "Активировать при отпускании блокирующего модификатора"
        }
        "Any one trigger modifier is enough" => "Достаточно любого одного триггер-модификатора",
        "Do not resend the trigger after override ends" => {
            "Не отправлять триггер повторно после окончания override"
        }
        "Stay active when another key is pressed" => {
            "Оставаться активным при нажатии другой клавиши"
        }
        "Select Combo slot" => "Выберите слот Combo",
        "Local name for this combo slot" => "Локальное имя этого слота Combo",
        "Keys that must be pressed together" => "Клавиши, которые нужно нажать вместе",
        "Keycode sent when the combo activates" => "Keycode, отправляемый при срабатывании Combo",
        "Maximum time between combo key presses" => {
            "Максимальное время между нажатиями клавиш Combo"
        }
        "Press 2-4 keys" => "Нажмите 2–4 клавиши",
        "Record 2-4 keys" => "Записать 2–4 клавиши",
        "Pick output" => "Выбрать выход",
        "Hold actions are limited to left/right modifiers and layers" => {
            "Hold-действия ограничены левыми/правыми модификаторами и слоями"
        }
        "Tap-then-hold actions are limited to left/right modifiers and layers" => {
            "Tap-then-hold действия ограничены левыми/правыми модификаторами и слоями"
        }
        "Left Control" => "Левый Control",
        "Right Control" => "Правый Control",
        "Left Shift" => "Левый Shift",
        "Right Shift" => "Правый Shift",
        "Left Alt" => "Левый Alt",
        "Right Alt" => "Правый Alt",
        "Clear all" => "Очистить всё",
        "↩ Undo" => "↩ Отменить",
        "Undo last change" => "Отменить последнее изменение",
        "Remove all actions from this macro" => "Удалить все действия из этого макроса",
        "Select a tap dance tab above to edit" => "Выберите Tap Dance выше для редактирования",
        "Key sent on single tap" => "Клавиша при одиночном tap",
        "Key sent when held" => "Клавиша при hold",
        "Key sent on double tap" => "Клавиша при double tap",
        "Key sent on tap then hold" => "Клавиша при tap затем hold",
        "Best for a second tap action, usually another normal key or command" => {
            "Лучше всего для второго tap-действия: обычная клавиша или команда"
        }
        "Basic keys — standard keyboard layout" => "Базовые клавиши — стандартная раскладка",
        "Universal symbols — same output in any language" => {
            "Универсальные символы — одинаковый ввод на любом языке"
        }
        "Layout symbols — follow the active keyboard language" => {
            "Символы раскладки — зависят от активного языка"
        }
        "Extra universal symbols — typography and math" => {
            "Дополнительные универсальные символы — типографика и математика"
        }
        "Custom keycodes — defined by this keyboard" => {
            "Кастомные keycodes — определены этой клавиатурой"
        }
        "Layers: choose a layer action, then pick the target layer" => {
            "Слои: выберите действие слоя, затем целевой слой"
        }
        "Plain modifiers" => "Обычные модификаторы",
        "Mod+Key — always sends modifier+key together" => {
            "Mod+Key — всегда отправляет модификатор+клавишу вместе"
        }
        "Mod-Tap — hold for modifier, tap for regular key" => {
            "Mod-Tap — hold для модификатора, tap для обычной клавиши"
        }
        "One-Shot Mod — active for next keypress only" => {
            "One-Shot Mod — активен только для следующего нажатия"
        }
        "Choose the key to pair with the modifier" => "Выберите клавишу для пары с модификатором",
        "This key will always be sent together with the selected modifier" => {
            "Эта клавиша всегда будет отправляться вместе с выбранным модификатором"
        }
        "Choose the tap key" => "Выберите tap-клавишу",
        "Hold will send the modifier; tap will send the key you pick" => {
            "Hold отправит модификатор; tap отправит выбранную клавишу"
        }
        "✕ Cancel" => "✕ Отмена",
        "Mod+Key — pick modifier, then key" => "Mod+Key — выберите модификатор, затем клавишу",
        "Mod-Tap — pick modifier, then tap key" => {
            "Mod-Tap — выберите модификатор, затем tap-клавишу"
        }
        "Choose macro" => "Выбрать макрос",
        "Select a macro above to edit" => "Выберите макрос выше для редактирования",
        "Macro name" => "Имя макроса",
        "Move up" => "Выше",
        "Move down" => "Ниже",
        "Text" => "Текст",
        "Types text characters one by one" => "Вводит символы текста по одному",
        "Tap" => "Tap",
        "Press and release a key" => "Нажать и отпустить клавишу",
        "Down" => "Down",
        "Press a key (hold until Up)" => "Нажать клавишу (удерживать до Up)",
        "Up" => "Up",
        "Release a previously pressed key" => "Отпустить ранее нажатую клавишу",
        "Delay" => "Задержка",
        "Wait before next action" => "Пауза перед следующим действием",
        "Type text here" => "Введите текст здесь",
        "Characters to type when this macro runs" => "Символы, которые макрос будет вводить",
        "Click to change key — press and release this key" => {
            "Нажмите, чтобы сменить клавишу — press/release"
        }
        "Click to change key — holds down until Up" => {
            "Нажмите, чтобы сменить клавишу — удерживать до Up"
        }
        "Click to change key — releases this key" => "Нажмите, чтобы сменить клавишу — отпустить",
        "Delay is in milliseconds" => "Задержка в миллисекундах",
        "Remove this action" => "Удалить это действие",
        "+ Text" => "+ Текст",
        "Type characters" => "Ввести символы",
        "+ Tap" => "+ Tap",
        "+ Down" => "+ Down",
        "Hold a key" => "Удерживать клавишу",
        "+ Up" => "+ Up",
        "Release a key" => "Отпустить клавишу",
        "+ Delay" => "+ Задержка",
        "Pause in milliseconds" => "Пауза в миллисекундах",
        "No Tap Dance slots available on this keyboard" => {
            "На этой клавиатуре нет слотов Tap Dance"
        }
        "Choose tap dance" => "Выбрать Tap Dance",
        "TD name" => "Имя TD",
        "On Tap" => "При Tap",
        "On Hold" => "При Hold",
        "On Double Tap" => "При Double Tap",
        "On Tap + Hold" => "При Tap + Hold",
        "Click to assign a key" => "Нажмите, чтобы назначить клавишу",
        "Tapping Term" => "Окно tap/hold",
        "Time in milliseconds to distinguish tap from hold (default: 200)" => {
            "Время в миллисекундах для различения tap и hold (по умолчанию 200)"
        }
        "Tapping term is in milliseconds" => "Окно tap/hold в миллисекундах",
        "Clear all actions for this tap dance" => "Очистить все действия этого Tap Dance",
        "Undo last tap dance change" => "Отменить последнее изменение Tap Dance",
        "Tap Dance Editor" => "Редактор Tap Dance",
        "No setup action is available for this OS" => "Для этой ОС нет действия настройки",
        "Install IBus" => "Установить IBus",
        "Install Fcitx5" => "Установить Fcitx5",
        "Keyboard is locked, unlock it to use Matrix Tester" => {
            "Клавиатура заблокирована — разблокируйте её для тестера матрицы"
        }
        "Click to reset Matrix Tester" => "Нажмите, чтобы сбросить тестер матрицы",
        "Matrix Tester is currently available only for Vial keyboards" => {
            "Тестер матрицы пока доступен только для Vial-клавиатур"
        }
        "Connect a Vial keyboard to start live switch testing" => {
            "Подключите Vial-клавиатуру, чтобы начать live-тест свитчей"
        }
        "Click Tested to reset progress" => "Нажмите на счётчик проверки, чтобы сбросить прогресс",
        "Tested" => "Проверено",
        "Toggle firmware layout/display option" => "Переключить опцию раскладки/дисплея в прошивке",
        "Choose firmware preset" => "Выбрать пресет прошивки",
        "Connect a Vial keyboard to edit Auto Shift settings" => {
            "Подключите Vial-клавиатуру, чтобы менять Auto Shift"
        }
        "Enable" => "Включить",
        "Turn Auto Shift on or off" => "Включить или выключить Auto Shift",
        "Enable for modifiers" => "Для модификаторов",
        "Allow Auto Shift behavior on modifier keys" => {
            "Разрешить Auto Shift на клавишах-модификаторах"
        }
        "No special keys" => "Без специальных клавиш",
        "Do not Auto Shift special keys such as Enter, Esc, Tab or Backspace" => {
            "Не применять Auto Shift к Enter, Esc, Tab и Backspace"
        }
        "No numeric keys" => "Без цифровых клавиш",
        "Do not Auto Shift number keys" => "Не применять Auto Shift к цифрам",
        "No alpha keys" => "Без буквенных клавиш",
        "Do not Auto Shift letter keys" => "Не применять Auto Shift к буквам",
        "Enable keyrepeat" => "Повтор клавиши",
        "Allow held Auto Shift keys to repeat" => "Разрешить повтор удерживаемых Auto Shift клавиш",
        "Stop repeat after timeout" => "Остановить повтор после таймаута",
        "Disable key repeat after the Auto Shift timeout is exceeded" => {
            "Отключать повтор после превышения таймаута Auto Shift"
        }
        "Timeout" => "Таймаут",
        "Hold time before Auto Shift sends the shifted key" => {
            "Время удержания перед отправкой shifted-клавиши"
        }
        "Timeout is in milliseconds" => "Таймаут в миллисекундах",
        "Light" => "Светлая",
        "Dark" => "Тёмная",
        "☀ Light" => "☀ Светлая",
        "🌙 Dark" => "🌙 Тёмная",
        "🔓 Unlock Keyboard" => "🔓 Разблокировать клавиатуру",
        "Unlock Keyboard" => "Разблокировать клавиатуру",
        "Press and hold the highlighted keys one by one" => {
            "Нажмите и удерживайте подсвеченные клавиши по очереди"
        }
        "Macros saved" => "Макросы сохранены",
        "Combos saved" => "Комбо сохранены",
        "Combo timeout saved" => "Таймаут комбо сохранён",
        "Entry" => "Слот",
        "Select Alt Repeat slot" => "Выбрать слот Alt Repeat",
        "Name" => "Имя",
        "Local name for this slot" => "Локальное имя этого слота",
        "Stored locally in Entropy" => "Хранится локально в Entropy",
        "Last key" => "Последняя клавиша",
        "Key that triggers alternate repeat behavior" => "Клавиша, запускающая alternate repeat",
        "Alt key" => "Alt-клавиша",
        "Key repeated when alternate repeat activates" => {
            "Клавиша, повторяемая при срабатывании alternate repeat"
        }
        "Ctrl mods" => "Ctrl-моды",
        "Shift mods" => "Shift-моды",
        "Alt mods" => "Alt-моды",
        "Allowed Ctrl modifiers" => "Разрешённые Ctrl-модификаторы",
        "Allowed Shift modifiers" => "Разрешённые Shift-модификаторы",
        "Allowed Alt modifiers" => "Разрешённые Alt-модификаторы",
        "Allowed OS modifiers" => "Разрешённые OS-модификаторы",
        "Right-side modifier" => "Правый модификатор",
        "Left-side modifier" => "Левый модификатор",
        "Default alt key" => "Alt-клавиша по умолчанию",
        "Use this alt key by default" => "Использовать эту Alt-клавишу по умолчанию",
        "Bidirectional" => "Двунаправленно",
        "Allow both keys to alternate each other" => {
            "Разрешить обеим клавишам чередовать друг друга"
        }
        "Ignore handedness" => "Игнорировать сторону",
        "Treat left and right modifiers as equivalent" => {
            "Считать левые и правые модификаторы одинаковыми"
        }
        "Clear" => "Очистить",
        "Undo" => "Отменить",
        "↶ Undo" => "↶ Отменить",
        "LED brightness" => "Яркость подсветки",
        "Global LED brightness for layer color lighting" => "Общая яркость цветной подсветки слоёв",
        "LED timeout" => "Таймаут подсветки",
        "Minutes before LEDs turn off automatically, 0 disables timeout" => {
            "Минут до автоотключения подсветки; 0 отключает таймаут"
        }
        "Off" => "Выкл",
        "Alt forces Esc" => "Alt отправляет Esc",
        "When Alt is held, Grave Escape sends Esc instead of ` or ~" => {
            "При удержании Alt Grave Escape отправляет Esc вместо ` или ~"
        }
        "Control forces Esc" => "Control отправляет Esc",
        "When Control is held, Grave Escape sends Esc instead of ` or ~" => {
            "При удержании Control Grave Escape отправляет Esc вместо ` или ~"
        }
        "Shift forces Esc" => "Shift отправляет Esc",
        "When Shift is held, Grave Escape sends Esc instead of ` or ~" => {
            "При удержании Shift Grave Escape отправляет Esc вместо ` или ~"
        }
        "Swap Caps Lock and Left Control" => "Поменять Caps Lock и левый Control",
        "Caps Lock sends Left Control and Left Control sends Caps Lock" => {
            "Caps Lock отправляет Left Control, а Left Control — Caps Lock"
        }
        "Treat Caps Lock as Control" => "Считать Caps Lock Control",
        "Caps Lock sends Control without swapping Left Control" => {
            "Caps Lock отправляет Control без обмена с Left Control"
        }
        "Disable OS keys" => "Отключить OS-клавиши",
        "Ignore both OS keys while this option is enabled" => {
            "Игнорировать обе OS-клавиши, пока опция включена"
        }
        "Swap ` and Escape" => "Поменять ` и Escape",
        "Grave sends Escape and Escape sends Grave" => "Grave отправляет Escape, а Escape — Grave",
        r"Swap \ and Backspace" => r"Поменять \ и Backspace",
        "Backslash sends Backspace and Backspace sends Backslash" => {
            "Backslash отправляет Backspace, а Backspace — Backslash"
        }
        "Enable N-key rollover" => "Включить N-key rollover",
        "Allow more simultaneous key presses when the keyboard supports it" => {
            "Разрешить больше одновременных нажатий, если клавиатура это поддерживает"
        }
        "Tapping term" => "Окно tap/hold",
        "Global tap-vs-hold decision window for dual-role keys" => {
            "Общее окно выбора tap/hold для dual-role клавиш"
        }
        "Permissive hold" => "Разрешающий hold",
        "Nested taps choose hold for Mod-Tap and Layer-Tap keys" => {
            "Вложенные taps выбирают hold для Mod-Tap и Layer-Tap"
        }
        "Hold on other key" => "Hold при другой клавише",
        "Pressing another key immediately chooses hold for dual-role keys" => {
            "Нажатие другой клавиши сразу выбирает hold для dual-role клавиш"
        }
        "Retro tapping" => "Retro tap",
        "A held-and-released-alone dual-role key still sends its tap action" => {
            "Dual-role клавиша, удержанная и отпущенная отдельно, всё равно отправляет tap"
        }
        "Chordal hold" => "Аккордный hold",
        "Same-hand chords prefer tap to reduce home-row mod accidents" => {
            "Аккорды одной рукой предпочитают tap, чтобы снизить ошибки home-row mods"
        }
        "Quick tap term" => "Окно quick tap",
        "Tap-then-hold repeat window for dual-role key tap actions" => {
            "Окно повтора tap-then-hold для tap-действий dual-role клавиш"
        }
        "Tap code delay" => "Задержка tap_code",
        "Delay between register and unregister in tap_code" => {
            "Задержка между register и unregister в tap_code"
        }
        "Tap hold caps delay" => "Задержка Caps Lock tap",
        "Extra delay for LT/MT keys whose tap action is Caps Lock" => {
            "Дополнительная задержка для LT/MT, где tap-действие — Caps Lock"
        }
        "Tapping toggle" => "Переключение TT",
        "Number of taps needed for TT layer toggle" => {
            "Количество taps для переключения слоя через TT"
        }
        "Flow tap" => "Таймаут flow tap",
        "Fast typing timeout that forces MT/LT keys to tap" => {
            "Таймаут быстрого набора, принудительно выбирающий tap для MT/LT"
        }
        "One Shot Keys" => "One Shot клавиши",
        "One-shot tap toggle" => "One-shot toggle",
        "Tap this many times to keep a one-shot key held until tapped again" => {
            "Столько taps удерживают one-shot до следующего нажатия"
        }
        "One-shot timeout" => "One-shot таймаут",
        "How long one-shot state waits before it is released" => {
            "Сколько one-shot состояние ждёт перед сбросом"
        }
        "Value is in milliseconds" => "Значение в миллисекундах",
        "DPI" => "DPI",
        "Touchpad pointer resolution in dots per inch" => {
            "Разрешение указателя тачпада в точках на дюйм"
        }
        "Sniper sens" => "Чувств. sniper",
        "Sniper divisor: lower is faster, higher is more precise" => {
            "Делитель sniper: ниже — быстрее, выше — точнее"
        }
        "Scroll sens" => "Чувств. скролла",
        "Scroll divisor: lower is faster, higher is smoother" => {
            "Делитель скролла: ниже — быстрее, выше — плавнее"
        }
        "Text sens" => "Чувств. текста",
        "Text mode divisor: lower is faster, higher is slower" => {
            "Делитель текстового режима: ниже — быстрее, выше — медленнее"
        }
        "Invert scroll" => "Инвертировать скролл",
        "Reverse the touchpad scroll direction" => "Развернуть направление скролла тачпада",
        "Acceleration" => "Ускорение",
        "Use firmware pointer acceleration for touchpad movement" => {
            "Использовать ускорение указателя из прошивки"
        }
        "Sticky mode" => "Фиксировать режим",
        "Keep the selected touchpad mode active until another mode is selected" => {
            "Оставлять выбранный режим тачпада активным до выбора другого"
        }
        "Auto layer enable" => "Автослой",
        "Automatically switch to the selected layer while the touchpad is active" => {
            "Автоматически переключаться на выбранный слой, пока тачпад активен"
        }
        "Auto layer" => "Автослой",
        "Layer selected automatically while the touchpad is active" => {
            "Слой, выбираемый автоматически при активности тачпада"
        }
        "Entropy background" => "Entropy в фоне",
        "Keep Entropy running in the background for live firmware data" => {
            "Держать Entropy в фоне для live-данных прошивки"
        }
        "Time sync" => "Синхронизация времени",
        "Uses the local system clock" => "Использует локальные системные часы",
        "Volume sync" => "Синхронизация громкости",
        "native Windows audio" => "нативное аудио Windows",
        "Uses the Windows default output device" => "Использует устройство вывода Windows по умолчанию",
        "Uses PipeWire default sink volume" => "Использует громкость default sink PipeWire",
        "Uses PulseAudio/PipeWire Pulse default sink volume" => {
            "Использует громкость default sink PulseAudio/PipeWire Pulse"
        }
        "missing wpctl/pactl" => "нет wpctl/pactl",
        "Install wireplumber or pulseaudio-utils/pavucontrol package for volume sync" => {
            "Установите wireplumber или pulseaudio-utils/pavucontrol для синхронизации громкости"
        }
        "Uses macOS system output volume" => "Использует системную громкость вывода macOS",
        "unsupported OS" => "ОС не поддерживается",
        "Volume sync is implemented for Windows, Linux and macOS" => {
            "Синхронизация громкости реализована для Windows, Linux и macOS"
        }
        "native Windows media session" => "нативная медиа-сессия Windows",
        "Uses Windows global media session metadata" => {
            "Использует метаданные глобальной медиа-сессии Windows"
        }
        "Uses MPRIS metadata from the active player" => "Использует MPRIS-метаданные активного плеера",
        "missing playerctl" => "нет playerctl",
        "Install playerctl and use an MPRIS-compatible player for media info" => {
            "Установите playerctl и используйте MPRIS-совместимый плеер для медиа-информации"
        }
        "Spotify / Music via AppleScript" => "Spotify / Music через AppleScript",
        "macOS may ask for Automation permission for Entropy, System Events, Spotify or Music" => {
            "macOS может запросить Automation-разрешение для Entropy, System Events, Spotify или Music"
        }
        "Media sync is implemented for Windows, Linux and macOS" => {
            "Медиа-синхронизация реализована для Windows, Linux и macOS"
        }
        "Media info" => "Медиа-информация",
        "ready" => "готово",
        "needs setup" => "нужна настройка",
        "active" => "активно",
        "starting" => "запуск",
        _ => text,
    }
}

pub fn tr_text(language: Language, text: &str) -> String {
    if !matches!(language, Language::Russian) {
        return text.to_owned();
    }

    match text {
        "No extra setup is required on Windows" => "В Windows дополнительная настройка не нужна".to_owned(),
        "Keep Entropy running while using Universal Symbols" => "Оставляйте Entropy запущенной при использовании универсальных символов".to_owned(),
        "Assign keys from Symbols → Universal symbols in the key picker" => "Назначьте клавиши из Symbols → Universal symbols в пикере клавиш".to_owned(),
        "Open Privacy & Security" => "Откройте Privacy & Security".to_owned(),
        "Allow Entropy in Accessibility" => "Разрешите Entropy в Accessibility".to_owned(),
        "If prompted, allow Entropy in Input Monitoring too" => "Если потребуется, разрешите Entropy и в Input Monitoring".to_owned(),
        "Restart Entropy after changing permissions" => "Перезапустите Entropy после изменения разрешений".to_owned(),
        "X11: install xdotool and keep Entropy running" => "X11: установите xdotool и оставляйте Entropy запущенной".to_owned(),
        "Wayland + IBus: install Entropy Universal Symbols and select it as an input source" => "Wayland + IBus: установите Entropy Universal Symbols и выберите его источником ввода".to_owned(),
        "Wayland + Fcitx5: install the addon, restart Fcitx5, and enable Entropy Universal Symbols" => "Wayland + Fcitx5: установите addon, перезапустите Fcitx5 и включите Entropy Universal Symbols".to_owned(),
        "Universal Symbols are not supported on this OS yet" => "Universal Symbols пока не поддерживаются на этой ОС".to_owned(),
        "Open Config → Universal Symbols to finish permissions setup" => "Откройте Настройки → Универсальные символы, чтобы завершить настройку разрешений".to_owned(),
        "Open Config → Universal Symbols to finish Linux setup" => "Откройте Настройки → Универсальные символы, чтобы завершить настройку Linux".to_owned(),
        "disabled" | "Disabled" => "Отключено".to_owned(),
        "oled master" | "OLED Master" | "OLED master" => "OLED мастер".to_owned(),
        "oled slave" | "OLED Slave" | "OLED slave" => "OLED ведомый".to_owned(),
        "clock" | "Clock" => "Часы".to_owned(),
        "volume" | "Volume" => "Громкость".to_owned(),
        "media" | "Media" => "Медиа".to_owned(),
        "Default" | "default" => "По умолчанию".to_owned(),
        "Unknown" => "Неизвестно".to_owned(),
        "Universal output backend: Windows native" => "Бэкенд универсального вывода: Windows native".to_owned(),
        "Universal output backend: macOS native — requires Accessibility/Input Monitoring permission" => "Бэкенд универсального вывода: macOS native — нужны разрешения Accessibility/Input Monitoring".to_owned(),
        "Universal output backend: unsupported on this OS" => "Бэкенд универсального вывода: эта ОС не поддерживается".to_owned(),
        "Esc" => "Esc".to_owned(),
        "Escape" => "Escape".to_owned(),
        "Backspace" => "Backspace".to_owned(),
        "Insert" => "Insert".to_owned(),
        "Delete" => "Delete".to_owned(),
        "Caps\nLock" | "Caps Lock" => "Caps\nLock".to_owned(),
        "Print\nScreen" | "Print Screen" => "Print\nScreen".to_owned(),
        "Scroll\nLock" | "Scroll Lock" => "Scroll\nLock".to_owned(),
        "Page\nUp" | "Page Up" => "Page\nUp".to_owned(),
        "Page\nDown" | "Page Down" => "Page\nDown".to_owned(),
        "Space" => "Пробел".to_owned(),
        "Menu" => "Меню".to_owned(),
        "Pause" => "Pause".to_owned(),
        "Home" => "Home".to_owned(),
        "End" => "End".to_owned(),
        "Left\nCtrl" => "Левый\nCtrl".to_owned(),
        "Right\nCtrl" => "Правый\nCtrl".to_owned(),
        "Left\nShift" => "Левый\nShift".to_owned(),
        "Right\nShift" => "Правый\nShift".to_owned(),
        "Left\nAlt" => "Левый\nAlt".to_owned(),
        "Right\nAlt" => "Правый\nAlt".to_owned(),
        "No key — this key does nothing" => "No key — эта клавиша ничего не делает".to_owned(),
        "Transparent — uses the key assigned on the layer below" => "Transparent — использует клавишу со слоя ниже".to_owned(),
        "Enter — confirm / new line" => "Enter — подтвердить / новая строка".to_owned(),
        "Escape — cancel / close" => "Escape — отмена / закрыть".to_owned(),
        "Backspace — delete character before cursor" => "Backspace — удалить символ перед курсором".to_owned(),
        "Tab — indent / move focus forward" => "Tab — отступ / фокус вперёд".to_owned(),
        "Caps Lock — toggle uppercase input" => "Caps Lock — переключить верхний регистр".to_owned(),
        "Menu key — open right-click context menu" => "Menu — открыть контекстное меню".to_owned(),
        "Minus — type -, Shift gives underscore (_)" => "Минус — вводит -, Shift даёт нижнее подчёркивание (_)".to_owned(),
        "Equals — type =, Shift gives plus (+)" => "Равно — вводит =, Shift даёт плюс (+)".to_owned(),
        "Left bracket — type [, Shift gives left brace ({)" => "Левая скобка — вводит [, Shift даёт левую фигурную скобку ({)".to_owned(),
        "Right bracket — type ], Shift gives right brace (})" => "Правая скобка — вводит ], Shift даёт правую фигурную скобку (})".to_owned(),
        "Backslash — type \\, Shift gives pipe (|)" => "Обратный слэш — вводит \\, Shift даёт вертикальную черту (|)".to_owned(),
        "Non-US hash key — type #, Shift gives tilde (~)" => "Non-US решётка — вводит #, Shift даёт тильду (~)".to_owned(),
        "Semicolon key — tap for semicolon (;), Shift gives colon (:)" => "Точка с запятой — tap вводит ;, Shift даёт двоеточие (:)".to_owned(),
        "Quote — type apostrophe ('), Shift gives double quote (\")" => "Кавычка — вводит апостроф ('), Shift даёт двойную кавычку (\")".to_owned(),
        "Grave accent — type `, Shift gives tilde (~)" => "Гравис — вводит `, Shift даёт тильду (~)".to_owned(),
        "Comma — type comma (,), Shift gives less-than (<)" => "Запятая — вводит запятую (,), Shift даёт знак «меньше» (<)".to_owned(),
        "Period — type dot (.), Shift gives greater-than (>)" => "Точка — вводит точку (.), Shift даёт знак «больше» (>)".to_owned(),
        "Slash — type /, Shift gives question mark (?)" => "Слэш — вводит /, Shift даёт вопросительный знак (?)".to_owned(),
        "Non-US backslash key — type \\, Shift gives pipe (|)" => "Non-US обратный слэш — вводит \\, Shift даёт вертикальную черту (|)".to_owned(),
        "Home — jump to beginning of line" => "Home — перейти к началу строки".to_owned(),
        "End — jump to end of line" => "End — перейти к концу строки".to_owned(),
        "Page Up — scroll up one page" => "Page Up — прокрутить на страницу вверх".to_owned(),
        "Page Down — scroll down one page" => "Page Down — прокрутить на страницу вниз".to_owned(),
        "Insert — toggle insert/overwrite mode" => "Insert — переключить режим вставки/замены".to_owned(),
        "Delete — delete character after cursor" => "Delete — удалить символ после курсора".to_owned(),
        "Print Screen — take a screenshot" => "Print Screen — сделать скриншот".to_owned(),
        "Pause / Break" => "Pause / Break".to_owned(),
        "Execute — run the currently selected action or file" => "Execute — запустить выбранное действие или файл".to_owned(),
        "Help — open help for the current app or context" => "Help — открыть справку текущего приложения или контекста".to_owned(),
        "Select — select the current item" => "Select — выбрать текущий элемент".to_owned(),
        "Stop — cancel the current action or loading" => "Stop — отменить текущее действие или загрузку".to_owned(),
        "Again — repeat the previous action" => "Again — повторить предыдущее действие".to_owned(),
        "Undo — revert the last action" => "Undo — отменить последнее действие".to_owned(),
        "Cut — remove selection and copy it to clipboard" => "Cut — вырезать выделение в буфер обмена".to_owned(),
        "Copy — copy selection to clipboard" => "Copy — копировать выделение в буфер обмена".to_owned(),
        "Paste — insert clipboard contents" => "Paste — вставить содержимое буфера обмена".to_owned(),
        "Find — search in the current document or view" => "Find — поиск в текущем документе или окне".to_owned(),
        "Mouse acceleration 0 — slowest cursor speed profile" => "Ускорение мыши 0 — самый медленный профиль скорости курсора".to_owned(),
        "Mouse acceleration 1 — medium cursor speed profile" => "Ускорение мыши 1 — средний профиль скорости курсора".to_owned(),
        "Mouse acceleration 2 — fastest cursor speed profile" => "Ускорение мыши 2 — самый быстрый профиль скорости курсора".to_owned(),
        "JIS \\ and _" => "JIS \\ и _".to_owned(),
        "JIS Katakana/Hiragana" => "JIS Katakana/Hiragana".to_owned(),
        "JIS ¥ and |" => "JIS ¥ и |".to_owned(),
        "JIS Henkan" => "JIS Henkan".to_owned(),
        "JIS Muhenkan" => "JIS Muhenkan".to_owned(),
        "JIS Numpad ," => "JIS нампад ,".to_owned(),
        "Hangul/English" => "Hangul/English".to_owned(),
        "Hanja" => "Hanja".to_owned(),
        "JIS Katakana" => "JIS Katakana".to_owned(),
        "JIS Hiragana" => "JIS Hiragana".to_owned(),
        "JIS Zenkaku/Hankaku" => "JIS Zenkaku/Hankaku".to_owned(),
        "Compile firmware" => "Скомпилировать прошивку".to_owned(),
        "RGB lighting — toggle on/off" => "RGB-подсветка — вкл/выкл".to_owned(),
        "RGB lighting — next animation mode" => "RGB-подсветка — следующий режим анимации".to_owned(),
        "RGB lighting — previous animation mode" => "RGB-подсветка — предыдущий режим анимации".to_owned(),
        "RGB lighting — hue +" => "RGB-подсветка — оттенок +".to_owned(),
        "RGB lighting — hue −" => "RGB-подсветка — оттенок −".to_owned(),
        "RGB lighting — saturation +" => "RGB-подсветка — насыщенность +".to_owned(),
        "RGB lighting — saturation −" => "RGB-подсветка — насыщенность −".to_owned(),
        "RGB lighting — brightness +" => "RGB-подсветка — яркость +".to_owned(),
        "RGB lighting — brightness −" => "RGB-подсветка — яркость −".to_owned(),
        "RGB lighting — animation speed +" => "RGB-подсветка — скорость анимации +".to_owned(),
        "RGB lighting — animation speed −" => "RGB-подсветка — скорость анимации −".to_owned(),
        "✕\nNone" => "✕\nНет".to_owned(),
        "▽\nInherit" => "▽\nНиже".to_owned(),
        "🔒\nLock" => "🔒\nLock".to_owned(),
        "Combo\nToggle" => "Combo\nВкл/выкл".to_owned(),
        "KC_NO — disables this key completely, it sends nothing when pressed" => {
            "KC_NO — полностью отключает клавишу, при нажатии ничего не отправляется".to_owned()
        }
        "KC_TRNS — inherits the key from the layer below" => {
            "KC_TRNS — наследует клавишу со слоя ниже".to_owned()
        }
        other if other.starts_with("Grave/Escape — sends Esc normally") => other
            .replace("Grave/Escape — sends Esc normally, ` when Shift or", "Grave/Escape — обычно отправляет Esc, ` при удержании Shift или")
            .replace("is held", ""),
        "QK_BOOT — put keyboard into flash mode" | "Bootloader — put keyboard into flash mode" => {
            "QK_BOOT — перевести клавиатуру в режим прошивки".to_owned()
        }
        "DB_TOGG — toggle debug mode" | "Debug toggle — enable/disable debug output" => {
            "DB_TOGG — переключить debug-режим".to_owned()
        }
        "QK_LOCK — hold to lock remaining keys until pressed again"
        | "Lock — lock a key in pressed state until pressed again" => {
            "QK_LOCK — удерживать клавишу нажатой до повторного нажатия".to_owned()
        }
        "Toggles the state of the Auto Shift feature" => "Переключает состояние Auto Shift".to_owned(),
        "Toggles Combo feature on and off" => "Включает или выключает Combo".to_owned(),
        "Capitalizes until end of current word" => "Верхний регистр до конца текущего слова".to_owned(),
        "Repeats the last pressed key" => "Повторяет последнюю нажатую клавишу".to_owned(),
        "Alt repeats the last pressed key" => "Alt-повтор последней нажатой клавиши".to_owned(),
        "Mouse\nUp" => "Мышь\n↑".to_owned(),
        "Mouse\nDown" => "Мышь\n↓".to_owned(),
        "Mouse\nLeft" => "Мышь\n←".to_owned(),
        "Mouse\nRight" => "Мышь\n→".to_owned(),
        "Scroll\nUp" => "Скролл\n↑".to_owned(),
        "Scroll\nDown" => "Скролл\n↓".to_owned(),
        "Scroll\nLeft" => "Скролл\n←".to_owned(),
        "Scroll\nRight" => "Скролл\n→".to_owned(),
        "Accel\n0" => "Ускор.\n0".to_owned(),
        "Accel\n1" => "Ускор.\n1".to_owned(),
        "Accel\n2" => "Ускор.\n2".to_owned(),
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
        "⏻\nPower" => "⏻\nПитание".to_owned(),
        "🌙\nSleep" => "🌙\nСон".to_owned(),
        "☀\nWake" => "☀\nWake".to_owned(),
        "🔇\nMute" => "🔇\nMute".to_owned(),
        "🔉\nVol-" => "🔉\nГр-".to_owned(),
        "🔊\nVol+" => "🔊\nГр+".to_owned(),
        "⏮\nPrev" => "⏮\nНазад".to_owned(),
        "⏭\nNext" => "⏭\nВперёд".to_owned(),
        "⏹\nStop" => "⏹\nСтоп".to_owned(),
        "🎵\nMedia" => "🎵\nМедиа".to_owned(),
        "✉\nMail" => "✉\nПочта".to_owned(),
        "🖩\nCalc" => "🖩\nКальк".to_owned(),
        "💻\nFiles" => "💻\nФайлы".to_owned(),
        "🔍\nSearch" => "🔍\nПоиск".to_owned(),
        "⬅\nBack" => "⬅\nНазад".to_owned(),
        "➡\nForward" => "➡\nВперёд".to_owned(),
        "↻\nRefresh" => "↻\nОбнов".to_owned(),
        "★\nFavs" => "★\nИзбр".to_owned(),
        "⏩\nFwd" => "⏩\nВпер".to_owned(),
        "⏪\nRew" => "⏪\nНазад".to_owned(),
        "☀+\nBright" => "☀+\nЯрк".to_owned(),
        "☀-\nBright" => "☀-\nЯрк".to_owned(),
        "Ctrl\nView" => "Ctrl\nОкна".to_owned(),
        "Launch\nPad" => "Launch\nPad".to_owned(),
        "Mute / Unmute audio" => "Включить/выключить звук".to_owned(),
        "Volume Up" => "Громкость выше".to_owned(),
        "Volume Down" => "Громкость ниже".to_owned(),
        "Next Track" => "Следующий трек".to_owned(),
        "Previous Track" => "Предыдущий трек".to_owned(),
        "Stop playback" => "Остановить воспроизведение".to_owned(),
        "Play / Pause" => "Воспроизведение / пауза".to_owned(),
        "Open media player" => "Открыть медиаплеер".to_owned(),
        "Open email client" => "Открыть почтовый клиент".to_owned(),
        "Open calculator" => "Открыть калькулятор".to_owned(),
        "Open My Computer / file manager" => "Открыть файловый менеджер".to_owned(),
        "Browser search" => "Поиск в браузере".to_owned(),
        "Browser home page" => "Домашняя страница браузера".to_owned(),
        "Browser back" => "Браузер назад".to_owned(),
        "Browser forward" => "Браузер вперёд".to_owned(),
        "Browser stop loading" => "Остановить загрузку в браузере".to_owned(),
        "Browser refresh" => "Обновить страницу".to_owned(),
        "Browser favourites" => "Избранное браузера".to_owned(),
        "Sleep — put computer to sleep" => "Sleep — перевести компьютер в сон".to_owned(),
        "Wake — wake computer from sleep" => "Wake — вывести компьютер из сна".to_owned(),
        "Brightness Up" => "Яркость выше".to_owned(),
        "Brightness Down" => "Яркость ниже".to_owned(),
        "Power — system power button" => "Power — системная кнопка питания".to_owned(),
        "Eject — eject removable media" => "Eject — извлечь съёмный носитель".to_owned(),
        "Fast Forward — jump forward in media" => "Fast Forward — перемотка вперёд".to_owned(),
        "Rewind — jump backward in media" => "Rewind — перемотка назад".to_owned(),
        "Mission Control / Task View — show open windows and spaces" => "Mission Control / Task View — показать открытые окна".to_owned(),
        "Launchpad / app launcher" => "Launchpad / запуск приложений".to_owned(),
        "Undo" => "Отмена".to_owned(),
        "On" => "Вкл".to_owned(),
        "Off" => "Выкл".to_owned(),
        "⏯\nPlay" => "⏯\nPlay".to_owned(),
        "⏏\nEject" => "⏏\nEject".to_owned(),
        "🏠\nHome" => "🏠\nHome".to_owned(),
        "Redo" => "Повтор".to_owned(),
        "Cut" => "Вырез".to_owned(),
        "Copy" => "Копия".to_owned(),
        "Paste" => "Встав".to_owned(),
        "Find" => "Поиск".to_owned(),
        "Prev\nWord" => "Пред.\nслово".to_owned(),
        "Next\nWord" => "След.\nслово".to_owned(),
        "Prev\nApp" => "Пред.\nприл.".to_owned(),
        "Next\nApp" => "След.\nприл.".to_owned(),
        "Lock" => "Lock".to_owned(),
        "Swap" => "Обмен".to_owned(),
        "Restore" => "Сброс".to_owned(),
        "Toggle" => "Вкл/выкл".to_owned(),
        "as Caps" => "как Caps".to_owned(),
        "as Ctrl" => "как Ctrl".to_owned(),
        "Left" => "Лево".to_owned(),
        "Right" => "Право".to_owned(),
        "Num Lock — toggle numpad number input" => "Num Lock — переключить цифровой ввод нампада".to_owned(),
        "Numpad ÷ (divide)" => "Нампад ÷ (деление)".to_owned(),
        "Numpad × (multiply)" => "Нампад × (умножение)".to_owned(),
        "Numpad − (minus)" => "Нампад − (минус)".to_owned(),
        "Numpad + (plus)" => "Нампад + (плюс)".to_owned(),
        "Numpad Enter" => "Нампад Enter".to_owned(),
        "Numpad . (decimal point)" => "Нампад . (десятичная точка)".to_owned(),
        "Numpad = (equals)" => "Нампад = (равно)".to_owned(),
        "Numpad , (comma)" => "Нампад , (запятая)".to_owned(),
        "Left Control when held, ( when tapped" => "Левый Control при hold, ( при tap".to_owned(),
        "Right Control when held, ) when tapped" => "Правый Control при hold, ) при tap".to_owned(),
        "Left Shift when held, ( when tapped" => "Левый Shift при hold, ( при tap".to_owned(),
        "Right Shift when held, ) when tapped" => "Правый Shift при hold, ) при tap".to_owned(),
        "Left Alt when held, ( when tapped" => "Левый Alt при hold, ( при tap".to_owned(),
        "Right Alt when held, ) when tapped" => "Правый Alt при hold, ) при tap".to_owned(),
        "Right Shift when held, Enter when tapped" => "Правый Shift при hold, Enter при tap".to_owned(),
        "Left Ctrl — modifier key (hold to activate shortcuts)" => "Левый Ctrl — модификатор для сочетаний клавиш".to_owned(),
        "Right Ctrl — modifier key (hold to activate shortcuts)" => "Правый Ctrl — модификатор для сочетаний клавиш".to_owned(),
        "Left Shift — hold to type uppercase / shifted symbols" => "Левый Shift — удерживайте для верхнего регистра и shifted-символов".to_owned(),
        "Right Shift — hold to type uppercase / shifted symbols" => "Правый Shift — удерживайте для верхнего регистра и shifted-символов".to_owned(),
        "Left Alt — modifier key (hold to activate shortcuts)" => "Левый Alt — модификатор для сочетаний клавиш".to_owned(),
        "Right Alt / AltGr — access special characters" => "Правый Alt / AltGr — доступ к спецсимволам".to_owned(),
        other if other.starts_with("Left Cmd, ") => other
            .replace("Left Cmd, macOS modifier key and app shortcuts", "Левый Cmd — модификатор macOS и сочетания приложений"),
        other if other.starts_with("Right Cmd, ") => other
            .replace("Right Cmd, macOS modifier key and app shortcuts", "Правый Cmd — модификатор macOS и сочетания приложений"),
        other if other.starts_with("Left Win, ") => other
            .replace("Left Win, Windows modifier key and OS shortcuts", "Левый Win — модификатор Windows и системные сочетания"),
        other if other.starts_with("Right Win, ") => other
            .replace("Right Win, Windows modifier key and OS shortcuts", "Правый Win — модификатор Windows и системные сочетания"),
        other if other.starts_with("Left Super, ") => other
            .replace("Left Super, desktop modifier key and OS shortcuts", "Левый Super — модификатор рабочего стола и системные сочетания"),
        other if other.starts_with("Right Super, ") => other
            .replace("Right Super, desktop modifier key and OS shortcuts", "Правый Super — модификатор рабочего стола и системные сочетания"),
        "Plain modifier — hold for left/right side, tap nothing" => "Обычный модификатор — удержание левой/правой стороны, tap ничего не отправляет".to_owned(),
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
        other if other.starts_with("Hold ") && other.contains(" with the key you choose next") => other
            .replace("Hold ", "Удерживать ")
            .replace("Left ", "левый ")
            .replace("Right ", "правый ")
            .replace(" together with the key you choose next", " вместе со следующей выбранной клавишей"),
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
            .replace(" — active for the next keypress only", " — активен только для следующего нажатия")
            .replace(" — applies ", " — применяет ")
            .replace(" to the next keypress only", " только к следующему нажатию")
            .replace(" — activates ", " — активирует ")
            .replace(" for the very next keypress only", " только для следующего нажатия"),
        other if other.starts_with("RGB Matrix: solid color") => "RGB Matrix: сплошной цвет без анимации".to_owned(),
        other if other.starts_with("RGB Matrix: breathing effect") => "RGB Matrix: breathing-эффект с плавным изменением яркости".to_owned(),
        other if other.starts_with("RGB Matrix: rainbow gradient") => "RGB Matrix: радужный градиент по всем клавишам".to_owned(),
        other if other.starts_with("RGB Matrix: swirling rainbow") => "RGB Matrix: вращающийся радужный паттерн".to_owned(),
        other if other.starts_with("RGB Matrix: snake animation") => "RGB Matrix: анимация змейки по клавишам".to_owned(),
        other if other.starts_with("RGB Matrix: Knight Rider") => "RGB Matrix: сканирующий эффект Knight Rider".to_owned(),
        other if other.starts_with("RGB Matrix: alternating red and green") => "RGB Matrix: чередование красного и зелёного как рождественская подсветка".to_owned(),
        other if other.starts_with("RGB Matrix: static gradient") => "RGB Matrix: статичный градиент".to_owned(),
        other if other.starts_with("RGB Matrix: test mode") => "RGB Matrix: тестовый режим, циклически R/G/B".to_owned(),
        "Arrow Up" => "Стрелка вверх".to_owned(),
        "Arrow Down" => "Стрелка вниз".to_owned(),
        "Arrow Left" => "Стрелка влево".to_owned(),
        "Arrow Right" => "Стрелка вправо".to_owned(),
        "Left Control" => "Левый Control".to_owned(),
        "Right Control" => "Правый Control".to_owned(),
        "Left Shift" => "Левый Shift".to_owned(),
        "Right Shift" => "Правый Shift".to_owned(),
        "Left Alt" => "Левый Alt".to_owned(),
        "Right Alt" => "Правый Alt".to_owned(),
        "Swap Caps Lock and Left Control" => "Поменять Caps Lock и левый Control".to_owned(),
        "Unswap Caps Lock and Left Control" => "Вернуть Caps Lock и левый Control".to_owned(),
        "Toggle Caps Lock and Left Control swap" => "Переключить обмен Caps Lock и левого Control".to_owned(),
        "Stop treating Caps Lock as Control" => "Перестать считать Caps Lock клавишей Control".to_owned(),
        "Treat Caps Lock as Control" => "Считать Caps Lock клавишей Control".to_owned(),
        "Swap ` and Escape" => "Поменять ` и Escape".to_owned(),
        "Unswap ` and Escape" => "Вернуть ` и Escape".to_owned(),
        "Swap \\ and Backspace" => "Поменять \\ и Backspace".to_owned(),
        "Unswap \\ and Backspace" => "Вернуть \\ и Backspace".to_owned(),
        "Toggle \\ and Backspace swap state" => "Переключить обмен \\ и Backspace".to_owned(),
        "Enable N-key rollover" => "Включить N-key rollover".to_owned(),
        "Disable N-key rollover" => "Отключить N-key rollover".to_owned(),
        "Toggle N-key rollover" => "Переключить N-key rollover".to_owned(),
        "Set the master half of a split keyboard as the left hand (for EE_HANDS)" => "Назначить мастер-половину сплита левой стороной (для EE_HANDS)".to_owned(),
        "Set the master half of a split keyboard as the right hand (for EE_HANDS)" => "Назначить мастер-половину сплита правой стороной (для EE_HANDS)".to_owned(),
        "Swap Caps Lock and Escape" => "Поменять Caps Lock и Escape".to_owned(),
        "Unswap Caps Lock and Escape" => "Вернуть Caps Lock и Escape".to_owned(),
        "Toggle Caps Lock and Escape swap" => "Переключить обмен Caps Lock и Escape".to_owned(),
        other if other.starts_with("Swap Left Alt and ") => other
            .replace("Swap Left Alt and ", "Поменять левый Alt и "),
        other if other.starts_with("Unswap Left Alt and ") => other
            .replace("Unswap Left Alt and ", "Вернуть левый Alt и "),
        other if other.starts_with("Swap Right Alt and ") => other
            .replace("Swap Right Alt and ", "Поменять правый Alt и "),
        other if other.starts_with("Unswap Right Alt and ") => other
            .replace("Unswap Right Alt and ", "Вернуть правый Alt и "),
        other if other.starts_with("Enable the ") && other.ends_with(" keys") => other
            .replace("Enable the ", "Включить клавиши ")
            .replace(" keys", ""),
        other if other.starts_with("Disable the ") && other.ends_with(" keys") => other
            .replace("Disable the ", "Отключить клавиши ")
            .replace(" keys", ""),
        other if other.starts_with("Toggles the status of the ") => other
            .replace("Toggles the status of the ", "Переключить состояние клавиш ")
            .replace(" keys", ""),
        other if other.starts_with("Swap Alt and ") && other.ends_with(" on both sides") => other
            .replace("Swap Alt and ", "Поменять Alt и ")
            .replace(" on both sides", " с обеих сторон"),
        other if other.starts_with("Unswap Alt and ") && other.ends_with(" on both sides") => other
            .replace("Unswap Alt and ", "Вернуть Alt и ")
            .replace(" on both sides", " с обеих сторон"),
        other if other.starts_with("Toggle Alt and ") && other.ends_with(" swap on both sides") => other
            .replace("Toggle Alt and ", "Переключить обмен Alt и ")
            .replace(" swap on both sides", " с обеих сторон"),
        other if other.starts_with("Swap Left Control and ") => other
            .replace("Swap Left Control and ", "Поменять левый Control и "),
        other if other.starts_with("Unswap Left Control and ") => other
            .replace("Unswap Left Control and ", "Вернуть левый Control и "),
        other if other.starts_with("Swap Right Control and ") => other
            .replace("Swap Right Control and ", "Поменять правый Control и "),
        other if other.starts_with("Unswap Right Control and ") => other
            .replace("Unswap Right Control and ", "Вернуть правый Control и "),
        other if other.starts_with("Swap Control and ") && other.ends_with(" on both sides") => other
            .replace("Swap Control and ", "Поменять Control и ")
            .replace(" on both sides", " с обеих сторон"),
        other if other.starts_with("Unswap Control and ") && other.ends_with(" on both sides") => other
            .replace("Unswap Control and ", "Вернуть Control и ")
            .replace(" on both sides", " с обеих сторон"),
        other if other.starts_with("Toggle Control and ") && other.ends_with(" swap on both sides") => other
            .replace("Toggle Control and ", "Переключить обмен Control и ")
            .replace(" swap on both sides", " с обеих сторон"),
        other if other.starts_with("Layer ") && other[6..].chars().all(|ch| ch.is_ascii_digit()) => {
            other.replace("Layer ", "Слой ")
        }
        other if other.starts_with("Pick key for ") => other.replace("Pick key for ", "Выбрать клавишу для "),
        other if other.starts_with("Momentarily activate layer ") => other
            .replace("Momentarily activate layer ", "Моментально активировать слой ")
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
        other if other.ends_with(" function key") => other.replace(" function key", " — функциональная клавиша"),
        other if other.starts_with("Numpad ") => other.replace("Numpad ", "Нампад "),
        other if other.starts_with("Shortcut: ") => other.replace("Shortcut: ", "Сочетание: ").replace("Right ", "Правый "),
        other if other.starts_with("Macro ") && other.contains(" — sends a sequence of keystrokes") => other
            .replace("Macro ", "Макрос ")
            .replace(" — sends a sequence of keystrokes", " — отправляет последовательность нажатий"),
        other if other.starts_with("Tap Dance ") && other.contains(" — different actions on tap, hold, double tap") => other
            .replace(" — different actions on tap, hold, double tap", " — разные действия на tap, hold и double tap"),
        other if other.contains(" — macro ") => other.replace(" — macro ", " — макрос "),
        other if other.contains(" — tap dance ") => other.replace(" — tap dance ", " — Tap Dance "),
        other if other.starts_with("MO(") => other
            .replace(" — activate layer ", " — активировать слой ")
            .replace(" — activate ", " — активировать ")
            .replace(" while held, return when released", " при удержании, вернуть при отпускании"),
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
            .replace(", hold to activate while held", ", hold активирует при удержании"),
        other if other.starts_with("Layer Tap — tap for ") => other
            .replace("Layer Tap — tap for ", "Layer Tap — tap для ")
            .replace(", hold to activate layer ", ", hold активирует слой ")
            .replace(", hold to activate ", ", hold активирует "),
        other if other.starts_with("LM(") => other
            .replace(" — activate layer ", " — активировать слой ")
            .replace(" with ", " с ")
            .replace(" held while key is pressed", " при удержании во время нажатия клавиши"),
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
            .replace("consistently regardless of the active keyboard language", "одинаково независимо от активного языка клавиатуры")
            .replace("hold Shift for", "удерживайте Shift для"),
        other if other.starts_with("Universal output backend: Wayland via IBus/Fcitx5 input method") => other.replacen("Universal output backend: Wayland via IBus/Fcitx5 input method", "Бэкенд универсального вывода: Wayland через IBus/Fcitx5", 1),
        other if other.starts_with("Universal output backend: Linux X11 native") => other.replacen("Universal output backend: Linux X11 native; Wayland uses IBus/Fcitx5", "Бэкенд универсального вывода: Linux X11 native; Wayland использует IBus/Fcitx5", 1),
        other if other.starts_with("Universal output backend: Linux; use IBus/Fcitx5 for Wayland") => other.replacen("Universal output backend: Linux; use IBus/Fcitx5 for Wayland", "Бэкенд универсального вывода: Linux; для Wayland используйте IBus/Fcitx5", 1),
        other => other.to_owned(),
    }
}
