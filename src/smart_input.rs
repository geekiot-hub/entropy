#![allow(non_snake_case)]

#[derive(Clone, Copy, Debug)]
pub struct SmartSymbol {
    pub trigger_keycode: u16,
    pub symbol: char,
    pub name: &'static str,
}

const KC_F13: u16 = 0x0068;
const MOD_CTRL: u16 = 0x0100;
const MOD_SHIFT: u16 = 0x0200;
const MOD_ALT: u16 = 0x0400;

pub const SMART_SYMBOLS: &[SmartSymbol] = &[
    // F13..F24
    SmartSymbol { trigger_keycode: KC_F13,      symbol: '{', name: "Left brace" },
    SmartSymbol { trigger_keycode: KC_F13 + 1,  symbol: '}', name: "Right brace" },
    SmartSymbol { trigger_keycode: KC_F13 + 2,  symbol: '[', name: "Left bracket" },
    SmartSymbol { trigger_keycode: KC_F13 + 3,  symbol: ']', name: "Right bracket" },
    SmartSymbol { trigger_keycode: KC_F13 + 4,  symbol: '(', name: "Left parenthesis" },
    SmartSymbol { trigger_keycode: KC_F13 + 5,  symbol: ')', name: "Right parenthesis" },
    SmartSymbol { trigger_keycode: KC_F13 + 6,  symbol: '<', name: "Less-than" },
    SmartSymbol { trigger_keycode: KC_F13 + 7,  symbol: '>', name: "Greater-than" },
    SmartSymbol { trigger_keycode: KC_F13 + 8,  symbol: '#', name: "Number sign" },
    SmartSymbol { trigger_keycode: KC_F13 + 9,  symbol: '@', name: "At sign" },
    SmartSymbol { trigger_keycode: KC_F13 + 10, symbol: '№', name: "Numero sign" },
    SmartSymbol { trigger_keycode: KC_F13 + 11, symbol: '₽', name: "Ruble sign" },

    // Shift+F13..F24
    SmartSymbol { trigger_keycode: MOD_SHIFT | KC_F13,      symbol: '!', name: "Exclamation mark" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 1),  symbol: '"', name: "Quotation mark" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 2),  symbol: '$', name: "Dollar sign" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 3),  symbol: '%', name: "Percent sign" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 4),  symbol: '&', name: "Ampersand" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 5),  symbol: '\'', name: "Apostrophe" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 6),  symbol: '*', name: "Asterisk" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 7),  symbol: '+', name: "Plus sign" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 8),  symbol: '=', name: "Equals sign" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 9),  symbol: '?', name: "Question mark" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 10), symbol: '|', name: "Vertical bar" },
    SmartSymbol { trigger_keycode: MOD_SHIFT | (KC_F13 + 11), symbol: '\\', name: "Backslash" },

    // Ctrl+F13..F24
    SmartSymbol { trigger_keycode: MOD_CTRL | KC_F13,      symbol: '«', name: "Left guillemet" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 1),  symbol: '»', name: "Right guillemet" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 2),  symbol: '€', name: "Euro sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 3),  symbol: '—', name: "Em dash" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 4),  symbol: '–', name: "En dash" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 5),  symbol: '•', name: "Bullet" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 6),  symbol: '×', name: "Multiplication sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 7),  symbol: '±', name: "Plus-minus sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 8),  symbol: '≠', name: "Not equal sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 9),  symbol: '≈', name: "Almost equal sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 10), symbol: '✓', name: "Check mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | (KC_F13 + 11), symbol: '§', name: "Section sign" },

    // Alt+F13..F24
    SmartSymbol { trigger_keycode: MOD_ALT | KC_F13,      symbol: '.', name: "Full stop" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 1),  symbol: ',', name: "Comma" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 2),  symbol: ';', name: "Semicolon" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 3),  symbol: ':', name: "Colon" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 4),  symbol: '/', name: "Slash" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 5),  symbol: '`', name: "Grave accent" },
    SmartSymbol { trigger_keycode: MOD_ALT | (KC_F13 + 6),  symbol: '^', name: "Caret" },

    // Ctrl+Alt+F13..F19
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | KC_F13,      symbol: 'Б', name: "Cyrillic Be" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 1),  symbol: 'Ю', name: "Cyrillic Yu" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 2),  symbol: 'Ж', name: "Cyrillic Zhe" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 3),  symbol: 'Э', name: "Cyrillic E" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 4),  symbol: 'Х', name: "Cyrillic Ha" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 5),  symbol: 'Ъ', name: "Cyrillic Hard Sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_ALT | (KC_F13 + 6),  symbol: 'Ё', name: "Cyrillic Yo" },

    // Ctrl+Shift+F13..F24
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | KC_F13,      symbol: '°', name: "Degree sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 1),  symbol: '‰', name: "Per mille sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 2),  symbol: '′', name: "Prime" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 3),  symbol: '″', name: "Double prime" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 4),  symbol: '‘', name: "Left single quotation mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 5),  symbol: '’', name: "Right single quotation mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 6),  symbol: '„', name: "Double low quotation mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 7),  symbol: '“', name: "Left double quotation mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 8),  symbol: '”', name: "Right double quotation mark" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 9),  symbol: '™', name: "Trade mark sign" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 10), symbol: '~', name: "Tilde" },
    SmartSymbol { trigger_keycode: MOD_CTRL | MOD_SHIFT | (KC_F13 + 11), symbol: '_', name: "Underscore" },
];

