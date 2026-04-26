#![allow(non_snake_case)]

#[derive(Clone, Copy, Debug)]
pub struct SmartSymbol {
    pub trigger_keycode: u16,
    pub symbol: char,
    pub name: &'static str,
}

pub const SMART_SYMBOLS: &[SmartSymbol] = &[
    SmartSymbol { trigger_keycode: 0x0068, symbol: '{', name: "Left brace" },
    SmartSymbol { trigger_keycode: 0x0069, symbol: '}', name: "Right brace" },
    SmartSymbol { trigger_keycode: 0x006A, symbol: '[', name: "Left bracket" },
    SmartSymbol { trigger_keycode: 0x006B, symbol: ']', name: "Right bracket" },
    SmartSymbol { trigger_keycode: 0x006C, symbol: '(', name: "Left parenthesis" },
    SmartSymbol { trigger_keycode: 0x006D, symbol: ')', name: "Right parenthesis" },
    SmartSymbol { trigger_keycode: 0x006E, symbol: '<', name: "Less-than" },
    SmartSymbol { trigger_keycode: 0x006F, symbol: '>', name: "Greater-than" },
    SmartSymbol { trigger_keycode: 0x0070, symbol: '#', name: "Number sign" },
    SmartSymbol { trigger_keycode: 0x0071, symbol: '@', name: "At sign" },
    SmartSymbol { trigger_keycode: 0x0072, symbol: '№', name: "Numero sign" },
    SmartSymbol { trigger_keycode: 0x0073, symbol: '₽', name: "Ruble sign" },
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
            run_windows_hotkey_loop();
        });
    });
}

#[cfg(not(target_os = "windows"))]
pub fn start() {}

#[cfg(target_os = "windows")]
fn symbol_for_hotkey_id(id: i32) -> Option<char> {
    let index = id.checked_sub(SMART_HOTKEY_BASE)? as usize;
    SMART_SYMBOLS.get(index).map(|symbol| symbol.symbol)
}

#[cfg(target_os = "windows")]
fn vk_for_smart_symbol(symbol: SmartSymbol) -> Option<u32> {
    // QMK KC_F13..KC_F24 keycodes are HID usages 0x68..0x73.
    // Windows virtual key codes VK_F13..VK_F24 are 0x7C..0x87.
    match symbol.trigger_keycode {
        0x0068..=0x0073 => Some(0x7C + (symbol.trigger_keycode - 0x0068) as u32),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
unsafe fn run_windows_hotkey_loop() {
    let mut registered = Vec::new();
    for (index, symbol) in SMART_SYMBOLS.iter().copied().enumerate() {
        let Some(vk) = vk_for_smart_symbol(symbol) else {
            continue;
        };
        let id = SMART_HOTKEY_BASE + index as i32;
        if RegisterHotKey(std::ptr::null_mut(), id, MOD_NOREPEAT, vk) != 0 {
            registered.push(id);
        } else {
            log::warn!(
                "Smart Input: failed to register hotkey for {} ({})",
                symbol.name,
                symbol.symbol
            );
        }
    }

    if registered.is_empty() {
        log::warn!("Smart Input: no smart symbol hotkeys were registered");
        return;
    }

    let mut msg = MSG::default();
    while GetMessageW(&mut msg as *mut MSG, std::ptr::null_mut(), 0, 0) > 0 {
        if msg.message == WM_HOTKEY {
            if let Some(symbol) = symbol_for_hotkey_id(msg.wParam as i32) {
                send_unicode_char(symbol);
            }
        } else {
            TranslateMessage(&msg as *const MSG);
            DispatchMessageW(&msg as *const MSG);
        }
    }

    for id in registered {
        UnregisterHotKey(std::ptr::null_mut(), id);
    }
}

#[cfg(target_os = "windows")]
unsafe fn send_unicode_char(symbol: char) {
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
const SMART_HOTKEY_BASE: i32 = 0x5A00;
#[cfg(target_os = "windows")]
const WM_HOTKEY: u32 = 0x0312;
#[cfg(target_os = "windows")]
const MOD_NOREPEAT: u32 = 0x4000;
#[cfg(target_os = "windows")]
const INPUT_KEYBOARD: u32 = 1;
#[cfg(target_os = "windows")]
const KEYEVENTF_KEYUP: u32 = 0x0002;
#[cfg(target_os = "windows")]
const KEYEVENTF_UNICODE: u32 = 0x0004;

#[cfg(target_os = "windows")]
type HWND = *mut core::ffi::c_void;

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
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
union INPUT_UNION {
    ki: KEYBDINPUT,
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
    fn RegisterHotKey(hWnd: HWND, id: i32, fsModifiers: u32, vk: u32) -> i32;
    fn UnregisterHotKey(hWnd: HWND, id: i32) -> i32;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32;
}
