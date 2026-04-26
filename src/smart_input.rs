#![allow(non_snake_case)]

use std::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, AtomicUsize, Ordering};

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

static HOTKEY_REGISTERED: AtomicUsize = AtomicUsize::new(0);
static HOTKEY_FAILED: AtomicUsize = AtomicUsize::new(0);
static HOTKEY_EVENTS: AtomicU64 = AtomicU64::new(0);
static HOOK_EVENTS: AtomicU64 = AtomicU64::new(0);
static SEND_OK: AtomicU64 = AtomicU64::new(0);
static SEND_FAIL: AtomicU64 = AtomicU64::new(0);
static LAST_TRIGGER: AtomicU32 = AtomicU32::new(0);
static LAST_SYMBOL: AtomicU32 = AtomicU32::new(0);
static LAST_ERROR: AtomicU32 = AtomicU32::new(0);
static LAST_EMIT_TICK: AtomicU64 = AtomicU64::new(0);
static LAST_SOURCE: AtomicI32 = AtomicI32::new(0);

pub fn smart_symbol_for_keycode(keycode: u16) -> Option<SmartSymbol> {
    SMART_SYMBOLS
        .iter()
        .copied()
        .find(|symbol| symbol.trigger_keycode == keycode)
}

pub fn status_text() -> String {
    #[cfg(target_os = "windows")]
    {
        let source = match LAST_SOURCE.load(Ordering::Relaxed) {
            1 => "hook",
            2 => "hotkey",
            _ => "none",
        };
        let trigger = LAST_TRIGGER.load(Ordering::Relaxed);
        let symbol = char::from_u32(LAST_SYMBOL.load(Ordering::Relaxed)).unwrap_or('—');
        format!(
            "Smart Input: hotkeys {}/{}, hook events {}, hotkey events {}, sent ok {}, failed {}, last {} VK 0x{:02X} → {}, err {}",
            HOTKEY_REGISTERED.load(Ordering::Relaxed),
            HOTKEY_FAILED.load(Ordering::Relaxed),
            HOOK_EVENTS.load(Ordering::Relaxed),
            HOTKEY_EVENTS.load(Ordering::Relaxed),
            SEND_OK.load(Ordering::Relaxed),
            SEND_FAIL.load(Ordering::Relaxed),
            source,
            trigger,
            symbol,
            LAST_ERROR.load(Ordering::Relaxed),
        )
    }
    #[cfg(not(target_os = "windows"))]
    {
        "Smart Input runtime is Windows-only in this build".to_string()
    }
}

#[cfg(target_os = "windows")]
pub fn start() {
    use std::sync::Once;
    static START: Once = Once::new();
    START.call_once(|| {
        std::thread::spawn(|| unsafe {
            run_windows_hotkey_loop();
        });
        std::thread::spawn(|| unsafe {
            run_windows_keyboard_hook_loop();
        });
    });
}

#[cfg(not(target_os = "windows"))]
pub fn start() {}

#[cfg(target_os = "windows")]
fn symbol_for_vk(vk: u32) -> Option<char> {
    let keycode = match vk {
        0x7C..=0x87 => 0x0068 + (vk - 0x7C) as u16,
        _ => return None,
    };
    smart_symbol_for_keycode(keycode).map(|symbol| symbol.symbol)
}

#[cfg(target_os = "windows")]
fn symbol_for_hotkey_id(id: i32) -> Option<(u32, char)> {
    let index = id.checked_sub(SMART_HOTKEY_BASE)? as usize;
    let smart = SMART_SYMBOLS.get(index).copied()?;
    Some((vk_for_smart_symbol(smart)?, smart.symbol))
}

#[cfg(target_os = "windows")]
fn vk_for_smart_symbol(symbol: SmartSymbol) -> Option<u32> {
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
            HOTKEY_REGISTERED.fetch_add(1, Ordering::Relaxed);
            registered.push(id);
        } else {
            HOTKEY_FAILED.fetch_add(1, Ordering::Relaxed);
            LAST_ERROR.store(GetLastError(), Ordering::Relaxed);
        }
    }

    let mut msg = MSG::default();
    while GetMessageW(&mut msg as *mut MSG, std::ptr::null_mut(), 0, 0) > 0 {
        if msg.message == WM_HOTKEY {
            HOTKEY_EVENTS.fetch_add(1, Ordering::Relaxed);
            if let Some((vk, symbol)) = symbol_for_hotkey_id(msg.wParam as i32) {
                emit_symbol(vk, symbol, 2);
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
unsafe fn run_windows_keyboard_hook_loop() {
    let module = GetModuleHandleW(std::ptr::null());
    let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0);
    if hook.is_null() {
        LAST_ERROR.store(GetLastError(), Ordering::Relaxed);
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
            if let Some(symbol) = symbol_for_vk(info.vkCode) {
                HOOK_EVENTS.fetch_add(1, Ordering::Relaxed);
                if is_key_down {
                    emit_symbol(info.vkCode, symbol, 1);
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
unsafe fn emit_symbol(vk: u32, symbol: char, source: i32) {
    let now = GetTickCount64();
    let previous = LAST_EMIT_TICK.swap(now, Ordering::Relaxed);
    if now.saturating_sub(previous) < 30 {
        return;
    }

    LAST_TRIGGER.store(vk, Ordering::Relaxed);
    LAST_SYMBOL.store(symbol as u32, Ordering::Relaxed);
    LAST_SOURCE.store(source, Ordering::Relaxed);
    send_unicode_char(symbol);
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
        if sent == inputs.len() as u32 {
            SEND_OK.fetch_add(1, Ordering::Relaxed);
        } else {
            SEND_FAIL.fetch_add(1, Ordering::Relaxed);
            LAST_ERROR.store(GetLastError(), Ordering::Relaxed);
        }
    }
}

#[cfg(target_os = "windows")]
const SMART_HOTKEY_BASE: i32 = 0x5A00;
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
const WM_HOTKEY: u32 = 0x0312;
#[cfg(target_os = "windows")]
const MOD_NOREPEAT: u32 = 0x4000;
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
    fn RegisterHotKey(hWnd: HWND, id: i32, fsModifiers: u32, vk: u32) -> i32;
    fn UnregisterHotKey(hWnd: HWND, id: i32) -> i32;
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
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    fn GetLastError() -> u32;
    fn GetTickCount64() -> u64;
}