pub fn smart_symbol_for_keycode(keycode: u16) -> Option<SmartSymbol> {
    SMART_SYMBOLS
        .iter()
        .copied()
        .find(|symbol| symbol.trigger_keycode == keycode)
}

#[cfg(target_os = "windows")]
pub fn start() {
    use std::sync::Once;
    static START: Once = Once::new();
    START.call_once(|| {
        std::thread::spawn(|| unsafe {
            run_windows_keyboard_hook_loop();
        });
    });
}

#[cfg(not(target_os = "windows"))]
pub fn start() {}

#[cfg(target_os = "windows")]
fn symbol_for_vk(vk: u32) -> Option<(char, u16)> {
    let base_keycode = match vk {
        0x7C..=0x87 => KC_F13 + (vk - 0x7C) as u16,
        _ => return None,
    };
    let mut trigger_keycode = base_keycode;
    if modifier_down(VK_CONTROL) {
        trigger_keycode |= MOD_CTRL;
    }
    if modifier_down(VK_SHIFT) {
        trigger_keycode |= MOD_SHIFT;
    }
    if modifier_down(VK_MENU) {
        trigger_keycode |= MOD_ALT;
    }
    smart_symbol_for_keycode(trigger_keycode).map(|symbol| (symbol.symbol, trigger_keycode))
}

#[cfg(target_os = "windows")]
fn modifier_down(vk: i32) -> bool {
    unsafe { GetAsyncKeyState(vk) & 0x8000u16 as i16 != 0 }
}

