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
}

pub fn tr(language: Language, key: Key) -> &'static str {
    match language {
        Language::English => en(key),
        Language::Russian => ru(key),
    }
}

fn en(key: Key) -> &'static str {
    match key {
        Key::MainTabLayout => "Layout",
        Key::MainTabAdvanced => "Advanced",
        Key::MainTabConfig => "Config",
        Key::NoDevicesFound => "No devices found",
        Key::LockAction => "Lock",
        Key::UnlockAction => "Unlock",
        Key::ComboTitle => "Combo",
        Key::AutoShiftTitle => "Auto Shift",
        Key::KeyOverridesTitle => "Key Overrides",
        Key::MouseKeysTitle => "Mouse Keys",
        Key::MatrixTesterTitle => "Matrix Tester",
        Key::UniversalSymbolsSetupTitle => "Universal Symbols Setup",
        Key::UniversalSymbolsTitle => "Universal Symbols",
        Key::RgbTitle => "RGB",
        Key::LayerLedsTitle => "Layer LEDs",
        Key::EncodersTitle => "Encoders",
        Key::DisplayPresetsTitle => "Display Presets",
        Key::TouchpadTitle => "Touchpad",
        Key::LiveFeaturesTitle => "Live Features",
        Key::MagicTitle => "Magic",
        Key::TapHoldOneShotTitle => "Tap-Hold & One Shot",
        Key::AltRepeatTitle => "Alt Repeat",
        Key::GraveEscapeTitle => "Grave Escape",
        Key::RgbUnavailableTooltip => "RGB settings are not available on this firmware",
        Key::AppSettingsTitle => "App Settings",
        Key::AppSettingsDescription => "Configure Entropy behavior and visual accent",
        Key::LanguageLabel => "Language",
        Key::LanguageTooltip => "Choose the application interface language",
        Key::CloseToTrayLabel => "Close to tray",
        Key::CloseToTrayTooltip => {
            "Hide Entropy in the system tray instead of quitting when the window is closed"
        }
        Key::ShiftedNumberSymbolsLabel => "Shifted number symbols",
        Key::ShiftedNumberSymbolsTooltip => {
            "Show shifted symbols above number-row keys, like ! over 1 and @ over 2"
        }
        Key::LayerHoverPreviewLabel => "Layer hover preview",
        Key::LayerHoverPreviewTooltip => {
            "Preview the target layer while hovering layer keys on the layout"
        }
        Key::EncoderHoverZoomLabel => "Encoder hover zoom",
        Key::EncoderHoverZoomTooltip => {
            "Enlarge encoder controls while hovering them on the layout"
        }
        Key::AccentColorLabel => "Accent color",
        Key::AccentColorTooltip => "Choose the color used for active states and highlights",
    }
}

fn ru(key: Key) -> &'static str {
    match key {
        Key::MainTabLayout => "Раскладка",
        Key::MainTabAdvanced => "Дополнительно",
        Key::MainTabConfig => "Настройки",
        Key::NoDevicesFound => "Устройства не найдены",
        Key::LockAction => "Заблокировать",
        Key::UnlockAction => "Разблокировать",
        Key::ComboTitle => "Комбо",
        Key::AutoShiftTitle => "Auto Shift",
        Key::KeyOverridesTitle => "Переопределения клавиш",
        Key::MouseKeysTitle => "Mouse Keys",
        Key::MatrixTesterTitle => "Тестер матрицы",
        Key::UniversalSymbolsSetupTitle => "Настройка Universal Symbols",
        Key::UniversalSymbolsTitle => "Universal Symbols",
        Key::RgbTitle => "RGB",
        Key::LayerLedsTitle => "Layer LEDs",
        Key::EncodersTitle => "Энкодеры",
        Key::DisplayPresetsTitle => "Пресеты дисплея",
        Key::TouchpadTitle => "Тачпад",
        Key::LiveFeaturesTitle => "Live Features",
        Key::MagicTitle => "Magic",
        Key::TapHoldOneShotTitle => "Tap-Hold и One Shot",
        Key::AltRepeatTitle => "Alt Repeat",
        Key::GraveEscapeTitle => "Grave Escape",
        Key::RgbUnavailableTooltip => "RGB-настройки недоступны в этой прошивке",
        Key::AppSettingsTitle => "Настройки приложения",
        Key::AppSettingsDescription => "Поведение Entropy и визуальный акцент",
        Key::LanguageLabel => "Язык",
        Key::LanguageTooltip => "Выбрать язык интерфейса приложения",
        Key::CloseToTrayLabel => "Закрывать в трей",
        Key::CloseToTrayTooltip => {
            "Прятать Entropy в системный трей вместо выхода при закрытии окна"
        }
        Key::ShiftedNumberSymbolsLabel => "Символы Shift на цифрах",
        Key::ShiftedNumberSymbolsTooltip => {
            "Показывать символы с Shift над цифровым рядом, например ! над 1 и @ над 2"
        }
        Key::LayerHoverPreviewLabel => "Предпросмотр слоя",
        Key::LayerHoverPreviewTooltip => {
            "Показывать целевой слой при наведении на клавиши слоя на раскладке"
        }
        Key::EncoderHoverZoomLabel => "Увеличение энкодера",
        Key::EncoderHoverZoomTooltip => "Увеличивать энкодеры при наведении на раскладке",
        Key::AccentColorLabel => "Акцентный цвет",
        Key::AccentColorTooltip => "Выбрать цвет активных состояний и подсветки",
    }
}
