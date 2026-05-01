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
        Key::EncoderHoverZoomTooltip => "Enlarge encoder controls while hovering them on the layout",
        Key::AccentColorLabel => "Accent color",
        Key::AccentColorTooltip => "Choose the color used for active states and highlights",
    }
}

fn ru(key: Key) -> &'static str {
    match key {
        Key::AppSettingsTitle => "Настройки приложения",
        Key::AppSettingsDescription => "Поведение Entropy и визуальный акцент",
        Key::LanguageLabel => "Язык",
        Key::LanguageTooltip => "Выбрать язык интерфейса приложения",
        Key::CloseToTrayLabel => "Закрывать в трей",
        Key::CloseToTrayTooltip => "Прятать Entropy в системный трей вместо выхода при закрытии окна",
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
