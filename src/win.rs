//! Thin Win32 helpers: read the pixel under the cursor, detect clicks,
//! and run a global hotkey message loop on a background thread.

use std::sync::mpsc::Sender;
use windows::Win32::Foundation::POINT;
use windows::Win32::Graphics::Gdi::{CLR_INVALID, GetDC, GetPixel, ReleaseDC};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, RegisterHotKey, VK_ESCAPE, VK_LBUTTON,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetMessageW, GetWindowThreadProcessId, MSG, WM_HOTKEY, WindowFromPoint,
};

pub const HOTKEY_LABEL: &str = "Ctrl + Shift + C";
const HOTKEY_ID: i32 = 1;
const VK_C: u32 = 0x43;

/// Opt into per-monitor DPI awareness v2 *before* any window exists.
///
/// This is what makes `GetCursorPos` and `GetPixel` speak the same language:
/// without it Windows virtualises both into the primary monitor's 96-DPI space
/// and rounds them differently, so on a scaled display the sampled pixel drifts
/// away from the one under the cursor. winit also does this when it builds its
/// event loop, but we do not want to depend on that ordering — whoever calls
/// first wins, and a second call is a harmless no-op.
pub fn set_dpi_aware() {
    // SAFETY: process-wide setting, no pointers involved. Failure just means it
    // was already set (by a manifest or by winit), which is the outcome we want.
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

/// Screen coordinates of the mouse cursor (virtual-screen space, all monitors,
/// so `x`/`y` can be negative when a monitor sits left of / above the primary).
pub fn cursor_pos() -> (i32, i32) {
    let mut p = POINT::default();
    // SAFETY: POINT is a plain out-param.
    unsafe {
        let _ = GetCursorPos(&mut p);
    }
    (p.x, p.y)
}

/// RGB of the screen pixel at the given virtual-screen coordinates.
///
/// Returns `None` when the point is off-screen: `GetPixel` answers `CLR_INVALID`
/// there, which naively decoded would look like pure white.
pub fn pixel_at(x: i32, y: i32) -> Option<[u8; 3]> {
    // SAFETY: GetDC(None) is the whole virtual desktop; we release it right after.
    unsafe {
        let hdc = GetDC(None);
        if hdc.is_invalid() {
            return None;
        }
        let c = GetPixel(hdc, x, y).0; // COLORREF is 0x00BBGGRR
        ReleaseDC(None, hdc);
        if c == CLR_INVALID {
            return None;
        }
        Some([(c & 0xFF) as u8, ((c >> 8) & 0xFF) as u8, ((c >> 16) & 0xFF) as u8])
    }
}

/// True when the window under the given screen point belongs to us.
///
/// We are always-on-top, so a click meant for the pixel behind us can easily
/// land on our own card instead; picking must ignore those.
pub fn point_over_own_window(x: i32, y: i32) -> bool {
    // SAFETY: pure queries; a null HWND from WindowFromPoint yields pid 0.
    unsafe {
        let hwnd = WindowFromPoint(POINT { x, y });
        if hwnd.is_invalid() {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid != 0 && pid == GetCurrentProcessId()
    }
}

fn key_down(vk: u16) -> bool {
    // SAFETY: pure query.
    unsafe { (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0 }
}

/// Physical left mouse button, regardless of which window has focus.
/// Honours the "swap buttons" setting because `VK_LBUTTON` is the logical
/// primary button.
pub fn left_button_down() -> bool {
    key_down(VK_LBUTTON.0)
}

pub fn escape_down() -> bool {
    key_down(VK_ESCAPE.0)
}

#[derive(Debug, Clone, Copy)]
pub enum HotkeyEvent {
    Pick,
}

/// Registers the global hotkey and pumps messages forever on this thread.
/// Call from a dedicated `std::thread`.
pub fn hotkey_loop(tx: Sender<HotkeyEvent>, wake: impl Fn() + Send + 'static) {
    // SAFETY: registering a thread-scoped hotkey (hwnd = None) and pumping
    // messages on the same thread is the documented usage.
    unsafe {
        if RegisterHotKey(None, HOTKEY_ID, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_C).is_err() {
            eprintln!("hexer: could not register {HOTKEY_LABEL} (already in use?)");
            return;
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_HOTKEY && msg.wParam.0 as i32 == HOTKEY_ID {
                if tx.send(HotkeyEvent::Pick).is_err() {
                    return;
                }
                wake();
            }
        }
    }
}
