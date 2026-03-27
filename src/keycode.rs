/// QMK/Vial keycode definitions.
/// Reference: https://github.com/qmk/qmk_firmware/blob/master/quantum/keycodes.h

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
    Navigation,
    Numpad,
    Media,
    Mouse,
    Layer,
    Special,
}

pub const KEYCODES: &[Keycode] = &[
    // ── Special ──────────────────────────────────────────────────────────────
    Keycode { value: 0x0000, name: "KC_NO",      label: "∅",      category: KeycodeCategory::Special },
    Keycode { value: 0x0001, name: "KC_TRNS",    label: "▽",      category: KeycodeCategory::Special },

    // ── Letters ──────────────────────────────────────────────────────────────
    Keycode { value: 0x0004, name: "KC_A",       label: "A",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0005, name: "KC_B",       label: "B",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0006, name: "KC_C",       label: "C",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0007, name: "KC_D",       label: "D",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0008, name: "KC_E",       label: "E",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0009, name: "KC_F",       label: "F",      category: KeycodeCategory::Basic },
    Keycode { value: 0x000A, name: "KC_G",       label: "G",      category: KeycodeCategory::Basic },
    Keycode { value: 0x000B, name: "KC_H",       label: "H",      category: KeycodeCategory::Basic },
    Keycode { value: 0x000C, name: "KC_I",       label: "I",      category: KeycodeCategory::Basic },
    Keycode { value: 0x000D, name: "KC_J",       label: "J",      category: KeycodeCategory::Basic },
    Keycode { value: 0x000E, name: "KC_K",       label: "K",      category: KeycodeCategory::Basic },
    Keycode { value: 0x000F, name: "KC_L",       label: "L",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0010, name: "KC_M",       label: "M",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0011, name: "KC_N",       label: "N",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0012, name: "KC_O",       label: "O",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0013, name: "KC_P",       label: "P",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0014, name: "KC_Q",       label: "Q",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0015, name: "KC_R",       label: "R",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0016, name: "KC_S",       label: "S",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0017, name: "KC_T",       label: "T",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0018, name: "KC_U",       label: "U",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0019, name: "KC_V",       label: "V",      category: KeycodeCategory::Basic },
    Keycode { value: 0x001A, name: "KC_W",       label: "W",      category: KeycodeCategory::Basic },
    Keycode { value: 0x001B, name: "KC_X",       label: "X",      category: KeycodeCategory::Basic },
    Keycode { value: 0x001C, name: "KC_Y",       label: "Y",      category: KeycodeCategory::Basic },
    Keycode { value: 0x001D, name: "KC_Z",       label: "Z",      category: KeycodeCategory::Basic },

    // ── Numbers ──────────────────────────────────────────────────────────────
    Keycode { value: 0x001E, name: "KC_1",       label: "1",      category: KeycodeCategory::Basic },
    Keycode { value: 0x001F, name: "KC_2",       label: "2",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0020, name: "KC_3",       label: "3",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0021, name: "KC_4",       label: "4",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0022, name: "KC_5",       label: "5",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0023, name: "KC_6",       label: "6",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0024, name: "KC_7",       label: "7",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0025, name: "KC_8",       label: "8",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0026, name: "KC_9",       label: "9",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0027, name: "KC_0",       label: "0",      category: KeycodeCategory::Basic },

    // ── Common control ───────────────────────────────────────────────────────
    Keycode { value: 0x0028, name: "KC_ENT",     label: "↵",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0029, name: "KC_ESC",     label: "Esc",    category: KeycodeCategory::Basic },
    Keycode { value: 0x002A, name: "KC_BSPC",    label: "⌫",     category: KeycodeCategory::Basic },
    Keycode { value: 0x002B, name: "KC_TAB",     label: "Tab",    category: KeycodeCategory::Basic },
    Keycode { value: 0x002C, name: "KC_SPC",     label: "Spc",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0039, name: "KC_CAPS",    label: "Caps",   category: KeycodeCategory::Basic },

    // ── Punctuation ──────────────────────────────────────────────────────────
    Keycode { value: 0x002D, name: "KC_MINS",    label: "-",      category: KeycodeCategory::Basic },
    Keycode { value: 0x002E, name: "KC_EQL",     label: "=",      category: KeycodeCategory::Basic },
    Keycode { value: 0x002F, name: "KC_LBRC",    label: "[",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0030, name: "KC_RBRC",    label: "]",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0031, name: "KC_BSLS",    label: "\\",     category: KeycodeCategory::Basic },
    Keycode { value: 0x0033, name: "KC_SCLN",    label: ";",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0034, name: "KC_QUOT",    label: "'",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0035, name: "KC_GRV",     label: "`",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0036, name: "KC_COMM",    label: ",",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0037, name: "KC_DOT",     label: ".",      category: KeycodeCategory::Basic },
    Keycode { value: 0x0038, name: "KC_SLSH",    label: "/",      category: KeycodeCategory::Basic },

    // ── Function keys ────────────────────────────────────────────────────────
    Keycode { value: 0x003A, name: "KC_F1",      label: "F1",     category: KeycodeCategory::Function },
    Keycode { value: 0x003B, name: "KC_F2",      label: "F2",     category: KeycodeCategory::Function },
    Keycode { value: 0x003C, name: "KC_F3",      label: "F3",     category: KeycodeCategory::Function },
    Keycode { value: 0x003D, name: "KC_F4",      label: "F4",     category: KeycodeCategory::Function },
    Keycode { value: 0x003E, name: "KC_F5",      label: "F5",     category: KeycodeCategory::Function },
    Keycode { value: 0x003F, name: "KC_F6",      label: "F6",     category: KeycodeCategory::Function },
    Keycode { value: 0x0040, name: "KC_F7",      label: "F7",     category: KeycodeCategory::Function },
    Keycode { value: 0x0041, name: "KC_F8",      label: "F8",     category: KeycodeCategory::Function },
    Keycode { value: 0x0042, name: "KC_F9",      label: "F9",     category: KeycodeCategory::Function },
    Keycode { value: 0x0043, name: "KC_F10",     label: "F10",    category: KeycodeCategory::Function },
    Keycode { value: 0x0044, name: "KC_F11",     label: "F11",    category: KeycodeCategory::Function },
    Keycode { value: 0x0045, name: "KC_F12",     label: "F12",    category: KeycodeCategory::Function },
    Keycode { value: 0x0068, name: "KC_F13",     label: "F13",    category: KeycodeCategory::Function },
    Keycode { value: 0x0069, name: "KC_F14",     label: "F14",    category: KeycodeCategory::Function },
    Keycode { value: 0x006A, name: "KC_F15",     label: "F15",    category: KeycodeCategory::Function },
    Keycode { value: 0x006B, name: "KC_F16",     label: "F16",    category: KeycodeCategory::Function },
    Keycode { value: 0x006C, name: "KC_F17",     label: "F17",    category: KeycodeCategory::Function },
    Keycode { value: 0x006D, name: "KC_F18",     label: "F18",    category: KeycodeCategory::Function },
    Keycode { value: 0x006E, name: "KC_F19",     label: "F19",    category: KeycodeCategory::Function },
    Keycode { value: 0x006F, name: "KC_F20",     label: "F20",    category: KeycodeCategory::Function },
    Keycode { value: 0x0070, name: "KC_F21",     label: "F21",    category: KeycodeCategory::Function },
    Keycode { value: 0x0071, name: "KC_F22",     label: "F22",    category: KeycodeCategory::Function },
    Keycode { value: 0x0072, name: "KC_F23",     label: "F23",    category: KeycodeCategory::Function },
    Keycode { value: 0x0073, name: "KC_F24",     label: "F24",    category: KeycodeCategory::Function },

    // ── Navigation ───────────────────────────────────────────────────────────
    Keycode { value: 0x0046, name: "KC_PSCR",    label: "PrtSc",  category: KeycodeCategory::Navigation },
    Keycode { value: 0x0047, name: "KC_SCRL",    label: "ScrLk",  category: KeycodeCategory::Navigation },
    Keycode { value: 0x0048, name: "KC_PAUS",    label: "Pause",  category: KeycodeCategory::Navigation },
    Keycode { value: 0x0049, name: "KC_INS",     label: "Ins",    category: KeycodeCategory::Navigation },
    Keycode { value: 0x004A, name: "KC_HOME",    label: "Home",   category: KeycodeCategory::Navigation },
    Keycode { value: 0x004B, name: "KC_PGUP",    label: "PgUp",   category: KeycodeCategory::Navigation },
    Keycode { value: 0x004C, name: "KC_DEL",     label: "Del",    category: KeycodeCategory::Navigation },
    Keycode { value: 0x004D, name: "KC_END",     label: "End",    category: KeycodeCategory::Navigation },
    Keycode { value: 0x004E, name: "KC_PGDN",    label: "PgDn",   category: KeycodeCategory::Navigation },
    Keycode { value: 0x004F, name: "KC_RGHT",    label: "→",      category: KeycodeCategory::Navigation },
    Keycode { value: 0x0050, name: "KC_LEFT",    label: "←",      category: KeycodeCategory::Navigation },
    Keycode { value: 0x0051, name: "KC_DOWN",    label: "↓",      category: KeycodeCategory::Navigation },
    Keycode { value: 0x0052, name: "KC_UP",      label: "↑",      category: KeycodeCategory::Navigation },

    // ── Numpad ───────────────────────────────────────────────────────────────
    Keycode { value: 0x0053, name: "KC_NUM",     label: "NmLk",   category: KeycodeCategory::Numpad },
    Keycode { value: 0x0054, name: "KC_PSLS",    label: "N/",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x0055, name: "KC_PAST",    label: "N*",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x0056, name: "KC_PMNS",    label: "N-",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x0057, name: "KC_PPLS",    label: "N+",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x0058, name: "KC_PENT",    label: "N↵",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x0059, name: "KC_P1",      label: "N1",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x005A, name: "KC_P2",      label: "N2",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x005B, name: "KC_P3",      label: "N3",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x005C, name: "KC_P4",      label: "N4",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x005D, name: "KC_P5",      label: "N5",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x005E, name: "KC_P6",      label: "N6",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x005F, name: "KC_P7",      label: "N7",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x0060, name: "KC_P8",      label: "N8",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x0061, name: "KC_P9",      label: "N9",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x0062, name: "KC_P0",      label: "N0",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x0063, name: "KC_PDOT",    label: "N.",     category: KeycodeCategory::Numpad },
    Keycode { value: 0x0067, name: "KC_PEQL",    label: "N=",     category: KeycodeCategory::Numpad },

    // ── Modifiers ────────────────────────────────────────────────────────────
    Keycode { value: 0x00E0, name: "KC_LCTL",    label: "LCtl",   category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E1, name: "KC_LSFT",    label: "LSft",   category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E2, name: "KC_LALT",    label: "LAlt",   category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E3, name: "KC_LGUI",    label: "LGui",   category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E4, name: "KC_RCTL",    label: "RCtl",   category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E5, name: "KC_RSFT",    label: "RSft",   category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E6, name: "KC_RALT",    label: "RAlt",   category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E7, name: "KC_RGUI",    label: "RGui",   category: KeycodeCategory::Modifier },

    // ── Media (QMK values from quantum/keycodes.h) ───────────────────────────
    Keycode { value: 0x00A8, name: "KC_MUTE",    label: "Mute",   category: KeycodeCategory::Media },
    Keycode { value: 0x00A9, name: "KC_VOLU",    label: "Vol+",   category: KeycodeCategory::Media },
    Keycode { value: 0x00AA, name: "KC_VOLD",    label: "Vol-",   category: KeycodeCategory::Media },
    Keycode { value: 0x00AB, name: "KC_MNXT",    label: "⏭",     category: KeycodeCategory::Media },
    Keycode { value: 0x00AC, name: "KC_MPRV",    label: "⏮",     category: KeycodeCategory::Media },
    Keycode { value: 0x00AD, name: "KC_MSTP",    label: "⏹",     category: KeycodeCategory::Media },
    Keycode { value: 0x00AE, name: "KC_MPLY",    label: "⏯",     category: KeycodeCategory::Media },
    Keycode { value: 0x00AF, name: "KC_MSEL",    label: "MSel",   category: KeycodeCategory::Media },
    Keycode { value: 0x00B0, name: "KC_MAIL",    label: "Mail",   category: KeycodeCategory::Media },
    Keycode { value: 0x00B1, name: "KC_CALC",    label: "Calc",   category: KeycodeCategory::Media },
    Keycode { value: 0x00B2, name: "KC_MYCM",    label: "MyPC",   category: KeycodeCategory::Media },
    Keycode { value: 0x00B3, name: "KC_WSCH",    label: "Srch",   category: KeycodeCategory::Media },
    Keycode { value: 0x00B4, name: "KC_WHOM",    label: "WWW",    category: KeycodeCategory::Media },
    Keycode { value: 0x00B5, name: "KC_WBAK",    label: "Back",   category: KeycodeCategory::Media },
    Keycode { value: 0x00B6, name: "KC_WFWD",    label: "Fwd",    category: KeycodeCategory::Media },
    Keycode { value: 0x00B7, name: "KC_WSTP",    label: "Stop",   category: KeycodeCategory::Media },
    Keycode { value: 0x00B8, name: "KC_WREF",    label: "Rfsh",   category: KeycodeCategory::Media },
    Keycode { value: 0x00B9, name: "KC_WFAV",    label: "Favs",   category: KeycodeCategory::Media },

    // ── Extra keys ───────────────────────────────────────────────────────────
    Keycode { value: 0x0065, name: "KC_APP",     label: "App",    category: KeycodeCategory::Basic },
    Keycode { value: 0x0066, name: "KC_PWR",     label: "Pwr",    category: KeycodeCategory::Special },
    Keycode { value: 0x00A5, name: "KC_SLEP",    label: "Sleep",  category: KeycodeCategory::Special },
    Keycode { value: 0x00A6, name: "KC_WAKE",    label: "Wake",   category: KeycodeCategory::Special },
    Keycode { value: 0x00A7, name: "KC_BRIU",    label: "Bri+",   category: KeycodeCategory::Special },
    Keycode { value: 0x00BB, name: "KC_BRID",    label: "Bri-",   category: KeycodeCategory::Special },

    // ── Mouse ────────────────────────────────────────────────────────────────
    Keycode { value: 0x00F0, name: "KC_MS_U",    label: "M↑",     category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F1, name: "KC_MS_D",    label: "M↓",     category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F2, name: "KC_MS_L",    label: "M←",     category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F3, name: "KC_MS_R",    label: "M→",     category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F4, name: "KC_BTN1",    label: "MB1",    category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F5, name: "KC_BTN2",    label: "MB2",    category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F6, name: "KC_BTN3",    label: "MB3",    category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F7, name: "KC_BTN4",    label: "MB4",    category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F8, name: "KC_BTN5",    label: "MB5",    category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F9, name: "KC_WH_U",    label: "WH↑",    category: KeycodeCategory::Mouse },
    Keycode { value: 0x00FA, name: "KC_WH_D",    label: "WH↓",    category: KeycodeCategory::Mouse },
    Keycode { value: 0x00FB, name: "KC_WH_L",    label: "WH←",    category: KeycodeCategory::Mouse },
    Keycode { value: 0x00FC, name: "KC_WH_R",    label: "WH→",    category: KeycodeCategory::Mouse },
    Keycode { value: 0x00FD, name: "KC_ACL0",    label: "MAc0",   category: KeycodeCategory::Mouse },
    Keycode { value: 0x00FE, name: "KC_ACL1",    label: "MAc1",   category: KeycodeCategory::Mouse },
    Keycode { value: 0x00FF, name: "KC_ACL2",    label: "MAc2",   category: KeycodeCategory::Mouse },

    // ── Layer keys ───────────────────────────────────────────────────────────
    Keycode { value: 0x5100, name: "MO(0)",      label: "MO(0)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5101, name: "MO(1)",      label: "MO(1)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5102, name: "MO(2)",      label: "MO(2)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5103, name: "MO(3)",      label: "MO(3)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5104, name: "MO(4)",      label: "MO(4)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5105, name: "MO(5)",      label: "MO(5)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5300, name: "TG(0)",      label: "TG(0)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5301, name: "TG(1)",      label: "TG(1)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5302, name: "TG(2)",      label: "TG(2)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5303, name: "TG(3)",      label: "TG(3)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5304, name: "TG(4)",      label: "TG(4)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5305, name: "TG(5)",      label: "TG(5)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5000, name: "TO(0)",      label: "TO(0)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5001, name: "TO(1)",      label: "TO(1)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5002, name: "TO(2)",      label: "TO(2)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5003, name: "TO(3)",      label: "TO(3)",  category: KeycodeCategory::Layer },
    Keycode { value: 0x5400, name: "OSL(0)",     label: "OSL(0)", category: KeycodeCategory::Layer },
    Keycode { value: 0x5401, name: "OSL(1)",     label: "OSL(1)", category: KeycodeCategory::Layer },
    Keycode { value: 0x5402, name: "OSL(2)",     label: "OSL(2)", category: KeycodeCategory::Layer },
    Keycode { value: 0x5403, name: "OSL(3)",     label: "OSL(3)", category: KeycodeCategory::Layer },

    // ── One-shot modifiers ───────────────────────────────────────────────────
    Keycode { value: 0x5500, name: "OSM(MOD_LSFT)", label: "OSM\nLSft", category: KeycodeCategory::Modifier },
    Keycode { value: 0x5501, name: "OSM(MOD_LCTL)", label: "OSM\nLCtl", category: KeycodeCategory::Modifier },
    Keycode { value: 0x5502, name: "OSM(MOD_LALT)", label: "OSM\nLAlt", category: KeycodeCategory::Modifier },
    Keycode { value: 0x5504, name: "OSM(MOD_LGUI)", label: "OSM\nLGui", category: KeycodeCategory::Modifier },
    Keycode { value: 0x5510, name: "OSM(MOD_RSFT)", label: "OSM\nRSft", category: KeycodeCategory::Modifier },
    Keycode { value: 0x5520, name: "OSM(MOD_RCTL)", label: "OSM\nRCtl", category: KeycodeCategory::Modifier },
    Keycode { value: 0x5540, name: "OSM(MOD_RALT)", label: "OSM\nRAlt", category: KeycodeCategory::Modifier },

    // ── QMK special ─────────────────────────────────────────────────────────
    Keycode { value: 0x7C77, name: "QK_BOOT",    label: "Boot",   category: KeycodeCategory::Special },
    Keycode { value: 0x7C16, name: "QK_RBT",     label: "Reset",  category: KeycodeCategory::Special },
    Keycode { value: 0x5C16, name: "EE_CLR",     label: "EEClr",  category: KeycodeCategory::Special },
    Keycode { value: 0x7800, name: "QK_LOCK",    label: "Lock",   category: KeycodeCategory::Special },
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

    // QMK layer keycode ranges
    if value & 0xFF00 == 0x5100 { return format!("MO({})", value & 0xFF); }
    if value & 0xFF00 == 0x5000 { return format!("TO({})", value & 0xFF); }
    if value & 0xFF00 == 0x5200 { return format!("DF({})", value & 0xFF); }
    if value & 0xFF00 == 0x5300 { return format!("TG({})", value & 0xFF); }
    if value & 0xFF00 == 0x5400 { return format!("OSL({})", value & 0xFF); }

    // LT(layer, kc): bit 14 set, bits [13:8] = layer, bits [7:0] = kc
    if value & 0xC000 == 0x4000 {
        let layer = (value >> 8) & 0xF;
        let kc = value & 0xFF;
        let kc_str = find_keycode(kc as u16).map(|k| k.label).unwrap_or("?");
        return format!("LT{}/{}", layer, kc_str);
    }

    // MT(mod, kc): bit 13 set
    if value & 0xE000 == 0x2000 {
        let kc = value & 0xFF;
        let kc_str = find_keycode(kc as u16).map(|k| k.label).unwrap_or("?");
        let mods = (value >> 8) & 0x1F;
        let mod_str = match mods {
            0x01 => "LSft",
            0x02 => "LCtl",
            0x04 => "LAlt",
            0x08 => "LGui",
            0x11 => "RSft",
            0x12 => "RCtl",
            0x14 => "RAlt",
            0x18 => "RGui",
            _ => "Mod",
        };
        return format!("{}/{}", mod_str, kc_str);
    }

    // KC_TRNS fallback
    if value == 0x0001 { return "▽".to_string(); }

    format!("{:04X}", value)
}
