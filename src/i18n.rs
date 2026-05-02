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
        "Active for next keypress only" => "Активен только для следующего нажатия",
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
        "Plain modifier — hold for left/right side, tap nothing" => "Обычный модификатор — удержание левой/правой стороны, tap ничего не отправляет".to_owned(),
        other if other.starts_with("Hold ") && other.contains(" with the key you choose next") => other
            .replace("Hold ", "Удерживать ")
            .replace(" together with the key you choose next", " вместе со следующей выбранной клавишей"),
        other if other.starts_with("Dual-role key: hold for ") => other
            .replace("Dual-role key: hold for ", "Dual-role клавиша: hold для ")
            .replace(", tap for the key you choose next", ", tap для следующей выбранной клавиши"),
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
        other if other.starts_with("Key: ") => other.replace("Key: ", "Клавиша: "),
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
