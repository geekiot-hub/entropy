/// QMK/Vial keycode definitions — protocol v6
/// Reference: vial-gui/src/main/python/keycodes/keycodes_v6.py

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
    Keycode { value: 0x0000, name: "KC_NO",     label: "NO",    category: KeycodeCategory::Special },
    Keycode { value: 0x0001, name: "KC_TRNS",   label: "TRNS",  category: KeycodeCategory::Special },

    // ── Letters ──────────────────────────────────────────────────────────────
    Keycode { value: 0x0004, name: "KC_A", label: "A", category: KeycodeCategory::Basic },
    Keycode { value: 0x0005, name: "KC_B", label: "B", category: KeycodeCategory::Basic },
    Keycode { value: 0x0006, name: "KC_C", label: "C", category: KeycodeCategory::Basic },
    Keycode { value: 0x0007, name: "KC_D", label: "D", category: KeycodeCategory::Basic },
    Keycode { value: 0x0008, name: "KC_E", label: "E", category: KeycodeCategory::Basic },
    Keycode { value: 0x0009, name: "KC_F", label: "F", category: KeycodeCategory::Basic },
    Keycode { value: 0x000A, name: "KC_G", label: "G", category: KeycodeCategory::Basic },
    Keycode { value: 0x000B, name: "KC_H", label: "H", category: KeycodeCategory::Basic },
    Keycode { value: 0x000C, name: "KC_I", label: "I", category: KeycodeCategory::Basic },
    Keycode { value: 0x000D, name: "KC_J", label: "J", category: KeycodeCategory::Basic },
    Keycode { value: 0x000E, name: "KC_K", label: "K", category: KeycodeCategory::Basic },
    Keycode { value: 0x000F, name: "KC_L", label: "L", category: KeycodeCategory::Basic },
    Keycode { value: 0x0010, name: "KC_M", label: "M", category: KeycodeCategory::Basic },
    Keycode { value: 0x0011, name: "KC_N", label: "N", category: KeycodeCategory::Basic },
    Keycode { value: 0x0012, name: "KC_O", label: "O", category: KeycodeCategory::Basic },
    Keycode { value: 0x0013, name: "KC_P", label: "P", category: KeycodeCategory::Basic },
    Keycode { value: 0x0014, name: "KC_Q", label: "Q", category: KeycodeCategory::Basic },
    Keycode { value: 0x0015, name: "KC_R", label: "R", category: KeycodeCategory::Basic },
    Keycode { value: 0x0016, name: "KC_S", label: "S", category: KeycodeCategory::Basic },
    Keycode { value: 0x0017, name: "KC_T", label: "T", category: KeycodeCategory::Basic },
    Keycode { value: 0x0018, name: "KC_U", label: "U", category: KeycodeCategory::Basic },
    Keycode { value: 0x0019, name: "KC_V", label: "V", category: KeycodeCategory::Basic },
    Keycode { value: 0x001A, name: "KC_W", label: "W", category: KeycodeCategory::Basic },
    Keycode { value: 0x001B, name: "KC_X", label: "X", category: KeycodeCategory::Basic },
    Keycode { value: 0x001C, name: "KC_Y", label: "Y", category: KeycodeCategory::Basic },
    Keycode { value: 0x001D, name: "KC_Z", label: "Z", category: KeycodeCategory::Basic },

    // ── Numbers ───────────────────────────────────────────────────────────────
    Keycode { value: 0x001E, name: "KC_1", label: "1", category: KeycodeCategory::Basic },
    Keycode { value: 0x001F, name: "KC_2", label: "2", category: KeycodeCategory::Basic },
    Keycode { value: 0x0020, name: "KC_3", label: "3", category: KeycodeCategory::Basic },
    Keycode { value: 0x0021, name: "KC_4", label: "4", category: KeycodeCategory::Basic },
    Keycode { value: 0x0022, name: "KC_5", label: "5", category: KeycodeCategory::Basic },
    Keycode { value: 0x0023, name: "KC_6", label: "6", category: KeycodeCategory::Basic },
    Keycode { value: 0x0024, name: "KC_7", label: "7", category: KeycodeCategory::Basic },
    Keycode { value: 0x0025, name: "KC_8", label: "8", category: KeycodeCategory::Basic },
    Keycode { value: 0x0026, name: "KC_9", label: "9", category: KeycodeCategory::Basic },
    Keycode { value: 0x0027, name: "KC_0", label: "0", category: KeycodeCategory::Basic },

    // ── Control ───────────────────────────────────────────────────────────────
    Keycode { value: 0x0028, name: "KC_ENTER",  label: "Enter", category: KeycodeCategory::Basic },
    Keycode { value: 0x0029, name: "KC_ESCAPE", label: "Esc",   category: KeycodeCategory::Basic },
    Keycode { value: 0x002A, name: "KC_BSPACE", label: "BkSp",  category: KeycodeCategory::Basic },
    Keycode { value: 0x002B, name: "KC_TAB",    label: "Tab",   category: KeycodeCategory::Basic },
    Keycode { value: 0x002C, name: "KC_SPACE",  label: "Space", category: KeycodeCategory::Basic },
    Keycode { value: 0x0039, name: "KC_CAPSLOCK",label: "Caps", category: KeycodeCategory::Basic },
    Keycode { value: 0x0065, name: "KC_APPLICATION", label: "Menu", category: KeycodeCategory::Basic },

    // ── Punctuation ───────────────────────────────────────────────────────────
    Keycode { value: 0x002D, name: "KC_MINUS",   label: "- _",  category: KeycodeCategory::Basic },
    Keycode { value: 0x002E, name: "KC_EQUAL",   label: "= +",  category: KeycodeCategory::Basic },
    Keycode { value: 0x002F, name: "KC_LBRACKET",label: "[ {",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0030, name: "KC_RBRACKET",label: "] }",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0031, name: "KC_BSLASH",  label: "\\ |", category: KeycodeCategory::Basic },
    Keycode { value: 0x0032, name: "KC_NONUS_HASH", label: "# ~", category: KeycodeCategory::Basic },
    Keycode { value: 0x0033, name: "KC_SCOLON",  label: "; :",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0034, name: "KC_QUOTE",   label: "' \"", category: KeycodeCategory::Basic },
    Keycode { value: 0x0035, name: "KC_GRAVE",   label: "` ~",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0036, name: "KC_COMMA",   label: ", <",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0037, name: "KC_DOT",     label: ". >",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0038, name: "KC_SLASH",   label: "/ ?",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0064, name: "KC_NONUS_BSLASH", label: "\\ |", category: KeycodeCategory::Basic },

    // ── Shifted symbols (LSFT(kc) shortcuts) ─────────────────────────────────
    // These are 0x0200 | basic_kc
    Keycode { value: 0x021E, name: "KC_EXLM",  label: "!",  category: KeycodeCategory::Basic },
    Keycode { value: 0x021F, name: "KC_AT",    label: "@",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0220, name: "KC_HASH",  label: "#",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0221, name: "KC_DLR",   label: "$",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0222, name: "KC_PERC",  label: "%",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0223, name: "KC_CIRC",  label: "^",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0224, name: "KC_AMPR",  label: "&",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0225, name: "KC_ASTR",  label: "*",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0226, name: "KC_LPRN",  label: "(",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0227, name: "KC_RPRN",  label: ")",  category: KeycodeCategory::Basic },
    Keycode { value: 0x022D, name: "KC_UNDS",  label: "_",  category: KeycodeCategory::Basic },
    Keycode { value: 0x022E, name: "KC_PLUS",  label: "+",  category: KeycodeCategory::Basic },
    Keycode { value: 0x022F, name: "KC_LCBR",  label: "{",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0230, name: "KC_RCBR",  label: "}",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0231, name: "KC_PIPE",  label: "|",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0233, name: "KC_COLN",  label: ":",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0234, name: "KC_DQUO",  label: "\"", category: KeycodeCategory::Basic },
    Keycode { value: 0x0235, name: "KC_TILD",  label: "~",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0236, name: "KC_LT",    label: "<",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0237, name: "KC_GT",    label: ">",  category: KeycodeCategory::Basic },
    Keycode { value: 0x0238, name: "KC_QUES",  label: "?",  category: KeycodeCategory::Basic },

    // ── Function keys ─────────────────────────────────────────────────────────
    Keycode { value: 0x003A, name: "KC_F1",  label: "F1",  category: KeycodeCategory::Function },
    Keycode { value: 0x003B, name: "KC_F2",  label: "F2",  category: KeycodeCategory::Function },
    Keycode { value: 0x003C, name: "KC_F3",  label: "F3",  category: KeycodeCategory::Function },
    Keycode { value: 0x003D, name: "KC_F4",  label: "F4",  category: KeycodeCategory::Function },
    Keycode { value: 0x003E, name: "KC_F5",  label: "F5",  category: KeycodeCategory::Function },
    Keycode { value: 0x003F, name: "KC_F6",  label: "F6",  category: KeycodeCategory::Function },
    Keycode { value: 0x0040, name: "KC_F7",  label: "F7",  category: KeycodeCategory::Function },
    Keycode { value: 0x0041, name: "KC_F8",  label: "F8",  category: KeycodeCategory::Function },
    Keycode { value: 0x0042, name: "KC_F9",  label: "F9",  category: KeycodeCategory::Function },
    Keycode { value: 0x0043, name: "KC_F10", label: "F10", category: KeycodeCategory::Function },
    Keycode { value: 0x0044, name: "KC_F11", label: "F11", category: KeycodeCategory::Function },
    Keycode { value: 0x0045, name: "KC_F12", label: "F12", category: KeycodeCategory::Function },
    Keycode { value: 0x0068, name: "KC_F13", label: "F13", category: KeycodeCategory::Function },
    Keycode { value: 0x0069, name: "KC_F14", label: "F14", category: KeycodeCategory::Function },
    Keycode { value: 0x006A, name: "KC_F15", label: "F15", category: KeycodeCategory::Function },
    Keycode { value: 0x006B, name: "KC_F16", label: "F16", category: KeycodeCategory::Function },
    Keycode { value: 0x006C, name: "KC_F17", label: "F17", category: KeycodeCategory::Function },
    Keycode { value: 0x006D, name: "KC_F18", label: "F18", category: KeycodeCategory::Function },
    Keycode { value: 0x006E, name: "KC_F19", label: "F19", category: KeycodeCategory::Function },
    Keycode { value: 0x006F, name: "KC_F20", label: "F20", category: KeycodeCategory::Function },
    Keycode { value: 0x0070, name: "KC_F21", label: "F21", category: KeycodeCategory::Function },
    Keycode { value: 0x0071, name: "KC_F22", label: "F22", category: KeycodeCategory::Function },
    Keycode { value: 0x0072, name: "KC_F23", label: "F23", category: KeycodeCategory::Function },
    Keycode { value: 0x0073, name: "KC_F24", label: "F24", category: KeycodeCategory::Function },

    // ── Navigation ────────────────────────────────────────────────────────────
    Keycode { value: 0x0046, name: "KC_PSCREEN",   label: "PrtSc", category: KeycodeCategory::Navigation },
    Keycode { value: 0x0047, name: "KC_SCROLLLOCK", label: "ScrLk", category: KeycodeCategory::Navigation },
    Keycode { value: 0x0048, name: "KC_PAUSE",     label: "Pause", category: KeycodeCategory::Navigation },
    Keycode { value: 0x0049, name: "KC_INSERT",    label: "Ins",   category: KeycodeCategory::Navigation },
    Keycode { value: 0x004A, name: "KC_HOME",      label: "Home",  category: KeycodeCategory::Navigation },
    Keycode { value: 0x004B, name: "KC_PGUP",      label: "PgUp",  category: KeycodeCategory::Navigation },
    Keycode { value: 0x004C, name: "KC_DELETE",    label: "Del",   category: KeycodeCategory::Navigation },
    Keycode { value: 0x004D, name: "KC_END",       label: "End",   category: KeycodeCategory::Navigation },
    Keycode { value: 0x004E, name: "KC_PGDOWN",    label: "PgDn",  category: KeycodeCategory::Navigation },
    Keycode { value: 0x004F, name: "KC_RIGHT",     label: "Right", category: KeycodeCategory::Navigation },
    Keycode { value: 0x0050, name: "KC_LEFT",      label: "Left",  category: KeycodeCategory::Navigation },
    Keycode { value: 0x0051, name: "KC_DOWN",      label: "Down",  category: KeycodeCategory::Navigation },
    Keycode { value: 0x0052, name: "KC_UP",        label: "Up",    category: KeycodeCategory::Navigation },

    // ── Numpad ────────────────────────────────────────────────────────────────
    Keycode { value: 0x0053, name: "KC_NUMLOCK",     label: "NmLk",  category: KeycodeCategory::Numpad },
    Keycode { value: 0x0054, name: "KC_KP_SLASH",    label: "N/",    category: KeycodeCategory::Numpad },
    Keycode { value: 0x0055, name: "KC_KP_ASTERISK", label: "N*",    category: KeycodeCategory::Numpad },
    Keycode { value: 0x0056, name: "KC_KP_MINUS",    label: "N-",    category: KeycodeCategory::Numpad },
    Keycode { value: 0x0057, name: "KC_KP_PLUS",     label: "N+",    category: KeycodeCategory::Numpad },
    Keycode { value: 0x0058, name: "KC_KP_ENTER",    label: "NEntr", category: KeycodeCategory::Numpad },
    Keycode { value: 0x0059, name: "KC_KP_1",  label: "N1", category: KeycodeCategory::Numpad },
    Keycode { value: 0x005A, name: "KC_KP_2",  label: "N2", category: KeycodeCategory::Numpad },
    Keycode { value: 0x005B, name: "KC_KP_3",  label: "N3", category: KeycodeCategory::Numpad },
    Keycode { value: 0x005C, name: "KC_KP_4",  label: "N4", category: KeycodeCategory::Numpad },
    Keycode { value: 0x005D, name: "KC_KP_5",  label: "N5", category: KeycodeCategory::Numpad },
    Keycode { value: 0x005E, name: "KC_KP_6",  label: "N6", category: KeycodeCategory::Numpad },
    Keycode { value: 0x005F, name: "KC_KP_7",  label: "N7", category: KeycodeCategory::Numpad },
    Keycode { value: 0x0060, name: "KC_KP_8",  label: "N8", category: KeycodeCategory::Numpad },
    Keycode { value: 0x0061, name: "KC_KP_9",  label: "N9", category: KeycodeCategory::Numpad },
    Keycode { value: 0x0062, name: "KC_KP_0",  label: "N0", category: KeycodeCategory::Numpad },
    Keycode { value: 0x0063, name: "KC_KP_DOT",   label: "N.",  category: KeycodeCategory::Numpad },
    Keycode { value: 0x0067, name: "KC_KP_EQUAL", label: "N=",  category: KeycodeCategory::Numpad },
    Keycode { value: 0x0085, name: "KC_KP_COMMA", label: "N,",  category: KeycodeCategory::Numpad },

    // ── Modifiers ─────────────────────────────────────────────────────────────
    Keycode { value: 0x00E0, name: "KC_LCTRL",  label: "LCtl", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E1, name: "KC_LSHIFT", label: "LSft", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E2, name: "KC_LALT",   label: "LAlt", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E3, name: "KC_LGUI",   label: "LGui", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E4, name: "KC_RCTRL",  label: "RCtl", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E5, name: "KC_RSHIFT", label: "RSft", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E6, name: "KC_RALT",   label: "RAlt", category: KeycodeCategory::Modifier },
    Keycode { value: 0x00E7, name: "KC_RGUI",   label: "RGui", category: KeycodeCategory::Modifier },

    // ── Media ─────────────────────────────────────────────────────────────────
    Keycode { value: 0x00A8, name: "KC_MUTE",  label: "Mute", category: KeycodeCategory::Media },
    Keycode { value: 0x00A9, name: "KC_VOLU",  label: "Vol+", category: KeycodeCategory::Media },
    Keycode { value: 0x00AA, name: "KC_VOLD",  label: "Vol-", category: KeycodeCategory::Media },
    Keycode { value: 0x00AB, name: "KC_MNXT",  label: "Next", category: KeycodeCategory::Media },
    Keycode { value: 0x00AC, name: "KC_MPRV",  label: "Prev", category: KeycodeCategory::Media },
    Keycode { value: 0x00AD, name: "KC_MSTP",  label: "Stop", category: KeycodeCategory::Media },
    Keycode { value: 0x00AE, name: "KC_MPLY",  label: "Play", category: KeycodeCategory::Media },
    Keycode { value: 0x00AF, name: "KC_MSEL",  label: "MSel", category: KeycodeCategory::Media },
    Keycode { value: 0x00B0, name: "KC_MAIL",  label: "Mail", category: KeycodeCategory::Media },
    Keycode { value: 0x00B1, name: "KC_CALC",  label: "Calc", category: KeycodeCategory::Media },
    Keycode { value: 0x00B2, name: "KC_MYCM",  label: "MyPC", category: KeycodeCategory::Media },
    Keycode { value: 0x00B3, name: "KC_WSCH",  label: "Srch", category: KeycodeCategory::Media },
    Keycode { value: 0x00B4, name: "KC_WHOM",  label: "WWW",  category: KeycodeCategory::Media },
    Keycode { value: 0x00B5, name: "KC_WBAK",  label: "Back", category: KeycodeCategory::Media },
    Keycode { value: 0x00B6, name: "KC_WFWD",  label: "Fwd",  category: KeycodeCategory::Media },
    Keycode { value: 0x00B7, name: "KC_WSTP",  label: "WStop",category: KeycodeCategory::Media },
    Keycode { value: 0x00B8, name: "KC_WREF",  label: "Rfsh", category: KeycodeCategory::Media },
    Keycode { value: 0x00B9, name: "KC_WFAV",  label: "Favs", category: KeycodeCategory::Media },
    Keycode { value: 0x00A5, name: "KC_SLEP",  label: "Sleep",category: KeycodeCategory::Media },
    Keycode { value: 0x00A6, name: "KC_WAKE",  label: "Wake", category: KeycodeCategory::Media },
    Keycode { value: 0x00A7, name: "KC_BRIU",  label: "Bri+", category: KeycodeCategory::Media },
    Keycode { value: 0x00BB, name: "KC_BRID",  label: "Bri-", category: KeycodeCategory::Media },
    Keycode { value: 0x0066, name: "KC_PWR",   label: "Power",category: KeycodeCategory::Media },

    // ── Mouse ─────────────────────────────────────────────────────────────────
    Keycode { value: 0x00F0, name: "KC_MS_U", label: "Ms U",category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F1, name: "KC_MS_D", label: "Ms D",category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F2, name: "KC_MS_L", label: "Ms L",category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F3, name: "KC_MS_R", label: "Ms R",category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F4, name: "KC_BTN1", label: "MB1", category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F5, name: "KC_BTN2", label: "MB2", category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F6, name: "KC_BTN3", label: "MB3", category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F7, name: "KC_BTN4", label: "MB4", category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F8, name: "KC_BTN5", label: "MB5", category: KeycodeCategory::Mouse },
    Keycode { value: 0x00F9, name: "KC_WH_U", label: "Wh U",category: KeycodeCategory::Mouse },
    Keycode { value: 0x00FA, name: "KC_WH_D", label: "Wh D",category: KeycodeCategory::Mouse },
    Keycode { value: 0x00FB, name: "KC_WH_L", label: "Wh L",category: KeycodeCategory::Mouse },
    Keycode { value: 0x00FC, name: "KC_WH_R", label: "Wh R",category: KeycodeCategory::Mouse },

    // ── QMK special ───────────────────────────────────────────────────────────
    Keycode { value: 0x7C16, name: "KC_GESC",  label: "GEsc",  category: KeycodeCategory::Special },
    Keycode { value: 0x7C77, name: "QK_BOOT",  label: "Boot",  category: KeycodeCategory::Special },
    Keycode { value: 0x7C00, name: "DB_TOGG",  label: "Debug", category: KeycodeCategory::Special },
    Keycode { value: 0x7800, name: "QK_LOCK",  label: "Lock",  category: KeycodeCategory::Special },
    Keycode { value: 0x7C1A, name: "KC_LSPO",  label: "LSPO",  category: KeycodeCategory::Special },
    Keycode { value: 0x7C1B, name: "KC_RSPC",  label: "RSPC",  category: KeycodeCategory::Special },
    Keycode { value: 0x7C18, name: "KC_LCPO",  label: "LCPO",  category: KeycodeCategory::Special },
    Keycode { value: 0x7C19, name: "KC_RCPC",  label: "RCPC",  category: KeycodeCategory::Special },
    Keycode { value: 0x7C1E, name: "KC_SFTENT",label: "SftEnt",category: KeycodeCategory::Special },
    Keycode { value: 0x7C14, name: "QK_MAKE",  label: "Make",  category: KeycodeCategory::Special },
    Keycode { value: 0x7C15, name: "KC_ASTG",  label: "ASpeed",category: KeycodeCategory::Special },
    // RGB Light
    Keycode { value: 0x7A00, name: "RGB_TOG",  label: "RGB\nTog",  category: KeycodeCategory::Special },
    Keycode { value: 0x7A01, name: "RGB_MOD",  label: "RGB\nMod",  category: KeycodeCategory::Special },
    Keycode { value: 0x7A02, name: "RGB_RMOD", label: "RGB\nRMod", category: KeycodeCategory::Special },
    Keycode { value: 0x7A03, name: "RGB_HUI",  label: "RGB\nH+",   category: KeycodeCategory::Special },
    Keycode { value: 0x7A04, name: "RGB_HUD",  label: "RGB\nH-",   category: KeycodeCategory::Special },
    Keycode { value: 0x7A05, name: "RGB_SAI",  label: "RGB\nS+",   category: KeycodeCategory::Special },
    Keycode { value: 0x7A06, name: "RGB_SAD",  label: "RGB\nS-",   category: KeycodeCategory::Special },
    Keycode { value: 0x7A07, name: "RGB_VAI",  label: "RGB\nV+",   category: KeycodeCategory::Special },
    Keycode { value: 0x7A08, name: "RGB_VAD",  label: "RGB\nV-",   category: KeycodeCategory::Special },
    Keycode { value: 0x7A09, name: "RGB_SPI",  label: "RGB\nSpd+", category: KeycodeCategory::Special },
    Keycode { value: 0x7A0A, name: "RGB_SPD",  label: "RGB\nSpd-", category: KeycodeCategory::Special },
];

