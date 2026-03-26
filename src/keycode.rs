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

/// Returns a human-readable label for a keycode.
/// For unknown keycodes, decodes QMK special forms or falls back to hex.
pub fn keycode_label(value: u16) -> String {
    if let Some(kc) = find_keycode(value) {
        return kc.label.to_string();
    }

    // QMK special keycode ranges
    // MO(n) = 0x5100 + n  (momentary layer)
    // LT(n, kc) = 0x4000 | (n << 8) | kc  — bit 14 set + layer in bits 8-11
    // OSL(n) = 0x5400 + n
    // TO(n)  = 0x5000 + n
    // TG(n)  = 0x5300 + n
    // DF(n)  = 0x5200 + n

    if value & 0xFF00 == 0x5100 { return format!("MO({})", value & 0xFF); }
    if value & 0xFF00 == 0x5000 { return format!("TO({})", value & 0xFF); }
    if value & 0xFF00 == 0x5200 { return format!("DF({})", value & 0xFF); }
    if value & 0xFF00 == 0x5300 { return format!("TG({})", value & 0xFF); }
    if value & 0xFF00 == 0x5400 { return format!("OSL({})", value & 0xFF); }

    // LT(layer, kc): bits [13:8] = layer, bits [7:0] = basic keycode, bit 14 set
    if value & 0xC000 == 0x4000 {
        let layer = (value >> 8) & 0xF;
        let kc = value & 0xFF;
        let kc_str = find_keycode(kc as u16).map(|k| k.label).unwrap_or("?");
        return format!("LT{}/{}", layer, kc_str);
    }

    // MT(mod, kc): bits [12:8] = mod, bit 13 set
    if value & 0xE000 == 0x2000 {
        let kc = value & 0xFF;
        let kc_str = find_keycode(kc as u16).map(|k| k.label).unwrap_or("?");
        return format!("MT/{}", kc_str);
    }

    // KC_TRNS
    if value == 0x0001 { return "▽".to_string(); }

    // Unknown — show hex
    format!("{:04X}", value)
}
