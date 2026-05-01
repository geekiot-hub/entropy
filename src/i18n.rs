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

pub fn tr_static(language: Language, text: &'static str) -> &'static str {
    if !matches!(language, Language::Russian) {
        return text;
    }

    match text {
        "Right-click or Esc to return to layout" => "ПКМ или Esc — вернуться к раскладке",
        "Open Privacy Settings" => "Открыть настройки приватности",
        "Effect" => "Эффект",
        "Color" => "Цвет",
        "Speed" => "Скорость",
        "Brightness" => "Яркость",
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
        "Waiting for a keyboard" => "Ожидание клавиатуры",
        "Connect a Vial device" => "Подключите Vial-устройство",
        "Light" => "Светлая",
        "Dark" => "Тёмная",
        "☀ Light" => "☀ Светлая",
        "🌙 Dark" => "🌙 Тёмная",
        "🔓 Unlock Keyboard" => "🔓 Разблокировать клавиатуру",
        "Unlock Keyboard" => "Разблокировать клавиатуру",
        "Press and hold the highlighted keys one by one" => {
            "Нажмите и удерживайте подсвеченные клавиши по очереди"
        }
        "Keyboard is locked, unlock it to edit macros" => {
            "Клавиатура заблокирована — разблокируйте её для редактирования макросов"
        }
        "Macros saved" => "Макросы сохранены",
        "Combos saved" => "Комбо сохранены",
        "Combo timeout saved" => "Таймаут комбо сохранён",
        "Pick key" => "Выбрать клавишу",
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
        "Flow tap" => "Flow tap timeout",
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
        "clock" | "Clock" => "Часы".to_owned(),
        "volume" | "Volume" => "Громкость".to_owned(),
        "media" | "Media" => "Медиа".to_owned(),
        "Default" | "default" => "По умолчанию".to_owned(),
        "Unknown" => "Неизвестно".to_owned(),
        "Universal output backend: Windows native" => "Бэкенд универсального вывода: Windows native".to_owned(),
        "Universal output backend: macOS native — requires Accessibility/Input Monitoring permission" => "Бэкенд универсального вывода: macOS native — нужны разрешения Accessibility/Input Monitoring".to_owned(),
        "Universal output backend: unsupported on this OS" => "Бэкенд универсального вывода: эта ОС не поддерживается".to_owned(),
        other if other.starts_with("Universal output backend: Wayland via IBus/Fcitx5 input method") => other.replacen("Universal output backend: Wayland via IBus/Fcitx5 input method", "Бэкенд универсального вывода: Wayland через IBus/Fcitx5", 1),
        other if other.starts_with("Universal output backend: Linux X11 native") => other.replacen("Universal output backend: Linux X11 native; Wayland uses IBus/Fcitx5", "Бэкенд универсального вывода: Linux X11 native; Wayland использует IBus/Fcitx5", 1),
        other if other.starts_with("Universal output backend: Linux; use IBus/Fcitx5 for Wayland") => other.replacen("Universal output backend: Linux; use IBus/Fcitx5 for Wayland", "Бэкенд универсального вывода: Linux; для Wayland используйте IBus/Fcitx5", 1),
        other => other.to_owned(),
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
