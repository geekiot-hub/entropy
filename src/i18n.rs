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
        Key::MatrixTesterDescription => {
            "Press switches on the keyboard to verify every matrix position"
        }
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
        Key::AltRepeatDescription => "Configure alternate repeat keys and modifier behavior",
        Key::AltRepeatUnavailable => "Alt Repeat is not supported by this keyboard",
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
        Key::MouseKeysDescription => "Tune mouse cursor and wheel movement parameters",
        Key::MouseKeysUnavailable => "Mouse keys settings are not available on this firmware",
        Key::MouseKeysEnableHint => {
            "Enable MOUSEKEY_ENABLE and QMK_SETTINGS in the keyboard rules.mk to use this page"
        }
        Key::RgbDescription => "Adjust lighting, effects, color and brightness",
        Key::RgbConnect => "Connect a Vial keyboard to edit RGB settings",
        Key::KeyOverridesDescription => "Override one key with custom modifier rules",
        Key::ComboDescription => "Press multiple keys together to send a separate keycode",
        Key::EncodersDescription => "Show or hide encoder controls for this device",
        Key::EncodersUnavailable => "No encoder controls are available for this device",
        Key::DisplayPresetsDescription => "Configure OLED and display presets",
        Key::DisplayPresetsUnavailable => "No display presets are exposed by this keyboard",
        Key::DisplayPresetsConnect => "Connect a Vial keyboard to edit display presets",
        Key::AutoShiftDescription => "Adjust hold threshold and typing behavior",
        Key::AutoShiftUnavailable => "Auto Shift is not enabled in this firmware",
        Key::AutoShiftEnableHint => {
            "Enable AUTO_SHIFT_ENABLE in the keyboard rules.mk to use this page"
        }
        Key::KeyboardLocked => "Keyboard is locked",
        Key::AutoShiftUnlockHint => "Unlock it from the Device menu to edit Auto Shift settings",
        Key::LayerLedsDescription => "Set LED brightness, timeout and per-layer colors",
        Key::LayerLedsUnavailable => "Layer LED settings are not available on this firmware",
        Key::LayerLedsEnableHint => {
            "Enable Ergohaven LED QMK_SETTINGS in the keyboard firmware to use this page"
        }
        Key::LayerLedsConnect => "Connect a Vial keyboard to edit Layer LED settings",
        Key::GraveEscapeDescription => "Choose which modifiers make Grave Escape send Esc",
        Key::GraveEscapeUnavailable => "Grave Escape settings are not available on this firmware",
        Key::GraveEscapeEnableHint => {
            "Enable QMK_SETTINGS and Grave Escape support in firmware to use this page"
        }
        Key::GraveEscapeConnect => "Connect a Vial keyboard to edit Grave Escape settings",
        Key::MagicDescription => "Tune global QMK keyboard behavior swaps",
        Key::MagicUnavailable => "Magic settings are not available on this firmware",
        Key::MagicEnableHint => {
            "Enable QMK_SETTINGS and Magic in the keyboard firmware to use this page"
        }
        Key::MagicConnect => "Connect a Vial keyboard to edit Magic settings",
        Key::TapHoldOneShotDescription => {
            "Tune dual-role keys, one-shot modifiers and one-shot layers"
        }
        Key::TapHoldOneShotUnavailable => {
            "Tap-Hold & One Shot settings are not available on this firmware"
        }
        Key::QmkSettingsEnableHint => {
            "Enable QMK_SETTINGS in the keyboard rules.mk to use this page"
        }
        Key::TapHoldOneShotConnect => {
            "Connect a Vial keyboard to edit Tap-Hold & One Shot settings"
        }
        Key::LiveFeaturesDescription => "Entropy-powered live data for firmware features",
        Key::LiveFeaturesInactive => "Live Features are not active for this keyboard",
        Key::LiveFeaturesSelectHint => {
            "Select a preset or connect firmware that uses Entropy-powered live data"
        }
        Key::LiveFeaturesReadyNote => "No manual setup is needed when all rows are ready",
        Key::TouchpadDescription => "Tune K:03 Pro touchpad pointer, scroll and mode behavior",
        Key::TouchpadUnavailable => "Touchpad settings are not available on this firmware",
        Key::TouchpadEnableHint => {
            "Connect a K:03 Pro firmware with Touchpad QMK_SETTINGS to use this page"
        }
        Key::TouchpadConnect => "Connect a Vial keyboard to edit Touchpad settings",
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
        Key::AutoShiftTitle => "Авто-Shift",
        Key::KeyOverridesTitle => "Переопределения клавиш",
        Key::MouseKeysTitle => "Клавиши мыши",
        Key::MatrixTesterTitle => "Тестер матрицы",
        Key::MatrixTesterDescription => "Нажимайте свитчи, чтобы проверить каждую позицию матрицы",
        Key::UniversalSymbolsSetupTitle => "Настройка универсальных символов",
        Key::UniversalSymbolsTitle => "Универсальные символы",
        Key::RgbTitle => "RGB",
        Key::LayerLedsTitle => "Подсветка слоёв",
        Key::EncodersTitle => "Энкодеры",
        Key::DisplayPresetsTitle => "Пресеты дисплея",
        Key::TouchpadTitle => "Тачпад",
        Key::LiveFeaturesTitle => "Live-интеграции",
        Key::MagicTitle => "Magic-настройки",
        Key::TapHoldOneShotTitle => "Tap-Hold и One Shot",
        Key::AltRepeatTitle => "Alt Repeat",
        Key::AltRepeatDescription => "Configure alternate repeat keys and modifier behavior",
        Key::AltRepeatUnavailable => "Alt Repeat is not supported by this keyboard",
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
        Key::MouseKeysDescription => "Параметры движения курсора и колеса мыши",
        Key::MouseKeysUnavailable => "Клавиши мыши недоступны в этой прошивке",
        Key::MouseKeysEnableHint => "Включите MOUSEKEY_ENABLE и QMK_SETTINGS в rules.mk клавиатуры",
        Key::RgbDescription => "Подсветка, эффекты, цвет и яркость",
        Key::RgbConnect => "Подключите Vial-клавиатуру, чтобы менять RGB-настройки",
        Key::KeyOverridesDescription => "Замена клавиш по правилам модификаторов",
        Key::ComboDescription => "Несколько клавиш вместе отправляют отдельный keycode",
        Key::EncodersDescription => "Показать или скрыть энкодеры для этого устройства",
        Key::EncodersUnavailable => "Для этого устройства нет доступных энкодеров",
        Key::DisplayPresetsDescription => "OLED и пресеты дисплея",
        Key::DisplayPresetsUnavailable => "Эта клавиатура не сообщает пресеты дисплея",
        Key::DisplayPresetsConnect => "Подключите Vial-клавиатуру, чтобы менять пресеты дисплея",
        Key::AutoShiftDescription => "Порог удержания и поведение при наборе",
        Key::AutoShiftUnavailable => "Auto Shift не включён в этой прошивке",
        Key::AutoShiftEnableHint => "Включите AUTO_SHIFT_ENABLE в rules.mk клавиатуры",
        Key::KeyboardLocked => "Клавиатура заблокирована",
        Key::AutoShiftUnlockHint => "Разблокируйте её в меню устройства, чтобы менять Auto Shift",
        Key::LayerLedsDescription => "Яркость, таймаут и цвета подсветки по слоям",
        Key::LayerLedsUnavailable => "Подсветка слоёв недоступна в этой прошивке",
        Key::LayerLedsEnableHint => "Включите Ergohaven LED QMK_SETTINGS в прошивке клавиатуры",
        Key::LayerLedsConnect => "Подключите Vial-клавиатуру, чтобы менять подсветку слоёв",
        Key::GraveEscapeDescription => "Модификаторы, при которых Grave Escape отправляет Esc",
        Key::GraveEscapeUnavailable => "Grave Escape недоступен в этой прошивке",
        Key::GraveEscapeEnableHint => "Включите QMK_SETTINGS и поддержку Grave Escape в прошивке",
        Key::GraveEscapeConnect => "Подключите Vial-клавиатуру, чтобы менять Grave Escape",
        Key::MagicDescription => "Глобальные переключатели поведения QMK",
        Key::MagicUnavailable => "Magic-настройки недоступны в этой прошивке",
        Key::MagicEnableHint => "Включите QMK_SETTINGS и Magic в прошивке клавиатуры",
        Key::MagicConnect => "Подключите Vial-клавиатуру, чтобы менять Magic-настройки",
        Key::TapHoldOneShotDescription => {
            "Dual-role клавиши, one-shot модификаторы и one-shot слои"
        }
        Key::TapHoldOneShotUnavailable => "Tap-Hold и One Shot недоступны в этой прошивке",
        Key::QmkSettingsEnableHint => "Включите QMK_SETTINGS в rules.mk клавиатуры",
        Key::TapHoldOneShotConnect => {
            "Подключите Vial-клавиатуру, чтобы менять Tap-Hold и One Shot"
        }
        Key::LiveFeaturesDescription => "Live-данные Entropy для функций прошивки",
        Key::LiveFeaturesInactive => "Live-интеграции не активны для этой клавиатуры",
        Key::LiveFeaturesSelectHint => {
            "Выберите пресет или подключите прошивку с live-данными Entropy"
        }
        Key::LiveFeaturesReadyNote => "Когда все строки готовы, ручная настройка не нужна",
        Key::TouchpadDescription => "Указатель, скролл и режимы тачпада K:03 Pro",
        Key::TouchpadUnavailable => "Тачпад недоступен в этой прошивке",
        Key::TouchpadEnableHint => "Подключите прошивку K:03 Pro с Touchpad QMK_SETTINGS",
        Key::TouchpadConnect => "Подключите Vial-клавиатуру, чтобы менять настройки тачпада",
    }
}