pub fn find_keycode(value: u16) -> Option<&'static Keycode> {
    KEYCODES.iter().find(|k| k.value == value)
}

/// Returns a human-readable label for a keycode.
/// Uses vial protocol v6 keycode ranges.
pub fn keycode_label_with_custom(value: u16, custom: &[(String, String)]) -> String {
    if let Some(kc) = find_keycode(value) {
        return kc.label.to_string();
    }

    // Custom ergohaven keycodes start at 0x7E40 (USER00 in QMK)
    const USER_BASE: u16 = 0x7E40;
    if value >= USER_BASE && (value as usize) < USER_BASE as usize + custom.len() {
        let idx = (value - USER_BASE) as usize;
        if let Some((_, label)) = custom.get(idx) {
            return label.clone();
        }
    }

    // QK_LAYER_TAP: 0x4000 | (layer << 8) | kc
    if value & 0xC000 == 0x4000 {
        let layer = (value >> 8) & 0x3F;
        let kc = value & 0xFF;
        let kc_str = find_keycode(kc as u16).map(|k| k.label).unwrap_or("?");
        return format!("LT{}/{}", layer, kc_str);
    }

    // One-shot mod: 0x52A0 base
    if value & 0xFF00 == 0x52A0 || value & 0xFF00 == 0x52B0 {
        let mods = value & 0xFF;
        let mod_str = decode_mods(mods, value >= 0x52B0);
        return format!("OSM\n{}", mod_str);
    }

    // Layer ops — vial v6 protocol:
    // QK_TO             = 0x5200, stride 1
    // QK_MOMENTARY      = 0x5220, stride 1  → MO(n)
    // QK_DEF_LAYER      = 0x5240            → DF(n)
    // QK_TOGGLE_LAYER   = 0x5260            → TG(n)
    // QK_ONE_SHOT_LAYER = 0x5280            → OSL(n)
    // QK_LAYER_TAP_TOG  = 0x52C0            → TT(n)
    // QK_PERSISTENT_DEF = 0x52E0            → PDF(n)
    if value >= 0x5200 && value < 0x5300 {
        let sub = value & 0xFF;
        return match (value >> 5) & 0x7 {
            0 => format!("TO({})", sub & 0x1F),   // 0x5200
            1 => format!("MO({})", sub & 0x1F),   // 0x5220
            2 => format!("DF({})", sub & 0x1F),   // 0x5240
            3 => format!("TG({})", sub & 0x1F),   // 0x5260
            4 => format!("OSL({})", sub & 0x1F),  // 0x5280
            5 => format!("OSM({})", sub),          // 0x52A0
            6 => format!("TT({})", sub & 0x1F),   // 0x52C0
            7 => format!("PDF({})", sub & 0x1F),  // 0x52E0
            _ => format!("{:04X}", value),
        };
    }

    // QK_LAYER_MOD: 0x5000 | (layer << 4) | mods
    if value >= 0x5000 && value < 0x5200 {
        let layer = (value >> 4) & 0xF;
        return format!("LM({})", layer);
    }

    // QK_MOD_TAP: 0x2000 | (mods << 8) | kc
    if value & 0xE000 == 0x2000 {
        let kc = value & 0xFF;
        let mods = (value >> 8) & 0x1F;
        let right = (value >> 12) & 0x1 != 0;
        let kc_str = find_keycode(kc as u16).map(|k| k.label).unwrap_or("?");
        let mod_str = decode_mods(mods as u16, right);
        return format!("{}/{}", mod_str, kc_str);
    }

    // Modifier+key combos: 0x0100..0x1F00 | kc
    // LCTL=0x0100, LSFT=0x0200, LALT=0x0400, LGUI=0x0800
    // RCTL=0x1100, RSFT=0x1200, RALT=0x1400, RGUI=0x1800
    // MEH=0x0700, HYPR=0x0F00, LSA=0x0600, LCA=0x0500
    if value >= 0x0100 && value < 0x2000 && (value & 0xFF) != 0 {
        let mods = value >> 8;
        let kc = value & 0xFF;
        let kc_str = find_keycode(kc as u16).map(|k| k.label).unwrap_or("?");
        let mod_str = match mods {
            0x01 => "LCtl",
            0x02 => "LSft",
            0x04 => "LAlt",
            0x08 => "LGui",
            0x03 => "LCS",
            0x05 => "LCA",
            0x06 => "LSA",
            0x07 => "Meh",
            0x09 => "LCG",
            0x0C => "LAG",
            0x0F => "Hypr",
            0x0A => "LSG",
            0x11 => "RCtl",
            0x12 => "RSft",
            0x14 => "RAlt",
            0x18 => "RGui",
            _ => "Mod",
        };
        return format!("{}/{}", mod_str, kc_str);
    }

    if value == 0x0001 { return "TRNS".to_string(); }
    if value == 0x0000 { return "NO".to_string(); }

    format!("{:04X}", value)
}

fn decode_mods(mods: u16, _right: bool) -> &'static str {
    match mods {
        0x01 | 0x11 => "Ctl",
        0x02 | 0x12 => "Sft",
        0x04 | 0x14 => "Alt",
        0x08 | 0x18 => "Gui",
        0x03 | 0x13 => "CS",
        0x05 | 0x15 => "CA",
        0x06 | 0x16 => "SA",
        0x07 | 0x17 => "Meh",
        0x0F | 0x1F => "Hypr",
        _ => "Mod",
    }
}

pub fn keycode_label(value: u16) -> String {
    keycode_label_with_custom(value, &[])
}