#[cfg(target_os = "windows")]
unsafe fn run_windows_keyboard_hook_loop() {
    let module = GetModuleHandleW(std::ptr::null());
    let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0);
    if hook.is_null() {
        log::warn!("Smart Input: failed to install keyboard hook");
        return;
    }

    let mut msg = MSG::default();
    while GetMessageW(&mut msg as *mut MSG, std::ptr::null_mut(), 0, 0) > 0 {
        TranslateMessage(&msg as *const MSG);
        DispatchMessageW(&msg as *const MSG);
    }

    UnhookWindowsHookEx(hook);
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    if n_code == HC_ACTION {
        let info = &*(l_param as *const KBDLLHOOKSTRUCT);
        let is_key_down = w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN;
        let is_key_up = w_param == WM_KEYUP || w_param == WM_SYSKEYUP;
        let injected = info.flags & LLKHF_INJECTED != 0;
        if !injected {
            if let Some((symbol, trigger_keycode)) = symbol_for_vk(info.vkCode) {
                if is_key_down {
                    send_unicode_char(symbol, trigger_keycode);
                }
                if is_key_down || is_key_up {
                    return 1;
                }
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
}

#[cfg(target_os = "windows")]
unsafe fn send_unicode_char(symbol: char, trigger_keycode: u16) {
    release_transport_modifiers(trigger_keycode);
    for unit in symbol.encode_utf16(&mut [0; 2]) {
        let down = INPUT::keyboard_unicode(*unit, false);
        let up = INPUT::keyboard_unicode(*unit, true);
        let inputs = [down, up];
        let sent = SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
        if sent != inputs.len() as u32 {
            log::warn!("Smart Input: SendInput failed for U+{:04X}", *unit as u32);
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn release_transport_modifiers(trigger_keycode: u16) {
    if trigger_keycode & MOD_SHIFT != 0 {
        send_vk_keyup(VK_SHIFT as u16);
    }
    if trigger_keycode & MOD_ALT != 0 {
        send_vk_keyup(VK_MENU as u16);
    }
    if trigger_keycode & MOD_CTRL != 0 {
        send_vk_keyup(VK_CONTROL as u16);
    }
}

#[cfg(target_os = "windows")]
unsafe fn send_vk_keyup(vk: u16) {
    let input = INPUT::keyboard_vk(vk, true);
    let sent = SendInput(1, &input as *const INPUT, std::mem::size_of::<INPUT>() as i32);
    if sent != 1 {
        log::warn!("Smart Input: SendInput failed for VK keyup 0x{:02X}", vk);
    }
}

#[cfg(target_os = "windows")]
const WH_KEYBOARD_LL: i32 = 13;
#[cfg(target_os = "windows")]
const HC_ACTION: i32 = 0;
#[cfg(target_os = "windows")]
const WM_KEYDOWN: usize = 0x0100;
#[cfg(target_os = "windows")]
const WM_SYSKEYDOWN: usize = 0x0104;
#[cfg(target_os = "windows")]
const WM_KEYUP: usize = 0x0101;
#[cfg(target_os = "windows")]
const WM_SYSKEYUP: usize = 0x0105;
#[cfg(target_os = "windows")]
const VK_SHIFT: i32 = 0x10;
#[cfg(target_os = "windows")]
const VK_CONTROL: i32 = 0x11;
#[cfg(target_os = "windows")]
const VK_MENU: i32 = 0x12;
#[cfg(target_os = "windows")]
const LLKHF_INJECTED: u32 = 0x10;
#[cfg(target_os = "windows")]
const INPUT_KEYBOARD: u32 = 1;
#[cfg(target_os = "windows")]
const KEYEVENTF_KEYUP: u32 = 0x0002;
#[cfg(target_os = "windows")]
const KEYEVENTF_UNICODE: u32 = 0x0004;

#[cfg(target_os = "windows")]
type HHOOK = *mut core::ffi::c_void;
#[cfg(target_os = "windows")]
type HINSTANCE = *mut core::ffi::c_void;
#[cfg(target_os = "windows")]
type HWND = *mut core::ffi::c_void;

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct KBDLLHOOKSTRUCT {
    vkCode: u32,
    scanCode: u32,
    flags: u32,
    time: u32,
    dwExtraInfo: usize,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct MSG {
    hwnd: HWND,
    message: u32,
    wParam: usize,
    lParam: isize,
    time: u32,
    pt_x: i32,
    pt_y: i32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct INPUT {
    input_type: u32,
    u: INPUT_UNION,
}

#[cfg(target_os = "windows")]
impl INPUT {
    fn keyboard_unicode(unit: u16, key_up: bool) -> Self {
        let flags = KEYEVENTF_UNICODE | if key_up { KEYEVENTF_KEYUP } else { 0 };
        Self {
            input_type: INPUT_KEYBOARD,
            u: INPUT_UNION {
                ki: KEYBDINPUT {
                    wVk: 0,
                    wScan: unit,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn keyboard_vk(vk: u16, key_up: bool) -> Self {
        let flags = if key_up { KEYEVENTF_KEYUP } else { 0 };
        Self {
            input_type: INPUT_KEYBOARD,
            u: INPUT_UNION {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
union INPUT_UNION {
    mi: MOUSEINPUT,
    ki: KEYBDINPUT,
}

#[cfg(all(target_os = "windows", target_pointer_width = "64"))]
const _: [(); 40] = [(); std::mem::size_of::<INPUT>()];
#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
const _: [(); 28] = [(); std::mem::size_of::<INPUT>()];

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct MOUSEINPUT {
    dx: i32,
    dy: i32,
    mouseData: u32,
    dwFlags: u32,
    time: u32,
    dwExtraInfo: usize,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
struct KEYBDINPUT {
    wVk: u16,
    wScan: u16,
    dwFlags: u32,
    time: u32,
    dwExtraInfo: usize,
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
extern "system" {
    fn SetWindowsHookExW(
        idHook: i32,
        lpfn: Option<unsafe extern "system" fn(i32, usize, isize) -> isize>,
        hmod: HINSTANCE,
        dwThreadId: u32,
    ) -> HHOOK;
    fn CallNextHookEx(hhk: HHOOK, nCode: i32, wParam: usize, lParam: isize) -> isize;
    fn UnhookWindowsHookEx(hhk: HHOOK) -> i32;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32;
    fn GetAsyncKeyState(vKey: i32) -> i16;
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
}
