/// Basic QMK keycode definitions.
/// Full list: https://github.com/qmk/qmk_firmware/blob/master/quantum/keycodes.h

#[derive(Debug, Clone, PartialEq)]
pub struct Keycode {
    pub value: u16,
    pub name: &'static str,
    pub label: &'static str,
    pub category: KeycodeCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeycodeCategory {
    Basic,
    Modifier,
    Function,
    Layer,
    Media,
    Special,
}

pub const KEYCODES: &[Keycode] = &[
    Keycode { value: 0x0000, name: "KC_NO",      label: "∅",    category: KeycodeCategory::Special },
    Keycode { value: 0x0001, name: "KC_TRNS",    label: "▽",    category: KeycodeCategory::Special },
    Keycode { value: 0x0004, name: "KC_A",       label: "A",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0005, name: "KC_B",       label: "B",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0006, name: "KC_C",       label: "C",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0007, name: "KC_D",       label: "D",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0008, name: "KC_E",       label: "E",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0009, name: "KC_F",       label: "F",    category: KeycodeCategory::Basic },
    Keycode { value: 0x000A, name: "KC_G",       label: "G",    category: KeycodeCategory::Basic },
    Keycode { value: 0x000B, name: "KC_H",       label: "H",    category: KeycodeCategory::Basic },
    Keycode { value: 0x000C, name: "KC_I",       label: "I",    category: KeycodeCategory::Basic },
    Keycode { value: 0x000D, name: "KC_J",       label: "J",    category: KeycodeCategory::Basic },
    Keycode { value: 0x000E, name: "KC_K",       label: "K",    category: KeycodeCategory::Basic },
    Keycode { value: 0x000F, name: "KC_L",       label: "L",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0010, name: "KC_M",       label: "M",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0011, name: "KC_N",       label: "N",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0012, name: "KC_O",       label: "O",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0013, name: "KC_P",       label: "P",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0014, name: "KC_Q",       label: "Q",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0015, name: "KC_R",       label: "R",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0016, name: "KC_S",       label: "S",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0017, name: "KC_T",       label: "T",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0018, name: "KC_U",       label: "U",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0019, name: "KC_V",       label: "V",    category: KeycodeCategory::Basic },
    Keycode { value: 0x001A, name: "KC_W",       label: "W",    category: KeycodeCategory::Basic },
    Keycode { value: 0x001B, name: "KC_X",       label: "X",    category: KeycodeCategory::Basic },
    Keycode { value: 0x001C, name: "KC_Y",       label: "Y",    category: KeycodeCategory::Basic },
    Keycode { value: 0x001D, name: "KC_Z",       label: "Z",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0028, name: "KC_ENT",     label: "↵",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0029, name: "KC_ESC",     label: "Esc",  category: KeycodeCategory::Basic },
    Keycode { value: 0x002A, name: "KC_BSPC",    label: "⌫",   category: KeycodeCategory::Basic },
    Keycode { value: 0x002B, name: "KC_TAB",     label: "Tab",  category: KeycodeCategory::Basic },
    Keycode { value: 0x002C, name: "KC_SPC",     label: "Space",category: KeycodeCategory::Basic },
    Keycode { value: 0x00E0, name: "KC_LCTL",    label: "LCtl", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E1, name: "KC_LSFT",    label: "LSft", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E2, name: "KC_LALT",    label: "LAlt", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E3, name: "KC_LGUI",    label: "LGui", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E4, name: "KC_RCTL",    label: "RCtl", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E5, name: "KC_RSFT",    label: "RSft", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E6, name: "KC_RALT",    label: "RAlt", category: KeycodeCategory::Modifier },
    Keycode { value: 0x003A, name: "KC_F1",      label: "F1",   category: KeycodeCategory::Function },
    Keycode { value: 0x003B, name: "KC_F2",      label: "F2",   category: KeycodeCategory::Function },
    Keycode { value: 0x003C, name: "KC_F3",      label: "F3",   category: KeycodeCategory::Function },
    Keycode { value: 0x003D, name: "KC_F4",      label: "F4",   category: KeycodeCategory::Function },
    Keycode { value: 0x003E, name: "KC_F5",      label: "F5",   category: KeycodeCategory::Function },
    Keycode { value: 0x003F, name: "KC_F6",      label: "F6",   category: KeycodeCategory::Function },
    Keycode { value: 0x0040, name: "KC_F7",      label: "F7",   category: KeycodeCategory::Function },
    Keycode { value: 0x0041, name: "KC_F8",      label: "F8",   category: KeycodeCategory::Function },
    Keycode { value: 0x0042, name: "KC_F9",      label: "F9",   category: KeycodeCategory::Function },
    Keycode { value: 0x0043, name: "KC_F10",     label: "F10",  category: KeycodeCategory::Function },
    Keycode { value: 0x0044, name: "KC_F11",     label: "F11",  category: KeycodeCategory::Function },
    Keycode { value: 0x0045, name: "KC_F12",     label: "F12",  category: KeycodeCategory::Function },
];

pub fn find_keycode(value: u16) -> Option<&'static Keycode> {
    KEYCODES.iter().find(|k| k.value == value)
}

pub fn keycode_label(value: u16) -> &'static str {
    find_keycode(value).map(|k| k.label).unwrap_or("?")
}
